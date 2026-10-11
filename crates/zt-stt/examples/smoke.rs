//! End-to-end smoke test of the sherpa streaming adapter (CI: `stt-smoke`).
//!
//! cargo run -p zt-stt --release --features sherpa --example smoke -- \
//!   --name kroko-de --dir models/kroko-de --encoder encoder.onnx \
//!   --decoder decoder.onnx --joiner joiner.onnx --tokens tokens.txt \
//!   --wav test.wav --expect "wort1,wort2" [--hotwords "a,b"] [--legacy-check] \
//!   [--language de] [--threads 4]
//!
//! Creates the recognizer (fallback ladder logged), optionally applies
//! hotwords, streams the WAV in 100 ms chunks, prints the recognized text,
//! RTF, per-chunk latency, the partial cadence (audio time between text
//! changes = what the prompter cursor sees), the time from speech onset in
//! the WAV (noise-floor relative, `zt_audio::OnsetDetector`) to the first
//! partial, and fails if an expected keyword is missing.
//! `--legacy-check` reproduces the 0.x bug: the old config
//! (`modeling_unit = "bpe"` without `bpe_vocab`) must make
//! `OnlineRecognizer::create` return `None`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zt_audio::{rms_dbfs, OnsetConfig, OnsetDetector};
use zt_stt::{Hotword, LogSink, SherpaConfig, SherpaEngine, SpeechEngine, SttEvent, SAMPLE_RATE};

/// Nearest-rank percentile of an ascending slice (`NaN` if empty).
fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

/// Speech onset in the WAV (seconds), 10 ms blocks.
fn wav_onset(samples: &[f32]) -> Option<f64> {
    let block = (SAMPLE_RATE / 100) as usize;
    let mut d = OnsetDetector::<usize>::new(OnsetConfig::default());
    for (i, b) in samples.chunks(block).enumerate() {
        d.push(rms_dbfs(b), b.len(), i);
        if d.in_speech() {
            return d.claim().map(|i| (i * block) as f64 / SAMPLE_RATE as f64);
        }
    }
    None
}

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter().position(|a| a == key).and_then(|i| args.get(i + 1)).cloned()
}

fn list(s: Option<String>) -> Vec<String> {
    s.map(|s| s.split(',').map(|w| w.trim().to_string()).filter(|w| !w.is_empty()).collect())
        .unwrap_or_default()
}

/// Linear resampling to 16 kHz (sufficient for a smoke test).
fn resample(x: &[f32], from: u32) -> Vec<f32> {
    if from == SAMPLE_RATE {
        return x.to_vec();
    }
    let ratio = from as f64 / SAMPLE_RATE as f64;
    let n = (x.len() as f64 / ratio) as usize;
    (0..n)
        .map(|i| {
            let pos = i as f64 * ratio;
            let j = pos as usize;
            let f = (pos - j as f64) as f32;
            let a = x[j];
            let b = *x.get(j + 1).unwrap_or(&a);
            a + (b - a) * f
        })
        .collect()
}

fn legacy_check(cfg: &SherpaConfig) -> bool {
    let p = |x: &Path| x.to_string_lossy().into_owned();
    let mut c = sherpa_onnx::OnlineRecognizerConfig::default();
    c.model_config.transducer.encoder = Some(p(&cfg.encoder));
    c.model_config.transducer.decoder = Some(p(&cfg.decoder));
    c.model_config.transducer.joiner = Some(p(&cfg.joiner));
    c.model_config.tokens = Some(p(&cfg.tokens));
    c.model_config.num_threads = 2;
    c.decoding_method = Some("modified_beam_search".into());
    c.max_active_paths = 4;
    c.enable_endpoint = true;
    // Exactly the pre-fix configuration:
    c.model_config.modeling_unit = Some("bpe".into());
    c.model_config.bpe_vocab = None;
    let t0 = Instant::now();
    let old = sherpa_onnx::OnlineRecognizer::create(&c).is_some();
    let old_ms = t0.elapsed().as_millis();
    c.model_config.modeling_unit = None;
    let t1 = Instant::now();
    let fixed = sherpa_onnx::OnlineRecognizer::create(&c).is_some();
    println!(
        "LEGACY modeling_unit=bpe,bpe_vocab=None -> created={old} ({old_ms} ms); \
         modeling_unit=None -> created={fixed} ({} ms)",
        t1.elapsed().as_millis()
    );
    !old && fixed
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let need = |k: &str| arg(&args, k).unwrap_or_else(|| panic!("missing {k}"));
    let name = need("--name");
    let dir = PathBuf::from(need("--dir"));
    let wav = need("--wav");
    let expect = list(arg(&args, "--expect"));
    let hotwords = list(arg(&args, "--hotwords"));
    let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);

    let t_start = Instant::now();
    let mut cfg = SherpaConfig::zipformer(&dir, name.clone());
    cfg.encoder = dir.join(need("--encoder"));
    cfg.decoder = dir.join(need("--decoder"));
    cfg.joiner = dir.join(need("--joiner"));
    cfg.tokens = dir.join(need("--tokens"));
    cfg.debug = std::env::var_os("ZT_STT_DEBUG").is_some();
    cfg.language = arg(&args, "--language");
    if let Some(t) = arg(&args, "--threads") {
        cfg.num_threads = t.parse().expect("--threads");
    }
    let threads = cfg.num_threads;
    cfg.log = Some(LogSink(Arc::new(move |m: &str| {
        println!("[{:>7.3}s] {m}", t_start.elapsed().as_secs_f64())
    })));

    let mut ok = true;
    if args.iter().any(|a| a == "--legacy-check") {
        let reproduced = legacy_check(&cfg);
        println!("LEGACY root cause reproduced: {reproduced}");
        ok &= reproduced;
    }

    let t0 = Instant::now();
    let mut eng = SherpaEngine::new(cfg).unwrap_or_else(|e| panic!("engine: {e}"));
    let create_ms = t0.elapsed().as_millis();
    if !hotwords.is_empty() {
        let hw: Vec<Hotword> =
            hotwords.iter().map(|t| Hotword { text: t.clone(), score: 1.5 }).collect();
        let r = eng.set_hotwords(&hw);
        println!(
            "hotwords {} -> {:?}, stage={}",
            hw.len(),
            r.as_ref().map_err(|e| e.to_string()),
            eng.active_stage()
        );
        if r.is_err() || eng.active_stage() != "beam+hotwords" {
            println!("FAIL: hotwords not active");
            ok = false;
        }
    }
    let stage = eng.active_stage().to_string();

    let w = sherpa_onnx::Wave::read(&wav).unwrap_or_else(|| panic!("cannot read {wav}"));
    let samples = resample(w.samples(), w.sample_rate() as u32);
    let audio_s = samples.len() as f64 / SAMPLE_RATE as f64;
    let onset_s = wav_onset(&samples);
    let chunk = (SAMPLE_RATE / 10) as usize;
    let mut tail = samples;
    tail.resize(tail.len() + SAMPLE_RATE as usize * 2, 0.0);

    let mut lat: Vec<Duration> = Vec::new();
    let mut finals: Vec<String> = Vec::new();
    let mut partial = String::new();
    let mut first_partial_at: Option<f64> = None;
    // Audio time of every text change (partial event).
    let mut partial_times: Vec<f64> = Vec::new();
    let mut fed = 0usize;
    for c in tail.chunks(chunk) {
        let t = Instant::now();
        eng.accept_waveform(c);
        lat.push(t.elapsed());
        fed += c.len();
        for ev in eng.poll() {
            match ev {
                SttEvent::Partial(p) => {
                    let t_audio = fed as f64 / SAMPLE_RATE as f64;
                    first_partial_at.get_or_insert(t_audio);
                    partial_times.push(t_audio);
                    println!("PARTIAL t={t_audio:.2}s {p}");
                    partial = p;
                }
                SttEvent::Final(f) => {
                    finals.push(f);
                    partial.clear();
                }
                _ => {}
            }
        }
    }
    if !partial.is_empty() {
        finals.push(partial);
    }
    let text = finals.join(" ");
    let total: Duration = lat.iter().sum();
    let rtf = total.as_secs_f64() / (tail.len() as f64 / SAMPLE_RATE as f64);
    let mut ms: Vec<f64> = lat.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut gaps: Vec<f64> = partial_times.windows(2).map(|w| w[1] - w[0]).collect();
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let first_word_s = match (first_partial_at, onset_s) {
        (Some(p), Some(o)) => format!("{:.2}", p - o),
        _ => "-".into(),
    };
    let q = |p: f64| pct(&ms, p);
    let avg = ms.iter().sum::<f64>() / ms.len() as f64;

    let low = text.to_lowercase();
    let missing: Vec<&String> =
        expect.iter().filter(|k| !low.contains(&k.to_lowercase())).collect();
    ok &= missing.is_empty() && !text.trim().is_empty();

    println!("TEXT: {text}");
    // One machine-readable line; scripts/stt-smoke.py builds the comparison
    // table (log + step summary) from it.
    println!(
        "SMOKE model={name} platform={platform} stage={stage} create_ms={create_ms} audio_s={audio_s:.2} \
         rtf={rtf:.3} chunk_ms_avg={avg:.1} p50={:.1} p95={:.1} max={:.1} first_partial_s={} \
         onset_s={} first_word_s={first_word_s} partials={} cadence_s_p50={:.2} cadence_s_p95={:.2} \
         threads={threads} missing={missing:?} ok={ok}",
        q(0.5),
        q(0.95),
        q(1.0),
        first_partial_at.map_or("-".into(), |s| format!("{s:.2}")),
        onset_s.map_or("-".into(), |s| format!("{s:.2}")),
        partial_times.len(),
        pct(&gaps, 0.5),
        pct(&gaps, 0.95),
    );
    if !ok {
        std::process::exit(1);
    }
}
