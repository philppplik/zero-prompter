//! Windows speech recognition as system fallback engine (ADR-008).
//!
//! Chain: installed sherpa models (Nemotron / Zipformer EN 20M) ->
//! Windows speech recognition -> error status (or demo mode when no model is
//! installed and Windows speech is unavailable). The user can also pick it
//! directly as model "Windows Spracherkennung (System)" (id
//! [`zt_stt::WINSPEECH_ID`], nothing to download).
//!
//! Only in Windows builds with feature `winspeech`; elsewhere `AVAILABLE` is
//! false and `build` returns an error.

use zt_stt::SpeechEngine;

/// True if this build can use Windows speech recognition.
pub const AVAILABLE: bool = cfg!(all(windows, feature = "winspeech"));

/// The user picked Windows speech for the script language.
pub fn chosen(prefs: &crate::models::VoiceModelPrefs, lang: &str) -> bool {
    prefs
        .streaming
        .get(&zt_models::primary_language(lang))
        .is_some_and(|id| id == zt_stt::WINSPEECH_ID)
}

/// Creates the engine (starts Windows' own microphone capture).
#[cfg(all(windows, feature = "winspeech"))]
pub fn build(lang: &str) -> Result<Box<dyn SpeechEngine>, String> {
    use zt_stt::{WinSpeechConfig, WinSpeechEngine};
    let t0 = std::time::Instant::now();
    tracing::info!(target: "zt::engine", model = zt_stt::WINSPEECH_ID, language = %lang,
        "creating recognizer");
    match WinSpeechEngine::new(WinSpeechConfig::new(lang)) {
        Ok(e) => {
            tracing::info!(target: "zt::engine", model = zt_stt::WINSPEECH_ID, name = e.name(),
                init_ms = e.init_ms(), ms = t0.elapsed().as_millis() as u64,
                self_capturing = true, "recognizer ready");
            Ok(Box::new(e))
        }
        Err(err) => {
            tracing::error!(target: "zt::engine", model = zt_stt::WINSPEECH_ID, error = %err,
                ms = t0.elapsed().as_millis() as u64, "recognizer failed");
            Err(err.to_string())
        }
    }
}

#[cfg(not(all(windows, feature = "winspeech")))]
pub fn build(_lang: &str) -> Result<Box<dyn SpeechEngine>, String> {
    Err("Windows-Spracherkennung ist in diesem Build nicht enthalten (nur Windows, Feature `winspeech`)."
        .into())
}
