#!/usr/bin/env python
"""check-doc-markup.py — Markdown this repository writes but no reader sees.

WHAT THIS GATE IS FOR
=====================

Every other documentation gate here asks whether a sentence is TRUE. This one
asks whether it is **visible**. The two failures it catches have the same
signature and it is the worst one available: the file on disk is complete, the
text editor shows an ordinary line, `git diff` shows nothing unusual, and a
renderer throws the content away without a warning anywhere.

MECHANISM 1 — A TABLE ROW WITH MORE CELLS THAN ITS HEADER
---------------------------------------------------------

GitHub-flavoured Markdown resolves a cell-count disagreement in two opposite
directions, and only one of them is safe:

* a row with **fewer** cells than the header -> empty cells are inserted;
* a row with **more** cells than the header -> **the excess is discarded.**

So one unescaped pipe inside a cell does not shift the layout. It deletes every
character after the header's last column boundary, for the reader, permanently,
in silence.

★★★ Measured on 2026-09-12, before this gate existed: **five rows** in four of
this project's own documents were being truncated, and what was lost was not
incidental:

| file | what a reader never saw |
|---|---|
| `FEATURES.md` Source row | roughly two thousand characters -- every superseded measurement in the stack, back to the first |
| `FEATURES.md` `/F` flags row | the whole second half, including the rule that an unknown flag must fail the build |
| `RIBBON_IA.md` Format-tab row | the **entire** "which side moved, and why" cell, which is the column the row exists for |
| `GLYPH_ADOPTION.md` `line-width` | the evidence sentence for the deferral |
| `ENGINE_BACKLOG.md` rotation row | the file list naming where the work landed |

★★ Four of the five were broken by **a document quoting a command or a literal
that contains a pipe** -- a shell pipeline, a PDF flag pair (`Print` and
`NoZoom`), a closure parameter, another table row. That is the same shape as
the `Source` row broken on 2026-09-06 by the `xargs` invocation it quoted, and
it names the rule: **a document that quotes its own measurement command can be
broken by it.** Escape the pipe; do not reword the quotation.

MECHANISM 2 — AN EMPHASIS MARKER THAT CAN NEITHER OPEN NOR CLOSE
-----------------------------------------------------------------

CommonMark's flanking rules mean a `**` with whitespace on the outside cannot
open a strong run, and a `**` with whitespace on the inside cannot close one.
A line **ending in a space followed by `**`** is therefore inert in both
directions: whatever it was meant to bold renders with two literal asterisks in
it and no emphasis at all.

That is easy to produce when a long bold heading is wrapped by hand across two
lines, and impossible to see in an editor. One was written into `HANDOFF.md` on
2026-09-12 and caught by eye; this gate is so that the next one is not.

WHAT IS DELIBERATELY NOT FLAGGED
--------------------------------

* **A row with FEWER cells than its header.** It renders correctly. Flagging it
  would be noise, and noise is how a gate gets carved out until it means
  nothing. The distinction is the finding; it is not softness.
* **Anything inside a fenced code block.** Rust closure syntax (`|e| ...`) and
  shell pipelines inside fences are code, not tables. The first draft of
  mechanism 1 had no fence handling and reported a `DEFECTS.md` code sample as
  a broken table -- a gate that reports a correct file is a gate that gets
  disabled.
* **A run of pipe-leading lines with no delimiter row under the first.** GFM
  requires `|---|---|` to make a table at all. Without it the lines are
  ordinary paragraph text and their pipes mean nothing.

USAGE
=====

  tools/gates/check-doc-markup.py              scan every tracked *.md
  tools/gates/check-doc-markup.py --self-test  falsify the mechanism

Exit: 0 clean, 1 violations, 2 could not run (not a git checkout).

★ The self-test falsifies in BOTH directions for both mechanisms, per this
repository's rule that a check which cannot fail is not evidence: it plants an
over-wide row, an under-wide row, an escaped pipe, a fenced code sample and a
pipe-leading paragraph with no delimiter row, and asserts exactly one hit.
"""

import os
import subprocess
import sys

# The backslash is never written literally in this file. It has broken a patch
# script in this project more than once, and a gate about invisible characters
# should not contain one.
BACKSLASH = chr(92)
PIPE = "|"
NL = chr(10)
CR = chr(13)


# --------------------------------------------------------------------------
# Mechanism 1
# --------------------------------------------------------------------------
def delimiters(line):
    """Indices of the pipes a Markdown renderer treats as cell boundaries.

    A pipe preceded by a backslash is an escaped literal and bounds nothing.
    Written as a scan rather than a regular expression so the escape can be
    spelled with `chr(92)`.
    """
    out = []
    i = 0
    while i < len(line):
        if line[i] == BACKSLASH and i + 1 < len(line) and line[i + 1] == PIPE:
            i += 2
            continue
        if line[i] == PIPE:
            out.append(i)
        i += 1
    return out


def is_delimiter_row(line):
    """`|---|:--:|` and friends -- the row GFM requires to make a table."""
    body = line.strip()
    if not body.startswith(PIPE):
        return False
    if not any(c == "-" for c in body):
        return False
    return all(c in (PIPE, "-", ":", " ", BACKSLASH) for c in body)


def strip_fences(lines):
    """Return `lines` with fenced code blocks blanked out.

    Blanked rather than removed so every surviving index still equals its real
    line number -- a gate that reports the wrong line number costs the reader
    the same minute twice.
    """
    out = []
    fence = None
    for line in lines:
        body = line.strip()
        opener = None
        if body.startswith("```"):
            opener = "```"
        elif body.startswith("~~~"):
            opener = "~~~"
        if fence is None and opener is not None:
            fence = opener
            out.append("")
            continue
        if fence is not None:
            if opener == fence:
                fence = None
            out.append("")
            continue
        out.append(line)
    return out


def scan_tables(lines):
    """Rows whose excess cells a renderer discards. Returns (lineno, want, got)."""
    lines = strip_fences(lines)
    hits = []
    i = 0
    while i < len(lines):
        if not lines[i].lstrip().startswith(PIPE):
            i += 1
            continue
        j = i
        while j < len(lines) and lines[j].lstrip().startswith(PIPE):
            j += 1
        # A table needs a delimiter row directly under its header. Without one
        # this is a paragraph whose lines happen to begin with a pipe.
        if j - i >= 2 and is_delimiter_row(lines[i + 1]):
            want = len(delimiters(lines[i]))
            for n in range(i + 2, j):
                got = len(delimiters(lines[n]))
                if got > want:
                    hits.append((n + 1, want, got))
        i = j
    return hits


# --------------------------------------------------------------------------
# Mechanism 2
# --------------------------------------------------------------------------
def scan_emphasis(lines):
    """Lines ending in whitespace + `**`, which can neither open nor close."""
    lines = strip_fences(lines)
    hits = []
    for n, line in enumerate(lines):
        body = line.rstrip()
        if not body.endswith("**"):
            continue
        head = body[:-2]
        if head and head[-1] in (" ", chr(9)):
            hits.append((n + 1, body.strip()[-60:]))
    return hits


# --------------------------------------------------------------------------
def self_test():
    """Both mechanisms, falsified in both directions."""
    ok = True

    # ---- mechanism 1 -----------------------------------------------------
    doc = [
        "| Key | Why |",
        "|---|---|",
        "| fine | an ordinary two-cell row |",
        "| short |",
        "| escaped | a cell quoting " + BACKSLASH + PIPE + " a pipe |",
        "| broken | a cell quoting | a pipe | and losing this |",
        "",
        "```rust",
        "| doc.entered.is_some_and(|e| e.subpath.is_some()) |",
        "```",
        "",
        "| not a table because there is no delimiter row |",
        "| so these pipes | mean | nothing at all |",
    ]
    hits = scan_tables(doc)
    if [h[0] for h in hits] != [6]:
        print("SELF-TEST FAIL: table scan expected only line 6, got " + str(hits))
        ok = False

    # A file with no tables at all must be silent.
    if scan_tables(["ordinary prose", "with no pipes in it"]):
        print("SELF-TEST FAIL: plain prose reported as a table")
        ok = False

    # ---- mechanism 2 -----------------------------------------------------
    emph = [
        "- **A heading that closes properly.**",
        "- **A heading whose closer cannot close **",
        "Some prose ending in an ordinary sentence.",
        "A phrase that is **emphasised across",
        "the line break.**",
        "```",
        "let x = 1; // ends with a marker **",
        "```",
    ]
    hits = scan_emphasis(emph)
    if [h[0] for h in hits] != [2]:
        print("SELF-TEST FAIL: emphasis scan expected only line 2, got "
              + str(hits))
        ok = False

    # ---- the fence handling itself ---------------------------------------
    # Falsified directly: without `strip_fences` the code sample above is a
    # four-pipe row under a two-pipe header, which is exactly the false
    # positive that made the first draft unusable.
    unfenced = ["| Key | Why |", "|---|---|",
                "| doc.entered.is_some_and(|e| e.subpath.is_some()) |"]
    if not scan_tables(unfenced):
        print("SELF-TEST FAIL: the detector cannot see an over-wide row at all")
        ok = False

    print("self-test: " + ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def main():
    if "--self-test" in sys.argv[1:]:
        return self_test()

    try:
        listing = subprocess.check_output(["git", "ls-files", "*.md"], text=True)
    except Exception as exc:
        print("check-doc-markup: SKIPPED — not a git checkout (" + str(exc) + ")")
        return 2

    files = [f for f in listing.split(NL) if f.strip()]
    if not files:
        print("check-doc-markup: SKIPPED — git tracks no Markdown here")
        return 2

    table_hits = []
    emph_hits = []
    scanned = 0
    for path in files:
        if not os.path.isfile(path):
            continue
        try:
            with open(path, encoding="utf-8") as handle:
                raw = handle.read()
        except Exception:
            continue
        scanned += 1
        lines = [ln.rstrip(CR) for ln in raw.split(NL)]
        for lineno, want, got in scan_tables(lines):
            table_hits.append((path, lineno, want, got))
        for lineno, tail in scan_emphasis(lines):
            emph_hits.append((path, lineno, tail))

    if not table_hits and not emph_hits:
        print("check-doc-markup: " + str(scanned)
              + " Markdown files, every table row inside its header's column"
              + " count and no inert emphasis marker")
        return 0

    for path, lineno, want, got in table_hits:
        print(path + ":" + str(lineno) + ": this row has " + str(got)
              + " cell boundaries where its header has " + str(want)
              + " — a renderer DISCARDS everything past boundary "
              + str(want) + ".")
        print("    Escape the stray pipe as " + BACKSLASH + PIPE
              + " rather than rewording; the quotation is usually the point.")
    for path, lineno, tail in emph_hits:
        print(path + ":" + str(lineno)
              + ": this line ends in whitespace before `**`, which can neither"
              + " open nor close a strong run — it renders as two asterisks.")
        print("    ..." + tail)

    print("")
    print("check-doc-markup: " + str(len(table_hits)) + " truncated table row(s), "
          + str(len(emph_hits)) + " inert emphasis marker(s), in "
          + str(scanned) + " files.")
    print("Both failures are invisible in a text editor and invisible to every")
    print("other gate here. The file on disk is complete; the reader's copy is not.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
