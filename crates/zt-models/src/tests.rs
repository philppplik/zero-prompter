//! Unit tests for the manifest, the store and the install flow.

use super::*;
use std::io::Write;

const EMPTY_SHA: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const ABC_SHA: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn manifest_json(sha: &str, size: u64) -> String {
    format!(
        r#"{{"schemaVersion":1,"models":[{{"id":"kroko-de","name":"Kroko DE","languages":["de"],"tier":"s",
        "engine":"sherpaOnlineTransducer","license":"unknown","files":[
        {{"path":"tokens.txt","url":"https://example.invalid/tokens.txt","sha256":"{sha}","size":{size},"role":"tokens"}}]}}]}}"#
    )
}

fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zt-models-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn sha256_known_vectors() {
    assert_eq!(sha256_reader(&b""[..]).unwrap(), EMPTY_SHA);
    assert_eq!(sha256_reader(&b"abc"[..]).unwrap(), ABC_SHA);
}

#[test]
fn manifest_roundtrip_and_validation() {
    let m = Manifest::parse(&manifest_json(ABC_SHA, 3)).unwrap();
    assert_eq!(m.get("kroko-de").unwrap().file_by_role("tokens").unwrap().size, 3);
    assert_eq!(m.for_language("de").count(), 1);
    let again = Manifest::parse(&m.to_json().unwrap()).unwrap();
    assert_eq!(m, again);
    assert!(Manifest::parse(&manifest_json("zz", 3)).is_err());
    let traversal = manifest_json(ABC_SHA, 3).replace("tokens.txt\",\"url", "../x\",\"url");
    assert!(Manifest::parse(&traversal).is_err());
}

#[test]
fn safe_paths() {
    assert!(is_safe_relative("a/b.onnx"));
    for bad in ["", "/etc/x", "../x", "a/../b", "a\\b", "C:/x", "a//b", "./a"] {
        assert!(!is_safe_relative(bad), "{bad}");
    }
}

#[test]
fn store_status_and_finalize() {
    let root = tmpdir("store");
    let m = Manifest::parse(&manifest_json(ABC_SHA, 3)).unwrap();
    let e = m.get("kroko-de").unwrap();
    let f = &e.files[0];
    let store = ModelStore::new(&root);
    assert_eq!(store.status(e), ModelStatus::NotInstalled);

    std::fs::create_dir_all(store.model_dir(e)).unwrap();
    File::create(store.partial_path(e, f)).unwrap().write_all(b"abd").unwrap();
    assert!(matches!(store.finalize_download(e, f), Err(ModelError::Checksum { .. })));
    assert!(!store.partial_path(e, f).exists(), "corrupt .part must be deleted");

    File::create(store.partial_path(e, f)).unwrap().write_all(b"abc").unwrap();
    assert_eq!(store.finalize_download(e, f).unwrap().1, Verified::Hash);
    assert_eq!(store.status(e), ModelStatus::Installed);
    let _ = std::fs::remove_dir_all(&root);
}

struct FakeDl(&'static [u8]);
impl Downloader for FakeDl {
    fn download(
        &mut self,
        _url: &str,
        dest: &Path,
        _size: u64,
        progress: Progress<'_>,
    ) -> Result<(), ModelError> {
        std::fs::write(dest, self.0).map_err(|e| ModelError::Download(e.to_string()))?;
        progress(self.0.len() as u64, self.0.len() as u64);
        Ok(())
    }
}

#[test]
fn install_with_fake_downloader() {
    let root = tmpdir("install");
    let m = Manifest::parse(&manifest_json(ABC_SHA, 3)).unwrap();
    let e = m.get("kroko-de").unwrap();
    let store = ModelStore::new(&root);
    let mut last = (0, 0);
    let report = install(&store, e, &mut FakeDl(b"abc"), &mut |d, t| last = (d, t)).unwrap();
    assert_eq!(last, (3, 3));
    assert!(report.unverified.is_empty());
    assert_eq!(store.status(e), ModelStatus::Installed);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn missing_sha_is_a_warning_not_an_error() {
    // Empty string (bundled tokens.txt) and absent field both mean "unverified".
    let m = Manifest::parse(&manifest_json("", 3)).unwrap();
    assert_eq!(m.get("kroko-de").unwrap().files[0].sha256, None);
    assert_eq!(m.warnings().len(), 1);
    // A malformed hash is still fatal.
    assert!(Manifest::parse(&manifest_json("abc", 3)).is_err());
    // Upper-case hashes are normalised.
    let up = Manifest::parse(&manifest_json(&ABC_SHA.to_uppercase(), 3)).unwrap();
    assert_eq!(up.models[0].files[0].sha256.as_deref(), Some(ABC_SHA));
}

#[test]
fn unverified_install_reports_computed_hash_and_size_still_guards() {
    let root = tmpdir("unverified");
    let m = Manifest::parse(&manifest_json("", 3)).unwrap();
    let e = m.get("kroko-de").unwrap();
    let store = ModelStore::new(&root);
    let report = install(&store, e, &mut FakeDl(b"abc"), &mut |_, _| {}).unwrap();
    assert_eq!(report.unverified, vec![("tokens.txt".to_string(), ABC_SHA.to_string())]);
    assert_eq!(store.status(e), ModelStatus::Installed);
    // Wrong size is rejected even without a hash.
    let root2 = tmpdir("unverified-size");
    let store2 = ModelStore::new(&root2);
    assert!(matches!(
        install(&store2, e, &mut FakeDl(b"abcd"), &mut |_, _| {}),
        Err(ModelError::Size { .. })
    ));
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&root2);
}

const BUNDLED: &str = include_str!("../../../apps/desktop/src-tauri/resources/models.json");

#[test]
fn bundled_manifest_is_valid_and_complete() {
    let m = Manifest::parse(BUNDLED).expect("bundled models.json must parse");
    assert_eq!(m.models.len(), 8);
    // Every file is pinned and hashed (no unverified files in the release manifest).
    assert!(m.warnings().is_empty(), "{:?}", m.warnings());
    for e in &m.models {
        for f in &e.files {
            assert!(!f.url.contains("/resolve/main/"), "{}/{} not pinned", e.id, f.path);
        }
    }
    let vad: Vec<_> = m.vad_models().collect();
    assert_eq!(vad.len(), 1);
    assert_eq!(vad[0].id, "silero-vad");
    assert_eq!(vad[0].license, "MIT");
    assert!(vad[0].file_by_role("vad").is_some());
    assert!(vad[0].effective_roles().is_empty(), "VAD is never a streaming/resync candidate");
    for id in [
        "nemotron-3.5-streaming-0.6b-160ms-int8",
        "nemotron-3.5-streaming-0.6b-80ms-int8",
        "nemotron-3.5-streaming-0.6b-320ms-int8",
        "zipformer-de-kroko-2025-08-06",
        "zipformer-en-kroko-2025-08-06",
        "zipformer-en-20m-2023-02-17",
        "parakeet-tdt-0.6b-v3-int8",
    ] {
        let e = m.get(id).unwrap();
        for role in ["encoder", "decoder", "joiner", "tokens"] {
            assert!(e.file_by_role(role).is_some(), "{id} lacks {role}");
        }
    }
    let p = m.get("parakeet-tdt-0.6b-v3-int8").unwrap();
    assert_eq!(p.tier, Tier::L);
    assert_eq!(p.model_type(), Some(ModelType::OfflineNemoTransducer));
    assert_eq!(p.model_type().unwrap().sherpa_model_type(), Some("nemo_transducer"));
    assert!(p.has_role(ModelRole::Resync) && !p.has_role(ModelRole::Streaming));
}

#[test]
fn default_selection_and_fallback_chain() {
    let m = Manifest::parse(BUNDLED).unwrap();
    let ids = |v: Vec<&ModelEntry>| v.into_iter().map(|e| e.id.clone()).collect::<Vec<_>>();
    const N160: &str = "nemotron-3.5-streaming-0.6b-160ms-int8";
    const N80: &str = "nemotron-3.5-streaming-0.6b-80ms-int8";
    const N320: &str = "nemotron-3.5-streaming-0.6b-320ms-int8";
    // Nemotron 160 ms is the default for DE and EN, Kroko/20M stay selectable;
    // the other chunk variants are user choices only (no defaultRank).
    assert_eq!(
        ids(m.candidates("de-DE", ModelRole::Streaming)),
        [N160, "zipformer-de-kroko-2025-08-06", N80, N320]
    );
    assert_eq!(
        ids(m.candidates("en", ModelRole::Streaming)),
        [N160, "zipformer-en-kroko-2025-08-06", "zipformer-en-20m-2023-02-17", N80, N320]
    );
    // Nemotron not installed (existing users): Kroko keeps being used.
    assert_eq!(
        m.select("de", ModelRole::Streaming, None, |e| !e.id.starts_with("nemotron")).unwrap().id,
        "zipformer-de-kroko-2025-08-06"
    );
    let n = m.get(N160).unwrap();
    assert_eq!(n.model_type(), Some(ModelType::OnlineNemoTransducer));
    assert!(n.model_type().unwrap().is_online());
    assert_eq!(n.model_type().unwrap().sherpa_model_type(), None);
    assert_eq!(n.language_prompt("de-DE").as_deref(), Some("de"));
    assert_eq!(n.language_prompt("fr").as_deref(), Some("auto"));
    assert!(n.requires_attribution());
    assert_eq!(m.get("zipformer-de-kroko-2025-08-06").unwrap().language_prompt("de"), None);
    // Legacy expectations below: Nemotron unranked (as for users who pin Kroko).
    let old = format!("\"id\": \"{N160}\", \"defaultRank\": 0");
    let new = format!("\"id\": \"{N160}\"");
    let m = Manifest::parse(&BUNDLED.replace(&old, &new)).unwrap();
    assert_eq!(ids(m.candidates("de", ModelRole::Resync)), ["parakeet-tdt-0.6b-v3-int8"]);
    // User preference moves to the front of the chain.
    assert_eq!(
        ids(m.fallback_chain("en", ModelRole::Streaming, Some("zipformer-en-20m-2023-02-17")))[0],
        "zipformer-en-20m-2023-02-17"
    );
    // A preference for another language is ignored.
    assert_eq!(
        ids(m.fallback_chain("en", ModelRole::Streaming, Some("zipformer-de-kroko-2025-08-06")))[0],
        "zipformer-en-kroko-2025-08-06"
    );
    // Kroko EN not installed -> falls back to the Apache-2.0 20M model.
    let sel =
        m.select("en-US", ModelRole::Streaming, None, |e| e.id != "zipformer-en-kroko-2025-08-06");
    assert_eq!(sel.unwrap().id, "zipformer-en-20m-2023-02-17");
    assert!(m.select("fr", ModelRole::Streaming, None, |_| true).is_none());
}

#[test]
fn streaming_plan_language_fallback() {
    let m = Manifest::parse(BUNDLED).unwrap();
    let ids = |p: &StreamingPlan<'_>| p.chain.iter().map(|e| e.id.clone()).collect::<Vec<_>>();
    // Only the EN 20M model installed, script language German:
    // use it (reported as language fallback) instead of demo mode.
    let only_en20 = |e: &ModelEntry| e.id == "zipformer-en-20m-2023-02-17";
    let p = m.streaming_plan("de", None, only_en20);
    assert_eq!(ids(&p), ["zipformer-en-20m-2023-02-17"]);
    assert_eq!(p.language_fallback.as_deref(), Some("en"));
    // Matching language installed: no fallback flag.
    let p = m.streaming_plan("de-DE", None, |e| e.id == "zipformer-de-kroko-2025-08-06");
    assert_eq!(ids(&p), ["zipformer-de-kroko-2025-08-06"]);
    assert_eq!(p.language_fallback, None);
    // Resync-only (Parakeet) or VAD-only installs give no streaming model.
    let p = m.streaming_plan("de", None, |e| {
        e.id == "parakeet-tdt-0.6b-v3-int8" || e.id == "silero-vad"
    });
    assert!(p.chain.is_empty());
    assert_eq!(p.language_fallback, None);
}

#[test]
fn schema_v1_still_accepted_and_roles_derived() {
    let m = Manifest::parse(&manifest_json(ABC_SHA, 3)).unwrap();
    let e = m.get("kroko-de").unwrap();
    assert_eq!(e.model_type(), Some(ModelType::OnlineTransducer));
    assert_eq!(e.effective_roles(), [ModelRole::Streaming]);
    assert!(Manifest::parse(
        &manifest_json(ABC_SHA, 3).replace("\"schemaVersion\":1", "\"schemaVersion\":9")
    )
    .is_err());
}
