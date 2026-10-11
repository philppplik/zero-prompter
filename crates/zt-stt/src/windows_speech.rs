//! Windows speech recognition (`Windows.Media.SpeechRecognition`) as a
//! system fallback engine (ADR-008). Windows only, feature `winspeech`.
//!
//! Unlike the sherpa engines this adapter is *self-capturing*: Windows opens
//! the default microphone itself, so [`SpeechEngine::accept_waveform`] is a
//! no-op and [`SpeechEngine::self_capturing`] returns `true`. Partial results
//! come from `SpeechRecognizer.HypothesisGenerated`, finals from
//! `ContinuousRecognitionSession.ResultGenerated`, VAD from `StateChanged`.
//!
//! Threading: all WinRT objects live on a dedicated `zt-winspeech` thread
//! (multithreaded apartment: windows-rs joins the MTA implicitly via
//! `CoIncrementMTAUsage` on first activation). Async operations are awaited
//! blocking with `.get()` on that thread, never on the UI thread. Event
//! handlers run on the WinRT thread pool and only push into a channel.
//!
//! Constraint modes:
//! - [`WinSpeechMode::Dictation`]: `SpeechRecognitionTopicConstraint`
//!   (scenario Dictation). Needs the speech language installed and the
//!   privacy setting "Online-Spracherkennung" enabled (Windows uses the
//!   cloud dictation service for this constraint).
//! - [`WinSpeechMode::ScriptList`]: `SpeechRecognitionListConstraint` built
//!   from the script words (hotwords); works offline but only recognizes
//!   exactly those phrases. Started once hotwords are set.

use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use windows::core::{Interface, HSTRING};
use windows::Foundation::{TimeSpan, TypedEventHandler};
use windows::Globalization::Language;
use windows::Media::SpeechRecognition::{
    ISpeechRecognitionConstraint, SpeechContinuousRecognitionCompletedEventArgs,
    SpeechContinuousRecognitionResultGeneratedEventArgs, SpeechContinuousRecognitionSession,
    SpeechRecognitionConfidence, SpeechRecognitionHypothesisGeneratedEventArgs,
    SpeechRecognitionListConstraint, SpeechRecognitionResultStatus, SpeechRecognitionScenario,
    SpeechRecognitionTopicConstraint, SpeechRecognizer, SpeechRecognizerState,
    SpeechRecognizerStateChangedEventArgs,
};

use windows_collections::{IIterable, IVectorView};

use crate::{Hotword, SpeechEngine, SttError, SttEvent, WINSPEECH_NAME};

/// HRESULT 0x80045509 (SPERR_SPEECH_PRIVACY_POLICY_NOT_ACCEPTED).
const HR_PRIVACY: i32 = 0x8004_5509_u32 as i32;
/// HRESULT 0x80070005 (E_ACCESSDENIED, microphone access denied).
const HR_ACCESS_DENIED: i32 = 0x8007_0005_u32 as i32;
/// Max phrases for the offline list constraint.
const MAX_LIST_PHRASES: usize = 2000;

/// Hint for the operator UI (settings pages, `ms-settings:` URIs).
pub const HINT_PRIVACY: &str =
    "Windows-Einstellungen › Datenschutz und Sicherheit › Spracherkennung › \
     „Online-Spracherkennung“ einschalten (ms-settings:privacy-speech)";
pub const HINT_LANGUAGE: &str = "Windows-Einstellungen › Zeit und Sprache › Sprache und Region › \
     Deutsch › Sprachoptionen › „Spracherkennung“ installieren (ms-settings:regionlanguage), \
     dann unter Zeit und Sprache › Spracherkennung die Sprache wählen (ms-settings:speech)";
pub const HINT_MICROPHONE: &str =
    "Windows-Einstellungen › Datenschutz und Sicherheit › Mikrofon › \
     „Desktop-Apps Zugriff auf das Mikrofon erlauben“ (ms-settings:privacy-microphone)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinSpeechMode {
    Dictation,
    ScriptList,
}

#[derive(Debug, Clone)]
pub struct WinSpeechConfig {
    /// BCP-47 tag, e.g. "de-DE"; a bare "de" is matched against the installed
    /// languages ("de-DE", "de-AT", ...).
    pub language: String,
    pub mode: WinSpeechMode,
    /// Dictation: if the privacy setting blocks it, switch to `ScriptList`
    /// instead of failing (offline, script words only).
    pub list_fallback: bool,
    /// End-silence timeout (finalizes an utterance).
    pub end_silence: Duration,
}

impl WinSpeechConfig {
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            mode: WinSpeechMode::Dictation,
            list_fallback: true,
            end_silence: Duration::from_millis(400),
        }
    }
}

/// Result of a non-streaming initialization check (CI smoke, diagnostics).
#[derive(Debug, Clone)]
pub struct ProbeReport {
    pub topic_languages: Vec<String>,
    pub grammar_languages: Vec<String>,
    pub system_language: Option<String>,
    pub chosen_language: Option<String>,
    /// `(mode, Ok(status) | Err(message))` of the compile attempts.
    pub compile: Vec<(WinSpeechMode, Result<String, String>)>,
}

/// Completion status of a session that ended on its own.
type Completed = Arc<Mutex<Option<String>>>;

enum Cmd {
    Phrases(Vec<String>),
    Reset,
    Stop,
}

/// Self-capturing Windows speech engine.
pub struct WinSpeechEngine {
    name: String,
    cmd: Sender<Cmd>,
    events: Receiver<SttEvent>,
    thread: Option<JoinHandle<()>>,
    init_ms: u64,
}

impl std::fmt::Debug for WinSpeechEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WinSpeechEngine").field("name", &self.name).finish_non_exhaustive()
    }
}

fn hr_msg(e: &windows::core::Error) -> String {
    format!("{} (HRESULT 0x{:08X})", e.message(), e.code().0 as u32)
}

/// Maps WinRT errors to precise operator messages.
fn map_err(stage: &str, e: &windows::core::Error) -> SttError {
    let code = e.code().0;
    let hint = if code == HR_PRIVACY {
        format!("Online-Spracherkennung ist in Windows deaktiviert. {HINT_PRIVACY}")
    } else if code == HR_ACCESS_DENIED {
        format!("Mikrofonzugriff verweigert. {HINT_MICROPHONE}")
    } else {
        String::new()
    };
    SttError::Engine(format!("Windows-Spracherkennung ({stage}): {}. {hint}", hr_msg(e)))
}

fn tags(v: windows::core::Result<IVectorView<Language>>) -> Vec<String> {
    v.map(|v| v.into_iter().filter_map(|l| l.LanguageTag().ok()).map(|t| t.to_string()).collect())
        .unwrap_or_default()
}

/// Picks an installed language tag matching `wanted` ("de-DE" or "de").
fn match_language(wanted: &str, installed: &[String]) -> Option<String> {
    let w = wanted.to_ascii_lowercase();
    installed
        .iter()
        .find(|t| t.to_ascii_lowercase() == w)
        .or_else(|| {
            let prim = w.split(['-', '_']).next().unwrap_or("");
            // Prefer the "main" region (de-DE, en-US) over others.
            let main = format!("{prim}-{}", if prim == "en" { "us" } else { prim });
            installed.iter().find(|t| t.to_ascii_lowercase() == main).or_else(|| {
                installed.iter().find(|t| t.to_ascii_lowercase().split('-').next() == Some(prim))
            })
        })
        .cloned()
}

fn timespan(d: Duration) -> TimeSpan {
    TimeSpan { Duration: (d.as_nanos() / 100) as i64 }
}

fn make_recognizer(lang: &str) -> Result<SpeechRecognizer, SttError> {
    let l = Language::CreateLanguage(&HSTRING::from(lang)).map_err(|e| map_err("Sprache", &e))?;
    SpeechRecognizer::Create(&l).map_err(|e| {
        SttError::Engine(format!(
            "Windows-Spracherkennung: Sprache {lang} nicht nutzbar: {}. {HINT_LANGUAGE}",
            hr_msg(&e)
        ))
    })
}

fn add_constraint(
    r: &SpeechRecognizer,
    mode: WinSpeechMode,
    phrases: &[String],
) -> windows::core::Result<()> {
    let c: ISpeechRecognitionConstraint = match mode {
        WinSpeechMode::Dictation => SpeechRecognitionTopicConstraint::Create(
            SpeechRecognitionScenario::Dictation,
            &HSTRING::from("dictation"),
        )?
        .cast()?,
        WinSpeechMode::ScriptList => {
            let list: Vec<HSTRING> = phrases.iter().map(HSTRING::from).collect();
            let it: IIterable<HSTRING> = IIterable::from(list);
            SpeechRecognitionListConstraint::Create(&it)?.cast()?
        }
    };
    let cs = r.Constraints()?;
    cs.Clear()?;
    cs.Append(&c)
}

/// Compiles the constraints; `Ok(())` only for status `Success`.
fn compile(r: &SpeechRecognizer) -> Result<(), SttError> {
    let res = r
        .CompileConstraintsAsync()
        .and_then(|op| op.get())
        .map_err(|e| map_err("CompileConstraints", &e))?;
    let status = res.Status().map_err(|e| map_err("CompileConstraints", &e))?;
    if status == SpeechRecognitionResultStatus::Success {
        return Ok(());
    }
    let hint = match status {
        SpeechRecognitionResultStatus::TopicLanguageNotSupported
        | SpeechRecognitionResultStatus::GrammarLanguageMismatch => HINT_LANGUAGE,
        SpeechRecognitionResultStatus::NetworkFailure => {
            "Keine Verbindung zum Windows-Diktierdienst (Internet nötig)."
        }
        _ => "",
    };
    Err(SttError::Engine(format!(
        "Windows-Spracherkennung: Constraints nicht kompilierbar (Status {status:?}). {hint}"
    )))
}

/// Initialization check without starting the microphone (CI smoke /
/// diagnostics): lists languages, creates recognizers, compiles constraints.
pub fn probe(language: &str) -> ProbeReport {
    let topic_languages = tags(SpeechRecognizer::SupportedTopicLanguages());
    let grammar_languages = tags(SpeechRecognizer::SupportedGrammarLanguages());
    let system_language = SpeechRecognizer::SystemSpeechLanguage()
        .and_then(|l| l.LanguageTag())
        .ok()
        .map(|t| t.to_string());
    let mut all = topic_languages.clone();
    all.extend(grammar_languages.iter().cloned());
    let chosen_language = match_language(language, &all);
    let mut compile_res = Vec::new();
    if let Some(lang) = &chosen_language {
        for mode in [WinSpeechMode::Dictation, WinSpeechMode::ScriptList] {
            let r = (|| {
                let r = make_recognizer(lang)?;
                let phrases = vec!["Guten Morgen".to_string(), "Hallo Welt".to_string()];
                add_constraint(&r, mode, &phrases).map_err(|e| map_err("Constraints", &e))?;
                compile(&r)?;
                let _ = r.Close();
                Ok::<_, SttError>("Success".to_string())
            })();
            compile_res.push((mode, r.map_err(|e| e.to_string())));
        }
    }
    ProbeReport {
        topic_languages,
        grammar_languages,
        system_language,
        chosen_language,
        compile: compile_res,
    }
}

/// Everything living on the WinRT thread.
struct Session {
    recognizer: SpeechRecognizer,
    session: SpeechContinuousRecognitionSession,
}

impl Session {
    fn stop(self) {
        if let Ok(op) = self.session.StopAsync() {
            let _ = op.get();
        }
        let _ = self.recognizer.Close();
    }
}

fn start_session(
    lang: &str,
    mode: WinSpeechMode,
    phrases: &[String],
    cfg: &WinSpeechConfig,
    tx: &Sender<SttEvent>,
    completed: &Completed,
) -> Result<Session, SttError> {
    let recognizer = make_recognizer(lang)?;
    add_constraint(&recognizer, mode, phrases).map_err(|e| map_err("Constraints", &e))?;
    compile(&recognizer)?;
    if let Ok(t) = recognizer.Timeouts() {
        let _ = t.SetEndSilenceTimeout(timespan(cfg.end_silence));
    }
    let session = recognizer.ContinuousRecognitionSession().map_err(|e| map_err("Session", &e))?;
    // Never auto-stop on long silence (teleprompter pauses).
    let _ = session.SetAutoStopSilenceTimeout(timespan(Duration::from_secs(24 * 3600)));

    let txh = tx.clone();
    recognizer
        .HypothesisGenerated(&TypedEventHandler::<
            SpeechRecognizer,
            SpeechRecognitionHypothesisGeneratedEventArgs,
        >::new(move |_, args| {
            if let Ok(a) = args.ok() {
                let text = a.Hypothesis()?.Text()?.to_string();
                if !text.is_empty() {
                    let _ = txh.send(SttEvent::Partial(text));
                }
            }
            Ok(())
        }))
        .map_err(|e| map_err("HypothesisGenerated", &e))?;
    let txs = tx.clone();
    recognizer
        .StateChanged(
            &TypedEventHandler::<SpeechRecognizer, SpeechRecognizerStateChangedEventArgs>::new(
                move |_, args| {
                    if let Ok(a) = args.ok() {
                        match a.State()? {
                            SpeechRecognizerState::SpeechDetected => {
                                let _ = txs.send(SttEvent::Vad(true));
                            }
                            SpeechRecognizerState::SoundEnded => {
                                let _ = txs.send(SttEvent::Vad(false));
                            }
                            _ => {}
                        }
                    }
                    Ok(())
                },
            ),
        )
        .map_err(|e| map_err("StateChanged", &e))?;
    let txr = tx.clone();
    session
        .ResultGenerated(&TypedEventHandler::<
            SpeechContinuousRecognitionSession,
            SpeechContinuousRecognitionResultGeneratedEventArgs,
        >::new(move |_, args| {
            if let Ok(a) = args.ok() {
                let r = a.Result()?;
                let conf = r.Confidence()?;
                let text = r.Text()?.to_string();
                if conf != SpeechRecognitionConfidence::Rejected && !text.is_empty() {
                    let _ = txr.send(SttEvent::Final(text));
                }
            }
            Ok(())
        }))
        .map_err(|e| map_err("ResultGenerated", &e))?;
    // Session ended on its own (mic lost, network, timeout): the worker
    // restarts it; the status is kept for the error message.
    let done = completed.clone();
    session
        .Completed(&TypedEventHandler::<
            SpeechContinuousRecognitionSession,
            SpeechContinuousRecognitionCompletedEventArgs,
        >::new(move |_, args| {
            if let Ok(a) = args.ok() {
                let st = a.Status()?;
                if let Ok(mut g) = done.lock() {
                    *g = Some(format!("{st:?}"));
                }
            }
            Ok(())
        }))
        .map_err(|e| map_err("Completed", &e))?;
    session.StartAsync().and_then(|op| op.get()).map_err(|e| map_err("StartAsync", &e))?;
    let _ = tx.send(SttEvent::Vad(false));
    Ok(Session { recognizer, session })
}

impl WinSpeechEngine {
    /// Time from construction to a running session (or list mode ready).
    pub fn init_ms(&self) -> u64 {
        self.init_ms
    }

    /// Creates the recognizer, compiles the constraints and starts continuous
    /// recognition (microphone). Errors carry a precise operator hint.
    pub fn new(cfg: WinSpeechConfig) -> Result<Self, SttError> {
        let (cmd_tx, cmd_rx) = channel::<Cmd>();
        let (ev_tx, ev_rx) = channel::<SttEvent>();
        let (init_tx, init_rx) = channel::<Result<String, SttError>>();
        let thread = std::thread::Builder::new()
            .name("zt-winspeech".into())
            .spawn(move || worker(cfg, cmd_rx, ev_tx, init_tx))
            .map_err(|e| SttError::Engine(format!("Thread: {e}")))?;
        let t0 = Instant::now();
        match init_rx.recv_timeout(Duration::from_secs(20)) {
            Ok(Ok(name)) => Ok(Self {
                name,
                cmd: cmd_tx,
                events: ev_rx,
                thread: Some(thread),
                init_ms: t0.elapsed().as_millis() as u64,
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = cmd_tx.send(Cmd::Stop);
                Err(SttError::Engine(
                    "Windows-Spracherkennung: Initialisierung antwortet nicht (Timeout 20 s)"
                        .into(),
                ))
            }
        }
    }
}

fn worker(
    cfg: WinSpeechConfig,
    cmd_rx: Receiver<Cmd>,
    ev_tx: Sender<SttEvent>,
    init_tx: Sender<Result<String, SttError>>,
) {
    let topic = tags(SpeechRecognizer::SupportedTopicLanguages());
    let grammar = tags(SpeechRecognizer::SupportedGrammarLanguages());
    let mut mode = cfg.mode;
    let installed = if mode == WinSpeechMode::Dictation { &topic } else { &grammar };
    let Some(lang) = match_language(&cfg.language, installed) else {
        let _ = init_tx.send(Err(SttError::Model(format!(
            "Windows-Sprachpaket für „{}“ fehlt (installiert: {}). {HINT_LANGUAGE}",
            cfg.language,
            if installed.is_empty() { "keine".to_string() } else { installed.join(", ") }
        ))));
        return;
    };
    let completed: Completed = Arc::new(Mutex::new(None));
    let mut phrases: Vec<String> = Vec::new();
    let mut session = None;
    let mut fallback_note = String::new();
    if mode == WinSpeechMode::Dictation {
        match start_session(&lang, mode, &phrases, &cfg, &ev_tx, &completed) {
            Ok(s) => session = Some(s),
            Err(e) if cfg.list_fallback && match_language(&lang, &grammar).is_some() => {
                // Dictation blocked (privacy setting / no network): offline
                // list constraint, started once the script words arrive.
                fallback_note = format!(" [Skriptwort-Modus, Diktat nicht verfügbar: {e}]");
                mode = WinSpeechMode::ScriptList;
            }
            Err(e) => {
                let _ = init_tx.send(Err(e));
                return;
            }
        }
    }
    let label = match mode {
        WinSpeechMode::Dictation => "Diktat",
        WinSpeechMode::ScriptList => "Skriptwörter, offline",
    };
    let _ = init_tx.send(Ok(format!("{WINSPEECH_NAME} {lang} ({label}){fallback_note}")));
    loop {
        match cmd_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Cmd::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(Cmd::Phrases(p)) => {
                if mode == WinSpeechMode::ScriptList && !p.is_empty() && p != phrases {
                    phrases = p;
                    if let Some(s) = session.take() {
                        s.stop();
                    }
                    session = start_session(&lang, mode, &phrases, &cfg, &ev_tx, &completed).ok();
                }
            }
            Ok(Cmd::Reset) => {
                // Dropping the running utterance: restart the session.
                if let Some(s) = session.take() {
                    s.stop();
                    session = start_session(&lang, mode, &phrases, &cfg, &ev_tx, &completed).ok();
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // Session ended by Windows (mic lost, network, timeout):
                // restart (at most every 500 ms).
                let ended = completed.lock().ok().and_then(|mut g| g.take());
                if ended.is_some() && session.is_some() {
                    if let Some(s) = session.take() {
                        s.stop();
                    }
                    session = start_session(&lang, mode, &phrases, &cfg, &ev_tx, &completed).ok();
                }
            }
        }
    }
    if let Some(s) = session.take() {
        s.stop();
    }
}

impl Drop for WinSpeechEngine {
    fn drop(&mut self) {
        let _ = self.cmd.send(Cmd::Stop);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl SpeechEngine for WinSpeechEngine {
    fn name(&self) -> &str {
        &self.name
    }
    fn accept_waveform(&mut self, _samples: &[f32]) {}
    fn poll(&mut self) -> Vec<SttEvent> {
        self.events.try_iter().collect()
    }
    fn set_hotwords(&mut self, hotwords: &[Hotword]) -> Result<(), SttError> {
        let mut p: Vec<String> = hotwords.iter().map(|h| h.text.to_lowercase()).collect();
        p.sort();
        p.dedup();
        p.truncate(MAX_LIST_PHRASES);
        self.cmd.send(Cmd::Phrases(p)).map_err(|_| SttError::Engine("thread gone".into()))
    }
    fn reset(&mut self) {
        let _ = self.cmd.send(Cmd::Reset);
    }
    fn self_capturing(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_matching() {
        let inst = vec!["de-AT".to_string(), "de-DE".to_string(), "en-GB".to_string()];
        assert_eq!(match_language("de", &inst).as_deref(), Some("de-DE"));
        assert_eq!(match_language("de-AT", &inst).as_deref(), Some("de-AT"));
        assert_eq!(match_language("en", &inst).as_deref(), Some("en-GB"));
        assert_eq!(match_language("fr", &inst), None);
    }

    #[test]
    fn privacy_error_has_hint() {
        let e = windows::core::Error::from_hresult(windows::core::HRESULT(HR_PRIVACY));
        assert!(map_err("x", &e).to_string().contains("privacy-speech"));
    }
}
