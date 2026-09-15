#!/usr/bin/env python
"""check-patch-residue.py -- damage done to a file by the tool that wrote it.

WHAT THIS GATE IS FOR
=====================

Almost every source edit in this project is applied by a short Python script
that writes a payload into a `.rs` or `.md` file. Those scripts use two devices
that have now silently corrupted committed source on separate occasions:

* a **marker translated into a glyph** -- the payload is written in plain ASCII
  and a helper maps a short token onto a star, an arrow or a warning sign,
  because a literal non-ASCII character in a heredoc has been mangled by the
  shell before now;
* a **backslash**, which is eaten once by a `<<'EOF'` heredoc, once more by
  Python's own escape handling if the payload is not a raw string, and NOT
  decoded at all if it is.

Both failures share the worst possible signature, which is why they need a gate
rather than care: **the file compiles, `cargo fmt` is happy, `clippy` is happy,
every test passes, and every other gate here is green.** The damage is inside a
doc comment or a string, so no machine downstream has an opinion about it, and
the only oracle is a human reading the emitted region back.

MECHANISM 1 -- A MARKER TRANSLATED INSIDE A WORD
-------------------------------------------------

A helper that maps `S` -> star and `SS` -> two stars does not know what a word
is. Applied to a payload containing the ordinary English word `ASSERTION`, it
emits `A<star><star>ERTION`. Measured 2026-09-14, by eye, while reading an
unrelated function -- not by any gate, and not by any test:

| file | what shipped | what was written |
|---|---|---|
| `tools/ui-verify/src/checks/font_group.rs` | `AN A<star><star>ERTION ABOUT A CLICK` | `AN ASSERTION ABOUT A CLICK` |
| `crates/pdfcer-gui/src/canvas/textsel.rs` | `THE STALENE<star><star> RULE` | `THE STALENESS RULE` |

Both had been in the tree for days.

The detector is deliberately narrow: a star or a warning sign **immediately
preceded by an ASCII letter**. Every legitimate use in this repository has
whitespace, a pipe, a line start or a Markdown delimiter before it -- a marker
is a marker precisely because it stands alone. The one exception that is not a
defect is a Rust newline escape immediately before the glyph, which is a string
literal starting a new line with a marker on it, and it is excluded by name.

MECHANISM 2 -- A UNICODE ESCAPE THAT WAS NEVER DECODED
--------------------------------------------------------

A backslash-u escape inside a Python **raw** string is six characters, not a
character. It reaches the file verbatim. In a `.rs` file that sequence is never
valid: Rust spells a unicode escape with braces, so a brace-less one is either
dead text inside a doc comment -- where it renders as itself and nobody notices
-- or a compile error. Three occurrences have been caught by hand; this catches
the fourth.

MECHANISM 3 -- A PLACEHOLDER THAT WAS NEVER SUBSTITUTED
--------------------------------------------------------

The patch scripts in this project spell their glyphs two ways: by CONCATENATING
a named constant (`"... " + STAR * 3 + " ..."`), and by writing a brace token
into the payload (`"...{STAR}..."`) for a later `.format()` or an f-string. Both
are fine. **Mixing them in one payload is not**, because the brace form is inert
unless something formats it, and a payload assembled by concatenation never is.

Measured 2026-09-15 in `DESIGNS.md`: a block built almost entirely by
concatenation carried a single `*{LQ}one status sentence{RQ}*`, which reached
the file verbatim and rendered as itself. It was found by eye, immediately, only
because the session that wrote it happened to re-grep -- the rest of the block
was correct, so nothing looked wrong at any distance.

MARKDOWN ONLY, and the narrowing is MEASURED rather than assumed. The same
pattern in Rust is a captured format identifier and is correct and idiomatic:
`{PREFIX}`, `{MIN_COLUMN_WIDTH}`. Counted on 2026-09-15 across `crates/` and
`tools/`: **2,311** legitimate occurrences in `.rs`, and **zero** in `.md`
anywhere in the tree. A gate whose claim is "this construct never appears here"
is only worth registering where the current count is zero; in Rust it would
need 2,311 carve-outs and would mean nothing. In Markdown it means exactly one
thing.

RUST ONLY, and the narrowness is deliberate. In a `.py` file the brace-less
escape is correct and idiomatic -- it is how the patch scripts spell their own
markers -- and in a `.md` file it is ambiguous, because a document quoting a
Python snippet is quoting a valid one. Rust is the single language walked here
in which the sequence cannot be right, so Rust is the only place it is called
wrong. A gate that fires on legitimate content gets carved out until it means
nothing, and this one has exactly one claim to make.

THE INPUT SET IS THE WORKING TREE, NOT THE INDEX
-------------------------------------------------

Walked from disk, never asked of git. `check-gate-input-scope.py` exists
because four gates in this repository have been written asking git which files
exist, and a file written and not yet added is invisible to that question --
which is exactly the state a file is in when a patch script has just damaged
it. A gate that goes green on a defect until it is committed is worse than no
gate, because the green is read as a measurement.

FALSIFICATION IS BUILT IN
-------------------------

`--self-test` plants both mechanisms in synthetic lines and asserts the scanner
finds exactly those and nothing else -- in both directions, so a scanner that
answered "yes" unconditionally fails it too. It is registered in `run-all.sh`
ahead of the real run, for the reason recorded across this project: a check
that has never been watched fail is not evidence, and a falsification that
lives only in a session's memory has to be re-derived by the next one.

EXIT CODES
----------

0  no residue found (or the self-test passed).
1  residue found; every occurrence printed with `file:line`.
2  SKIPPED -- no source tree to walk.
"""

from __future__ import annotations

import os
import re
import sys

# Directories never walked: build output, VCS internals, Python caches, and the
# harness's own artifact and profile trees, which hold captured traces that
# legitimately contain anything at all.
SKIP_DIRS = {
    ".git",
    "target",
    "__pycache__",
    ".ui-verify-profiles",
    "node_modules",
    ".venv",
}

# Extensions walked. Deliberately NOT `.txt` or `.jsonl`: gate snapshots and
# captured traces hold whatever the program emitted, and a gate that reports its
# own recorded evidence is a gate that gets carved out until it means nothing.
EXTENSIONS = (".rs", ".md", ".py", ".sh", ".toml")

BACKSLASH = chr(92)
STAR = "★"
WARN = "⚠"

# A marker glued to the end of a word. The carve-out is a Rust string literal
# beginning a new line with a marker on it, where the two characters before the
# glyph are a backslash and an `n`.
GLUED = re.compile(r"(?<!" + BACKSLASH * 2 + r"n)(?<=[A-Za-z])[" + STAR + WARN + r"]")

# A brace-less unicode escape. The braced spelling is the valid Rust one and
# passes.
UNDECODED = re.compile(BACKSLASH * 2 + r"u[0-9a-fA-F]{4}(?!" + BACKSLASH + r"{)")

# An all-capitals brace token. Markdown only -- see MECHANISM 3. The length
# bound keeps it from matching a long prose fragment that merely happens to be
# shouted inside braces.
UNSUBSTITUTED = re.compile(r"\{[A-Z][A-Z0-9_]{0,15}\}")

# A Markdown code fence. Mechanism 3 does not look inside one, and the reason is
# the header's own rule turned on itself: a document quoting a Rust `writeln!`
# with a captured identifier is quoting CORRECT code, and a gate that calls a
# correct quotation a defect gets carved out until it means nothing. Today the
# count inside fences is also zero -- this is a guard against the first true
# quotation, not a concession to an existing one.
FENCE = re.compile(r"^\s*(```|~~~)")

# A line that is ABOUT the hazard rather than suffering from it. A document
# recording the lesson has to be able to spell the thing it forbids.
ABOUT_THE_HAZARD = re.compile(
    r"never decoded|raw string|hazard|residue|verbatim|brace-less"
)

# A line that is ABOUT mechanism 3 rather than suffering from it, kept separate
# from `ABOUT_THE_HAZARD` on purpose: widening one carve-out to excuse a second
# mechanism is how a detector loses the claim it was registered to make.
ABOUT_THE_PLACEHOLDER = re.compile(r"unsubstituted|placeholder|residue|hazard")

GLUED_MECHANISM = "a marker translated INSIDE a word"
UNDECODED_MECHANISM = "an undecoded unicode escape"
UNSUBSTITUTED_MECHANISM = "a placeholder that was never substituted"


def scan(
    lines: list[str], rust: bool, markdown: bool = False
) -> list[tuple[int, str, str]]:
    """Every residue hit in one file's lines, as `(line number, mechanism, line)`.

    Split from [`offences`] so it can be falsified without a file on disk, which
    is what `--self-test` does. A detector that can only be exercised by walking
    a real tree can only be proved non-vacuous by planting a defect in a real
    file, and planting in a real file is how an experiment gets left behind.
    """
    hits: list[tuple[int, str, str]] = []
    fenced = False
    for number, line in enumerate(lines, start=1):
        if markdown and FENCE.match(line):
            fenced = not fenced
            continue
        if GLUED.search(line):
            hits.append((number, GLUED_MECHANISM, line))
        if rust and UNDECODED.search(line) and not ABOUT_THE_HAZARD.search(line):
            hits.append((number, UNDECODED_MECHANISM, line))
        if (
            markdown
            and not fenced
            and UNSUBSTITUTED.search(line)
            and not ABOUT_THE_PLACEHOLDER.search(line)
        ):
            hits.append((number, UNSUBSTITUTED_MECHANISM, line))
    return hits


def offences(path: str) -> list[tuple[int, str, str]]:
    """[`scan`] applied to one file on disk.

    Read as UTF-8 with replacement rather than strictly: a gate that crashes on
    one unreadable file reports as a broken tool, and sends the reader to debug
    the tool while the defect it was pointed at sits unfixed. That has happened
    in this directory before.
    """
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as handle:
            lines = handle.read().splitlines()
    except OSError:
        return []
    # The brace-less escape is a defect in Rust and nowhere else -- see the
    # header. Python uses it correctly; Markdown may be quoting Python.
    return scan(lines, rust=path.endswith(".rs"), markdown=path.endswith(".md"))


def self_test() -> int:
    """Both mechanisms, planted, and both directions asserted."""
    ok = True

    # ---- mechanism 1: a marker glued to the end of a word ----------------
    glued = [
        "//! " + STAR * 3 + " A HEADING, with the marker standing alone.",
        "//! AN A" + STAR * 2 + "ERTION ABOUT A CLICK -- the real 2026-09-14 damage.",
        '             opened a second time.' + BACKSLASH + 'n' + BACKSLASH + 'n'
        + STAR + ' Read the trace before believing any of this.",',
        "| # | Command id | " + STAR + "P3 | Line | a marker opening a table cell |",
        "    // " + WARN + " an ordinary warning, preceded by a space.",
        "//! THE STALENE" + STAR * 2 + " RULE IS NOT RELAXED.",
    ]
    hits = [h[0] for h in scan(glued, rust=True)]
    if hits != [2, 6]:
        print("SELF-TEST FAIL: glued-marker scan expected [2, 6], got " + str(hits))
        ok = False

    # The other direction: prose with no marker at all must be silent, so a
    # detector that answered "yes" unconditionally fails here.
    if scan(["ordinary prose with no markers in it at all"], rust=True):
        print("SELF-TEST FAIL: clean prose reported as glued")
        ok = False

    # ---- mechanism 2: an undecoded escape --------------------------------
    escapes = [
        "const DASH: char = '" + BACKSLASH + "u{2014}';  // the valid Rust spelling",
        'let s = "' + BACKSLASH + 'u2014";  // never valid Rust',
        "//! a line about a raw string, which may spell "
        + BACKSLASH + "u2014 to name it",
    ]
    hits = [h[0] for h in scan(escapes, rust=True)]
    if hits != [2]:
        print("SELF-TEST FAIL: escape scan expected [2], got " + str(hits))
        ok = False

    # And the Rust-only narrowing, falsified: the identical line in a Python
    # file is correct code and must not be reported.
    if scan([escapes[1]], rust=False):
        print("SELF-TEST FAIL: a valid Python escape reported outside Rust")
        ok = False

    # ---- mechanism 3: a placeholder that was never substituted -----------
    # The first line is the real 2026-09-15 damage from `DESIGNS.md`, copied
    # exactly. The second is the legitimate Rust form, asserted NOT to fire
    # outside Markdown -- 2,311 of those are in this tree and every one of them
    # is correct. The third is a document explaining the mechanism, which has to
    # be able to spell the thing it forbids.
    braces = [
        "ordinary prose with no braces anywhere in it",
        "> prediction of *{LQ}one status sentence{RQ}* would not have budgeted",
        'writeln!(f, "{PREFIX} the dock reported {MIN_COLUMN_WIDTH}")?;',
        "an unsubstituted {TOKEN} is residue, and this line is ABOUT it",
    ]
    hits = [h[0] for h in scan(braces, rust=False, markdown=True)]
    if hits != [2, 3]:
        print("SELF-TEST FAIL: placeholder scan expected [2, 3], got " + str(hits))
        ok = False

    # The Markdown-only narrowing, falsified in the direction that matters: the
    # identical Rust line is captured-identifier formatting and must be silent.
    if scan([braces[2]], rust=True, markdown=False):
        print("SELF-TEST FAIL: a Rust captured identifier reported as a placeholder")
        ok = False

    # And the fence carve-out, falsified BOTH ways: the same Rust line inside a
    # Markdown fence is a quotation and must be silent, while a line after the
    # fence closes must be seen again -- a toggle that latched open would make
    # every mechanism-3 claim below the first fence vacuous.
    fenced = [
        "```rust",
        braces[2],
        "```",
        "> and back in prose, *{LQ}still wrong{RQ}*",
    ]
    hits = [h[0] for h in scan(fenced, rust=False, markdown=True)]
    if hits != [4]:
        print("SELF-TEST FAIL: fenced scan expected [4], got " + str(hits))
        ok = False

    print("self-test: " + ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return self_test()

    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    walked = 0
    found: list[tuple[str, int, str, str]] = []
    for here, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in files:
            if not name.endswith(EXTENSIONS):
                continue
            path = os.path.join(here, name)
            walked += 1
            for number, mechanism, line in offences(path):
                found.append((os.path.relpath(path, root), number, mechanism, line))

    if walked == 0:
        print("check-patch-residue: SKIPPED - no source files under " + root)
        return 2

    if not found:
        print(
            "check-patch-residue: clean - "
            + str(walked)
            + " files walked, no marker translated inside a word, no undecoded"
            + " escape and no unsubstituted placeholder"
        )
        return 0

    print("check-patch-residue: " + str(len(found)) + " occurrence(s):")
    print("")
    for path, number, mechanism, line in found:
        print(path + ":" + str(number) + " - " + mechanism)
        print("    " + line.strip()[:160])
    print("")
    print("Each of these was written by a patch script, not typed. The file compiles and")
    print("every other gate is green, because the damage is inside prose. Repair the word,")
    print("then fix the script's helper: a token-translating helper must not be applied to")
    print("a payload it was not written for, and a payload containing a backslash must")
    print("spell it with a placeholder rather than as an escape.")
    print("")
    print("A placeholder that was never substituted is the third mechanism and it has a")
    print("different repair: the payload mixed two spellings. Pick ONE -- concatenate the")
    print("named constant, or format the whole payload -- because a brace token inside a")
    print("concatenated payload is inert and reaches the file as itself.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
