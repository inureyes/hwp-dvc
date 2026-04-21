#!/usr/bin/env python3
"""Scrub personal metadata from HWPX fixture files.

한컴오피스 saves the author, title, and last-save identity into
`Contents/content.hpf`. When test fixtures are authored by a real
user with "저장 시 개인 정보 제거" disabled, those strings end up in
the committed archive. This script rewrites each listed HWPX so that:

- `<opf:title>`   is cleared.
- `<opf:meta name="creator">`, `lastsaveby`, `keyword`, `subject`,
  `description`, `date` are cleared or set to a neutral value.
- `CreatedDate` / `ModifiedDate` are pinned to the epoch so git diffs
  remain stable across re-saves.

The repack preserves HWPX conventions: `mimetype` is first and
STORED; everything else is DEFLATED.

Usage:
    python3 scripts/scrub_fixture_metadata.py [file.hwpx ...]

With no arguments, scrubs every `*.hwpx` under
`crates/hwp-dvc-core/tests/fixtures/docs/`.
"""

from __future__ import annotations

import os
import re
import sys
import tempfile
import zipfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_DIR = REPO_ROOT / "crates/hwp-dvc-core/tests/fixtures/docs"

NEUTRAL_CREATOR = "hwp-dvc fixture"
NEUTRAL_DATE_ISO = "1980-01-01T00:00:00Z"
NEUTRAL_DATE_KR = ""

META_TARGETS = {
    "creator": NEUTRAL_CREATOR,
    "lastsaveby": NEUTRAL_CREATOR,
    "subject": "",
    "description": "",
    "keyword": "",
    "date": NEUTRAL_DATE_KR,
    "CreatedDate": NEUTRAL_DATE_ISO,
    "ModifiedDate": NEUTRAL_DATE_ISO,
}


def scrub_hpf(xml: str) -> str:
    """Return HPF XML with personal metadata cleared."""
    xml = re.sub(
        r'<opf:title([^>]*)>[^<]*</opf:title>',
        r'<opf:title\1></opf:title>',
        xml,
    )
    for name, replacement in META_TARGETS.items():
        pattern = (
            rf'(<opf:meta name="{re.escape(name)}"[^>]*>)[^<]*(</opf:meta>)'
        )
        xml = re.sub(pattern, rf'\g<1>{replacement}\g<2>', xml)
    return xml


def repack(work_dir: Path, out_path: Path) -> None:
    if out_path.exists():
        out_path.unlink()
    with zipfile.ZipFile(out_path, "w") as zf:
        mt_path = work_dir / "mimetype"
        zi = zipfile.ZipInfo("mimetype")
        zi.compress_type = zipfile.ZIP_STORED
        zf.writestr(zi, mt_path.read_bytes())
        for root, _dirs, files in os.walk(work_dir):
            for name in sorted(files):
                full = Path(root) / name
                rel = full.relative_to(work_dir).as_posix()
                if rel == "mimetype":
                    continue
                zf.write(full, rel, compress_type=zipfile.ZIP_DEFLATED)


def scrub_file(path: Path) -> bool:
    """Scrub one HWPX. Returns True when the HPF actually changed."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        with zipfile.ZipFile(path) as zf:
            zf.extractall(tmp_path)
        hpf = tmp_path / "Contents" / "content.hpf"
        before = hpf.read_text(encoding="utf-8")
        after = scrub_hpf(before)
        if before == after:
            return False
        hpf.write_text(after, encoding="utf-8")
        repack(tmp_path, path)
    return True


def main(argv: list[str]) -> int:
    targets = [Path(a) for a in argv[1:]]
    if not targets:
        targets = sorted(DEFAULT_DIR.glob("*.hwpx"))
    if not targets:
        print("no HWPX files found", file=sys.stderr)
        return 1
    changed = 0
    for p in targets:
        if not p.exists():
            print(f"skip (missing): {p}", file=sys.stderr)
            continue
        if scrub_file(p):
            changed += 1
            print(f"scrubbed: {p.relative_to(REPO_ROOT)}")
        else:
            print(f"clean:    {p.relative_to(REPO_ROOT)}")
    print(f"done: {changed}/{len(targets)} files changed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
