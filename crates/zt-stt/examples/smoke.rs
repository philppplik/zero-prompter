//! End-to-end smoke test of the sherpa streaming adapter (CI: `stt-smoke`).
//!
//! cargo run -p zt-stt --release --features sherpa --example smoke -- \
//!   --name kroko-de --dir models/kroko-de --encoder encoder.onnx \
//!   --decoder decoder.onnx --joiner joiner.onnx --tokens tokens.txt \
//!   --wav test.wav --expect "wort1,wort2" [--hotwords "a,b"] [--legacy-check]
//!
//! Creates the recognizer (fallback ladder logged), optionally applies
//! hotwords, streams the WAV in 100 ms chunks, prints the recognized text,
//! RTF and per-chunk latency, and fails if an expected keyword is missing.
//! `--legacy-check` reproduces the 0.x bug: the old config
//! (`modeling_unit = "bpe"` without `bpe_vocab`) must make
//! `OnlineRecognizer::create` return `None`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use zt_stt::{Hotword, LogSink, SherpaConfig, SherpaEngine, SpeechEngine, SttEvent, SAMPLE_RATE};

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
        let hw: Vec<Hotword> = hotwords.iter().map(|t| Hotword { text: t.clone(), score: 1.5 }).collect();
        let r = eng.set_hotwords(&hw);
        println!("hotwords {} -> {:?}, stage={}", hw.len(), r.as_ref().map_err(|e| e.to_string()), eng.active_stage());
        if r.is_err() || eng.active_stage() != "beam+hotwords" {
            println!("FAIL: hotwords not active");
            ok = false;
        }
    }
    let stage = eng.active_stage().to_string();

    let w = sherpa_onnx::Wave::read(&wav).unwrap_or_else(|| panic!("cannot read {wav}"));
    let samples = resample(w.samples(), w.sample_rate() as u32);
    let audio_s = samples.len() as f64 / SAMPLE_RATE as f64;
    let chunk = (SAMPLE_RATE / 10) as usize;
    let mut tail = samples;
    tail.resize(tail.len() + SAMPLE_RATE as usize * 2, 0.0);

    let mut lat: Vec<Duration> = Vec::new();
    let mut finals: Vec<String> = Vec::new();
    let mut partial = String::new();
    let mut first_partial_at: Option<f64> = None;
    let mut fed = 0usize;
    for c in tail.chunks(chunk) {
        let t = Instant::now();
        eng.accept_waveform(c);
        lat.push(t.elapsed());
        fed += c.len();
        for ev in eng.poll() {
            match ev {
                SttEvent::Partial(p) => {
                    first_partial_at.get_or_insert(fed as f64 / SAMPLE_RATE as f64);
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
    let pct = |p: f64| ms[((ms.len() - 1) as f64 * p).round() as usize];
    let avg = ms.iter().sum::<f64>() / ms.len() as f64;

    let low = text.to_lowercase();
    let missing: Vec<&String> = expect.iter().filter(|k| !low.contains(&k.to_lowercase())).collect();
    ok &= missing.is_empty() && !text.trim().is_empty();

    println!("TEXT: {text}");
    let line = format!(
        "SMOKE model={name} platform={platform} stage={stage} create_ms={create_ms} audio_s={audio_s:.2} \
         rtf={rtf:.3} chunk_ms_avg={avg:.1} p50={:.1} p95={:.1} max={:.1} first_partial_s={} missing={missing:?} ok={ok}",
        pct(0.5),
        pct(0.95),
        pct(1.0),
        first_partial_at.map_or("-".into(), |s| format!("{s:.1}")),
    );
    println!("{line}");
    if let Some(p) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open(p) {
            let _ = writeln!(
                f,
                "| {name} | {platform} | {stage} | {rtf:.3} | {avg:.1} / {:.1} / {:.1} | {create_ms} | {} | {text} |",
                pct(0.95),
                pct(1.0),
                if ok { "ok" } else { "FAIL" }
            );
        }
    }
    if !ok {
        std::process::exit(1);
    }
}
