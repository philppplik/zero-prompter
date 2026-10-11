//! Settings and themes derived from `docs/design/tokens.json`.
//!
//! The design tokens are the single source of truth; they are embedded at
//! compile time so the core never touches the filesystem.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Raw embedded design tokens (same file the frontend consumes).
pub const TOKENS_JSON: &str = include_str!("../../../docs/design/tokens.json");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrompterTheme {
    pub bg: String,
    pub text: String,
    pub read_opacity: f32,
    pub marker: String,
    pub word_highlight: String,
    pub direction: String,
    pub contrast: f32,
    pub wcag: String,
    #[serde(default)]
    pub letter_spacing: Option<String>,
    #[serde(default)]
    pub word_spacing: Option<String>,
    #[serde(default)]
    pub line_height: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateStyle {
    pub color: String,
    pub shape: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrompterTypeScale {
    pub size_min: u16,
    pub size_default: u16,
    pub size_max: u16,
    pub line_height: f32,
    pub measure_ch: u16,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeTokens {
    pub prompter: PrompterTypeScale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutTokens {
    pub focus_line_default: f32,
}

/// Subset of tokens.json the core needs. Unknown keys are ignored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tokens {
    pub state: BTreeMap<String, StateStyle>,
    pub prompter: BTreeMap<String, PrompterTheme>,
    #[serde(rename = "type")]
    pub typography: TypeTokens,
    pub layout: LayoutTokens,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid design tokens: {0}")]
pub struct TokensError(#[from] serde_json::Error);

impl Tokens {
    pub fn parse(json: &str) -> Result<Self, TokensError> {
        // Tolerate a UTF-8 BOM written by some Windows editors.
        Ok(serde_json::from_str(json.trim_start_matches('\u{feff}'))?)
    }

    /// The embedded tokens. Panics only if the repo file is broken, which a unit test guards.
    pub fn embedded() -> Self {
        Self::parse(TOKENS_JSON).expect("docs/design/tokens.json must be valid")
    }
}

/// Theme registry built from tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeSet {
    pub themes: BTreeMap<String, PrompterTheme>,
}

impl ThemeSet {
    pub const DEFAULT: &'static str = "studioDark";

    pub fn from_tokens(t: &Tokens) -> Self {
        Self { themes: t.prompter.clone() }
    }

    pub fn get(&self, id: &str) -> Option<&PrompterTheme> {
        self.themes.get(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.themes.keys().map(String::as_str)
    }
}

/// What to do when voice tracking is `lost` (CONCEPT.md §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum LostPolicy {
    #[default]
    Stop,
    ConstantSpeed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub theme: String,
    pub font_size_px: u16,
    pub line_height: f32,
    /// Focus/reading line as a fraction of viewport height.
    pub focus_line: f32,
    pub wpm: u16,
    pub countdown_secs: u8,
    pub lost_policy: LostPolicy,
    pub lost_timeout_ms: u32,
    pub manual_override_ms: u32,
    pub language: String,
    pub reduced_motion: bool,
    /// Action id -> key chord, e.g. "togglePlay" -> "Space".
    pub hotkeys: BTreeMap<String, String>,
    /// Microphone gain: automatic (AGC towards about -20 dBFS) when true,
    /// otherwise the fixed [Self::mic_gain_db].
    pub mic_gain_auto: bool,
    /// Manual microphone gain in dB ([Self::MIC_GAIN_MIN_DB]..=[Self::MIC_GAIN_MAX_DB]).
    pub mic_gain_db: f32,
    /// Multi-channel input -> mono: "auto" (loudest channel), "mix" (average)
    /// or a 1-based channel number ("1", "2", ...).
    pub mic_channel: String,
}

impl Settings {
    pub const WPM_MIN: u16 = 40;
    pub const WPM_MAX: u16 = 400;
    /// Focus line range as viewport fraction (DESIGN §3.7: 5–70 %). The
    /// frontend (Inspector slider, marker drag) uses the same bounds.
    pub const FOCUS_LINE_MIN: f32 = 0.05;
    pub const FOCUS_LINE_MAX: f32 = 0.70;
    /// Manual microphone gain range (dB); matches zt_audio::MANUAL_GAIN_RANGE_DB.
    pub const MIC_GAIN_MIN_DB: f32 = -20.0;
    pub const MIC_GAIN_MAX_DB: f32 = 40.0;

    pub fn from_tokens(t: &Tokens) -> Self {
        let p = t.typography.prompter;
        Self {
            font_size_px: p.size_default,
            line_height: p.line_height,
            focus_line: t.layout.focus_line_default,
            ..Self::base()
        }
    }

    fn base() -> Self {
        let hotkeys = [
            ("togglePlay", "Space"),
            ("stop", "Home"),
            ("faster", "ArrowUp"),
            ("slower", "ArrowDown"),
            ("nextParagraph", "PageDown"),
            ("prevParagraph", "PageUp"),
            ("toggleMirror", "M"),
            ("toggleFullscreen", "F"),
            ("toggleVoice", "V"),
            ("resyncHere", "R"),
            ("commandPalette", "Ctrl+K"),
            ("shortcutOverlay", "?"),
        ]
        .into_iter()
        .map(|(a, k)| (a.to_string(), k.to_string()))
        .collect();
        Self {
            theme: ThemeSet::DEFAULT.to_string(),
            font_size_px: 72,
            line_height: 1.4,
            focus_line: 0.35,
            wpm: 140,
            countdown_secs: 3,
            lost_policy: LostPolicy::Stop,
            lost_timeout_ms: 4000,
            manual_override_ms: 2000,
            language: "de".to_string(),
            reduced_motion: false,
            hotkeys,
            mic_gain_auto: true,
            mic_gain_db: 0.0,
            mic_channel: "auto".to_string(),
        }
    }

    /// Clamp all values into valid ranges (after loading user files).
    pub fn sanitized(mut self, t: &Tokens) -> Self {
        let p = t.typography.prompter;
        self.font_size_px = self.font_size_px.clamp(p.size_min, p.size_max);
        self.wpm = self.wpm.clamp(Self::WPM_MIN, Self::WPM_MAX);
        self.focus_line = if self.focus_line.is_finite() {
            self.focus_line.clamp(Self::FOCUS_LINE_MIN, Self::FOCUS_LINE_MAX)
        } else {
            t.layout.focus_line_default.clamp(Self::FOCUS_LINE_MIN, Self::FOCUS_LINE_MAX)
        };
        self.line_height = self.line_height.clamp(1.0, 3.0);
        self.mic_gain_db = if self.mic_gain_db.is_finite() {
            self.mic_gain_db.clamp(Self::MIC_GAIN_MIN_DB, Self::MIC_GAIN_MAX_DB)
        } else {
            0.0
        };
        let ch = self.mic_channel.trim().to_ascii_lowercase();
        let valid = ch == "auto" || ch == "mix" || ch.parse::<u8>().is_ok_and(|n| n >= 1);
        self.mic_channel = if valid { ch } else { "auto".to_string() };
        if !t.prompter.contains_key(&self.theme) {
            self.theme = ThemeSet::DEFAULT.to_string();
        }
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::base()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_tokens_parse() {
        let t = Tokens::embedded();
        assert!(t.prompter.contains_key("studioDark"));
        assert_eq!(t.state.len(), 6);
        for k in ["idle", "listening", "tracking", "hold", "lost", "resync"] {
            assert!(t.state.contains_key(k), "missing state {k}");
        }
        let s = Settings::from_tokens(&t);
        assert_eq!(s.font_size_px, t.typography.prompter.size_default);
    }

    #[test]
    fn sanitize_clamps() {
        let t = Tokens::embedded();
        let s = Settings { font_size_px: 5, wpm: 9999, theme: "nope".into(), ..Default::default() }
            .sanitized(&t);
        assert_eq!(s.font_size_px, t.typography.prompter.size_min);
        assert_eq!(s.wpm, Settings::WPM_MAX);
        assert_eq!(s.theme, ThemeSet::DEFAULT);
    }

    #[test]
    fn focus_line_range_matches_design() {
        let t = Tokens::embedded();
        let at = |f: f32| Settings { focus_line: f, ..Default::default() }.sanitized(&t).focus_line;
        assert_eq!(at(0.0), Settings::FOCUS_LINE_MIN);
        assert_eq!(at(0.05), 0.05);
        assert_eq!(at(0.7), 0.7);
        assert_eq!(at(0.9), Settings::FOCUS_LINE_MAX);
        assert_eq!(at(0.35), 0.35);
        assert!((Settings::FOCUS_LINE_MIN..=Settings::FOCUS_LINE_MAX).contains(&at(f32::NAN)));
        let range = Settings::FOCUS_LINE_MIN..=Settings::FOCUS_LINE_MAX;
        assert!(range.contains(&t.layout.focus_line_default));
    }

    #[test]
    fn mic_settings_sanitized() {
        let t = Tokens::embedded();
        let d = Settings::default();
        assert!(d.mic_gain_auto);
        assert_eq!(d.mic_channel, "auto");
        let s = Settings { mic_gain_db: 99.0, mic_channel: "0".into(), ..Default::default() }
            .sanitized(&t);
        assert_eq!(s.mic_gain_db, Settings::MIC_GAIN_MAX_DB);
        assert_eq!(s.mic_channel, "auto");
        let s =
            Settings { mic_gain_db: f32::NAN, mic_channel: " MIX ".into(), ..Default::default() }
                .sanitized(&t);
        assert_eq!(s.mic_gain_db, 0.0);
        assert_eq!(s.mic_channel, "mix");
        let s = Settings { mic_channel: "2".into(), ..Default::default() }.sanitized(&t);
        assert_eq!(s.mic_channel, "2");
    }

    #[test]
    fn dyslexia_has_spacing() {
        let set = ThemeSet::from_tokens(&Tokens::embedded());
        assert!(set.get("dyslexia").unwrap().letter_spacing.is_some());
    }
}
