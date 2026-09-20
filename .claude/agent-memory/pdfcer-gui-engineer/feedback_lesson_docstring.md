---
name: a-lesson-in-a-docstring-is-not-an-instrument
description: When a finding generalises, the next sibling gets written the same way anyway — build the sweep in the same session, don't write the paragraph
metadata:
  type: feedback
---

When a defect turns out to be a **class** rather than an instance, writing the
generalisation down does not stop the next instance. Build the thing that looks
for the class, in the same session, or it recurs.

**Why:** the index-vs-working-tree defect (see
[[a-gate-whose-input-set-comes-from-git-measures-the-index]]) was written
four times in this repository. The first instance,
`tools/check-suite-name-absent.py`, **recorded the correct generalisation in its
own docstring** — *"a gate whose input set is 'what is already committed' cannot
see the commit you are about to make"* — and it was written **before instances 2,
3 and 4 happened**. Instance 2 was repaired by excluding one directory. Instance 3
was the same gate again, and cost a release pre-flight that declared 41 of 41
green on a tree the packager failed thirty minutes later. Instance 4,
`check-doc-markup`, had **never once fired** in its whole life and was found only
because an audit went looking.

The prose was correct, specific, and adjacent to the code. It propagated to
nobody, because **nothing swept for the pattern.** One ~470-line script closed
the class permanently; three paragraphs had not closed it in a week.

**How to apply:**

- The trigger is the **second** occurrence, not the fifth. A second instance is
  the announcement that there is a mechanism, and the mechanism is the finding —
  repairing only the instance buys about a day.
- Ask *what would I grep for to find the next one?* If that question has an
  answer, it is a script, and writing the script is the work. If it has no answer,
  the generalisation is not yet sharp enough to be worth writing down either.
- The instrument must not carry the defect it hunts. The gate for this class walks
  the tree with `os.walk` and never asks git anything, for exactly that reason.
- Pair it with an escape hatch that states a reason, not a name: a marker like
  `gate-input-scope-exempt: <reason>` keeps the legitimate cases legible and
  distinguishes them from the ones nobody thought about. A hard-coded exclusion
  list is how instance 2 got "fixed" — see
  [[an-unevidenced-excuse-is-worse-than-silence]].
- Register it in the runner in the same commit. A checker named in every document
  and registered in no runner runs when somebody remembers — see
  [[a-checker-named-in-every-document-and-registered-in-no-runner]].
- Then falsify it. This class falsifies in one step: plant the violation in an
  **untracked** file, which a gate with the hole cannot see at all.


## ★★★ TWO MORE BUILT — 2026-09-13, one of them at the sixth recurrence

**`check-completeness-tests.py`** — the class is *"a completeness test that
carries its own copy of the set is testing the copy"*, written into memory
**five times** under five separate incidents before anything was built. The
sixth instance is what paid for it. See
[[a-hand-written-list-inside-a-completeness-test-is-the-gap]].

**`check-memory-index.sh`** — and this one is the sharper illustration, because
the class it hunts is *this file's own class*. A memory topic file that
`MEMORY.md` does not name is unreachable: only the index is loaded into a
session, so a file the index omits is written, committed, reviewed and never
read again. One was found on 2026-09-13, and it was
`user_he_is_not_at_the_keyboard_unless_he_says_so.md` — the standing rule that
decides whether a session drives the release binary or defers the work back to
the operator. It surfaced from a `comm` run done for an unrelated reason.

⇒ **The generalisation that ties both to the rule above:** writing the
artifact and REGISTERING the artifact are two edits, and the second is the one
that gets skipped, because the first is where the thinking was. A memory file
and its index line. A gate and its `run` line. A `--self-test` and its dispatch.
Every instance of this class in this repository has that shape.

★ Practical note on adopting a gate onto an existing tree: the honest form is
a **debt register that prints its own count every run**, not an exemption list,
and both directions must be red — a new site AND a register line that matches
nothing. The second is what stops the register quietly ceasing to describe the
tree.

## A THIRD — 2026-09-16, and this one is the cleanest illustration yet

**`tools/gates/check-scroll-row-wrapping.sh`** — the class is *"a plain
`ui.horizontal` lays out past the end of its column, so inside a fixed-size
window it raises a scrollbar the operator cannot dismiss"*.

The illustration: the first instance was diagnosed, fixed, and written up at
length — in a RAG entry, in the module's own header, and in a comment block
immediately above the repaired row. **The second instance was in the same file,
about a hundred lines below that comment block, and had been there all along.**
Ken reported it in the same seven words he used the first time. A third latent
one was sitting in a group added the same day.

Seven candidate rows existed in the whole tree; five were converted in minutes
and two took a stated reason. The cost of the instrument was under an hour, and
the prose it replaces had already failed twice.

**How to apply, sharpened:** the question is not *"is this finding written
down?"* but *"what would I grep for?"* — and if the answer is a directory plus
a token, it is a gate and the writing is the work. Scope it to a **directory**
so a new module is covered the day it lands, exempt with a **stated reason**
rather than a path list, and make **both directions red**: a new violation AND
an exemption marker that no longer sits above the thing it excuses. Then
falsify it against the real files, not only the self-test — see
[[falsify-the-gate-against-the-real-files-and-the-fix-against-a-control-binary]].

## The trigger — "confirm you have built every X", and the list is on the OTHER side

When Ken asks to **confirm** that something is complete — *"confirm that you
have built every editable surface into the GUI that has been implemented in
pdfcer"* (2026-08-28) — the answer is a **script**, not a reading of this
project's documents.

**Why:** on 2026-08-28 the question could not be answered from `FEATURES.md`,
`NO_SURFACE.md` or `GUI_ROADMAP.md`, and the reason is structural rather than a
gap in any of them: **all three are keyed on what this shell does.** None is
keyed on the engine's verb list, so none can answer *"is there a verb
`pdfcer-core` implements that nothing here calls?"*

A 60-line script (`tools/verb-coverage.py` — parse `impl EditSession` out of
`edit.rs`, grep this crate for each `pub fn` name) answered it in two seconds
and found **twelve gaps**, including:

- two operator **settings** that were persisted, validated, drawn in a window
  and honoured by nothing;
- three capabilities the engine had shipped **in answer to this shell's own
  requests** and this shell had then never consumed.

★★★ The last one is the pattern worth carrying: **a reply arriving is not a
capability landing.** The engine session runs in parallel and answers within the
hour, and three separate times its answer sat in
`D:\Dev\FeatureRequests\pdfce_FeatureRequests\open\` while our own doc comments
still said the capability was blocked.

**How to apply:**

1. Any "have we done all of X" question → find the authoritative *list* of X
   (usually on the other side of a crate boundary), enumerate it mechanically,
   diff it against what this side names.
2. Commit the instrument, not just its output — a register that is trusted
   rather than re-measured becomes the next stale blocker. `EDITABLE_SURFACES.md`
   says *"re-run it before quoting any number in this file"* on its first screen.
3. State what the measurement is worth: a **miss** (identifier appears nowhere)
   is strong; a **hit** is weak (a call site behind a condition nothing sets is a
   hit here and dead in the running program).
4. The same shape works for guards: a funnel keyed on option *constructors* was
   blind to a setting delivered by a *setter*, and the way to find the second
   delivery mechanism was to enumerate the engine's API, not to re-read the
   guard. See [[a-backlog-row-is-a-record-not-evidence]].
