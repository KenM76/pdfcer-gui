#!/usr/bin/env python3
"""A `wanted` row of ``ENGINE_BACKLOG.md`` whose symbol the shell now calls.

WHAT THIS MEASURES THAT THE OTHER TWO INSTRUMENTS CANNOT
========================================================

``ENGINE_BACKLOG.md`` has two guards already and they share one blind spot,
which both of them state in their own words:

* ``tools/gates/check-engine-backlog.sh`` fails when a capability in the
  engine's ``FEATURES.md`` is discussed nowhere in the register.  It does not
  judge the verdict.
* ``tools/walk-engine-backlog.py`` counts the rows under each heading.  It
  counts a row "by the section it SITS IN, never by the verdict its body
  states".

So both prove a capability is **accounted for** and neither proves the account
is **true**.  A row can say `wanted` — *the engine has it, an operator would
use it, nobody has filed it* — for as long as anybody likes after the shell
started calling the thing, and every gate stays green, because the only
evidence that would contradict it lives in the Rust and nothing reads both.

**A register wrong in this direction is worse than one that is merely
incomplete: it schedules work already done, and it under-reports the program
to the person who paid for it.**  `FEATURES.md` reads this register, so a
stale `wanted` propagates into the acceptance contract.

THE HIT RULE, AND WHY IT IS SHAPED LIKE THIS
============================================

A bare grep on a symbol's last path segment is useless here, and the register
records why in its own prose: every ``page_objects`` hit in this repository is
the shell's own ``OpenDoc::page_objects``, never the engine's
``EditSession::page_objects``.  An absence claim has to name the receiver.  A
bare-name rule additionally drowns in one-word symbols — ``state``, ``style``,
``Standard`` — whose hit counts run to four figures, and a gate with that
signal-to-noise ratio is switched off within a week.

So an occurrence counts only when it is shaped like **consumption of an
engine symbol**:

``.IDENT``
    a method call or a **field read**.  The field form is not an afterthought:
    an engine report is consumed by reading its fields — ``report.appearance_lines``
    — and a call-shaped rule sees none of that.

``::IDENT``
    a path-qualified use — an associated function, an enum variant, a
    constructor.

bare ``IDENT`` in a file that imports ``IDENT`` from an engine crate
    ``MediaBoxChange`` is reached as ``&[MediaBoxChange]`` and never with a
    ``::`` or a ``.`` in front of it.  The per-file import set is what tells
    that apart from a shell type of the same name.

What survives is confined to rows that *cite* a wired symbol while being
*about* an absent one, and the per-symbol exemption below is for exactly those.
No hit count is quoted in this header: the remedy a hit asks for is an edit to
the register, so any figure written here recommends its own obsolescence.  Run
``--list`` for the live count.

THE EXEMPTION IS PER SYMBOL, NEVER PER ROW
==========================================

A row may exempt one symbol with a token naming it::

    <!--namesake:edit_text--> the row is about `run_repertoire`; `edit_text`
    is cited as the verb this would precede.

A row-wide exemption would be the cheaper thing to build and it is the wrong
shape: rows here name three and four symbols, and muting the row to silence
one of them hides the day the other three get wired.  The token names its
symbol; every other symbol in the row keeps firing.  A count of exempted
symbols is printed on a clean run, because an exemption nobody can see is a
rule quietly narrowing.

SCOPE, WHICH IS A CLAIM AND IS STATED SO IT CAN BE CHECKED
==========================================================

Scanned: ``crates/pdfcer-gui/src/**/*.rs`` and ``crates/pdfcer-gui-base/src/**/*.rs``,
comment lines excluded. Base is in scope because it calls the engine too
(``ocr``, ``acrobat``): a call site there is operator reach like any other.

**Not** scanned, deliberately: ``tools/ui-verify`` — a call site there is a
driven check, not operator reach, and treating it as evidence would let a
harness written for a gap close the row describing it.  **Not** scanned:
``crates/egui-shell`` — R7 forbids it naming a pdfcer crate at all.  **Not**
scanned: ``tests/`` — the register's own `shipped` standard is a call site
"reached through this shell's funnel", and an integration test is not the
funnel.  ``split_text_object`` is the live example: called from a probe test
and nowhere in ``src``, and correctly still `wanted`.

WHAT A HIT IS AND IS NOT
========================

A hit says **go and re-read this row**.  It does not say the row is wrong.  A
call site behind a condition nothing sets is dead in the running program, and
only ``tools/ui-verify`` answers that — which is the same caveat the register
attaches to its own `shipped` verdict.

USAGE
=====

    python tools/check-backlog-verdict-drift.py              # exit 1 on drift
    python tools/check-backlog-verdict-drift.py --list       # report, exit 0
    python tools/check-backlog-verdict-drift.py --self-test  # falsify the rule

Exit: 0 clean, 1 a row to re-read, 2 the register or the shell tree is absent.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

REGISTER = "ENGINE_BACKLOG.md"
SHELL_SRC = os.path.join("crates", "pdfcer-gui", "src")
#: Every GUI crate whose source is operator reach. `SHELL_SRC` must exist; the
#: others are scanned when present, so a self-test plant needs only the first.
SHELL_SRCS = (SHELL_SRC, os.path.join("crates", "pdfcer-gui-base", "src"))

# Sections whose rows assert an ABSENCE. A `shipped` or `declined` row makes no
# claim this file can falsify, and `unknown` says so on its face.
ABSENCE_SECTIONS = ("wanted", "blocked")

USE_RE = re.compile(r"use\s+(?:pdfcer_(?:core|render|print))\b([^;]*);", re.S)
NAMESAKE_RE = re.compile(r"<!--\s*namesake:([A-Za-z_][A-Za-z0-9_]*)\s*-->")
SECTION_RE = re.compile(r"^## `([a-z]+)`")
BACKTICK_RE = re.compile(r"`([^`]+)`")
SYMBOL_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_:]*$")
ARGS_RE = re.compile(r"\([^()]*\)\s*$")
GROUP_RE = re.compile(r"([A-Za-z_][A-Za-z0-9_:]*::)\{([^{}]*)\}")

MIN_IDENT = 6  # shorter names are namesakes far more often than they are hits


def engine_imports(text: str) -> set[str]:
    """Identifiers this file imports from an engine crate, braces included."""
    out: set[str] = set()
    for m in USE_RE.finditer(text):
        out.update(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", m.group(1)))
    return out


def load_shell(root: str) -> dict[str, tuple[set[str], list[tuple[int, str]]]]:
    files: dict[str, tuple[set[str], list[tuple[int, str]]]] = {}
    walked = [
        w for src in SHELL_SRCS for w in os.walk(os.path.join(root, src))
    ]
    for base, _dirs, names in walked:
        for fn in names:
            if not fn.endswith(".rs"):
                continue
            p = os.path.join(base, fn)
            with open(p, encoding="utf-8", errors="replace") as fh:
                text = fh.read()
            rel = os.path.relpath(p, root).replace(os.sep, "/")
            lines = [
                (i, ln)
                for i, ln in enumerate(text.splitlines(), 1)
                if not ln.lstrip().startswith("//")
            ]
            files[rel] = (engine_imports(text), lines)
    return files


def rows(path: str) -> list[tuple[int, str, str, str]]:
    """`(line, section, first cell, whole row)` for every row in an absence section."""
    out: list[tuple[int, str, str, str]] = []
    section = None
    with open(path, encoding="utf-8") as fh:
        for n, ln in enumerate(fh.read().splitlines(), 1):
            m = SECTION_RE.match(ln)
            if m:
                section = m.group(1)
                continue
            s = ln.lstrip()
            if section not in ABSENCE_SECTIONS or not s.startswith("|"):
                continue
            if s.startswith("|--"):
                continue
            cells = s.split("|")
            if len(cells) < 3:
                continue
            first = cells[1].strip()
            if first.startswith("Row (") or set(first) <= set("- :"):
                continue
            out.append((n, section, first, s))
    return out


def candidates(cell: str) -> set[str]:
    """Engine identifiers named in a row's capability cell.

    Takes the last path segment of every backticked symbol, keeping only names
    distinctive enough to be worth grepping: snake_case with an underscore, or
    CamelCase with two capitals. Both bars exist to keep `state` and `style`
    out, and both were chosen by measuring the false-positive count.

    Two normalisations happen before the bar is applied, and each one recovered
    a row that had been silently invisible:

    * a trailing argument list is dropped, so ``with_duplicate_keys(policy)``
      is read as ``with_duplicate_keys``.  Rows in this register are written
      for a human and name a verb's parameters when the parameters are the
      point, which is exactly when the row is worth checking.
    * a brace group is expanded, so ``DuplicateKeyPolicy::{KeepLast, Refuse}``
      yields both variants.  Rust's own import syntax, used here for the same
      reason Rust uses it.
    """
    out: set[str] = set()
    for tok in BACKTICK_RE.findall(cell):
        for piece in expand(tok.strip()):
            ident = piece.split("::")[-1]
            if len(ident) < MIN_IDENT:
                continue
            snake = "_" in ident
            camel = ident[0].isupper() and sum(1 for c in ident if c.isupper()) >= 2
            if snake or camel:
                out.add(ident)
    return out


def expand(tok: str) -> list[str]:
    """One backticked token to the zero or more paths it names."""
    tok = ARGS_RE.sub("", tok).strip()
    m = GROUP_RE.fullmatch(tok)
    if m:
        stem, inner = m.group(1), m.group(2)
        return [
            stem + part.strip()
            for part in inner.split(",")
            if SYMBOL_RE.fullmatch(part.strip())
        ]
    return [tok] if SYMBOL_RE.fullmatch(tok) else []


def hits(ident: str, files) -> list[str]:
    esc = re.escape(ident)
    used = re.compile(r"[.]" + esc + r"\b|::" + esc + r"\b")
    bare = re.compile(r"\b" + esc + r"\b")
    found: list[str] = []
    for rel, (imports, lines) in sorted(files.items()):
        imported = ident in imports
        for n, ln in lines:
            if used.search(ln) or (imported and bare.search(ln)):
                found.append(f"{rel}:{n}: {ln.strip()[:100]}")
    return found


def check(root: str, listing: bool) -> int:
    register = os.path.join(root, REGISTER)
    if not os.path.isfile(register) or not os.path.isdir(os.path.join(root, SHELL_SRC)):
        print("backlog-verdict-drift: SKIPPED — no register or no shell source.")
        return 2

    files = load_shell(root)
    flagged: list[str] = []
    exempt = 0

    for line, section, first, whole in rows(register):
        muted = set(NAMESAKE_RE.findall(whole))
        for ident in sorted(candidates(first)):
            if ident in muted:
                exempt += 1
                continue
            found = hits(ident, files)
            if not found:
                continue
            title = re.sub(r"\s+", " ", first)[:88]
            flagged.append(
                f"  {REGISTER}:{line} `{section}` — {title}\n"
                f"      `{ident}` has {len(found)} call site(s) in the shell:\n"
                + "".join(f"        {one}\n" for one in found[:3])
            )

    if flagged:
        print(f"backlog-verdict-drift: {len(flagged)} row/symbol pair(s) to re-read.")
        print("")
        for one in flagged:
            print(one)
        print(
            "A hit means the register asserts an absence the source contradicts.\n"
            "Re-read the row: move it to `shipped` with the call site named, narrow\n"
            "it to the part still missing, or — if the row is about a different\n"
            "symbol and merely cites this one — add `<!--namesake:IDENT-->` to the\n"
            "row with the reason beside it."
        )
        return 0 if listing else 1

    print(
        f"backlog-verdict-drift: clean — {len(rows(register))} absence row(s), "
        f"{exempt} exempted symbol(s), none contradicted by the shell's source."
    )
    return 0


# ---------------------------------------------------------------------------
# --self-test — seven plants: five the rule must catch, two it must leave
# alone. The two green plants are the ones that keep it honest: a rule with no
# green plant beside it is widened by whoever next finds it too quiet.
#
# Two of the red plants exist because the rule was once blind to them, and each
# blindness cost a real row: a verb written with its parameter named
# (`with_duplicate_keys(policy)`) and a brace group (`Policy::{KeepLast,
# Refuse}`) both failed a symbol regex anchored at end-of-token, so three rows
# naming shipped verbs were invisible to the instrument built to find them.
# ---------------------------------------------------------------------------
def plant(tmp: str, register_body: str, rust: str) -> str:
    root = tempfile.mkdtemp(dir=tmp)
    src = os.path.join(root, SHELL_SRC)
    os.makedirs(src)
    with open(os.path.join(root, REGISTER), "w", encoding="utf-8") as fh:
        fh.write(register_body)
    with open(os.path.join(src, "planted.rs"), "w", encoding="utf-8") as fh:
        fh.write(rust)
    return root


HEAD = "# register\n\n## `wanted` — a real gap — **1 of 1**\n\n| Row | Verdict |\n| --- | --- |\n"


def self_test() -> int:
    calls = "fn f(s: &mut S) { s.set_residual_scope(x); }\n"
    commented = "// s.set_residual_scope(x) is what we would call\n"
    row = "| **Scope it** — `EditSession::set_residual_scope` | **wanted.** |\n"
    muted = (
        "| **Scope it** — `EditSession::set_residual_scope` "
        "<!--namesake:set_residual_scope--> ours, not theirs | **wanted.** |\n"
    )
    shipped = (
        "# register\n\n## `shipped` — stale — **1 of 1**\n\n| Row | Verdict |\n"
        "| --- | --- |\n" + row.replace("**wanted.**", "**shipped.**")
    )

    # A verb the register writes with its parameter named, and a brace group
    # where only the SECOND variant is called — the two shapes a token regex
    # anchored at end-of-token silently drops.
    argrow = "| **Scope it** — `EditSession::set_residual_scope(scope)` | **wanted.** |\n"
    grouprow = "| **Pick one** — `Reach::{MarkedOnly, HiddenCarriers}` | **wanted.** |\n"
    grouped = "fn f() { let s = Reach::HiddenCarriers; }\n"

    cases = [
        ("a called symbol in a wanted row", 1, HEAD + row, calls),
        ("a verb written with its parameter named", 1, HEAD + argrow, calls),
        ("the second variant of a brace group", 1, HEAD + grouprow, grouped),
        ("the same symbol only in a comment", 0, HEAD + row, commented),
        ("an exempted symbol", 0, HEAD + muted, calls),
        ("a called symbol in a SHIPPED row", 0, shipped, calls),
        ("a register with no absence rows", 0, "# register\n", calls),
    ]

    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name, want, reg, rust in cases:
            root = plant(tmp, reg, rust)
            out = []
            keep, sys.stdout = sys.stdout, open(os.devnull, "w", encoding="utf-8")
            try:
                got = check(root, listing=False)
            finally:
                sys.stdout.close()
                sys.stdout = keep
            if got != want:
                print(f"  SELF-TEST FAILED: {name} — wanted rc={want}, got rc={got}")
                bad += 1
            out.clear()

    if bad:
        print(f"backlog-verdict-drift --self-test: {bad} of {len(cases)} plants wrong.")
        return 1
    print(
        f"backlog-verdict-drift --self-test: clean — all {len(cases)} plants behaved "
        "(called, parameterised, brace group, commented-out, exempted, "
        "wrong-section, empty register)."
    )
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--list", action="store_true", help="report but exit 0")
    ap.add_argument("--self-test", action="store_true", help="falsify the rule")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    return check(ROOT, args.list)


if __name__ == "__main__":
    sys.exit(main())
