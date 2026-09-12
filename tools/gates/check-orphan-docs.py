#!/usr/bin/env python3
"""check-orphan-docs.py — two items' doc comments must not run together.

===========================================================================
WHY THIS GATE EXISTS
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

The same end state arrives by a second route. `author` was split out of
`app/actions/forms.rs` into `forms/author.rs` under R2 on 2026-08-30 and **its
doc comment was left behind**, where it merged into the next item's. Different
cause, identical symptom; the repair differs, so the diagnosis has to come
first. The discriminator is whether the undocumented owner is still in the same
file.

★★★ NOTHING IN THE TOOLCHAIN CAN SEE THIS. `rustc` is happy — a doc comment is
valid prose wherever it sits. `cargo fmt` is happy. `cargo clippy -D warnings`
is happy. The only observer that would notice is `cargo doc`, and nobody runs
that on a binary crate.

★ How many, measured rather than asserted — because the sentence that used to
sit here said *"ten instances in this crate"* and was wrong in all three of
its clauses once the scope widened. As of **2026-09-12**:

    grep -rc 'Moved here on 2026-09-12' crates tools --include='*.rs'

reports **15** repaired in place, and one more was repaired across files
(`author.rs`, below), for **16** total — **11 in `pdfcer-gui`, 5 in
`tools/ui-verify/`**. The eleven accumulated over thirteen days and shipped in
four releases; the five in the harness never shipped, because the harness is
not a product — which is exactly why nothing was ever going to notice them.

The count is written beside the command that produces it on purpose. A bare
number in prose has no oracle, and this project has corrected eight of them —
one of which was in this file's own header. ⇒ *a statement that is accurate
about something other than what it is attached to is the defect this gate
exists for; a stale count in its header is the same defect in the instrument.*

★★★ AND IT FALSIFIES CITATIONS, WHICH IS WORSE THAN LOSING A DOC. The clearest
instance found: `sys/win32.rs` documented `is_foreground` with

    /// `key_stroke`'s own doc comment has said since it was written that "the
    /// input driver refuses to type when the foreground window is not the one
    /// under test".

while `key_stroke` had no doc comment at all — its doc had been absorbed by
`wheel`, a function about the mouse wheel, thirteen days earlier. The quotation
was **verbatim accurate**; only the claim about where the words lived was false.
A reader following it landed on a paragraph about scroll detents and would
reasonably have concluded the citing sentence was invented.

The repair needed no rewording: moving the doc to `key_stroke` made the citation
true, because the only thing wrong with it was the location of what it cited.
⇒ *an orphan does not merely leave an item undocumented — it silently falsifies
every cross-reference to that item without making a single word of those
cross-references wrong.* A missing doc is absent; a misdirected citation
**actively misleads, and reads as the citation's fault.**

What finally exposed one was unrelated and lucky: deleting an absorbing item
left a blank line after a doc comment, which clippy *does* lint
(`empty_line_after_doc_comments`). Chasing that one lint rather than silencing
it found the other nine. ⇒ *a lint firing on the shape next to a defect is the
only warning this class gets, so never paper one over.*

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

★ The first draft of this check looked for a bold title **after** a paragraph
break and found **zero**. That is the shape of an ordinary mid-doc heading; the
anomaly is the opposite one. ⇒ *a detector aimed at the normal shape reports
zero and looks exactly like a clean tree.* The second draft looked for `**`
anywhere in a doc line and found **717**, essentially all of them mid-sentence
emphasis wrapped across a line break. Only the third measured anything.

---------------------------------------------------------------------------
Two structural limits, and the one it cannot close
---------------------------------------------------------------------------

1. **An indented bold sentence inside a bullet list is not a title.**
   `pagesize.rs` has `///   **That asymmetry is the disclosure.**` as the last
   sentence of a `*` bullet. Closed precisely, by requiring **exactly one**
   space between `///` and `**`: a title is never indented, a list continuation
   always is.

2. **A bold sentence that merely begins at a line boundary is indistinguishable
   from a title.** `canvas/tool/arm.rs` documents `arm_text_edit` with

       /// Arm the caret tool with `kind`, or retire it if that kind is already armed.
       /// **The entry point the `edit.text` and `edit.add_text` dispatch arms call.**

   and its twin `arm_markup` carries the *same sentence pair*, differing only in
   where the wrap falls — there the bold starts mid-line, here at column 5. No
   line-oriented regex can separate those, because the distinguishing fact is
   semantic. So it goes on `ALLOWED` below rather than being papered over by a
   looser pattern that would also stop seeing real orphans.

3. **★★★ The bold-title convention is ONE shape, and this gate sees only it.**
   Not a tuning gap — a measured ceiling, established three ways on
   2026-09-12 so that nobody re-derives it:

   * A hand read of `sys/win32.rs` found an orphan whose title is plain
     prose — *"Turn the mouse wheel at the pointer's current position."* No
     pattern built on `**` will ever reach that one.
   * Dropping the bold requirement (any one-sentence `///` line above a
     completed sentence, bare `///` below) gives **126** candidates. Too
     loose to gate on: a two-sentence paragraph whose closing sentence
     happens to fill one line has the identical footprint, and most of the
     126 are precisely that.
   * Requiring **bold LEAD words** rather than a fully bold sentence gives
     **20**, which a person can read in ten minutes. Eighteen were benign,
     one was a real orphan on a constant in `launch.rs`, one was already
     known.

   ★ The discriminator that made that triage cheap is not a prose heuristic
   and is the right direction for any future widening: **a real orphan
   implies an undocumented item in the same file**, so
   `clippy::missing_docs_in_private_items` is the census every remaining
   orphan hides in. On 2026-09-12 it reported **429** undocumented items
   (130 in `pdfcer-gui`, 299 in `ui-verify`; 211 functions, 147 fields, 46
   constants, 18 methods, 3 variants, 3 associated functions, 1 struct).
   Far too many to turn on as a gate — closing that is an operator-scale R5
   decision, not a cleanup — but as a **filter over regex candidates** it is
   exact where a regex is only suggestive.

   ★★ And it has a trap that cost a wrong all-clear the same day. The triage
   narrowed that census to a handful of item kinds, then printed *"benign —
   no undocumented item below it"* for the `launch.rs` candidate. A constant
   sixteen lines below it **was** in the census; the narrowing produced the
   verdict, and stated it in the same words a real all-clear would use. ⇒ *a
   verdict computed from a filtered population inherits the filter's blind
   spot and reports it with undiminished confidence.*

★★ `ALLOWED` is checked for STALENESS. An entry that no longer matches anything
fails the gate. The lesson is this project's own, recorded twice: an exemption
outlives its reason, and a gate carrying a dead carve-out is a gate nobody knows
has been narrowed. If the arm.rs wording is ever reflowed, this gate says so
rather than quietly keeping a hole open.

===========================================================================
USAGE / EXIT CODES
===========================================================================
  tools/gates/check-orphan-docs.py              check the crate
  tools/gates/check-orphan-docs.py --self-test  falsify the mechanism

  0  clean
  1  an orphaned doc comment, or a stale `ALLOWED` entry
  2  SKIPPED — no source tree to scan (see run-all.sh's three-state model)
"""

import io
import os
import re
import sys

# ★★★ BOTH roots, and the reason is a measurement this gate failed.
#
# It shipped scanning `crates/` alone and printed "776 files scanned",
# which reads complete and is 77% of the tracked `.rs` files here. The
# missing 232 are `tools/ui-verify/` — the harness, 206 of them under
# `src/checks/`, each carrying the long doc comment R5 asks for. The one
# instrument aimed at misattached documentation was not reading the most
# documented tree in the project.
#
# Found by an unrelated number disagreeing: `git ls-files --eol` counted
# 147 CRLF `.rs` files against this gate's 126. The gap was not line
# endings, it was the denominator. ⇒ *a scope constant is a claim about
# coverage, and a file count is not evidence for it.*
#
# `check-file-size.sh` has scanned both roots since it was written. The
# divergence was never argued; it was a default nobody questioned.
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

# A COMPLETE bold sentence alone on a line, with EXACTLY one space after `///`.
# That one space is limit 1 above: a title is never indented inside a list.
TITLE = re.compile(r"^[ \t]*/// \*\*.+[.!?]\*\*[ \t]*$")

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
                # ★★★ CRLF, and this gate shipped blind to it for one run.
                #
                # 126 of this crate's 776 `.rs` files use CRLF. Both regexes below
                # anchor on `[ \t]*$`, and a trailing `\r` is neither a space nor a
                # tab — so every line in those files failed to match and the gate
                # reported CLEAN over 16% of the tree while printing a file count
                # that looked complete.
                #
                # It surfaced only because a falsification run against ten known-bad
                # files disagreed with the gate on one of them. ⇒ *`$` in a
                # line-oriented check is a claim about line endings*, and nothing in
                # a clean report distinguishes "no violations" from "never matched".
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
