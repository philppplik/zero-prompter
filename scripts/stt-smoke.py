#!/usr/bin/env python3
"""CI STT smoke test: fetch models from models.json (SHA-256 verified),
fetch pinned test WAVs, run the prebuilt smoke example per model (build it first:
`cargo build --release -p zt-stt --features sherpa --example smoke`), print a
comparison table (load time, RTF, chunk latency, partial cadence, time to first
word, peak RSS via psutil if installed) to the log and the step summary.

usage: stt-smoke.py <cache_dir> <model_id>...   (cargo is run from the repo root)
"""
import hashlib
import json
import os
import re
import subprocess
import sys
import threading
import time
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

NEMO = "nemotron-3.5-streaming-0.6b-160ms-int8"
NEMO320 = "nemotron-3.5-streaming-0.6b-320ms-int8"

# model id -> list of (wav, expected keywords, hotwords, legacy-check, language)
# Nemotron: no hotwords (sherpa-onnx 1.13.8 streaming NeMo = greedy_search only).
CASES = {
    NEMO: [
        ("de-kroko-0", "Ende,Wurst,zwei", "", False, "de"),
        ("en-librispeech-0", "yellow,lamps,quarter", "", False, "en"),
    ],
    NEMO320: [
        ("de-kroko-0", "Ende,Wurst,zwei", "", False, "de"),
        ("en-librispeech-0", "yellow,lamps,quarter", "", False, "en"),
    ],
    "zipformer-de-kroko-2025-08-06": [
        ("de-kroko-0", "Ende,Wurst,zwei", "Teleprompter,Mikrofon", True, None),
    ],
    "zipformer-en-kroko-2025-08-06": [
        ("en-librispeech-0", "yellow,lamps,quarter", "nightfall,brothels", True, None),
        ("en-kroko-0", "country", "", False, None),
    ],
    "zipformer-en-20m-2023-02-17": [
        ("en-librispeech-0", "yellow,lamps,quarter", "nightfall,brothels", True, None),
    ],
}
# ONNX Runtime threads (the app uses the same: 4 for Nemotron, 2 otherwise).
THREADS = {NEMO: 4, NEMO320: 4}  # informational; SherpaConfig::nemotron already uses 4
EXE = os.path.join(ROOT, "target", "release", "examples", "smoke" + (".exe" if os.name == "nt" else ""))

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
    rows = []
    for mid in ids:
        m = models[mid]
        mdir = os.path.join(cache, mid)
        roles = {}
        for f in m["files"]:
            fetch(f["url"], os.path.join(mdir, f["path"]), f["sha256"], f.get("size"))
            roles[f["role"]] = f["path"]
        for wav, expect, hot, legacy, lang in CASES[mid]:
            cmd = [
                EXE,
                "--name", mid, "--dir", mdir,
                "--encoder", roles["encoder"], "--decoder", roles["decoder"],
                "--joiner", roles["joiner"], "--tokens", roles["tokens"],
                "--wav", os.path.join(cache, "wavs", wav + ".wav"),
                "--expect", expect,
                "--threads", str(THREADS.get(mid, 2)),
            ]
            if hot:
                cmd += ["--hotwords", hot]
            if legacy:
                cmd.append("--legacy-check")
            if lang:
                cmd += ["--language", lang]
            print("::group::" + mid + " / " + wav, flush=True)
            rc, out, peak = run(cmd)
            print("::endgroup::", flush=True)
            row = parse(out)
            row.update(case=wav, peak_rss_mb=peak, rc=rc)
            rows.append(row)
            if rc != 0:
                failed.append(f"{mid}/{wav} (exit {rc})")
    report(rows)
    if failed:
        raise SystemExit("FAILED: " + ", ".join(failed))


def run(cmd):
    """Runs the smoke binary, echoes its output, samples the peak RSS."""
    proc = subprocess.Popen(cmd, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                            text=True, encoding="utf-8", errors="replace")
    peak = [0]

    def sample():
        try:
            import psutil
            p = psutil.Process(proc.pid)
            while proc.poll() is None:
                mi = p.memory_info()
                peak[0] = max(peak[0], getattr(mi, "peak_wset", 0), mi.rss)
                time.sleep(0.05)
        except Exception:
            pass

    t = threading.Thread(target=sample, daemon=True)
    t.start()
    lines = []
    for line in proc.stdout:
        print(line, end="", flush=True)
        lines.append(line)
    rc = proc.wait()
    t.join(timeout=1)
    return rc, "".join(lines), round(peak[0] / 2**20) if peak[0] else None


def parse(out):
    row = {}
    for line in out.splitlines():
        if line.startswith("SMOKE "):
            for kv in re.findall(r"(\w+)=(\[[^\]]*\]|\S+)", line):
                row[kv[0]] = kv[1]
        elif line.startswith("TEXT: "):
            row["text"] = line[6:].strip()
    return row


def report(rows):
    head = ["model", "case", "platform", "threads", "load ms", "RTF", "chunk p95 ms",
            "cadence p50/p95 s", "partials", "first word s", "peak RSS MB", "ok", "text"]
    lines = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    for r in rows:
        g = lambda k: str(r.get(k, "-"))
        lines.append("| " + " | ".join([
            g("model"), g("case"), g("platform"), g("threads"), g("create_ms"), g("rtf"), g("p95"),
            g("cadence_s_p50") + " / " + g("cadence_s_p95"), g("partials"), g("first_word_s"),
            g("peak_rss_mb"), g("ok"), g("text")[:120],
        ]) + " |")
    table = "\n".join(lines)
    print("\n=== STT comparison ===\n" + table, flush=True)
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as f:
            f.write("\n### STT smoke comparison\n\n" + table + "\n")

if __name__ == "__main__":
    main()
