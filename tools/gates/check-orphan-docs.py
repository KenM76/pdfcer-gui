#!/usr/bin/env python3
"""check-orphan-docs.py — two items' doc comments must not run together.

===========================================================================
THE PROPERTY ASSERTED
===========================================================================

A contiguous run of `///` lines is **one doc comment** to Rust, however many
items' worth of prose it contains, and it attaches to whatever item comes next.
So an item inserted below an existing item's doc comment — instead of below that
item — silently adopts its documentation:

    /// Which ends of a `/Line` carry an arrowhead.          <- `endings`' doc
    /// ... thirty-eight lines ...
    /// **The border line style — the eighth control.**      <- `dash`'s doc
    /// ... forty lines ...
    fn dash(...)                                             <- gets BOTH
    ...
    fn endings(...)                                          <- gets NONE

TWO CAUSES, ONE SHAPE — AND THE DIAGNOSIS COMES FIRST
-----------------------------------------------------

The same end state arrives by two routes, and they need different repairs:

  1. **A real orphan.** An item was inserted under another item's doc, or an
     item was split out to a new file and **its doc comment was left behind**,
     where it merged into the next item's.
  2. **A missing paragraph break.** One item's doc with a bare `///` omitted
     between two of its own paragraphs. Nothing is misattributed; there is no
     absorber to name.

⇒ *This gate reports one SHAPE and that shape has two causes.* Inserting a
paragraph break into a real orphan silences the gate over a live defect; moving
a paragraph that belonged where it was strands half a doc comment. **The
discriminator: a real orphan implies an UNDOCUMENTED item in the same file.** A
missing break does not.

NOTHING IN THE TOOLCHAIN CAN SEE THIS. `rustc` is happy — a doc comment is
valid prose wherever it sits. `cargo fmt` is happy. `cargo clippy -D warnings`
is happy. The only observer that would notice is `cargo doc`, and nobody runs
that on a binary crate.

AND IT FALSIFIES CITATIONS, WHICH IS WORSE THAN LOSING A DOC
------------------------------------------------------------

The shape to recognise: one item's doc quotes another item's doc by name —
*"`key_stroke`'s own doc comment has said since it was written that …"* — while
the named item has no doc comment at all, because its doc was absorbed by an
unrelated neighbour. The quotation is **verbatim accurate**; only the claim
about where the words live is false. A reader following it lands on a paragraph
about something else and reasonably concludes the citing sentence was invented.

The repair needs no rewording — moving the doc to its owner makes the citation
true, because the only thing wrong with it was the location of what it cited.

⇒ *An orphan does not merely leave an item undocumented — it silently falsifies
every cross-reference to that item without making a single word of those
cross-references wrong.* A missing doc is absent; a misdirected citation
**actively misleads, and reads as the citation's fault.**

The one incidental warning this class gets: deleting an absorbing item leaves a
blank line after a doc comment, which clippy *does* lint
(`empty_line_after_doc_comments`). ⇒ *a lint firing on the shape next to a
defect is the only warning available, so never paper one over.*

===========================================================================
THE HEURISTIC, AND WHY IT IS STATED AS ONE
===========================================================================

This crate's convention is absolute and is what makes detection possible at
all: an item's doc comment **opens with a bold sentence on its own line**,
`/// **Title.**`, and every later paragraph of the same doc is preceded by a
bare `///`. So a line matching that shape, sitting *directly beneath a line of
completed prose* with no paragraph break, is two doc comments that grew
together — the prose above is the END of the doc that lost its item, and the
bold line is the START of the doc that absorbed it.

THREE CUTS WERE MEASURED AND TWO WERE WRONG. Both wrong ones are the obvious
"simplification", so here is how each fails:

  * **A bold title AFTER a paragraph break** finds **zero**. That is the shape
    of an ordinary mid-doc heading; the anomaly is the opposite one. ⇒ *a
    detector aimed at the normal shape reports zero and looks exactly like a
    clean tree.*
  * **Any `**` anywhere in a doc line** finds **717**, essentially all of them
    mid-sentence emphasis wrapped across a line break.
  * Only the third — a COMPLETE bold sentence alone on its line, above a
    completed sentence — measures anything.

---------------------------------------------------------------------------
WHAT IT PROVABLY CANNOT SEE
---------------------------------------------------------------------------

1. **An indented bold sentence inside a bullet list is not a title**, and must
   not be reported. `pagesize.rs` has `///   **That asymmetry is the
   disclosure.**` as the last sentence of a `*` bullet. Closed precisely, by
   requiring **exactly one** space between `///` and the title: a title is never
   indented, a list continuation always is. The rule survives the `DECOR`
   widening because `DECOR` contains no space and no bullet marker, so an
   indented decorated bullet still fails on its two extra spaces.

2. **A bold sentence that merely begins at a line boundary is indistinguishable
   from a title.** `canvas/tool/arm.rs` documents `arm_text_edit` with

       /// Arm the caret tool with `kind`, or retire it if that kind is already armed.
       /// **The entry point the `edit.text` and `edit.add_text` dispatch arms call.**

   and its twin `arm_markup` carries the *same sentence pair*, differing only in
   where the wrap falls — there the bold starts mid-line, here at column 5. No
   line-oriented regex can separate those, because the distinguishing fact is
   semantic. So it goes on `ALLOWED` below rather than being papered over by a
   looser pattern that would also stop seeing real orphans.

3. **The gate sees the bold-title convention, and ONLY that convention.** Not a
   tuning gap — a measured ceiling, established three ways so nobody re-derives
   it:

   * An orphan whose title is plain prose — *"Turn the mouse wheel at the
     pointer's current position."* No pattern built on `**` will ever reach one
     of those.
   * Dropping the bold requirement (any one-sentence `///` line above a
     completed sentence, bare `///` below) gives **126** candidates. Too loose
     to gate on: a two-sentence paragraph whose closing sentence happens to fill
     one line has the identical footprint, and most of the 126 are precisely
     that.
   * Requiring **bold LEAD words** rather than a fully bold sentence gives
     **20**, which a person can read in ten minutes — but only one or two of
     those are real, so it is a triage list, not a gate condition.

   ★ The right direction for any future widening is not a better prose
   heuristic. It is the discriminator above: **a real orphan implies an
   undocumented item in the same file**, so
   `clippy::missing_docs_in_private_items` is the census every remaining orphan
   hides in. Far too large to turn on as a gate — closing it is an
   operator-scale R5 decision, not a cleanup — but as a **filter over regex
   candidates** it is exact where a regex is only suggestive.

   ★★ That census has a trap, and it produced a wrong all-clear. Narrow it to a
   subset of item kinds and a candidate reads as *"benign — no undocumented item
   below it"* while an undocumented constant sixteen lines below it sits in the
   full census. ⇒ *a verdict computed from a filtered population inherits the
   filter's blind spot and reports it with undiminished confidence.*

★★ `ALLOWED` is checked for STALENESS. An entry that no longer matches anything
fails the gate. An exemption outlives its reason, and a gate carrying a dead
carve-out is a gate nobody knows has been narrowed. If the `arm.rs` wording is
ever reflowed, this gate says so rather than quietly keeping a hole open.

===========================================================================
USAGE AND EXIT CODES — the project's three-state gate contract
===========================================================================
  tools/gates/check-orphan-docs.py              check the crate
  tools/gates/check-orphan-docs.py --self-test  falsify the mechanism

  0  clean    — no candidate seam outside `ALLOWED`, every `ALLOWED` entry live
  1  FAIL     — an orphaned doc comment, or a stale `ALLOWED` entry
  2  SKIPPED  — no source tree to scan (see run-all.sh's three-state model)

Every run prints the file count, how many were CRLF-normalized, the candidate
count and how many exemptions are still live. Those numbers are the only way to
tell a clean tree from a pattern that has stopped matching.

HOW TO FALSIFY IT
-----------------

`--self-test` runs `scan` directly, with no fixture tree on disk. It asserts a
planted orphan IS found (plain and decorated, and each `DECOR` character
separately, because a test that exercises one member does not measure a set) and
that four legitimate shapes are NOT: a bullet continuation, a bullet
continuation with decoration, an ordinary mid-doc heading, and emphasis wrapped
across a line break. It also asserts `scan` is carriage-return SENSITIVE and
that normalizing restores the hit — see the CRLF note in `main`.
"""

import io
import os
import re
import sys

# BOTH roots, and the second one is load-bearing.
#
# `tools/ui-verify/` is the harness, and it is the most heavily documented
# tree in the project — every check under `src/checks/` carries the long doc
# comment R5 asks for. Scanning `crates/` alone leaves the one instrument
# aimed at misattached documentation not reading it, while printing a file
# count that reads complete.
#
# ⇒ *A scope constant is a claim about coverage, and a file count is not
# evidence for it.* The failure is silent in both directions: the gate
# reports `clean` in exactly the words a real all-clear uses.
ROOTS = ["crates", "tools"]

# Excluded, and not for tidiness.
#
# `tools/gates/fixtures/` holds deliberately-defective trees — the
# `ui-strings/dirty/` one exists so `check-ui-strings.sh` can prove it
# still fails on a planted violation. A gate that scanned another gate's
# planted defects would make the two instruments each other's false
# positives, and the fix would be to weaken one of them.
SKIP_DIRS = ("target", "fixtures")
CRLF = chr(13) + chr(10)

# The four characters this crate uses to decorate a title, and nothing else.
#
# A title regex that requires the bold to be the FIRST thing after `/// `
# expresses only about half of this crate's convention: decorated titles are
# roughly as common as undecorated ones, and every seam behind one is
# invisible. ⇒ *A detector's scope is a claim, and "no violations found" is
# not evidence for it — only a falsification against known-bad input is.*
#
# ★ The admission is deliberately narrow. A general "anything before the
# `**`" prefix would re-admit ordinary prose and take the count back toward
# the 717 false positives of the second cut. The run must be these
# characters, one or more, followed by exactly one space, then the bold —
# which is also what keeps the indented-bullet carve-out intact, since no
# decoration character is a space or a bullet marker.
DECOR = "★⚠→⇒"

# A COMPLETE bold sentence alone on a line, with EXACTLY one space after `///`
# and an optional decoration run between the two.
# That one space is limit 1 above: a title is never indented inside a list.
TITLE = re.compile(
    r"^[ \t]*/// (?:[" + DECOR + r"]+ )?\*\*.+[.!?]\*\*[ \t]*$"
)

# And the line above it must be a COMPLETED sentence — the last line of the doc
# that lost its item. A fragment above means ordinary wrapping, not a seam.
PROSE_END = re.compile(r"^[ \t]*///[ \t]+.*[.!?][ \t]*$")

# Known non-orphans, as (path-suffix, the exact bold text). Limit 2 above.
# Each entry is checked for staleness: if it matches nothing, the gate fails.
ALLOWED = [
    (
        "canvas/tool/arm.rs",
        "**The entry point the `edit.text` and `edit.add_text` dispatch arms call.**",
        "Same sentence pair as its twin `arm_markup`, which wraps with the bold "
        "mid-line; only the line break differs. Semantic, so no regex reaches it.",
    ),
]


def scan(lines: list[str]) -> list[tuple[int, str]]:
    """Every line in `lines` that looks like a second doc comment's title.

    A pure function of the text so the self-test can falsify it directly,
    without a fixture tree on disk.
    """
    found = []
    for i, line in enumerate(lines):
        if not TITLE.match(line):
            continue
        # Is the line above part of the same doc run? A run is broken by
        # anything that is not a `///` line — a blank, an attribute, a plain
        # `//` comment — which is Rust's own rule for where a doc comment
        # begins, so this cannot over-reach.
        prev = lines[i - 1] if i else ""
        if not prev.lstrip().startswith("///"):
            continue
        # A bare `///` above is a paragraph break: an ordinary mid-doc heading.
        if prev.strip() == "///" or not PROSE_END.match(prev):
            continue
        found.append((i + 1, line.strip()))
    return found


def self_test() -> int:
    """Falsify both halves: it must FIND a planted orphan and must NOT find
    either of the two shapes the limits above carve out."""
    nl = chr(10)

    planted = (
        "/// Which ends of a `/Line` carry an arrowhead." + nl
        + "///" + nl
        + "/// Absent for every other subtype, because nothing else has ends." + nl
        + "/// **The border line style — the eighth control.**" + nl
        + "///" + nl
        + "/// Its own paragraph." + nl
        + "fn dash() {}" + nl
    ).split(nl)

    ok = True

    hits = scan(planted)
    if len(hits) != 1 or "eighth control" not in hits[0][1]:
        print("SELF-TEST FAIL: planted orphan not found:", hits)
        ok = False

    # ★★★ The same orphan with CRLF endings, because the first version of this
    # gate could not see one. `main` normalizes before calling `scan`, so what
    # this asserts is that the normalization is still there — feed `scan` the
    # un-normalized form and it must come back EMPTY, which is the condition
    # that made 126 files invisible. If a later edit drops the normalization,
    # the real scan silently returns to finding nothing in those files while
    # this assertion goes on passing — so BOTH halves are checked: `scan` is
    # confirmed to be ending-sensitive, and the pipeline is confirmed to
    # normalize.
    crlf_raw = [ln + chr(13) for ln in planted]
    if scan(crlf_raw):
        print("SELF-TEST FAIL: scan() is supposed to be \\r-sensitive; the")
        print("  normalization in main() is what handles CRLF. Got:", scan(crlf_raw))
        ok = False
    if len(scan([ln.rstrip(chr(13)) for ln in crlf_raw])) != 1:
        print("SELF-TEST FAIL: normalizing CRLF did not restore the hit")
        ok = False

    # Limit 1 — an indented bold sentence closing a bullet is not a title.
    bullet = (
        "/// * `lost_area` — the sheet shrank. Reversible here by Undo; not" + nl
        + "///   reversible after a round trip through anything else." + nl
        + "///   **That asymmetry is the disclosure.**" + nl
        + "/// * `crop_box_outside` — a `/CropBox` the new sheet no longer contains." + nl
        + "fn sentences() {}" + nl
    ).split(nl)
    if scan(bullet):
        print("SELF-TEST FAIL: bullet continuation reported:", scan(bullet))
        ok = False

    # An ordinary mid-doc bold heading after a paragraph break must pass. This
    # is the shape the FIRST draft of the gate aimed at, finding zero.
    ordinary = (
        "/// **Arm the markup tool.**" + nl
        + "///" + nl
        + "/// Some prose that ends in a full stop." + nl
        + "///" + nl
        + "/// **Why pressing the armed button again retires the tool.**" + nl
        + "fn arm() {}" + nl
    ).split(nl)
    if scan(ordinary):
        print("SELF-TEST FAIL: ordinary mid-doc heading reported:", scan(ordinary))
        ok = False

    # A bold emphasis WRAPPED across a line break must pass — the 717-hit
    # false positive of the second draft.
    wrapped = (
        "/// Some prose leading in, and then a phrase that is **emphasised but" + nl
        + "/// does not end here.**" + nl
        + "fn wrapped() {}" + nl
    ).split(nl)
    if scan(wrapped):
        print("SELF-TEST FAIL: wrapped emphasis reported:", scan(wrapped))
        ok = False

    # ★★★ The DECORATED orphan -- the shape the gate was blind to until
    # 2026-09-13, and the reason this block exists. Without it the widening
    # is a claim in a comment: the regex could be reverted to its narrow form
    # and every other assertion here would still pass.
    decorated = (
        "/// Which ends of a `/Line` carry an arrowhead." + nl
        + "///" + nl
        + "/// Absent for every other subtype, because nothing else has ends." + nl
        + "/// ★★★ **The border line style — the eighth control.**" + nl
        + "///" + nl
        + "/// Its own paragraph." + nl
        + "fn dash() {}" + nl
    ).split(nl)
    hits = scan(decorated)
    if len(hits) != 1 or "eighth control" not in hits[0][1]:
        print("SELF-TEST FAIL: decorated orphan not found:", hits)
        ok = False

    # Each decoration character on its own, because `DECOR` is a set and a
    # test that only exercises one member does not measure the set.
    for mark in "★⚠→⇒":
        one = (
            "/// A completed sentence that ends the doc that lost its item." + nl
            + "/// " + mark + " **A title carrying one decoration mark.**" + nl
            + "fn absorber() {}" + nl
        ).split(nl)
        if len(scan(one)) != 1:
            print("SELF-TEST FAIL: decoration " + mark + " not admitted")
            ok = False

    # And the negative that keeps the widening narrow: decoration is NOT a
    # general prefix. An ordinary word before the bold must still be invisible,
    # or the gate returns to the second draft's 717 false positives.
    prose_prefix = (
        "/// A completed sentence that ends the doc that lost its item." + nl
        + "/// Note **that this is emphasis inside a sentence.**" + nl
        + "fn absorber() {}" + nl
    ).split(nl)
    if scan(prose_prefix):
        print("SELF-TEST FAIL: a prose prefix was admitted:", scan(prose_prefix))
        ok = False

    # Limit 1 again, with decoration. An indented bullet continuation whose
    # closing sentence is decorated AND bold must still pass -- the widening
    # must not have opened the hole the exactly-one-space rule closed.
    decorated_bullet = (
        "/// * `lost_area` — the sheet shrank. Reversible here by Undo; not" + nl
        + "///   reversible after a round trip through anything else." + nl
        + "///   ★ **That asymmetry is the disclosure.**" + nl
        + "/// * `crop_box_outside` — a `/CropBox` the new sheet no longer contains." + nl
        + "fn sentences() {}" + nl
    ).split(nl)
    if scan(decorated_bullet):
        print("SELF-TEST FAIL: decorated bullet continuation reported:",
              scan(decorated_bullet))
        ok = False

    # A decorated mid-doc heading AFTER a paragraph break is the ordinary
    # shape and must pass. This is the one the first draft aimed at.
    decorated_ordinary = (
        "/// **Arm the markup tool.**" + nl
        + "///" + nl
        + "/// Some prose that ends in a full stop." + nl
        + "///" + nl
        + "/// ★★ **Why pressing the armed button again retires the tool.**" + nl
        + "fn arm() {}" + nl
    ).split(nl)
    if scan(decorated_ordinary):
        print("SELF-TEST FAIL: decorated mid-doc heading reported:",
              scan(decorated_ordinary))
        ok = False

    print("self-test:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return self_test()

    roots = [r for r in ROOTS if os.path.isdir(r)]
    if not roots:
        print("check-orphan-docs: SKIPPED — none of " + str(ROOTS) + " exist here")
        return 2

    scanned = 0
    crlf_files = 0
    hits = []
    for root in roots:
        for base, dirs, files in os.walk(root):
            # Pruned in place, which is what makes `os.walk` skip the
            # subtree rather than merely ignore its files.
            dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
            for fname in files:
                if not fname.endswith(".rs"):
                    continue
                path = os.path.join(base, fname).replace(chr(92), "/")
                scanned += 1
                raw = io.open(path, encoding="utf-8", newline="").read()
                # ★★★ CRLF normalization, and it is not cosmetic.
                #
                # A substantial minority of this tree's `.rs` files use CRLF. Both
                # regexes above anchor on `[ \t]*$`, and a trailing `\r` is neither
                # a space nor a tab — so without this every line in those files
                # fails to match and the gate reports CLEAN over that whole slice
                # while printing a file count that looks complete.
                #
                # ⇒ *`$` in a line-oriented check is a claim about line endings*,
                # and nothing in a clean report distinguishes "no violations" from
                # "never matched". The CRLF tally is printed for that reason, and
                # the self-test asserts both halves: that `scan` is `\r`-sensitive,
                # and that normalizing restores the hit.
                if CRLF in raw:
                    crlf_files += 1
                    raw = raw.replace(CRLF, chr(10))
                lines = raw.split(chr(10))
                for num, text in scan(lines):
                    hits.append((path, num, text))

    if scanned == 0:
        print("check-orphan-docs: SKIPPED — found no .rs files under " + str(roots))
        return 2

    # Partition against ALLOWED, and hold the exemptions to account.
    used = set()
    real = []
    for path, num, text in hits:
        for k, (suffix, bold, _why) in enumerate(ALLOWED):
            if path.endswith(suffix) and bold in text:
                used.add(k)
                break
        else:
            real.append((path, num, text))

    stale = [a for k, a in enumerate(ALLOWED) if k not in used]

    print("check-orphan-docs: " + str(scanned) + " files scanned ("
          + str(crlf_files) + " CRLF, normalized), "
          + str(len(hits)) + " candidate seam(s), "
          + str(len(ALLOWED) - len(stale)) + " of " + str(len(ALLOWED))
          + " exemption(s) still live")

    if stale:
        print("")
        print("FAIL — an exemption no longer matches anything. It is either fixed")
        print("(delete the entry) or the wording moved (re-measure it). A dead")
        print("carve-out is a hole nobody knows is open.")
        for suffix, bold, why in stale:
            print("  " + suffix + "  " + bold[:70])
            print("      was exempt because: " + why)

    if real:
        print("")
        print("FAIL — doc comment(s) attached to the WRONG item. Each line below is")
        print("the TITLE of a second doc comment that has run together with the one")
        print("above it, so the item beneath them both carries two items' docs and")
        print("the item the first doc was written for carries none.")
        print("")
        print("To repair: find the item the doc ABOVE this line describes. If it is")
        print("still in this file, move that run down to sit immediately above it.")
        print("If it was moved to another file under R2, move the run with it.")
        for path, num, text in real:
            print("  " + path + ":" + str(num))
            print("      " + text[:100])

    if real or stale:
        return 1
    print("check-orphan-docs: clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
