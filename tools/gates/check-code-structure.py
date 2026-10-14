#!/usr/bin/env python3
"""Structure gate: long functions, long files, copy-pasted helpers.

Measures production Rust under `crates/` and every `tools/<crate>/` (a file's
text before its first `#[cfg(test)]`; `tests/`, `benches/`, `fuzz/`,
`examples/` and `target/` are skipped, as is a `tests.rs` or `*_tests.rs`
file, which this repository only ever includes under `#[cfg(test)]`) for:

* a function longer than MAX_FN_LINES lines;
* a file whose production part is longer than MAX_FILE_LINES lines;
* a private free `fn` with the same name AND body (whitespace ignored) in two
  or more files of one crate -- the copy-paste signature.

`code-structure-baseline.txt` beside this script is DEBT, not an allowlist: a
new violation fails, and so does a baseline line that no longer occurs
(delete it), so the list only shrinks. Never regenerate it to admit a new one.

Usage: check-code-structure.py [--write-baseline | --self-test]
Exit 0 clean, 1 on any new or stale entry.
"""

import argparse
import re
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

MAX_FN_LINES = 80
MAX_FILE_LINES = 800
SKIP_DIRS = {"tests", "benches", "fuzz", "examples", "target", ".git", ".claude"}
FN_RE = re.compile(
    r"^\s*(?:pub(?:\([a-z]+\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)"
)
TEST_FILE = re.compile(r"(^|_)tests?\.rs$")
BASELINE = Path(__file__).resolve().parent / "code-structure-baseline.txt"


def production_lines(text: str) -> list[str]:
    lines = text.split("\n")
    for i, line in enumerate(lines):
        if line.strip().startswith("#[cfg(test)]"):
            return lines[:i]
    return lines


def functions(lines: list[str]):
    """Yield (name, first_line_index, length, body_text) per outermost fn."""
    i = 0
    while i < len(lines):
        m = FN_RE.match(lines[i])
        if not m:
            i += 1
            continue
        depth, started, j = 0, False, i
        while j < len(lines):
            code = lines[j].split("//", 1)[0]
            if not started and ";" in code and "{" not in code:
                break  # a trait method declaration has no body
            depth += code.count("{") - code.count("}")
            if "{" in code:
                started = True
            if started and depth <= 0:
                break
            j += 1
        if started:
            body = "".join("".join(l.split()) for l in lines[i + 1 : j + 1])
            yield m.group(1), i, j - i + 1, body
        i = j + 1


def crate_of(path: Path, root: Path) -> str:
    for parent in path.parents:
        if (parent / "Cargo.toml").exists():
            return parent.relative_to(root).as_posix() or "."
    return "."


def sources(root: Path):
    """Every production `.rs` under the crates this repository owns."""
    tops = [root / "crates"] + [
        d for d in sorted((root / "tools").glob("*")) if (d / "Cargo.toml").is_file()
    ]
    for top in tops:
        if not top.is_dir():
            continue
        for path in sorted(top.rglob("*.rs")):
            if TEST_FILE.search(path.name):
                continue
            if not any(p in SKIP_DIRS for p in path.relative_to(root).parts):
                yield path


def measure(root: Path) -> set[str]:
    found: set[str] = set()
    helpers: dict[tuple[str, str, str], set[str]] = defaultdict(set)
    for path in sources(root):
        name = path.relative_to(root).as_posix()
        lines = production_lines(path.read_text(encoding="utf-8", errors="replace"))
        if len(lines) > MAX_FILE_LINES:
            found.add(f"file {name}")
        for fn, start, length, body in functions(lines):
            if length > MAX_FN_LINES:
                found.add(f"fn {name}::{fn}")
            if lines[start].startswith("fn "):  # impl methods may legitimately repeat
                helpers[(crate_of(path, root), fn, body)].add(name)
    for (crate, fn, _), files in helpers.items():
        if len(files) > 1:
            found.add(f"dup {crate}::{fn} in {', '.join(sorted(files))}")
    return found


def self_test() -> int:
    """Plants one of each violation and a clean file; all three must be found
    and nothing else."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        src = root / "crates" / "c" / "src"
        src.mkdir(parents=True)
        (root / "crates" / "c" / "Cargo.toml").write_text("[package]\n")
        helper = "fn twin(a: u8) -> u8 {\n    a + 1\n}\n"
        long_fn = "fn long() {\n" + "    let _ = 1;\n" * MAX_FN_LINES + "}\n"
        (src / "a.rs").write_text(helper + long_fn, newline="")
        (src / "b.rs").write_text(helper, newline="")
        (src / "big.rs").write_text("const X: u8 = 1;\n" * (MAX_FILE_LINES + 1), newline="")
        (src / "x_tests.rs").write_text(long_fn, newline="")
        tested = "fn ok() {}\n#[cfg(test)]\nmod t {\n" + "    fn f() {}\n" * 900 + "}\n"
        (src / "clean.rs").write_text(tested, newline="")
        found = measure(root)
    want = {
        "fn crates/c/src/a.rs::long",
        "file crates/c/src/big.rs",
        "dup crates/c::twin in crates/c/src/a.rs, crates/c/src/b.rs",
    }
    ok = found == want
    if not ok:
        print(f"self-test: wanted {sorted(want)}, found {sorted(found)}")
    print("self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--write-baseline", action="store_true")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    found = measure(Path(__file__).resolve().parents[2])
    if args.write_baseline:
        BASELINE.write_bytes(("\n".join(sorted(found)) + "\n").encode())
        print(f"code-structure: baseline written, {len(found)} entr(ies)")
        return 0
    baseline = {
        l.strip()
        for l in BASELINE.read_text(encoding="utf-8").splitlines()
        if l.strip() and not l.startswith("#")
    } if BASELINE.exists() else set()
    new, stale = sorted(found - baseline), sorted(baseline - found)
    for entry in new:
        print(f"NEW   {entry}")
    for entry in stale:
        print(f"STALE {entry}  (fixed -- delete this baseline line)")
    if new or stale:
        print(f"code-structure: FAIL -- {len(new)} new, {len(stale)} stale; limits fn "
              f"{MAX_FN_LINES} lines, file {MAX_FILE_LINES} production lines, no "
              "duplicated private helpers")
        return 1
    print(f"code-structure: clean -- {len(found)} baseline entr(ies) of debt, none new")
    return 0


if __name__ == "__main__":
    sys.exit(main())
