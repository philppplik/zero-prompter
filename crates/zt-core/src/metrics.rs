//! Rolling latency statistics (pure, no clock): the voice pipeline pushes
//! measured durations in milliseconds, the diagnostics panel reads p50/p95.
//!
//! Percentiles use the nearest-rank method over the last `cap` samples.

use serde::Serialize;
use std::collections::VecDeque;

/// Fixed-size window of the most recent samples (ms).
#[derive(Debug, Clone)]
pub struct LatencyWindow {
    buf: VecDeque<f64>,
    cap: usize,
    total: u64,
}

/// Snapshot for the UI / log (`null` fields when no sample yet).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencySummary {
    /// Samples currently in the window.
    pub count: usize,
    /// Samples ever recorded.
    pub total: u64,
    pub p50_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub max_ms: Option<f64>,
    pub last_ms: Option<f64>,
}

impl LatencyWindow {
    pub fn new(cap: usize) -> Self {
        let cap = cap.max(1);
        Self { buf: VecDeque::with_capacity(cap), cap, total: 0 }
    }

    pub fn push(&mut self, ms: f64) {
        if !ms.is_finite() || ms < 0.0 {
            return;
        }
        if self.buf.len() == self.cap {
            self.buf.pop_front();
        }
        self.buf.push_back(ms);
        self.total += 1;
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Nearest-rank percentile, `q` in `[0, 1]`.
    pub fn percentile(&self, q: f64) -> Option<f64> {
        if self.buf.is_empty() {
            return None;
        }
        let mut v: Vec<f64> = self.buf.iter().copied().collect();
        v.sort_by(|a, b| a.total_cmp(b));
        let q = q.clamp(0.0, 1.0);
        let rank = ((q * v.len() as f64).ceil() as usize).clamp(1, v.len());
        Some(v[rank - 1])
    }

    pub fn summary(&self) -> LatencySummary {
        LatencySummary {
            count: self.buf.len(),
            total: self.total,
            p50_ms: self.percentile(0.5),
            p95_ms: self.percentile(0.95),
            max_ms: self.buf.iter().copied().reduce(f64::max),
            last_ms: self.buf.back().copied(),
        }
    }
}

impl Default for LatencyWindow {
    fn default() -> Self {
        Self::new(200)
    }
}

/// Events per second over a sliding one-second bucket (frames/s, ticks/s).
#[derive(Debug, Clone, Copy, Default)]
pub struct RateCounter {
    bucket_start_ms: u64,
    count: u32,
    rate: f64,
}

impl RateCounter {
    /// Counts one event at `t_ms`; returns `Some(rate)` whenever a bucket closes.
    pub fn hit(&mut self, t_ms: u64) -> Option<f64> {
        self.count += 1;
        let dt = t_ms.saturating_sub(self.bucket_start_ms);
        if dt >= 1000 {
            self.rate = self.count as f64 * 1000.0 / dt as f64;
            self.count = 0;
            self.bucket_start_ms = t_ms;
            return Some(self.rate);
        }
        None
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_window_has_no_percentiles() {
        let w = LatencyWindow::new(10);
        assert!(w.is_empty());
        assert_eq!(w.summary().p50_ms, None);
        assert_eq!(w.summary().count, 0);
    }

    #[test]
    fn nearest_rank_percentiles() {
        let mut w = LatencyWindow::new(100);
        for i in 1..=100 {
            w.push(i as f64);
        }
        assert_eq!(w.percentile(0.5), Some(50.0));
        assert_eq!(w.percentile(0.95), Some(95.0));
        assert_eq!(w.percentile(1.0), Some(100.0));
        assert_eq!(w.percentile(0.0), Some(1.0));
        let s = w.summary();
        assert_eq!(s.max_ms, Some(100.0));
        assert_eq!(s.last_ms, Some(100.0));
    }

    #[test]
    fn window_is_rolling() {
        let mut w = LatencyWindow::new(3);
        for v in [1000.0, 1000.0, 1000.0, 10.0, 20.0, 30.0] {
            w.push(v);
        }
        assert_eq!(w.len(), 3);
        assert_eq!(w.summary().total, 6);
        assert_eq!(w.percentile(0.95), Some(30.0));
    }

    #[test]
    fn rejects_garbage_samples() {
        let mut w = LatencyWindow::new(3);
        w.push(f64::NAN);
        w.push(-1.0);
        assert!(w.is_empty());
    }

    #[test]
    fn rate_counter_reports_per_second() {
        let mut r = RateCounter::default();
        let mut last = None;
        for i in 0..=100u64 {
            if let Some(v) = r.hit(i * 10) {
                last = Some(v);
            }
        }
        // 100 hits within 1000 ms
        assert!((last.unwrap() - 100.0).abs() < 1.5, "{last:?}");
    }
}
