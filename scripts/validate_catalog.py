#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "docs" / "catalog.json"
ID_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
SHA_RE = re.compile(r"^[0-9a-fA-F]{64}$")
STATUSES = {"stable", "experimental", "blocked"}

def fail(message):
    print(f"catalog: {message}", file=sys.stderr)
    return False

def https_url(value):
    try:
        parsed = urlparse(value)
        return parsed.scheme == "https" and bool(parsed.netloc)
    except Exception:
        return False

def main():
    data = json.loads(CATALOG.read_text())
    ok = True
    if data.get("schema") != 1:
        ok &= fail("schema must be 1")
    mods = data.get("mods")
    if not isinstance(mods, list):
        fail("mods must be an array")
        return 1

    ids = set()
    required = {
        "id", "name", "version", "author", "summary", "description",
        "category", "loader", "game_build", "download_url", "sha256"
    }
    for index, mod in enumerate(mods):
        prefix = f"mods[{index}]"
        missing = sorted(required - set(mod))
        if missing:
            ok &= fail(f"{prefix}: missing {', '.join(missing)}")
            continue

        mod_id = mod["id"]
        if not isinstance(mod_id, str) or not ID_RE.fullmatch(mod_id):
            ok &= fail(f"{prefix}: invalid id {mod_id!r}")
        if mod_id in ids:
            ok &= fail(f"{prefix}: duplicate id {mod_id!r}")
        ids.add(mod_id)

        if not https_url(mod["download_url"]):
            ok &= fail(f"{prefix}: download_url must use HTTPS")
        if not isinstance(mod["sha256"], str) or not SHA_RE.fullmatch(mod["sha256"]):
            ok &= fail(f"{prefix}: sha256 must contain exactly 64 hex characters")

        status = mod.get("status", "stable")
        if status not in STATUSES:
            ok &= fail(f"{prefix}: status must be one of {sorted(STATUSES)}")
        if status == "blocked" and not mod.get("status_note"):
            ok &= fail(f"{prefix}: blocked mods require status_note")

        for shot in mod.get("screenshots", []):
            if not isinstance(shot, str) or shot.startswith("/") or "://" in shot:
                ok &= fail(f"{prefix}: screenshots must be relative site paths")

    if ok:
        suffix = "s" if len(mods) != 1 else ""
        print(f"catalog: OK ({len(mods)} mod{suffix})")
        return 0
    return 1

if __name__ == "__main__":
    raise SystemExit(main())
