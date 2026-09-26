# `ui-verify/checks/off_page_census`

`the_off_page_census_finds_the_object_and_marks_it` — **the discovery
third of the operator's off-page question, driven end to end.**

# The question this exists to answer, in his words

> *"how do I view and edit objects that are off of the page? we added this
> feature but I didn't see how to enable it."*

Three verbs, and two of them were already true when he asked. This module's
siblings measure those:

| third | check | what it settles |
|---|---|---|
| **see** | `off_page_visible` | an object beyond the media box is actually painted |
| **reach** | `off_page_press` | a press in the grey is a gesture, so it can be banded and dragged |
| **zoom** | `off_page_zoom` | it survives being zoomed in on |
| **find** | *this one* | the operator can learn a sheet has anything out there **without already knowing** |

The fourth is the one that was missing, and it is the only one that
scales. Seeing and reaching are answers to *"I know something is over
there"*; on a thirty-six sheet drawing set nobody knows that, and scrolling
every sheet out to its pasteboard to check is not a procedure a person
performs. The census is what turns a capability into something he can use.

# What is asserted, in order, and why each line is the one chosen

1. **`offpage-opened pages=N`** — the window exists and knows how much work
   it has. A command with no dispatch arm reaches nothing and traces
   nothing, which is the failure `unembed_fonts` was written after.
2. **`offpage-scanned … dirty=D objects=O`** — the walk FINISHED. This is a
   window that does real work after it opens, one page per frame, so the
   difference between *"scanning"* and *"stalled after page one"* is exactly
   the difference a check has to be able to state. The line is emitted once,
   on the completing frame, which is what makes `Trace::last` usable here.
3. **`objects >= 1`** — the assertion that can actually fail. The fixture
   is chosen so a clean answer is a DEFECT rather than a fact about the
   input: `fixtures/off-page-object.pdf` is 485 bytes of hand-written syntax
   with one square on the page and one entirely beside it, and its sibling
   checks all depend on that second square being there. A census that
   reports nothing on this file has failed to find the thing the rest of the
   group proves is visible, reachable and zoomable.
4. **The `offpage.mark` region, then a click on it** — the control the
   operator presses, laid out and hit in the window's OWN frame.
5. **`offpage-mark-requested bands=B`** — the press became an action.
6. **`redact-mark-offpage … epoch=…`** — the action reached the document
   through the edit funnel.

## Why `objects` and not `dirty`

`dirty` counts PAGES with something outside; `objects` counts the marks. On
a one-page fixture `dirty` can only ever be 0 or 1, so an assertion on it is
one bit wide and is satisfied by a scan that found the page and
misattributed why. `objects` is the number the operator is actually shown,
and it is the number a wrong tolerance or a wrong page box moves first.

## Why the applied line is matched on a FIELD and not on its name

The edit funnel writes `redact-mark-offpage-refused page=… detail=…` on the
branch where the document is **untouched**, and that line shares the success
line's first token — which is what `Trace::last` matches on. The success
line carries an `epoch=`, the refusal carries a `detail=`. The field is what
tells them apart, so this check filters on the field. Asserting the bare
name would report a refusal as a pass, which is the precise shape of
`RESUME.md`'s recurring "a trace-grepping check passes on a build that
failed" finding.


It passed on its first drive, which is the moment a check is least worth
believing. So the fixture constant was swapped to `fixtures/paragraph.pdf`
— a document with nothing beside its sheet — and the run went **RED** on the
`objects >= 1` line, quoting `dirty=0 objects=0`. The oracle therefore reads
the census's actual answer rather than the fact that a window opened, which
is the failure mode every trace-grepping check in this suite has had at
least once. The clean-fixture run also never reached the press, so the
greyed-button branch below is reasoned, not measured.

What remains unfalsified and is named so nobody reads it as measured: the
`epoch=` filter on the applied line. No build has been driven in which the
funnel refuses this edit, so that clause is argued from the funnel's source
rather than from a red run.

## What this check does NOT assert, stated rather than implied

- **That the marks are in the right place.** The action carries BANDS —
  rectangles covering the strip of pasteboard outside the sheet — not the
  bounding box of each object. A band check would need a pixel oracle over a
  redaction preview, which is a surface that does not exist.
- **That anything is removed.** It must not: this command **marks**, and
  `edit.redact_apply` is a separate, deliberate second press. A check that
  asserted destruction here would be asserting the defect.
- **The multi-page case.** The disclosure that undo takes the marks back one
  page at a time only appears above one page, and this fixture has one. The
  engine has no undo-grouping verb — grepped, not assumed — so that sentence
  is the honest half of a limitation rather than a feature, and measuring it
  wants a multi-sheet fixture that does not exist yet.
