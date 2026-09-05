#!/usr/bin/env python3
"""verb-coverage.py — which of `EditSession`'s verbs this shell actually calls.

WHY THIS EXISTS
===============

On 2026-08-28 the operator asked a question this project could not answer from
its own documents:

    "confirm that you have built every editable surface into the GUI that has
     been implemented in pdfcer"

`FEATURES.md` describes what the GUI does. `NO_SURFACE.md` lists tunables with
no control. Neither is keyed on the ENGINE's verb list, so neither could answer
*"is there a verb pdfcer-core implements that nothing here calls?"* — and the
answer turned out to be yes, repeatedly, including for capabilities the engine
had shipped IN ANSWER TO THIS SHELL'S OWN REQUESTS and this shell had then not
consumed.

★★★ The failure mode this closes is specific and this project has now recorded
**seven** instances of it: a blocker written into a doc comment or a backlog
row, true on the day it was written, false within days, and re-read by nobody.
A blocker's reason is prose, and no test can check prose. This is the
instrument that makes the question mechanical instead.

WHAT IT MEASURES, AND WHAT THAT MEASUREMENT IS WORTH
====================================================

For every `pub fn` declared inside an `impl EditSession` block in
`pdfcer-core/src/edit.rs`, whether the identifier appears anywhere in
`crates/pdfcer-gui/src`.

★★ It is a **grep**, and its limits are worth stating plainly because a number
from a tool reads as authoritative:

  - A hit means the NAME appears, not that a reachable operator route calls it.
    A call site behind a condition nothing sets is a hit here and dead in the
    running program. `tools/ui-verify` is the instrument for that question and
    this one cannot answer it.
  - A miss is stronger: no occurrence of the identifier means nothing here
    calls it, full stop. **The miss list is the useful output.**
  - A miss is not automatically a gap. Roughly a third of the misses are
    builder methods on other types that happen to sit in the same file, or
    `*_with` variants of a verb this shell calls in its plain form. The
    register at `EDITABLE_SURFACES.md` carries a hand-written reason per miss;
    this tool produces the list that register must account for.

★★★ IT MEASURES THE LOCKED REVISION, NOT THE ENGINE'S WORKING TREE
==================================================================

The first cut read `D:/Dev/pdfcer/crates/pdfcer-core/src/edit.rs` off disk, and on
2026-08-29 it reported `move_outline_item` and `set_outline_open` as gaps. They
are not gaps. They were **uncommitted work in the engine session's worktree** —
that project runs in parallel and edits its own tree continuously — and the
revision this shell links is whatever `Cargo.lock` pins.

⇒ **A verb in the worktree and not in the lock is not callable**, and a register
that listed it would send the next session to write a call that does not
compile. Worse, it would look like a capability we were behind on.

So the scan reads `git show <locked-rev>:crates/pdfcer-core/src/edit.rs`, and it
reports the difference rather than hiding it: verbs the worktree has and the
lock does not are printed under COMING, so the two facts stay separate —
*"nothing here calls it"* and *"we could not call it if we wanted to."*

USAGE
=====

    python tools/verb-coverage.py                 # the miss list, one per line
    python tools/verb-coverage.py --all           # every verb with its count
    python tools/verb-coverage.py --markdown      # a table for the register

Exit code is 0 always: this is an instrument, not a gate. Making it a gate
would require a checked-in allow-list of "misses that are fine", which is
another prose blocker list, which is the thing it exists to replace.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys

# ★★★ DERIVED, NOT ASSUMED — see `tools/engine_path.py`.
#
# This was a hard-coded literal until 2026-09-03, when the project's rename
# pointed it at a directory the engine had not created yet and this instrument
# went silently blind. `engine_path.locate` reads the git URL out of the
# manifest Cargo actually builds from, so it follows the temporary
# `package = ...` shim and will follow the engine's rename without anybody
# remembering that this line exists.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import engine_path  # noqa: E402

ENGINE_REPO = engine_path.locate() or pathlib.Path("D:/Dev/pdfcer")
ENGINE_FILE = f"crates/{engine_path.crate_name('core')}/src/edit.rs"
LOCK = pathlib.Path("Cargo.lock")
GUI = pathlib.Path("crates/pdfcer-gui/src")


def locked_revision() -> str | None:
    """The `pdfcer-core` commit this workspace's `Cargo.lock` pins.

    The lock names it in the source URL's fragment:

        source = "git+file:///D:/Dev/pdfcer?branch=main#97d445f85f…"

    Read from the lock rather than from `cargo metadata`, which is a process
    spawn and a JSON parse for one hex string that is right there.
    """
    if not LOCK.exists():
        return None
    for line in LOCK.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith('source = "git+file:') and "#" in line:
            return line.rsplit("#", 1)[1].rstrip('"')
    return None


def engine_source(rev: str | None) -> tuple[str, str]:
    """`edit.rs` as of `rev`, and a one-word description of where it came from.

    Falls back to the working tree when there is no revision or git will not
    answer — and says so, because the fallback is the thing this function exists
    to avoid silently doing.
    """
    if rev:
        try:
            out = subprocess.run(
                ["git", "-C", str(ENGINE_REPO), "show", f"{rev}:{ENGINE_FILE}"],
                capture_output=True, check=True,
            )
            return out.stdout.decode("utf-8", errors="replace"), f"lock {rev[:7]}"
        except (OSError, subprocess.CalledProcessError):
            pass
    return (ENGINE_REPO / ENGINE_FILE).read_text(encoding="utf-8", errors="replace"), "WORKTREE"

IMPL = re.compile(r"^impl(?:<[^>]*>)?\s+([A-Za-z0-9_]+)")
# Four-space indent only: a `pub fn` nested deeper is inside a nested item or a
# test module, and `EditSession`'s own verbs are all at one level.
METHOD = re.compile(r"^    pub (?:const )?(?:unsafe )?fn ([a-z0-9_]+)")


def engine_verbs_from(text: str) -> list[str]:
    """Every `pub fn` declared at one indent inside an `impl EditSession`.

    The scan is a state machine over `impl` headers rather than a parse: the
    file is 34,000 lines and holds dozens of impls, so "every `pub fn` in the
    file" — which is what the first cut of this measurement did — sweeps in
    `MarkupNote::new`, `NewTextField::with_value` and forty other builders and
    reports them as uncalled engine verbs.
    """
    current = None
    out: list[str] = []
    for line in text.splitlines():
        m = IMPL.match(line)
        if m:
            current = m.group(1)
            continue
        if current != "EditSession":
            continue
        m = METHOD.match(line)
        if m:
            out.append(m.group(1))
    return sorted(set(out))


# A `//` line comment, to end of line, and a `/* */` block comment.
#
# ⚠ Deliberately naive about `//` inside a string literal — a URL in a `&str`
# swallows the rest of that line. **That direction of error is the safe one:**
# swallowing code can only make a verb look UNCALLED, which turns this gate red
# and sends someone to look. The opposite error — counting prose as a call — is
# the one that hides a capability for days, and it is what this function was
# changed to stop.
LINE_COMMENT = re.compile(r"//[^\n]*")
BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.S)


def strip_comments(text: str) -> str:
    """Blank out comments, preserving length and line structure.

    Each comment becomes the same number of spaces rather than being deleted,
    so the regex that runs next cannot accidentally join two identifiers that
    sat on either side of one.
    """
    text = BLOCK_COMMENT.sub(lambda m: " " * len(m.group(0)), text)
    return LINE_COMMENT.sub(lambda m: " " * len(m.group(0)), text)


def gui_hits(names: list[str], root: pathlib.Path) -> dict[str, int]:
    """How many times each verb is CALLED across the shell's sources.

    One pass over the tree holding every file in memory once, rather than a
    grep per name: 177 names over ~400 files is 70,000 file reads the naive
    way, which took long enough that the first version of this script was
    unusable on Windows.

    ## ★★★ Why this is no longer "the name appears anywhere" — 2026-09-05

    It was, and on 2026-09-05 the new `tools/gates/check-engine-api-drift.py`
    caught what that cost. `pdfcer-core` had shipped **`pdfcer_core::sign`** —
    101 public items, an entire digital-signing subsystem, written in answer to
    *this shell's own* 2026-09-03 request. This gate scored `EditSession::sign`
    **consumed**, because the word `sign` occurs 42 times under
    `crates/pdfcer-gui/src` and the first two are in `app/actions/bookmarks.rs`,
    in a doc table about **the arithmetic sign of `/Count`**.

    ⇒ A capability the operator asked for was discharged by a comment about
    positive and negative numbers.

    This is the shape the project keeps meeting: **a proxy condition that
    survives one correction.** Name-appears-anywhere was chosen because it is
    cheap and it *reads* like "does the shell know this verb exists". What it
    measures is "does this English word occur", so every verb whose name is
    also an ordinary word — `sign`, `merge`, `split`, `count`, `set`, `move`,
    `insert`, `close`, `open` — was permanently and silently exempt.

    ## What counts as a hit now

    The name must be **call-shaped**: followed by `(`, with no identifier
    character before it. That admits `session.sign(`, `EditSession::sign(` and
    a bare `sign(`; it rejects the word `sign` in a sentence.

    And comments are blanked first, so a doc comment that *mentions*
    `session.sign(…)` while describing work not yet done cannot discharge the
    verb either. Both filters are needed and neither subsumes the other: the
    first kills prose, the second kills aspirational examples, and this project
    has been fooled by both.

    ⚠ A hit remains WEAK evidence. A name being called somewhere is not a live
    route to the operator — that is `ui-verify`'s question, not this gate's.
    All this measurement now refuses to do is call prose a call.
    """
    blobs = [
        strip_comments(p.read_text(encoding="utf-8", errors="replace"))
        for p in root.rglob("*.rs")
    ]
    counts = {}
    for name in names:
        # `(?<![A-Za-z0-9_])` rather than `\b`, because the intent is stated
        # directly: nothing that could be part of a longer identifier may
        # precede the name. `\s*\(` is what makes it a CALL and not a word.
        pattern = re.compile(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"\s*\(")
        counts[name] = sum(len(pattern.findall(b)) for b in blobs)
    return counts


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--all", action="store_true", help="every verb, with counts")
    ap.add_argument("--markdown", action="store_true", help="a table for the register")
    args = ap.parse_args()

    if not (ENGINE_REPO / ENGINE_FILE).exists():
        # * EXIT NON-ZERO. This printed to stderr and carried on until
        # 2026-09-03, which let `check-verb-coverage.sh` report
        # "PASS: all 0 uncalled verb(s)" having read nothing at all.
        print(
            f"verb-coverage: FAIL - engine not found at "
            f"{ENGINE_REPO / ENGINE_FILE}. Nothing was examined, so this exits "
            f"non-zero rather than reporting an empty miss list - a check that "
            f"cannot fail is not evidence.",
            file=sys.stderr,
        )
        return 2
        return 0
    rev = locked_revision()
    text, origin = engine_source(rev)
    verbs = engine_verbs_from(text)
    counts = gui_hits(verbs, GUI)
    missing = [v for v in verbs if counts[v] == 0]

    # ★ What the engine's worktree has that the lock does not. Reported
    # separately and never mixed into the miss list: those verbs cannot be
    # called from this workspace at all until `cargo update` moves the pin.
    coming: list[str] = []
    if origin != "WORKTREE":
        live = set(engine_verbs_from(
            (ENGINE_REPO / ENGINE_FILE).read_text(encoding="utf-8", errors="replace")
        ))
        coming = sorted(live - set(verbs))

    if args.markdown:
        print(f"| verb | occurrences in `crates/pdfcer-gui/src` |")
        print("|---|---|")
        for v in verbs:
            print(f"| `{v}` | {counts[v]} |")
    elif args.all:
        for v in verbs:
            print(f"{counts[v]:5d}  {v}")
    else:
        for v in missing:
            print(v)

    if coming:
        print(
            f"\nCOMING ({len(coming)}): in the engine's WORKING TREE and not in the "
            f"locked revision, so not callable from here yet — "
            + ", ".join(coming),
            file=sys.stderr,
        )
    if origin == "WORKTREE":
        print(
            "\n⚠ read the engine's WORKING TREE, not the locked revision. The engine "
            "session edits that tree continuously, so this list may name verbs this "
            "workspace cannot call.",
            file=sys.stderr,
        )
    print(
        f"\n{len(verbs)} EditSession verbs ({origin}), {len(verbs) - len(missing)} named "
        f"somewhere in the shell, {len(missing)} named nowhere.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
