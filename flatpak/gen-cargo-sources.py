#!/usr/bin/env python3
"""Generates flatpak/cargo-sources.json from Cargo.lock (crates.io packages only).

Flatpak builds run without network access, so every crate is listed as a source:
flatpak-builder downloads (and checksums) the .crate archives up front, unpacks them
into cargo/vendor/, and a cargo config makes cargo build from that directory offline.

Usage: python3 flatpak/gen-cargo-sources.py [--check]   (re-run after Cargo.lock changes)
"""

import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "flatpak" / "cargo-sources.json"
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"
VENDOR = "cargo/vendor"


def sources() -> list[dict]:
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    out = []
    for pkg in lock["package"]:
        source = pkg.get("source")
        if source is None:
            continue  # rvp itself
        if source != REGISTRY:
            sys.exit(f"unsupported source for {pkg['name']}: {source}")
        name, version, checksum = pkg["name"], pkg["version"], pkg["checksum"]
        dest = f"{VENDOR}/{name}-{version}"
        out.append({
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": f"https://static.crates.io/crates/{name}/{name}-{version}.crate",
            "sha256": checksum,
            "dest": dest,
        })
        # cargo checks vendored crates against this file; the package checksum is enough.
        out.append({
            "type": "inline",
            "contents": json.dumps({"package": checksum, "files": {}}),
            "dest": dest,
            "dest-filename": ".cargo-checksum.json",
        })
    out.append({
        "type": "inline",
        "contents": (
            "[source.vendored-sources]\n"
            f'directory = "{VENDOR}"\n\n'
            "[source.crates-io]\n"
            'replace-with = "vendored-sources"\n'
        ),
        "dest": "cargo",
        "dest-filename": "config.toml",
    })
    return out


def main() -> None:
    text = json.dumps(sources(), indent=4) + "\n"
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text() != text:
            sys.exit("flatpak/cargo-sources.json is out of date: run python3 flatpak/gen-cargo-sources.py")
        return
    OUT.write_text(text)
    print(f"wrote {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
