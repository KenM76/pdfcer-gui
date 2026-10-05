#!/usr/bin/env python3
"""check-ink-roster: the render notes name every engine DIVERGENCE counter.

`app::status::notes::ink::ROSTER` is a hand-written list of the render
counters the engine's metrics table (the `//!` header of
`crates/pdfcer-cli/src/main.rs`, at the pinned revision) labels DIVERGENCE.
The engine offers no enumeration of them, so this gate is the tripwire: it
reads the table at the revision `Cargo.lock` pins and fails when a key is in
one list and not the other.

Exit: 0 clean, 1 the lists disagree, 2 SKIP (no engine clone, or git cannot
produce the pinned file). `--self-test` plants both disagreements.
"""

from __future__ import annotations

import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import engine_path  # noqa: E402

OURS = ROOT / "crates/pdfcer-gui/src/app/status/notes/ink.rs"
LOCK = ROOT / "Cargo.lock"
TABLE = "crates/pdfcer-cli/src/main.rs"

ROW = re.compile(r"^//! \| `([a-z0-9_]+)` \|(.*)$")
KEY = re.compile(r'key: "([a-z0-9_]+)"')


def divergence_keys(table: str) -> set[str]:
    """Metrics keys whose table row carries the upper-case word DIVERGENCE."""
    out = set()
    for line in table.splitlines():
        m = ROW.match(line)
        if m and re.search(r"\bDIVERGENCE\b", m.group(2)):
            out.add(m.group(1))
    return out


def roster_keys(source: str) -> set[str]:
    return set(KEY.findall(source))


def compare(engine: set[str], ours: set[str]) -> list[str]:
    problems = []
    for k in sorted(engine - ours):
        problems.append(f"engine labels `{k}` DIVERGENCE; the render notes never report it")
    for k in sorted(ours - engine):
        problems.append(f"the render notes report `{k}`, which the engine's table does not label DIVERGENCE")
    return problems


def locked_revision() -> str | None:
    for line in LOCK.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith('source = "git+file:') and "#" in line:
            return line.rsplit("#", 1)[1].rstrip('"')
    return None


def self_test() -> int:
    table = (
        "//! | `alpha` | `alpha` | \"q\" (DIVERGENCE. x) |\n"
        "//! | `beta` | `beta` | \"q\" (census, a divergence in lower case) |\n"
        "//! | `gamma` | `g.h` | \"q\" (DIVERGENCE) |\n"
    )
    engine = divergence_keys(table)
    ok = engine == {"alpha", "gamma"}
    ok &= compare(engine, {"alpha", "gamma"}) == []
    ok &= len(compare(engine, {"alpha"})) == 1
    ok &= len(compare(engine, {"alpha", "gamma", "beta"})) == 1
    ok &= roster_keys('InkNote { key: "alpha", read: |d| d.x') == {"alpha"}
    print("check-ink-roster --self-test: " + ("PASS" if ok else "FAIL")
          + " - catches a missing key and an extra one; ignores lower-case prose.")
    return 0 if ok else 1


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return self_test()
    repo = engine_path.locate(ROOT)
    rev = locked_revision()
    if repo is None or rev is None:
        print("SKIP: check-ink-roster - no engine clone or no pinned revision.")
        return 2
    p = subprocess.run(["git", "-C", str(repo), "show", f"{rev}:{TABLE}"], capture_output=True)
    if p.returncode != 0:
        print(f"SKIP: check-ink-roster - git cannot show {TABLE} at {rev[:8]}.")
        return 2
    engine = divergence_keys(p.stdout.decode("utf-8", errors="replace"))
    if not engine:
        print(f"FAIL: check-ink-roster - no DIVERGENCE rows parsed from {TABLE}; the table's shape changed.")
        return 1
    ours = roster_keys(OURS.read_text(encoding="utf-8"))
    problems = compare(engine, ours)
    if problems:
        print("FAIL: check-ink-roster")
        for line in problems:
            print("  " + line)
        print("Add or remove the counter in app::status::notes::ink::ROSTER with its catalog phrase.")
        return 1
    print(f"check-ink-roster: clean - {len(ours)} DIVERGENCE counters, all reported.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
