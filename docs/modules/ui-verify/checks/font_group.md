# `ui-verify/checks/font_group`

`the_format_tab_offers_font_controls_for_swept_text` — **the ribbon route
to a restyle, and the sentence that tells an operator how to reach it.**

# What this is for, and how it differs from `restyle_text`


O37 shipped with an admission written into its own row:

> You are in Edit mode, so a drag with the Select tool draws a marquee round
> objects. Press **T** first — that arms the text tool — then sweep across
> the words. **That is a discoverability gap and it is ours, not a
> limitation. Nothing on screen tells you to press T.**

Three surfaces now do. This check asserts two of them in the state an
operator is actually in when they need them, which is the state **before**
anything is swept — and that ordering is the whole design of the check.

## The two phases, and why the first one has to come first

| phase | the operator's state | what must be true |
|---|---|---|
| 1 | clicked a piece of text with the Select tool; **nothing swept** | the Format tab appears, it carries a **Font** group, and the Properties panel says how to get an operand |
| 2 | pressed `T`, swept the words | the ribbon's **Bold** commits a restyle to the document |

Phase 1 cannot be reached after phase 2, because a sweep is not undone by
clicking again — so a check that swept first would have destroyed the state
it is meant to observe. That is not a harness convenience; it is the
operator's own sequence. They click the thing they want to change *before*
they know a sweep is needed, which is precisely why the gap existed.

## What phase 1 can and cannot see, said plainly

The ribbon publishes `ribbon.item.<id>` for **every** command control,
enabled or greyed — deliberately, and `egui_shell::ribbon::control`'s own
note says why: *"the question a consumer asks is where is this control, and
a control that is greyed is still a control that was drawn somewhere."*

So a region tells this check the control is **on screen**. It does not tell
it the control is greyed. That is a real limit and it is not papered over:
the greying is asserted by
`app::conditions::tests::the_font_groups_visibility_follows_the_mode_and_its_enablement_the_sweep`,
which reads the registered command's own predicate against the published
conditions — the join, not either half. What *this* check adds, and what no
unit test can, is that the controls are **drawn at all**, on a real ribbon,
in a window, at a real width, on the tab that really appeared.

And the appearing is itself under test. The Format tab is contextual: its
`visible_when` moved from `selection.any` to `selection.formattable` when
the Font group landed, and a build that missed that change shows **no tab**
after a sweep and therefore no Font group — a whole feature with no surface,
which is exactly the shape of defect this project exists to catch.




The answer was in the trace the check was already holding:
`pdfcer-diag properties-panel object=832 kind=Path notes=0`. So
[`aimed_at_one_text_object`] now reads that line, plus
`canvas-selection … sel=N`, and **skips** — never fails — when the click did
not leave exactly one text object selected. Phase 2 had this guard from the
start (`chars == 0` is a skip, not a failure); phase 1 did not, and the
asymmetry is what let a correct program be blamed.

The correct aim for this fixture is `--doc-point 0,1140,62`, which
`RESUME.md`'s aim table gives and the sweep did not use: a 5 pt title-block
run at PDF (1135.7, 58.4)–(1190.5, 63.4).

# The oracle

Phase 1: the precondition above, then the regions `ribbon.tab.format`,
`ribbon.group.format.font`, the five `ribbon.item.format.*`, and
`properties.text` with `properties.text.face` inside it.


Phase 2: `text-style-applied … applied=N` **and** the `format-text` label
`vector_edit` writes when the edit reached the engine — the same two-line
oracle `restyle_text` uses, for its reason: the first without the second is
a module that decided to act and whose action never landed.

## Item notes

### `const VK_T`

Pressed only in **phase 2**, and the fact that phase 1 works without it is
the point of the check: the whole complaint is that an operator does not
know to press it, so the surfaces that tell them must be observed in the
state where they have not.

### `enum Aim`

Separated from the wording above so the READ is testable without a running
program: every variant here is reachable from a three-line trace, and the
tests at the foot of this module reach all four. That is the whole point of
the split — a guard against a harness misreading its own oracle is worth
nothing if the guard itself can only be exercised by driving the mouse.

### `fn aim_verdict`

**Order matters, and it is "what" before "how many".** A click that lands
on a path inside a marquee of eleven is an aim problem twice over, and the
kind is the more useful half to be told about: it names the fixture
coordinate that has to change. Reporting "11 objects selected" first would
send a reader looking for a stray Shift.

### `fn list_of`

`driving::list` takes owned `String`s and `driving::list_str` takes a slice
of `&str` — this is the latter, spelled locally only because the filter
above produces a `Vec<&str>` and handing it straight over reads better than
a collect-into-owned at the call site.

### `fn the_two_font_lists_describe_the_same_five_controls`

Two hand-written lists over one set is the shape a completeness check
goes blind in: a sixth control added to the band and to one list reads as
a measured group from either end. The pairing is mechanical — a region
is `ribbon.item.` followed by the command id — so it can be asserted
even though neither list is derived from the other.

### `fn a_click_that_landed_on_a_path_is_the_harnesss_aim_and_not_a_defect`

These are the actual values from a run at `--doc-point 0,300,500` — a
drawing view on `SW41177.pdf` — where the check reported the program's
correct silence as O37's complaint returning.
The verdict must be a SKIP that names the kind, so the reader is sent to
the fixture coordinate rather than to `panels::properties::text`.

### `fn a_multi_selection_whose_first_object_is_text_still_skips`

This is the case `properties-panel` alone cannot see: it describes the
first selected object and says nothing about how many there are, which
is why the count is read from `canvas-selection` instead of inferred.

### `fn a_missing_selection_count_is_zero_and_not_one`

`canvas-selection` is written through `diag::trace_changed`, so a run
that never changed its selection carries no line — and defaulting that
to one would let the guard pass on silence, which is the failure mode
the guard exists to end.

### `fn the_oracle_names_are_the_ones_the_program_writes`

Pinned here rather than trusted: the two lines are quoted verbatim from
`canvas::trace` and `panels::properties::mod::object_section`, and the
kind spelling is `summary::ObjectKind::Text` under `{:?}`.

### `const FONT_ITEMS`

Asserted as a **list**, not as "Bold is there". Three of the five are
`Item::Custom`s drawn by `app::fontband`, and a custom item that the
manifest names and no renderer matches draws **nothing** while the shell
reserves its space — which is the defect `COLOUR_SWATCH` shipped with for
the whole of v0.1.0, invisible because a gap in a band looks like a gap in a
band. Only naming all five catches it.

### `const FONT_COMMANDS`

This module PINS its fixture: it opens `fixtures/paragraph.pdf` at a
measured point and ignores `--pdf` and `--doc-point`, because its subject
is a discoverability route and a route needs a known page.
`font_group_real` is the twin that does the opposite -- it honours the aim
and drives whatever drawing the operator names -- and it shares these two
lists, the aim guard and the enablement renderer rather than copying them.

Copying would have been the ordinary move and it is the one this
repository has already paid for nine times over in private `click_tab`
helpers: a shared list diverges silently, and a group measured against a
stale copy of its own membership reports a measured group.

# A second list, because a region and an enablement are different facts

[`FONT_ITEMS`] holds published REGION names (`ribbon.item.format.bold`) and
answers *where is this control*. These are the ids the same five controls
are registered under, and they are what the enablement event is keyed by,
because that event is about a COMMAND rather than about a rectangle.

The two lists are asserted to line up by
[`the_two_font_lists_describe_the_same_five_controls`], which exists because
a check that read four regions and five enablements, or five regions and
four enablements, would report a measured group either way. The pairing is
`ribbon.item.` + the id, and it is spelled out rather than computed so that
a rename on either side is a compile-visible edit to a literal instead of a
silently-still-passing concatenation.

### `const FACE_ROW_REGION`

Asserted ALONGSIDE the section, not instead of it, for the reason
`FONT_ITEMS` gives about the ribbon band: a section that draws its heading
and then returns before any control is the exact shape of the regression
this check exists to catch, and a section-level region cannot see it.

### `fn aimed_at_one_text_object`

A bounded poll rather than a fixed sleep, for `restyle_text`'s reason: a
restyle re-resolves its pin from a fresh provenance extraction per run, so a
sweep across a title-block label is a dozen extractions, and a fixed sleep
long enough for the worst case makes every run slow while a pleasant one
reads the trace mid-gesture and reports "nothing happened" about a gesture
that is still running.
**Did the phase-1 click land on one text object?** `Ok(())` if it did; an
[`Error`] — which this check's `run` turns into a SKIP — if it did not.

# Why this exists, written on the day it was needed


The trace had the answer on the same frame the check was already reading:
`pdfcer-diag properties-panel object=832 kind=Path notes=0`. Nothing new had
to be published for this guard; the check simply had to look.

# The two facts, and why both are needed

`route` draws when **exactly one** object is selected **and** it is text.
Those are separate refusals with separate causes, so they are read
separately:

| fact | read from | why not the other line |
|---|---|---|
| how many are selected | `canvas-selection … sel=N` | the panel describes only the FIRST, so it cannot count |
| what the first one is | `properties-panel … kind=K` | the canvas line names a `TargetId`, not a kind |

# An absent `properties-panel` line is "selected nothing", not "unknown"

`object_section` writes that line unconditionally once it has an object to
describe, every frame, through `diag::trace` rather than `trace_changed`. So
its absence after a settled click means `object_indices_on` came back empty
— no page-content object under the pointer — which is an aim problem of its
own and is reported as one.

### `fn describe`

One line, both numbers, every id — including the ones that PASSED. A
message that lists only the offenders leaves a reader unable to tell
*"three of five are dead"* from *"three of five never reported"*, and those
two want different investigations. `live=?` marks a control whose renderer
has no second predicate, which is not the same as one whose second predicate
said no.
