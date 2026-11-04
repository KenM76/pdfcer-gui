#!/usr/bin/env python3
"""affected-checks.py — the ui-verify checks a change since a git ref can affect.

Between releases only these are driven (OPERATOR_REQUESTS O305); the full
sweep is for framework-wide changes or when asked.

A check is selected when any of these holds:

1. its own source file changed;
2. its source quotes, as a string literal, an identifier that a changed
   source file also quotes: a trace event (`canvas-form-shade`), a region
   (`ribbon.item.edit.insert_3d`), a command id (`edit.attachments`) or an
   environment variable (`PDFCER_DIAG_VIEWPORT`). Those literals are the only
   contract between a check and the program, so sharing one is what "this
   change can reach this check" means here;
3. it is in SMOKE, which is always driven.

An engine pin moved since REF counts as change too: every engine crate with a
source file changed between the two pins (`pdfcer-3d/src/raster.rs` is
`pdfcer_3d`) makes every program file naming that crate changed, so rule 2
reaches the checks of whatever calls it. The crate, not the module, because
engine crates re-export their modules' types at the root (`pdfcer_3d::Bounds`
is `bounds::Bounds`), so a caller need not name the module it reaches. A crate
the program never names (`pdfcer_cli`) reaches nothing. A pin this cannot read
selects every check, as `--all` does.

Usage:
    python tools/ui-verify/affected-checks.py [--since REF] [--why]

REF defaults to the newest release tag (`git describe --tags --abbrev=0`).
Uncommitted changes count. Prints one check name per line; `--why` prints the
reason for each to stderr. Exit 0 always when git answers; 2 when it does not.
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CHECKS = ROOT / "tools" / "ui-verify" / "src" / "checks"
# Launch, title, open and the off-screen pointer: what every release must do.
SMOKE = [
    "the_title_bar_carries_the_build_time",
    "read_mode_says_how_to_get_back_out",
    "fillable_fields_are_shaded_on_the_page",
]
# An identifier is lowercase words joined by `.`, `-` or `_` with at least one
# joiner, or an upper-case env var; bare words match too much to mean anything.
IDENT = re.compile(r'"([a-z][a-z0-9]*(?:[.\-][a-z0-9_]+)+|PDFCER_[A-Z0-9_]+)"')
# Literals so common that sharing one says nothing about reach.
COMMON = {"ui-rect", "ui-rect-visible", "frame-phase", "PDFCER_DIAG",
          "PDFCER_DIAG_VIEWPORT", "PDFCER_DIAG_INVOKE", "pdfcer-diag"}
NAME = re.compile(r'fn name\(&self\)[^{]*\{\s*"([a-z0-9_]+)"')


def git(*args):
    out = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"affected-checks: git {' '.join(args)} failed: {out.stderr.strip()}")
    return out.stdout


# `source = "git+file:///D:/Dev/pdfcer?branch=main#<commit>"` in Cargo.lock.
PIN = re.compile(r'source = "git\+file:///([^?"]+)\?[^#"]*#([0-9a-f]{40})"')
ENGINE_SRC = re.compile(r"^crates/([a-z0-9\-]+)/src/(.+)\.rs$")


def engine_pin(lock_text):
    pins = set(PIN.findall(lock_text))
    return pins.pop() if len(pins) == 1 else None


def engine_touched(since):
    """Engine crates changed between the pins, `None` when every check must
    run, `[]` when the pin did not move."""
    old = subprocess.run(["git", "show", f"{since}:Cargo.lock"], cwd=ROOT,
                         capture_output=True, text=True)
    new = engine_pin((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    if old.returncode != 0 or new is None:
        return None
    old = engine_pin(old.stdout)
    if old is None:
        return None
    if old == new:
        return []
    repo = new[0] if new[0][1:2] == ":" else new[0].lstrip("/")
    diff = subprocess.run(["git", "-C", repo, "diff", "--name-only", f"{old[1]}..{new[1]}"],
                          capture_output=True, text=True)
    if diff.returncode != 0:
        return None
    crates = set()
    for name in diff.stdout.split():
        m = ENGINE_SRC.match(name)
        if m and "/tests" not in name and not name.endswith("tests.rs"):
            crates.add(m.group(1).replace("-", "_"))
    return sorted(crates)


def engine_callers(crates):
    """Program files (not the harness) naming a changed engine crate."""
    hits = {}
    for path in (ROOT / "crates").rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="replace")
        for crate in crates:
            if re.search(rf"\b{crate}\b", text):
                hits[path.relative_to(ROOT).as_posix()] = crate
                break
    return hits


def changed_files(since):
    names = set(git("diff", "--name-only", f"{since}..HEAD").split())
    names |= set(git("diff", "--name-only", "HEAD").split())
    return sorted(n for n in names if n.endswith(".rs"))


# A file name is a fixture, shared by checks of unrelated subjects.
FILE = re.compile(r"\.(pdf|prc|u3d|png|jpe?g|txt|toml|json|ron|exe|dxf|svg|zip|md)$")


def idents(text):
    return {m for m in IDENT.findall(text) if m not in COMMON and not FILE.search(m)}


def main():
    argv = sys.argv[1:]
    why = "--why" in argv
    since = argv[argv.index("--since") + 1] if "--since" in argv else \
        git("describe", "--tags", "--abbrev=0").strip()
    changed = changed_files(since)
    touched = engine_touched(since)
    if touched is None or "--all" in argv:
        for path in sorted(CHECKS.rglob("*.rs")):
            for name in NAME.findall(path.read_text(encoding="utf-8", errors="replace")):
                print(name)
        print("affected-checks: every check (the engine's reach cannot be bounded)",
              file=sys.stderr)
        return
    via_engine = engine_callers(touched)
    changed = sorted(set(changed) | via_engine.keys())

    checks = {}  # path -> (names, idents)
    for path in CHECKS.rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="replace")
        names = NAME.findall(text)
        if names:
            checks[path] = (names, idents(text))

    reach = {}  # ident -> changed file that quotes it
    own = set()
    for rel in changed:
        path = ROOT / rel
        if not path.is_file():
            continue
        if path in checks:
            own.add(path)
            continue
        if "tools/ui-verify" in rel:
            continue  # harness plumbing: reaches every check, not one
        for ident in idents(path.read_text(encoding="utf-8", errors="replace")):
            reach.setdefault(ident, rel)

    picked = {}
    for name in SMOKE:
        picked[name] = "always driven"
    for path, (names, ids) in checks.items():
        rel = path.relative_to(ROOT).as_posix()
        reason = None
        if path in own:
            reason = f"its source {rel} changed"
        else:
            shared = sorted(ids & reach.keys())
            if shared:
                reason = f'quotes "{shared[0]}", as {reach[shared[0]]} does'
        if reason:
            for name in names:
                picked.setdefault(name, reason)

    for name in sorted(picked):
        print(name)
        if why:
            print(f"{name}: {picked[name]}", file=sys.stderr)
    print(f"affected-checks: {len(picked)} of {sum(len(n) for n, _ in checks.values())} "
          f"checks, {len(changed)} changed .rs files since {since} "
          f"({len(via_engine)} calling the changed engine crates {', '.join(touched) or 'none'})",
          file=sys.stderr)


if __name__ == "__main__":
    main()
