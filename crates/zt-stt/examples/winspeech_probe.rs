//! Initialization smoke test of the Windows speech engine (CI: `winspeech`).
//!
//! cargo run -p zt-stt --features winspeech --example winspeech_probe -- [de|en-US|...]
//!
//! Lists the installed speech languages, creates a `SpeechRecognizer` per
//! constraint mode and runs `CompileConstraintsAsync` (no microphone, no
//! session start). Exit code 1 only if a language with grammar support is
//! installed but the offline list constraint fails to compile; missing
//! language packs (typical on CI runners) and blocked dictation (privacy
//! setting "Online-Spracherkennung" off) are reported and skipped.

#[cfg(windows)]
fn main() {
    use zt_stt::{winspeech_probe, WinSpeechMode};
    let wanted = std::env::args().nth(1).unwrap_or_else(|| "de".into());
    let t0 = std::time::Instant::now();
    let r = winspeech_probe(&wanted);
    println!("topic (dictation) languages: {:?}", r.topic_languages);
    println!("grammar (list) languages:    {:?}", r.grammar_languages);
    println!("system speech language:      {:?}", r.system_language);
    let mut lang = r.chosen_language.clone();
    let mut report = r;
    if lang.is_none() {
        // Fall back to the system speech language to still exercise the API.
        if let Some(sys) = report.system_language.clone() {
            println!("SKIP {wanted}: language pack not installed; probing system language {sys}");
            report = winspeech_probe(&sys);
            lang = report.chosen_language.clone();
        }
    }
    let Some(lang) = lang else {
        println!("SKIP: no speech recognition language installed on this machine");
        return;
    };
    println!("chosen language: {lang}");
    let mut fail = false;
    for (mode, res) in &report.compile {
        match res {
            Ok(s) => println!("compile {mode:?}: {s}"),
            Err(e) => {
                println!("compile {mode:?}: FAILED {e}");
                let grammar_ok = report.grammar_languages.iter().any(|g| g == &lang);
                if *mode == WinSpeechMode::ScriptList && grammar_ok {
                    fail = true;
                }
            }
        }
    }
    println!("probe ms: {}", t0.elapsed().as_millis());
    if fail {
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    println!("SKIP: Windows speech recognition exists only on Windows");
}
