//! Voice state machine (CONCEPT.md §6.3) with jump hysteresis.
//!
//! | From              | Event                          | To        |
//! |-------------------|--------------------------------|-----------|
//! | idle              | Start                          | listening |
//! | listening         | match (conf >= θ)              | tracking  |
//! | tracking          | match                          | tracking  |
//! | tracking          | VAD silence / conf < θ         | hold      |
//! | hold              | match near cursor              | tracking  |
//! | hold/tracking     | speech without match > timeout | lost      |
//! | tracking/hold/lost| jump > N words, 2x confirmed   | resync    |
//! | lost              | global n-gram match (2x)       | resync    |
//! | resync            | after ease (300 ms)            | tracking  |
//! | *                 | Stop                           | idle      |
//! | *                 | manual seek                    | hold (override 2 s) |
//!
//! Backtrack / forward-jump handling follows the approach of
//! EdNutting/autocue (MIT, https://github.com/EdNutting/autocue): small
//! backward steps are accepted immediately (speaker repeats), large jumps
//! need repeated confirmation. Re-implemented from the description, no code
//! copied. See THIRD_PARTY_NOTICES.md.
//!
//! Time is injected (`t_ms`, monotonic milliseconds); the tracker never reads a clock.

use crate::index::ScriptIndex;
use crate::normalize::normalize_hypothesis;
use crate::sw::{align, QueryWord, SwHit, SwParams};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VoiceState {
    #[default]
    Idle,
    Listening,
    Tracking,
    Hold,
    Lost,
    Resync,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackerConfig {
    /// Window behind / ahead of the cursor in normalised words (−10 / +60).
    pub window_before: usize,
    pub window_after: usize,
    /// Number of recent hypothesis words matched (k).
    pub k: usize,
    /// Minimum confidence for a local match (θ).
    pub theta: f32,
    /// Minimum confidence for a global (lost) match.
    pub theta_global: f32,
    pub min_matches: u16,
    pub min_matches_global: u16,
    /// Forward distance (words) beyond which a match is a "jump" (≈ N lines).
    pub jump_words: usize,
    /// Backward distance accepted without confirmation (backtrack).
    pub backtrack_words: usize,
    /// Confirmations needed before a jump is executed.
    pub jump_confirmations: u8,
    /// Two candidates count as "the same jump" if within this many words.
    pub jump_tolerance: usize,
    /// Accumulated *speech* time without a match before `lost`.
    pub lost_timeout_ms: u64,
    pub resync_ease_ms: u64,
    pub manual_override_ms: u64,
    /// Weight of final vs. partial hypothesis words.
    pub final_weight: f32,
    pub partial_weight: f32,
    pub sw: SwParams,
    /// Resync (offline, high-confidence hypothesis; ADR-006): wider search window.
    pub resync: ResyncConfig,
}

/// Parameters for [`TrackerEvent::Resync`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResyncConfig {
    /// Window behind / ahead of the cursor (much wider than the streaming window).
    pub window_before: usize,
    pub window_after: usize,
    /// Last k words of the offline segment used as query.
    pub k: usize,
    pub theta: f32,
    pub min_matches: u16,
    /// Hits within this distance of the cursor are a confirmation, not a move.
    pub deadband_words: usize,
    /// The offline result describes speech that already happened. A hit up to
    /// this many words *behind* the cursor means streaming is simply ahead
    /// (consistent) and is ignored.
    pub lag_words: usize,
    pub sw: SwParams,
}

impl Default for ResyncConfig {
    fn default() -> Self {
        Self {
            window_before: 80,
            window_after: 400,
            k: 16,
            theta: 0.6,
            min_matches: 4,
            deadband_words: 3,
            lag_words: 15,
            // The segment end may be clipped by the VAD: allow the alignment
            // to end up to 3 words before the last query word.
            sw: SwParams { max_trailing: 3, ..SwParams::default() },
        }
    }
}

/// What a [`TrackerEvent::Resync`] did (for logging / UI / tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResyncOutcome {
    /// Tracker idle, manual override active, or hypothesis empty.
    Ignored,
    /// No sufficiently confident match in the resync window.
    NoMatch,
    /// Match agrees with the streaming cursor (deadband or lag); nothing moved.
    Confirmed,
    /// Cursor re-anchored to this normalised position.
    Reanchored { from: Option<usize>, to: usize },
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            window_before: 10,
            window_after: 60,
            k: 6,
            theta: 0.55,
            theta_global: 0.7,
            min_matches: 2,
            min_matches_global: 4,
            jump_words: 18, // ≈ 3 lines at ~6 words/line
            backtrack_words: 10,
            jump_confirmations: 2,
            jump_tolerance: 4,
            lost_timeout_ms: 4000,
            resync_ease_ms: 300,
            manual_override_ms: 2000,
            final_weight: 1.0,
            partial_weight: 0.8,
            sw: SwParams::default(),
            resync: ResyncConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackerEvent {
    Start,
    Stop,
    /// STT hypothesis. Partials carry the full current utterance text and
    /// replace the previous partial; a final commits it.
    Hypothesis { text: String, is_final: bool },
    VadSpeech,
    VadSilence,
    /// Operator click on a word / "resync here" / paragraph jump.
    ManualSeek { token: usize },
    /// Time advance (call regularly, e.g. every 50 ms).
    Tick,
    /// High-confidence hypothesis of a finished speech segment from the
    /// offline (resync) engine. Searched in a wider window and applied
    /// immediately (no 2x jump confirmation), see [`VoiceTracker::resync`].
    /// `age_ms`: time between the end of the segment and this event (decode
    /// latency); widens the tolerated lag by `rate_wps * age`.
    Resync { text: String, age_ms: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackerOutput {
    pub state: VoiceState,
    /// Script token of the last confirmed spoken word.
    pub cursor_token: Option<usize>,
    pub confidence: f32,
    /// Smoothed speaking rate in normalised words per second.
    pub rate_wps: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PendingJump {
    pos: usize,
    count: u8,
}

#[derive(Debug, Clone)]
pub struct VoiceTracker {
    cfg: TrackerConfig,
    index: ScriptIndex,
    state: VoiceState,
    /// Normalised position of the last confirmed spoken word (None = before start).
    cursor: Option<usize>,
    confidence: f32,
    committed: VecDeque<String>,
    partial: Vec<String>,
    last_query: Vec<String>,
    pending: Option<PendingJump>,
    speech: bool,
    nomatch_speech_ms: u64,
    last_t: Option<u64>,
    resync_until: u64,
    override_until: u64,
    rate_wps: f32,
    last_match: Option<(usize, u64)>,
    last_resync: ResyncOutcome,
}

impl VoiceTracker {
    pub fn new(index: ScriptIndex, cfg: TrackerConfig) -> Self {
        Self {
            cfg,
            index,
            state: VoiceState::Idle,
            cursor: None,
            confidence: 0.0,
            committed: VecDeque::new(),
            partial: Vec::new(),
            last_query: Vec::new(),
            pending: None,
            speech: false,
            nomatch_speech_ms: 0,
            last_t: None,
            resync_until: 0,
            override_until: 0,
            rate_wps: 0.0,
            last_match: None,
            last_resync: ResyncOutcome::Ignored,
        }
    }

    /// Feeds an offline resync hypothesis; returns what it did.
    pub fn resync(&mut self, text: &str, age_ms: u64, t_ms: u64) -> ResyncOutcome {
        self.last_resync = ResyncOutcome::Ignored;
        self.handle(TrackerEvent::Resync { text: text.to_string(), age_ms }, t_ms);
        self.last_resync
    }

    /// Outcome of the most recent [`TrackerEvent::Resync`].
    pub fn last_resync(&self) -> ResyncOutcome {
        self.last_resync
    }

    pub fn state(&self) -> VoiceState {
        self.state
    }

    pub fn config(&self) -> &TrackerConfig {
        &self.cfg
    }

    /// Replace the configuration (e.g. timeouts changed in settings) without
    /// losing cursor, state or hypothesis buffer.
    pub fn set_config(&mut self, cfg: TrackerConfig) {
        self.cfg = cfg;
    }

    /// Replace the script (e.g. after an edit); keeps the state, resets matching.
    pub fn set_index(&mut self, index: ScriptIndex) {
        self.index = index;
        self.cursor = None;
        self.clear_buffer();
    }

    pub fn output(&self) -> TrackerOutput {
        TrackerOutput {
            state: self.state,
            cursor_token: self.cursor.and_then(|c| self.index.token_of(c)),
            confidence: self.confidence,
            rate_wps: self.rate_wps,
        }
    }

    fn clear_buffer(&mut self) {
        self.committed.clear();
        self.partial.clear();
        self.last_query.clear();
        self.pending = None;
    }

    /// Feeds one event at monotonic time `t_ms`; returns the new output.
    pub fn handle(&mut self, ev: TrackerEvent, t_ms: u64) -> TrackerOutput {
        let dt = self.last_t.map_or(0, |l| t_ms.saturating_sub(l));
        self.last_t = Some(t_ms);
        if self.speech && self.state != VoiceState::Idle {
            self.nomatch_speech_ms += dt;
        }

        match ev {
            TrackerEvent::Start => {
                if self.state == VoiceState::Idle {
                    self.state = VoiceState::Listening;
                    self.nomatch_speech_ms = 0;
                    self.clear_buffer();
                }
            }
            TrackerEvent::Stop => {
                self.state = VoiceState::Idle;
                self.speech = false;
                self.clear_buffer();
            }
            _ if self.state == VoiceState::Idle => {}
            TrackerEvent::VadSpeech => self.speech = true,
            TrackerEvent::VadSilence => {
                self.speech = false;
                if self.state == VoiceState::Tracking {
                    self.state = VoiceState::Hold;
                }
            }
            TrackerEvent::ManualSeek { token } => {
                let n = self.index.norm_of_token(token);
                self.cursor = if self.index.is_empty() { None } else { Some(n) };
                self.state = VoiceState::Hold;
                self.override_until = t_ms + self.cfg.manual_override_ms;
                self.nomatch_speech_ms = 0;
                self.last_match = None;
                self.clear_buffer();
            }
            TrackerEvent::Hypothesis { text, is_final } => {
                self.speech = true;
                if t_ms >= self.override_until {
                    self.on_hypothesis(&text, is_final, t_ms);
                }
            }
            TrackerEvent::Tick => {}
            TrackerEvent::Resync { text, age_ms } => {
                self.last_resync = self.apply_resync(&text, age_ms, t_ms);
            }
        }
        self.on_time(t_ms);
        self.output()
    }

    fn on_time(&mut self, t_ms: u64) {
        match self.state {
            VoiceState::Resync if t_ms >= self.resync_until => self.state = VoiceState::Tracking,
            VoiceState::Tracking | VoiceState::Hold
                if self.nomatch_speech_ms > self.cfg.lost_timeout_ms =>
            {
                self.state = VoiceState::Lost;
                self.confidence = 0.0;
            }
            _ => {}
        }
    }

    fn build_query(&self) -> Vec<QueryWord> {
        let total = self.committed.len() + self.partial.len();
        let skip = total.saturating_sub(self.cfg.k);
        self.committed
            .iter()
            .map(|w| (w, self.cfg.final_weight))
            .chain(self.partial.iter().map(|w| (w, self.cfg.partial_weight)))
            .skip(skip)
            .map(|(w, weight)| QueryWord { text: w.clone(), weight })
            .collect()
    }

    fn on_hypothesis(&mut self, text: &str, is_final: bool, t_ms: u64) {
        let words = normalize_hypothesis(text, self.index.lang);
        if is_final {
            self.committed.extend(words);
            self.partial.clear();
            while self.committed.len() > self.cfg.k * 3 {
                self.committed.pop_front();
            }
        } else {
            self.partial = words;
        }
        let query = self.build_query();
        let qtexts: Vec<String> = query.iter().map(|q| q.text.clone()).collect();
        if query.is_empty() || qtexts == self.last_query {
            return;
        }
        self.last_query = qtexts;

        let expected = self.cursor.map_or(0, |c| c + 1);
        let n = self.index.len();
        let lo = expected.saturating_sub(self.cfg.window_before);
        let hi = (expected + self.cfg.window_after).min(n);

        let local = align(&query, &self.index.words, lo..hi, expected, &self.cfg.sw)
            .filter(|h| h.confidence >= self.cfg.theta && h.matches >= self.cfg.min_matches);

        if let Some(hit) = local {
            let back_ok = !self.cursor.is_some_and(|c| hit.end + self.cfg.backtrack_words < c);
            let fwd_ok = hit.end <= expected + self.cfg.jump_words;
            let is_jump = self.state == VoiceState::Lost || !(back_ok && fwd_ok);
            if !is_jump {
                self.accept(hit, t_ms);
                return;
            }
            self.consider_jump(hit, t_ms);
            return;
        }

        // Nothing near the cursor: global n-gram search (catches far jumps and `lost`).
        if query.len() >= self.cfg.min_matches_global as usize {
            let global = align(&query, &self.index.words, 0..n, expected, &self.cfg.sw).filter(|h| {
                h.confidence >= self.cfg.theta_global && h.matches >= self.cfg.min_matches_global
            });
            if let Some(hit) = global {
                self.consider_jump(hit, t_ms);
                return;
            }
        }
        if self.state == VoiceState::Tracking {
            self.state = VoiceState::Hold;
        }
    }

    fn accept(&mut self, hit: SwHit, t_ms: u64) {
        if let Some((prev, t_prev)) = self.last_match {
            let dt = t_ms.saturating_sub(t_prev);
            if dt > 0 && hit.end > prev {
                let inst = (hit.end - prev) as f32 * 1000.0 / dt as f32;
                self.rate_wps =
                    if self.rate_wps == 0.0 { inst } else { 0.8 * self.rate_wps + 0.2 * inst };
            }
        }
        self.last_match = Some((hit.end, t_ms));
        self.cursor = Some(hit.end);
        self.confidence = hit.confidence;
        self.pending = None;
        self.nomatch_speech_ms = 0;
        if self.state != VoiceState::Resync {
            self.state = VoiceState::Tracking;
        }
    }

    /// Offline resync (ADR-006). The offline engine sees a whole VAD segment,
    /// so its text is more reliable than streaming partials: it is searched in
    /// a much wider window and, if confident, applied without the 2x jump
    /// confirmation. Because it describes *past* speech, a hit slightly behind
    /// the cursor is the normal case and only confirms streaming.
    fn apply_resync(&mut self, text: &str, age_ms: u64, t_ms: u64) -> ResyncOutcome {
        let rc = self.cfg.resync;
        if t_ms < self.override_until || self.index.is_empty() {
            return ResyncOutcome::Ignored;
        }
        let words = normalize_hypothesis(text, self.index.lang);
        let skip = words.len().saturating_sub(rc.k);
        let query: Vec<QueryWord> =
            words.into_iter().skip(skip).map(|text| QueryWord { text, weight: 1.0 }).collect();
        if query.is_empty() {
            return ResyncOutcome::Ignored;
        }
        let expected = self.cursor.map_or(0, |c| c + 1);
        let n = self.index.len();
        let lo = expected.saturating_sub(rc.window_before);
        let hi = (expected + rc.window_after).min(n);
        let mut hit = align(&query, &self.index.words, lo..hi, expected, &rc.sw)
            .filter(|h| h.confidence >= rc.theta && h.matches >= rc.min_matches);
        if hit.is_none() && matches!(self.state, VoiceState::Lost | VoiceState::Listening) {
            let theta = rc.theta.max(self.cfg.theta_global);
            hit = align(&query, &self.index.words, 0..n, expected, &rc.sw)
                .filter(|h| h.confidence >= theta && h.matches >= rc.min_matches);
        }
        let Some(hit) = hit else { return ResyncOutcome::NoMatch };

        let lag = rc.lag_words + (self.rate_wps.max(0.0) * age_ms as f32 / 1000.0).ceil() as usize;
        if let Some(c) = self.cursor {
            if hit.end <= c + rc.deadband_words && hit.end + lag >= c {
                self.nomatch_speech_ms = 0;
                self.confidence = self.confidence.max(hit.confidence);
                return ResyncOutcome::Confirmed;
            }
        }
        let from = self.cursor;
        self.clear_buffer();
        self.last_match = None; // no rate across a re-anchor
        self.cursor = Some(hit.end);
        self.confidence = hit.confidence;
        self.nomatch_speech_ms = 0;
        self.state = VoiceState::Resync;
        self.resync_until = t_ms + self.cfg.resync_ease_ms;
        ResyncOutcome::Reanchored { from, to: hit.end }
    }

    /// Hysteresis: a jump is executed only after `jump_confirmations`
    /// consistent hypotheses (each new candidate must be at or slightly
    /// beyond the previous one).
    fn consider_jump(&mut self, hit: SwHit, t_ms: u64) {
        let consistent = self.pending.is_some_and(|p| {
            hit.end + self.cfg.jump_tolerance >= p.pos
                && hit.end <= p.pos + self.cfg.jump_tolerance * 2
        });
        let count = if consistent { self.pending.map_or(1, |p| p.count + 1) } else { 1 };
        if count >= self.cfg.jump_confirmations {
            self.last_match = None; // don't compute a rate across the jump
            self.state = VoiceState::Resync;
            self.resync_until = t_ms + self.cfg.resync_ease_ms;
            self.accept(hit, t_ms);
        } else {
            self.pending = Some(PendingJump { pos: hit.end, count });
            if self.state == VoiceState::Tracking {
                self.state = VoiceState::Hold;
            }
        }
    }
}

#[cfg(test)]
mod resync_tests {
    use super::*;
    use crate::normalize::Lang;

    const SYL: [&str; 8] = ["ka", "lo", "mi", "ru", "te", "so", "na", "pe"];

    /// 300 distinct pseudo-words; neighbours differ in the first syllable.
    fn words() -> Vec<String> {
        (0..300).map(|i| format!("{}{}{}", SYL[i % 8], SYL[(i / 8) % 8], SYL[(i / 64) % 8])).collect()
    }

    fn tracker() -> VoiceTracker {
        let w = words();
        let idx = ScriptIndex::build(w.iter().enumerate().map(|(i, s)| (i, s.as_str())), Lang::De);
        let mut t = VoiceTracker::new(idx, TrackerConfig::default());
        t.handle(TrackerEvent::Start, 0);
        t
    }

    fn text(a: usize, b: usize) -> String {
        words()[a..b].join(" ")
    }

    /// Seeks to `token` at t=0 (override ends at 2000 ms).
    fn at(token: usize) -> VoiceTracker {
        let mut t = tracker();
        t.handle(TrackerEvent::ManualSeek { token }, 0);
        t
    }

    #[test]
    fn forward_reanchor_in_wide_window_without_confirmation() {
        let mut t = at(10);
        // 100 words ahead: far outside the streaming window (+60) and above
        // `jump_words`, yet a single offline hypothesis suffices.
        let out = t.resync(&text(100, 112), 0, 3000);
        assert_eq!(out, ResyncOutcome::Reanchored { from: Some(10), to: 111 });
        assert_eq!(t.output().cursor_token, Some(111));
        assert_eq!(t.state(), VoiceState::Resync);
        t.handle(TrackerEvent::Tick, 3400);
        assert_eq!(t.state(), VoiceState::Tracking);
    }

    #[test]
    fn hit_slightly_behind_cursor_only_confirms() {
        let mut t = at(50);
        assert_eq!(t.resync(&text(40, 49), 0, 3000), ResyncOutcome::Confirmed);
        assert_eq!(t.output().cursor_token, Some(50));
        assert_ne!(t.state(), VoiceState::Resync);
    }

    #[test]
    fn decode_age_widens_tolerated_lag() {
        let mut t = at(80);
        // 79 - 59 = 20 words behind: beyond lag_words (15) -> re-anchor ...
        let mut t2 = t.clone();
        assert!(matches!(t2.resync(&text(48, 60), 0, 3000), ResyncOutcome::Reanchored { .. }));
        // ... but with 3 wps * 2 s of decode latency it is consistent.
        t.rate_wps = 3.0;
        assert_eq!(t.resync(&text(48, 60), 2000, 3000), ResyncOutcome::Confirmed);
    }

    #[test]
    fn backward_reanchor_when_streaming_ran_away() {
        let mut t = at(150);
        let out = t.resync(&text(75, 87), 0, 3000);
        assert_eq!(out, ResyncOutcome::Reanchored { from: Some(150), to: 86 });
        assert_eq!(t.output().cursor_token, Some(86));
    }

    #[test]
    fn ignored_during_manual_override_and_when_idle() {
        let mut t = at(10);
        assert_eq!(t.resync(&text(100, 112), 0, 500), ResyncOutcome::Ignored);
        assert_eq!(t.output().cursor_token, Some(10));
        t.handle(TrackerEvent::Stop, 600);
        assert_eq!(t.resync(&text(100, 112), 0, 5000), ResyncOutcome::Ignored);
        let mut fresh = tracker();
        assert_eq!(fresh.resync("", 0, 10), ResyncOutcome::Ignored);
    }

    #[test]
    fn unrelated_text_does_not_move_cursor() {
        let mut t = at(20);
        let out = t.resync("xylophon quatsch banane fernseher bluetooth wasserkocher", 0, 3000);
        assert_eq!(out, ResyncOutcome::NoMatch);
        assert_eq!(t.output().cursor_token, Some(20));
    }

    #[test]
    fn lost_tracker_is_recovered_globally() {
        let mut t = at(5);
        // Speech without match for > lost_timeout -> lost.
        t.handle(TrackerEvent::VadSpeech, 2100);
        t.handle(TrackerEvent::Tick, 7000);
        assert_eq!(t.state(), VoiceState::Lost);
        // Far ahead (beyond the streaming window): recovered in one step.
        let out = t.resync(&text(268, 280), 0, 7100);
        assert_eq!(out, ResyncOutcome::Reanchored { from: Some(5), to: 279 });
    }
}
