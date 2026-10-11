//! Speech-onset detection for the "Sprechbeginn -> erste Hypothese" metric.
//!
//! The first version used a fixed threshold (`RMS > -45 dBFS`) and was only
//! re-armed on an endpoint. With a quiet microphone + AGC the *noise* is
//! lifted above -45 dBFS, so the "onset" fired on room noise seconds (or
//! tens of seconds) before anybody spoke, and it was never reset during
//! pauses: field log p50 2.6 s, p95 46 s.
//!
//! Now the threshold is relative to a tracked noise floor (fast down,
//! slow up, frozen during speech), an onset needs `min_speech_ms` of
//! consecutive loud frames (time stamp = first loud frame), and the detector
//! re-arms after `reset_silence_ms` of quiet. An onset that ended before its
//! first hypothesis arrived (short word, slow model) stays claimable for
//! `pending_ms`, so slow models are measured instead of silently dropped.
//!
//! Generic over the time stamp type (`Instant` in the app, audio seconds in
//! the CI smoke test); all durations are derived from sample counts.

/// Tuning of [`OnsetDetector`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OnsetConfig {
    pub sample_rate: u32,
    /// Loud = level above noise floor + `margin_db`.
    pub margin_db: f32,
    /// Absolute lower bound of the threshold (digital silence + dither).
    pub min_level_dbfs: f32,
    /// Consecutive loud audio needed for an onset.
    pub min_speech_ms: f32,
    /// Quiet = level below noise floor + `release_db`.
    pub release_db: f32,
    /// Quiet audio after which the detector re-arms.
    pub reset_silence_ms: f32,
    /// Noise floor rise rate outside speech (dB per second).
    pub floor_rise_db_per_s: f32,
    /// How long an unclaimed onset stays claimable after its speech ended.
    pub pending_ms: f32,
}

impl Default for OnsetConfig {
    fn default() -> Self {
        Self {
            sample_rate: crate::TARGET_RATE,
            margin_db: 12.0,
            min_level_dbfs: -70.0,
            min_speech_ms: 60.0,
            release_db: 6.0,
            reset_silence_ms: 500.0,
            floor_rise_db_per_s: 3.0,
            pending_ms: 2500.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State<T> {
    Idle,
    Speech { at: T, claimed: bool },
}

/// Noise-floor relative onset detector (see module docs).
#[derive(Debug, Clone)]
pub struct OnsetDetector<T: Copy> {
    cfg: OnsetConfig,
    floor: Option<f32>,
    run_ms: f32,
    run_start: Option<T>,
    silence_ms: f32,
    state: State<T>,
    /// Unclaimed onset of an already finished utterance + its age.
    pending: Option<(T, f32)>,
}

impl<T: Copy> OnsetDetector<T> {
    pub fn new(cfg: OnsetConfig) -> Self {
        Self {
            cfg,
            floor: None,
            run_ms: 0.0,
            run_start: None,
            silence_ms: 0.0,
            state: State::Idle,
            pending: None,
        }
    }

    /// Current noise floor estimate (dBFS).
    pub fn noise_floor(&self) -> Option<f32> {
        self.floor
    }

    pub fn in_speech(&self) -> bool {
        matches!(self.state, State::Speech { .. })
    }

    /// Feeds the level of one audio block (`samples` long, captured at `at`).
    pub fn push(&mut self, db: f32, samples: usize, at: T) {
        let ms = samples as f32 * 1000.0 / self.cfg.sample_rate.max(1) as f32;
        let db = db.max(-100.0);
        if let Some((_, age)) = &mut self.pending {
            *age += ms;
            if *age > self.cfg.pending_ms {
                self.pending = None;
            }
        }
        let floor = *self.floor.get_or_insert(db);
        let floor = if db < floor {
            floor + (db - floor) * 0.3
        } else if self.in_speech() || self.run_start.is_some() {
            floor
        } else {
            floor + (db - floor).min(self.cfg.floor_rise_db_per_s * ms / 1000.0)
        };
        self.floor = Some(floor);
        let threshold = (floor + self.cfg.margin_db).max(self.cfg.min_level_dbfs);
        match self.state {
            State::Idle => {
                if db > threshold {
                    let start = *self.run_start.get_or_insert(at);
                    self.run_ms += ms;
                    if self.run_ms >= self.cfg.min_speech_ms {
                        self.state = State::Speech { at: start, claimed: false };
                        self.silence_ms = 0.0;
                        self.run_ms = 0.0;
                        self.run_start = None;
                    }
                } else {
                    self.run_ms = 0.0;
                    self.run_start = None;
                }
            }
            State::Speech { at: start, claimed } => {
                if db < floor + self.cfg.release_db {
                    self.silence_ms += ms;
                    if self.silence_ms >= self.cfg.reset_silence_ms {
                        if !claimed {
                            self.pending = Some((start, 0.0));
                        }
                        self.state = State::Idle;
                    }
                } else {
                    self.silence_ms = 0.0;
                }
            }
        }
    }

    /// Called on a hypothesis: returns the onset of the running utterance
    /// once (first hypothesis only), else a recent unclaimed onset.
    pub fn claim(&mut self) -> Option<T> {
        match &mut self.state {
            State::Speech { at, claimed: false } => {
                let at = *at;
                self.state = State::Speech { at, claimed: true };
                self.pending = None;
                Some(at)
            }
            State::Speech { .. } => None,
            State::Idle => self.pending.take().map(|(t, _)| t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 10 ms blocks; time stamp = block index.
    fn feed(d: &mut OnsetDetector<u32>, t: &mut u32, db: f32, blocks: u32) {
        for _ in 0..blocks {
            d.push(db, 160, *t);
            *t += 1;
        }
    }

    #[test]
    fn agc_lifted_noise_is_not_speech() {
        // Noise at -40 dBFS (above the old fixed -45 dBFS threshold).
        let mut d = OnsetDetector::new(OnsetConfig::default());
        let mut t = 0;
        feed(&mut d, &mut t, -40.0, 500);
        assert!(!d.in_speech());
        assert_eq!(d.claim(), None);
        // Speech 15 dB above the noise starts at block 500.
        feed(&mut d, &mut t, -25.0, 30);
        assert!(d.in_speech());
        assert_eq!(d.claim(), Some(500));
        assert_eq!(d.claim(), None, "only the first hypothesis is measured");
    }

    #[test]
    fn short_clicks_are_ignored_and_rearm_after_silence() {
        let mut d = OnsetDetector::new(OnsetConfig::default());
        let mut t = 0;
        feed(&mut d, &mut t, -60.0, 100);
        feed(&mut d, &mut t, -20.0, 3); // 30 ms click
        feed(&mut d, &mut t, -60.0, 20);
        assert!(!d.in_speech());
        feed(&mut d, &mut t, -30.0, 50); // utterance 1 at 123
        assert_eq!(d.claim(), Some(123));
        feed(&mut d, &mut t, -60.0, 60); // 600 ms pause -> re-armed
        assert!(!d.in_speech());
        let start2 = t;
        feed(&mut d, &mut t, -30.0, 50);
        assert_eq!(d.claim(), Some(start2));
    }

    #[test]
    fn short_pause_inside_speech_does_not_rearm() {
        let mut d = OnsetDetector::new(OnsetConfig::default());
        let mut t = 0;
        feed(&mut d, &mut t, -60.0, 100);
        feed(&mut d, &mut t, -30.0, 50);
        assert_eq!(d.claim(), Some(100));
        feed(&mut d, &mut t, -60.0, 20); // 200 ms
        feed(&mut d, &mut t, -30.0, 50);
        assert_eq!(d.claim(), None);
    }

    #[test]
    fn late_hypothesis_after_short_word_is_measured_but_expires() {
        let mut d = OnsetDetector::new(OnsetConfig::default());
        let mut t = 0;
        feed(&mut d, &mut t, -60.0, 100);
        feed(&mut d, &mut t, -30.0, 20); // "Ja"
        feed(&mut d, &mut t, -60.0, 80); // silence, hypothesis still pending
        assert!(!d.in_speech());
        assert_eq!(d.claim(), Some(100));
        // A cough without hypothesis expires instead of poisoning later samples.
        feed(&mut d, &mut t, -30.0, 20);
        feed(&mut d, &mut t, -60.0, 400);
        assert_eq!(d.claim(), None);
    }

    #[test]
    fn floor_follows_rising_noise_slowly() {
        let mut d = OnsetDetector::new(OnsetConfig::default());
        let mut t = 0;
        feed(&mut d, &mut t, -70.0, 100);
        // AGC ramps the noise up by 10 dB over 5 s: not an onset.
        for i in 0..500 {
            d.push(-70.0 + i as f32 * 0.02, 160, t);
            t += 1;
        }
        assert!(!d.in_speech());
        assert!(d.noise_floor().unwrap() > -64.0);
    }
}
