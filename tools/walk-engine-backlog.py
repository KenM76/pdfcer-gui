#!/usr/bin/env python3
"""Count the rows under each ``## `verdict`` section of ``ENGINE_BACKLOG.md``.

WHY THIS IS A FILE AND NOT FIFTEEN LINES IN A DOC COMMENT
=========================================================

``ENGINE_BACKLOG.md`` specifies this walk in prose, has done since
2026-09-06, and says of it: *"the only safe reading of any figure in this
file is the one you produce yourself"*.  Between then and 2026-09-10 the
five section headings went wrong **seven times**.

The seventh recurrence is the one that named the cause.  By then the rule
had already been tightened to *"move the heading in the same edit that
files the row"* — and the very next commit **honoured that rule and was
still wrong**, because it moved the headings by *arithmetic* (previous
figure, plus the inserts, minus the delete) rather than by re-running the
walk.  One of its two inserted rows replaced an existing row, so the
arithmetic was off by one and looked exactly like a correct edit.

⇒ **A walk described in prose is a walk that will be replaced by
arithmetic.**  Prose cannot be executed, so under time pressure it gets
approximated, and an approximation that moves the number in the right
direction is indistinguishable from a measurement.  This file exists so
that "re-walk, never increment" is a command someone can run in under a
second rather than a discipline someone has to remember.

WHAT IT COUNTS, PRECISELY
=========================

A **row** is a line that

  * starts with ``|`` (after leading whitespace), and
  * whose first cell is neither a Markdown separator (``---``) nor a
    table header — the register's headers all begin ``Row (``.

A row belongs to the **section it sits in**, delimited by ``## `verdict```
headings at the top level.  Sub-headings (``### Annotations & markup`` and
friends) are ignored; they group rows within a verdict, they do not change
which verdict a row has.

⚠ **Read what this does NOT measure.**  It counts a row by the section it
SITS IN, never by the verdict its body states.  A `wanted` row whose cell
now opens "✅ WIRED AND DRIVEN" is counted as `wanted`.  That is the same
blind spot ``check-engine-backlog.sh`` has and that ``ENGINE_BACKLOG.md``
already records: the gate proves each row is ACCOUNTED FOR, it proves
nothing about whether the account is true.  A large `wanted` figure is
therefore not a claim that sixty things are unbuilt.

USAGE
=====

    python tools/walk-engine-backlog.py                 # the working tree
    python tools/walk-engine-backlog.py --at HEAD       # what is committed
    python tools/walk-engine-backlog.py --check         # exit 1 if a heading disagrees
    python tools/walk-engine-backlog.py --write 2026-09-10   # set the headings FROM the walk

``--at`` is the half that tells *"rows this session added"* apart from
*"rows that were already uncounted at HEAD"*.  Every previous re-count in
``ENGINE_BACKLOG.md`` that got the attribution right ran both.

``--check`` reads each ``## `verdict` — **N of T**`` heading and compares
N and T against the walk.  It is what a future gate would call.

``--write`` is the ONLY supported way to move a heading.  It preserves each
heading's editorial sub-label verbatim and rewrites only the figures and the
provenance comment, so there is nothing left for a hand edit to get wrong.
Run it, then ``--check``, then commit all three together.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

# The register's five verdicts, in the order the file presents them.
VERDICTS = ["wanted", "blocked", "unknown", "declined", "shipped"]

# ★ THE ROW CAP — a register row is a verdict plus ONE paragraph.
#
# Set by the operator on 2026-09-10 after this file's rows had grown to
# essays: one of them ran 7,779 characters, the mean was 1,048, and the 181
# rows totalled 190 KB.  Reading the register cost a large slice of a session
# before any work started, and filing one row cost a page of writing.  The
# directive to preserve reasoning had been read as "never delete", which is a
# different rule and not the one that was given.
#
# The cap is deliberately generous — 1,200 characters is a long paragraph, not
# a tweet — and it is deliberately CHECKED, because "keep rows short" as prose
# is exactly the sort of rule this script exists to stop being approximated.
ROW_CAP = 1200

# ``## `wanted` — **60 of 160** — …`` — the heading whose two figures drift.
HEADING = re.compile(r"^##\s+`(?P<verdict>\w+)`(?P<rest>.*)$")
FIGURES = re.compile(r"\*\*(?P<rows>\d+)\s+of\s+(?P<total>\d+)\*\*")


def rows_by_verdict(text: str) -> dict[str, int]:
    """Walk ``text`` and return ``{verdict: row count}``.

    The state machine is deliberately trivial: a top-level ``## `verdict```
    heading opens a section, any other top-level ``## `` closes it, and
    every qualifying ``|`` line in between increments that section.
    """
    counts = {v: 0 for v in VERDICTS}
    current: str | None = None
    for line in text.splitlines():
        if line.startswith("## "):
            m = HEADING.match(line)
            current = m.group("verdict") if m and m.group("verdict") in counts else None
            continue
        stripped = line.strip()
        if current is None or not stripped.startswith("|"):
            continue
        first = stripped.split("|")[1].strip() if stripped.count("|") >= 2 else ""
        if first.startswith("---") or set(first) <= set("-: ") and first:
            continue
        if first.startswith("Row ("):
            continue
        if not first:
            continue
        counts[current] += 1
    return counts


def over_cap(text: str, cap: int) -> list[tuple[str, int, str]]:
    """Return ``(verdict, line number, opening)`` for every row longer than ``cap``."""
    out: list[tuple[str, int, str]] = []
    current: str | None = None
    for n, line in enumerate(text.splitlines(), start=1):
        if line.startswith("## "):
            m = HEADING.match(line)
            current = m.group("verdict") if m and m.group("verdict") in VERDICTS else None
            continue
        stripped = line.strip()
        if current is None or not stripped.startswith("|"):
            continue
        first = stripped.split("|")[1].strip() if stripped.count("|") >= 2 else ""
        if not first or first.startswith("---") or first.startswith("Row (") or set(first) <= set("-: "):
            continue
        if len(line) > cap:
            out.append((current, n, first[:60]))
    return out


def headings(text: str) -> dict[str, tuple[int, int] | None]:
    """Return the ``**N of T**`` figures each verdict heading claims."""
    claimed: dict[str, tuple[int, int] | None] = {}
    for line in text.splitlines():
        if not line.startswith("## "):
            continue
        m = HEADING.match(line)
        if not m or m.group("verdict") not in VERDICTS:
            continue
        f = FIGURES.search(m.group("rest"))
        claimed[m.group("verdict")] = (
            (int(f.group("rows")), int(f.group("total"))) if f else None
        )
    return claimed


def rewrite_headings(text: str, counts: dict[str, int], total: int, stamp: str) -> str:
    """Return ``text`` with every ``## `verdict``` heading carrying the walked figures.

    This exists so that "re-walk, never increment" needs no discipline at all.
    The seven heading defects this file's header records were every one of them
    a **hand edit** — a figure retyped, or reached by arithmetic from a figure
    nobody re-measured.  A rule that can be executed cannot be approximated, so
    the fix for the seventh recurrence is not an eighth paragraph: it is that
    the only supported way to move a heading is to run this.

    The sub-label (``a real gap``, ``waiting on something named``, …) is
    preserved verbatim from whatever the heading already says, because it is
    editorial and this function has no opinion about it.  Only the ``**N of
    T**`` figures and the provenance comment are rewritten.
    """
    out: list[str] = []
    nl = "\r\n" if "\r\n" in text else "\n"
    for line in text.split(nl):
        m = HEADING.match(line)
        if not m or m.group("verdict") not in counts:
            out.append(line)
            continue
        verdict = m.group("verdict")
        rest = m.group("rest")
        label = rest.split("—")[1].strip() if rest.count("—") >= 1 else ""
        label = re.sub(r"\*\*.*", "", label).strip(" —")
        out.append(
            f"## `{verdict}`"
            + (f" — {label}" if label else "")
            + f" — **{counts[verdict]} of {total}**"
            + f" <!-- counted by tools/walk-engine-backlog.py, {stamp}; do not retype -->"
        )
    return nl.join(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--at", metavar="REV", help="walk `git show REV:ENGINE_BACKLOG.md`")
    ap.add_argument("--check", action="store_true",
                    help="exit 1 if a heading disagrees or a row is over the cap")
    ap.add_argument("--cap", type=int, default=ROW_CAP,
                    help=f"maximum characters in one register row (default {ROW_CAP})")
    ap.add_argument("--write", metavar="YYYY-MM-DD",
                    help="rewrite the five headings FROM THIS WALK, stamped with this date")
    ap.add_argument("--file", default="ENGINE_BACKLOG.md")
    args = ap.parse_args()

    if args.at:
        text = subprocess.run(
            ["git", "show", f"{args.at}:{args.file}"],
            capture_output=True, text=True, check=True, encoding="utf-8",
        ).stdout
        where = f"{args.at}:{args.file}"
    else:
        text = Path(args.file).read_text(encoding="utf-8")
        where = f"{args.file} (working tree)"

    counts = rows_by_verdict(text)
    total = sum(counts.values())

    if args.write:
        if args.at:
            print("--write needs the working tree, not --at", file=sys.stderr)
            return 2
        Path(args.file).write_text(
            rewrite_headings(text, counts, total, args.write),
            encoding="utf-8", newline="",
        )
        text = Path(args.file).read_text(encoding="utf-8")
        print(f"rewrote the headings of {args.file} from this walk")

    claimed = headings(text)

    print(f"walked {where}")
    bad = False
    for v in VERDICTS:
        c = claimed.get(v)
        note = ""
        if c is None:
            note = "  <- heading states no figure"
            bad = True
        elif c != (counts[v], total):
            note = f"  <- heading claims {c[0]} of {c[1]}"
            bad = True
        print(f"  {v:<9} {counts[v]:>4} of {total}{note}")
    print(f"  {'TOTAL':<9} {total:>4}")

    long_rows = over_cap(text, args.cap)
    if long_rows:
        print()
        print(f"{len(long_rows)} row(s) exceed the {args.cap}-character cap:")
        for v, n, opening in long_rows:
            print(f"  {v:<9} line {n:<5} {opening}")
        print()
        print("A register row is a VERDICT PLUS ONE PARAGRAPH. Cut the narration of")
        print("how the conclusion was reached, the re-statements, and the quotations")
        print("from the engine's own docs — keep the opening clause verbatim (a gate")
        print("keys on it), the verdict, the one load-bearing reason, and the anchors")
        print("somebody would need to find the work.")
        bad = True

    if bad:
        print()
        print("The headings do not agree with the walk. Rewrite them FROM THESE")
        print("FIGURES — do not adjust the old ones arithmetically. A number")
        print("reached by arithmetic from a number nobody re-measured is not a")
        print("measurement; that is the seventh recurrence, 2026-09-10.")
    return 1 if (bad and args.check) else 0


if __name__ == "__main__":
    sys.exit(main())
