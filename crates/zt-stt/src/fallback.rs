//! Fallback chain for engine construction (ADR-006): the first constructor
//! that succeeds wins; failures are collected for the operator UI.

use crate::SttError;

/// Result of [`build_with_fallback`].
pub struct Fallback<T> {
    pub value: T,
    /// Id of the candidate that succeeded.
    pub chosen: String,
    /// `(id, error)` of candidates tried before it.
    pub skipped: Vec<(String, String)>,
}

// Manual impl: engines (`Box<dyn SpeechEngine>`) are not `Debug`.
impl<T> std::fmt::Debug for Fallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fallback")
            .field("chosen", &self.chosen)
            .field("skipped", &self.skipped)
            .finish_non_exhaustive()
    }
}

impl<T> Fallback<T> {
    /// True if the preferred (first) candidate failed.
    pub fn fell_back(&self) -> bool {
        !self.skipped.is_empty()
    }
}

/// Tries `candidates` in order. `Err(skipped)` if none succeeded (the caller
/// then uses demo/mock mode and shows the collected errors).
pub fn build_with_fallback<T, I, F>(candidates: I) -> Result<Fallback<T>, Vec<(String, String)>>
where
    I: IntoIterator<Item = (String, F)>,
    F: FnOnce() -> Result<T, SttError>,
{
    let mut skipped = Vec::new();
    for (id, make) in candidates {
        match make() {
            Ok(value) => return Ok(Fallback { value, chosen: id, skipped }),
            Err(e) => skipped.push((id, e.to_string())),
        }
    }
    Err(skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockEngine, SpeechEngine};

    type Make = Box<dyn FnOnce() -> Result<Box<dyn SpeechEngine>, SttError>>;

    fn ok() -> Make {
        Box::new(|| Ok(Box::new(MockEngine::default()) as Box<dyn SpeechEngine>))
    }
    fn broken() -> Make {
        Box::new(|| Err(SttError::Model("missing encoder.onnx".into())))
    }

    #[test]
    fn first_working_candidate_wins() {
        let r = build_with_fallback(vec![
            ("nemotron-3.5-streaming-0.6b-320ms-int8".to_string(), broken()),
            ("zipformer-en-20m-2023-02-17".to_string(), ok()),
        ])
        .unwrap();
        assert_eq!(r.chosen, "zipformer-en-20m-2023-02-17");
        assert!(r.fell_back());
        assert_eq!(r.skipped[0].0, "nemotron-3.5-streaming-0.6b-320ms-int8");
        assert!(r.skipped[0].1.contains("encoder"));
        assert_eq!(r.value.name(), "mock");
    }

    #[test]
    fn preferred_candidate_used_without_trying_others() {
        let r = build_with_fallback(vec![("a".to_string(), ok()), ("b".to_string(), broken())])
            .unwrap();
        assert_eq!(r.chosen, "a");
        assert!(!r.fell_back());
    }

    #[test]
    fn all_failing_returns_every_error() {
        let e = build_with_fallback(vec![("a".to_string(), broken()), ("b".to_string(), broken())])
            .unwrap_err();
        assert_eq!(e.len(), 2);
        let none: Vec<(String, Make)> = vec![];
        assert!(build_with_fallback(none).unwrap_err().is_empty());
    }
}
