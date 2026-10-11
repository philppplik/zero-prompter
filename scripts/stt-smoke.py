#!/usr/bin/env python3
"""CI STT smoke test: fetch models from models.json (SHA-256 verified),
fetch pinned test WAVs, run `cargo run --example smoke` per model.

usage: stt-smoke.py <cache_dir> <model_id>...   (cargo is run from the repo root)
"""
import hashlib
import json
import os
import subprocess
import sys
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MANIFEST = os.path.join(ROOT, "apps", "desktop", "src-tauri", "resources", "models.json")
HF = "https://huggingface.co/csukuangfj"

# Pinned test audio (HF commit hashes as in models.json) + SHA-256.
WAVS = {
    "en-librispeech-0": (
        f"{HF}/sherpa-onnx-streaming-zipformer-en-20M-2023-02-17/resolve/"
        "d42f2d9f7ca24806fb667456a18a9f1b60f70d16/test_wavs/0.wav",
        "6bc58a4efdf20daac252b6b1502632601a71efe0308f6757dc1eda34891a7e4f",
    ),
    "de-kroko-0": (
        f"{HF}/sherpa-onnx-streaming-zipformer-de-kroko-2025-08-06/resolve/"
        "887db3d083240198c2d2b99fb66cfcfe6948ced8/test_wavs/0.wav",
        "36d3c4845b9808a1656a2a2e92d884590e2db94389e6fe559643291ae0cd3710",
    ),
    "en-kroko-0": (
        f"{HF}/sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06/resolve/"
        "572aaf4e2e0c603c3fc2a574d096e755a178faa1/test_wavs/0.wav",
        "148b936b43ce7c546a866e64da059f0458aee2d65e617f16e9d94f06e8d99ed6",
    ),
}

# model id -> list of (wav, expected keywords, hotwords, legacy-check)
CASES = {
    "zipformer-de-kroko-2025-08-06": [
        ("de-kroko-0", "Ende,Wurst,zwei", "Teleprompter,Mikrofon", True),
    ],
    "zipformer-en-kroko-2025-08-06": [
        ("en-librispeech-0", "yellow,lamps,quarter", "nightfall,brothels", True),
        ("en-kroko-0", "country", "", False),
    ],
    "zipformer-en-20m-2023-02-17": [
        ("en-librispeech-0", "yellow,lamps,quarter", "nightfall,brothels", True),
    ],
}


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def fetch(url, dest, digest, size=None):
    if os.path.isfile(dest) and sha256(dest) == digest:
        print(f"cached  {dest}")
        return
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    print(f"fetch   {url}")
    urllib.request.urlretrieve(url, dest + ".part")
    got = sha256(dest + ".part")
    if got != digest:
        raise SystemExit(f"SHA mismatch {dest}: {got} != {digest}")
    if size is not None and os.path.getsize(dest + ".part") != size:
        raise SystemExit(f"size mismatch {dest}")
    os.replace(dest + ".part", dest)


def main():
    cache, ids = sys.argv[1], sys.argv[2:]
    models = {m["id"]: m for m in json.load(open(MANIFEST, encoding="utf-8"))["models"]}
    for name, (url, digest) in WAVS.items():
        fetch(url, os.path.join(cache, "wavs", name + ".wav"), digest)
    failed = []
    for mid in ids:
        m = models[mid]
        mdir = os.path.join(cache, mid)
        roles = {}
        for f in m["files"]:
            fetch(f["url"], os.path.join(mdir, f["path"]), f["sha256"], f.get("size"))
            roles[f["role"]] = f["path"]
        for wav, expect, hot, legacy in CASES[mid]:
            cmd = [
                "cargo", "run", "-q", "--release", "-p", "zt-stt", "--features", "sherpa",
                "--example", "smoke", "--",
                "--name", mid, "--dir", mdir,
                "--encoder", roles["encoder"], "--decoder", roles["decoder"],
                "--joiner", roles["joiner"], "--tokens", roles["tokens"],
                "--wav", os.path.join(cache, "wavs", wav + ".wav"),
                "--expect", expect,
            ]
            if hot:
                cmd += ["--hotwords", hot]
            if legacy:
                cmd.append("--legacy-check")
            print("::group::" + mid + " / " + wav, flush=True)
            rc = subprocess.call(cmd, cwd=ROOT)
            print("::endgroup::", flush=True)
            if rc != 0:
                failed.append(f"{mid}/{wav} (exit {rc})")
    if failed:
        raise SystemExit("FAILED: " + ", ".join(failed))


if __name__ == "__main__":
    main()
