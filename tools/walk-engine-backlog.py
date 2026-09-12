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


#: A verdict cell that opens by announcing the capability is already here.
#: Matched against the cell's FIRST bolded run only — see `misfiled` below.
CONSUMED_MARKERS = ("\u2705", "WIRED", "CONSUMED", "SHIPPED AND")


def opening_clause(cell: str) -> str:
    """The verdict-bearing opening of a ``Why`` cell.

    The register's convention is that a row's cell OPENS with its verdict, in
    bold, optionally behind a status glyph: ``⬜ **`wanted` — …**``,
    ``◑ **`/DV` and `/Q` CONSUMED …**``, ``**declined, deliberately …**``.
    So the opening clause is *the first bolded run*, and the glyph in front of
    it is decoration.

    ★ The previous implementation read the first bolded run **only when the
    cell began with ``**``** and otherwise fell back to the first 60 characters.
    That is the same class of defect this whole module exists to catch: every
    row that opens with a glyph — which is most of the interesting ones — was
    measured by a fixed-width prefix instead of by its own structure, so a
    verdict stated at character 61 was invisible, and prose at character 20 that
    merely MENTIONED another verdict was not.  Read the structure.
    """
    i = cell.find("**")
    if i == -1 or i > 12:  # a bold run further in than a glyph-plus-space is prose
        return cell[:80]
    run = cell[i + 2:].split("**")[0]
    return run if run else cell[:80]


def contrary_verdict(head: str, section: str) -> str | None:
    """A verdict word in ``head`` that is not ``section``'s — or ``None``.

    # ★★ Why this exists, and why it is the more important half

    `CONSUMED_MARKERS` catches a row that announces itself WIRED.  It cannot
    catch the other five ways a row can be misfiled, because those are spelled
    with an ordinary word rather than a tick.  On 2026-09-11, with the marker
    check green, a hand read of the ``wanted`` section found **eight** rows
    carrying somebody else's verdict — five opening ``**declined …**``, two
    ``**`shipped` …**``, one ``**✅ Accounted for …**`` — and only the last
    three were reported.  ``wanted`` read **46** where the real gap was **38**:
    a 17% overstatement of the work left, in the number a session reads to
    choose what to build.

    That is this project's recorded failure mode *a gate keyed on a name is
    discharged by prose*, arriving from the opposite direction.  The verdict
    word IS the name here, so the check is keyed on all five of them.

    # Why a row may legitimately name another verdict

    Constantly — and that is why the section's OWN word wins.  ``⬜ **`wanted`,
    small, and blocked on nothing.**`` is a correctly filed `wanted` row; so is
    ``**`wanted` — shipped 2026-09-09; the pin does not carry it yet**``, where
    *shipped* describes the ENGINE, not this shell.  A row whose opening names
    its own section is self-consistent and is never reported, whatever else it
    mentions.  Only a row that names another verdict and never names its own is
    worth a human's attention.
    """
    if section in head:
        return None
    for word in VERDICTS:
        if word in head:
            return word
    return None


def misfiled(text: str) -> list[tuple[str, int, str, str]]:
    """Rows whose verdict CELL contradicts the SECTION they are filed under.

    Two ways a cell can contradict its section, and the second was unguarded
    until 2026-09-11: it can announce the capability is already here
    (`CONSUMED_MARKERS`), or it can simply open with a different verdict WORD
    (:func:`contrary_verdict`). The second is the common one.

    Returns ``(section verdict, line number, opening clause, the marker found)``.

    # Why this check exists

    A row's verdict in this register is **the section it sits in** — that is
    what :func:`rows_by_verdict` counts, and it is the right choice, because a
    section heading cannot be quietly contradicted by a sentence three hundred
    characters into a cell.

    But the reverse is not guarded at all. A row can be wired, marked
    ``**✅ WIRED 2026-09-06 — …**`` in its own cell by the session that wired
    it, and still sit under ``## `wanted``` forever, because moving it is a
    separate act that nothing checks. Three rows in the `wanted` section are in
    exactly that state as this is written.

    ⇒ **The count is right about what it measures and wrong about what a reader
    takes it for**, which is the failure this whole file was written against.
    `wanted`'s own heading tells the reader *"these are the rows to read if you
    are choosing what to build next"*, so a wired row left there hands somebody
    work that is finished.

    # Why it is narrow, and why it reports rather than fails

    The marker must appear in the cell's **first bolded run** — the place this
    register's convention puts a verdict. That keeps it away from the very
    common prose *"…filed as X and shipped the same week"* in the body of a
    genuinely-wanted row, which describes the ENGINE shipping something, not
    this shell consuming it.

    It is a report, not an exit-1, for the same reason `check-engine-backlog.sh`
    is *"deliberately weak in one direction"*: there are legitimate rows whose
    cell opens `**✅ Accounted for …**` for a capability only partly consumed,
    and a gate that forced those to move would teach people to re-baseline it.
    Somebody has to look. The tool's job is to make sure somebody is told.
    """
    out: list[tuple[str, int, str, str]] = []
    current: str | None = None
    for n, line in enumerate(text.splitlines(), start=1):
        if line.startswith("## "):
            m = HEADING.match(line)
            current = m.group("verdict") if m and m.group("verdict") in VERDICTS else None
            continue
        stripped = line.strip()
        if current is None or not stripped.startswith("|"):
            continue
        cells = stripped.split("|")
        if len(cells) < 4:
            continue
        first = cells[1].strip()
        if not first or first.startswith("---") or first.startswith("Row (") or set(first) <= set("-: "):
            continue
        head = opening_clause(cells[2].strip())

        # (a) The cell announces the capability is HERE while the section says
        #     it is not.  Never asked of the `shipped` section, where a cell
        #     opening with a tick is the section agreeing with itself.
        if current != "shipped":
            for marker in CONSUMED_MARKERS:
                if marker in head:
                    out.append((current, n, first[:60], marker))
                    break
            else:
                other = contrary_verdict(head, current)
                if other:
                    out.append((current, n, first[:60], f"opens `{other}`"))
            continue

        other = contrary_verdict(head, current)
        if other:
            out.append((current, n, first[:60], f"opens `{other}`"))
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
    # ★★ A gate that cannot PRINT its finding has not found anything.
    #
    # Python on this machine opens stdout as cp1252, and every interesting row
    # in the register opens with `★`. On 2026-09-11 `--check` printed
    # "1 row(s) exceed the 1200-character cap:" and then died with
    # `UnicodeEncodeError` on the line that would have said WHICH row — so the
    # run looked like a broken tool rather than a register that needed an edit,
    # and the natural next act was to go and debug this file.
    #
    # `errors="replace"` rather than a bare utf-8 switch, deliberately: a
    # console that genuinely cannot draw a character should show `?` and keep
    # going. **A decoration must never be able to kill a measurement.**
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):  # not a real console; nothing to fix
            pass

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

    wrong_section = misfiled(text)
    if wrong_section:
        print()
        print(f"{len(wrong_section)} row(s) open with a verdict their section contradicts:")
        for v, n, opening, marker in wrong_section:
            print(f"  {v:<9} line {n:<5} [{marker}] {opening}")
        print()
        print("A row's VERDICT is the section it sits in - that is what the counts")
        print("above measure. A row left in the wrong one is handed to the next")
        print("reader as the section's own heading describes it, so a declined or")
        print("shipped row under `wanted` becomes work somebody is told to do.")
        print("Move it, then re-run with --write to rewrite the headings.")
        print()
        print("This is a REPORT, not a failure. A partly-consumed row may open")
        print("with a tick and still belong where it is. Somebody has to look -")
        print("on 2026-09-11 a hand read found EIGHT of these while this check")
        print("reported three, because it was keyed on the tick and five of the")
        print("eight spelled their verdict as an ordinary word.")

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
