//! Reading-marker customization (DESIGN §3.7). Pure data + clamping; the
//! renderer lives in the frontend (`ReadingMarker.svelte`, `lib/marker.ts`),
//! whose bounds must match the constants here.

use serde::{Deserialize, Serialize};

/// Visual style of the reading marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MarkerStyle {
    #[default]
    Line,
    Band,
    Arrows,
    Brackets,
    Spotlight,
    Gradient,
    Underline,
    Sidebar,
    None,
}

/// What the `Underline` style underlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum UnderlineScope {
    #[default]
    Word,
    Sentence,
}

/// Motion when the marker (or the underline) moves to a new line/word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MarkerAnimation {
    None,
    #[default]
    Glide,
    Pulse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MarkerSettings {
    pub style: MarkerStyle,
    /// `#rrggbb`; `None` = theme marker colour. Single source of the marker
    /// colour (Settings > Lesemarker; the old UI pref `markerColor` is migrated here).
    pub color: Option<String>,
    /// Marker opacity (spotlight/gradient: strength of the darkening).
    pub opacity: f32,
    /// Thickness / height multiplier (line 2 px, band one line, arrows 12 px … × size).
    pub size: f32,
    /// Legacy: dimming of already-read text. Read dimming now lives only in the
    /// UI prefs (`dimRead` / `readOpacity`, Settings > Darstellung); the frontend
    /// migrates a stored value there once and writes `None` back. Kept so old
    /// settings files still deserialize; not serialized when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_read: Option<f32>,
    /// Dimming of not-yet-read text below the reading zone (0 = none). Edited
    /// under Settings > Darstellung (text dimming), rendered by the marker layer.
    pub dim_unread: f32,
    /// Glow around the marker (0 = off).
    pub glow: f32,
    /// Soft-focus blur outside the reading zone in px (spotlight/gradient, 0 = off).
    pub blur: f32,
    pub animation: MarkerAnimation,
    pub underline_scope: UnderlineScope,
}

impl MarkerSettings {
    pub const OPACITY_MIN: f32 = 0.1;
    pub const OPACITY_MAX: f32 = 1.0;
    pub const SIZE_MIN: f32 = 0.5;
    pub const SIZE_MAX: f32 = 3.0;
    pub const DIM_MAX: f32 = 0.9;
    pub const GLOW_MAX: f32 = 1.0;
    pub const BLUR_MAX: f32 = 8.0;

    /// Clamp into valid ranges; non-finite numbers fall back to defaults.
    pub fn sanitized(self) -> Self {
        let d = Self::default();
        let dim = self.dim_read.filter(|v| v.is_finite());
        Self {
            style: self.style,
            color: self.color.and_then(|c| normalize_hex(&c)),
            opacity: fix(self.opacity, Self::OPACITY_MIN, Self::OPACITY_MAX, d.opacity),
            size: fix(self.size, Self::SIZE_MIN, Self::SIZE_MAX, d.size),
            dim_read: dim.map(|v| v.clamp(0.0, Self::DIM_MAX)),
            dim_unread: fix(self.dim_unread, 0.0, Self::DIM_MAX, d.dim_unread),
            glow: fix(self.glow, 0.0, Self::GLOW_MAX, d.glow),
            blur: fix(self.blur, 0.0, Self::BLUR_MAX, d.blur),
            animation: self.animation,
            underline_scope: self.underline_scope,
        }
    }
}

impl Default for MarkerSettings {
    fn default() -> Self {
        Self {
            style: MarkerStyle::Line,
            color: None,
            opacity: 1.0,
            size: 1.0,
            dim_read: None,
            dim_unread: 0.0,
            glow: 0.0,
            blur: 0.0,
            animation: MarkerAnimation::Glide,
            underline_scope: UnderlineScope::Word,
        }
    }
}

fn fix(v: f32, lo: f32, hi: f32, def: f32) -> f32 {
    if !v.is_finite() {
        return def;
    }
    v.clamp(lo, hi)
}

/// `#RGB` / `#RRGGBB` (case-insensitive) -> lowercase `#rrggbb`; otherwise `None`.
pub fn normalize_hex(s: &str) -> Option<String> {
    let h = s.trim().strip_prefix('#')?;
    if !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let full = match h.len() {
        3 => h.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => h.to_string(),
        _ => return None,
    };
    Some(format!("#{}", full.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let d = MarkerSettings::default();
        assert_eq!(d.clone().sanitized(), d);
        assert_eq!(d.style, MarkerStyle::Line);
    }

    #[test]
    fn clamps_and_repairs() {
        let s = MarkerSettings {
            opacity: 0.0,
            size: 99.0,
            dim_read: Some(f32::NAN),
            dim_unread: 2.0,
            glow: -1.0,
            blur: f32::INFINITY,
            color: Some("red".into()),
            ..Default::default()
        }
        .sanitized();
        assert_eq!(s.opacity, MarkerSettings::OPACITY_MIN);
        assert_eq!(s.size, MarkerSettings::SIZE_MAX);
        assert_eq!(s.dim_read, None);
        assert_eq!(s.dim_unread, MarkerSettings::DIM_MAX);
        assert_eq!(s.glow, 0.0);
        assert_eq!(s.blur, 0.0);
        assert_eq!(s.color, None);
    }

    #[test]
    fn legacy_dim_read_still_deserializes() {
        let s: MarkerSettings = serde_json::from_str(r#"{"dimRead":0.6}"#).unwrap();
        assert_eq!(s.clone().sanitized().dim_read, Some(0.6));
        let j = serde_json::to_value(s).unwrap();
        assert_eq!(j["dimRead"].as_f64().map(|v| (v * 10.0).round()), Some(6.0));
    }

    #[test]
    fn hex_normalization() {
        assert_eq!(normalize_hex("#ABC").as_deref(), Some("#aabbcc"));
        assert_eq!(normalize_hex(" #FF8800 ").as_deref(), Some("#ff8800"));
        assert_eq!(normalize_hex("#12345"), None);
        assert_eq!(normalize_hex("123456"), None);
        assert_eq!(normalize_hex("#gg0000"), None);
    }

    #[test]
    fn serde_camel_case_and_partial() {
        let json = r#"{"style":"spotlight","dimUnread":0.4,"underlineScope":"sentence"}"#;
        let s: MarkerSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.style, MarkerStyle::Spotlight);
        assert_eq!(s.dim_unread, 0.4);
        assert_eq!(s.underline_scope, UnderlineScope::Sentence);
        assert_eq!(s.opacity, 1.0);
        let j = serde_json::to_value(MarkerSettings::default()).unwrap();
        assert_eq!(j["animation"], "glide");
        assert!(j.get("dimRead").is_none(), "legacy field is not written when unset");
        let all = "line band arrows brackets spotlight gradient underline sidebar none";
        for st in all.split(' ') {
            let v: MarkerStyle = serde_json::from_value(serde_json::json!(st)).unwrap();
            assert_eq!(serde_json::to_value(v).unwrap(), st);
        }
    }
}
