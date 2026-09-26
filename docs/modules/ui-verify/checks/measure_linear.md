# `ui-verify/checks/measure_linear`

`measure_linear_places_a_dimension` — the regression test for **a feature
whose every link has a passing unit test and whose end-to-end effect on the
document nobody had ever observed**.

# The defect class this exists for

R1: a UI change is done when it has been asserted against the running
binary, not when a test passes. A green unit test is the floor.

The measure feature carries dozens of unit tests, including two that prove
a canvas-authored `DimensionKind` is byte-for-byte the one
`pdfcer dimension-add` builds. Every one of them runs without a window.
None of them can state that a ribbon click arms the tool, that a click on
the page becomes a pick, that the third pick raises an action, or that the
action reaches the engine — because each of those is a property of a **call
site**, and a call site's effect is observable only in a running process.

A status word in a table is a claim, and it decays. This check is the
opposite of a status word: six links, driven in order, through the
operating system.

| # | Link | Where | Its own test |
|---|---|---|---|
| 1 | the ribbon click reports the command | `egui-shell`'s `band::render_command` | yes |
| 2 | dispatch routes the id to a `MeasureKind` | `app/dispatch.rs`, via `shell::commands::measure_for_command` | yes |
| 3 | the kind arms the canvas tool | `canvas::tool::arm_measure` | yes |
| 4 | the armed tool renders the control **pressed** | `app/conditions.rs` publishing `selected:measure.linear`, read by `band::render_command` | yes |
| 5 | a canvas click becomes a **pick** | `canvas::interact`'s `Click` arm → `canvas::measure::click` | yes |
| 6 | the third pick's action is **accepted by the engine** | `app/actions.rs`'s `CommitDimension` arm → `vector_edit` → `EditSession::add_dimension` | yes |

Six passing tests, six joins, and no test anywhere observes two adjacent
links being connected.

# Link 6 is the assertion this check exists for, and it is not link 5

A `measure-pick … committed=true` line proves the **shell** raised
`Action::CommitDimension`. It says nothing whatever about whether the
engine accepted it. `app/actions.rs`'s `vector_edit` is explicit about the
difference, because it has two exits and only one of them changes the
document:

```text
Ok  → doc.edit_epoch += 1; doc.page_texture = None
      pdfcer-diag add-dimension page=P n=1 epoch=E disclosures=…
Err → the document is left ALONE
      pdfcer-diag add-dimension-refused page=P n=1 detail=…
```

A build in which `add_dimension` refuses every kind — a bad group id, a
degenerate length, a borrowed session — emits an **identical**
`measure-pick committed=true` and places nothing. That is precisely the
class of defect this harness exists for, so the check asserts on
`add-dimension` (the success event, carrying the bumped `epoch=`) and reads
`add-dimension-refused` only to say *why* in a failure message.

The epoch is the honest signal for a second reason: it is the same counter
`canvas::interact` re-resolves the selection against and
`render::worker`'s key rebuilds the raster on, so a bumped epoch is not a
label the edit path prints about itself — it is the value the rest of the
application reacts to.

# What it does, through the operating system

Mouse only, because a linear dimension is three clicks and *that is the
feature*. Synthetic keyboard input does reach the target window — see
[`crate::checks::add_text`], which types real characters into a caret draft
and asserts they landed — so this check being mouse-only is a statement
about the gesture, not about the harness.

1. Click the **Review** mode segment. Measure is in Review's and Edit's tab
   lists and not in Read's, and Read is the default, so without this step
   there is no Measure tab to activate.
2. Click the **Measure** tab.
3. Capture the window — the *before* picture.
4. Click **Linear** in the Dimension group.
5. Capture the window again — the *after* picture.
6. Click **three points on the page**: A, B, and where the dimension sits —
   clicking a second time at the same point whenever the application says
   the pick it found needs confirming (see the rule-4 section below).

# The assertions, split by oracle

## Trace evidence — that the arm happened

| Assertion | Line | What its absence means |
|---|---|---|
| the click reached the control | `ribbon-command-invoked id=measure.linear` | the click missed, or the control is disabled |
| the tool was armed | `measure-tool tool=Measure(Linear)` | **link 2 or 3** |

The second is genuinely necessary: an armed measure tool is invisible from
outside the process. A crosshair is a cursor, and a screenshot of an armed
canvas and an unarmed one are the same picture, so the only way to see the
arm is to have the running program print what it chose.
`canvas::measure`'s own comment above the `measure-pick` line makes the
same point about picks: *"a first pick and a second are the same
screenshot."*

## Pixel evidence — that the control renders pressed

A trace line is written by the code under test, about itself. `arm_measure`
traces unconditionally the moment it is called, so `measure-tool` proves
links 2 and 3 and says nothing about link 4: a build whose ribbon never
renders a pressed state emits an identical trace and looks identical to a
reader of that trace. So the pressed state is asserted from the captured
window, three ways, exactly as [`crate::checks::markup_rectangle`] does:

| # | Comparison | What it rules out |
|---|---|---|
| P1 | Linear after ≠ Linear before | the control never changed |
| P2 | **Linear after ≠ Two-line after**, in one capture | a *global* repaint — a theme change, a hover, a resize — masquerading as a pressed state |
| P3 | Two-line after = Two-line before | the whole band changing, i.e. P1 passing for a reason unrelated to the click |

P2 is the load-bearing one: a differential inside a single frame, which
nothing that happens to *both* controls can satisfy.

## Gesture evidence — that three picks are taken, and the third commits

```text
pdfcer-diag measure-pick kind=Linear in_progress=true  committed=false   ← A
pdfcer-diag measure-pick kind=Linear in_progress=true  committed=false   ← B
pdfcer-diag measure-pick kind=Linear in_progress=false committed=true    ← where
```

The shape of that sequence *is* the feature. `canvas::measure::click`'s
header states the rule it encodes — **the third pick is the commit, and
there is no accept box** — and records why: decision 024 and
`shell-redesign.md` §2.4 exist because the operator disliked *"a separate
accept / reject box somewhere on the screen"*, and `MODES_AND_PANELS.md`
now makes application-initiated floating surfaces default to Never. A build
that quietly restored the old two-click commit would emit `committed=true`
on the **second** line, and this check is what would say so.

`committed=false` on the first two is asserted as strictly as
`committed=true` on the third, and it is the half that catches the reverse
regression: a tool that committed a zero-length dimension on pick A would
otherwise satisfy "a dimension was placed" perfectly.

### A pick is not always one click, and that is rule 4 rather than a
wobble

Snapping landed after this check was first written, and it changes the
click-to-pick arithmetic in a way that has to be modelled rather than
papered over. `canvas::measure::snapped` resolves every pick through
`pdfcer_core::vector::snap::snap_candidates`, and when the winning candidate
is **derived** — a centreline pdfcer *inferred* rather than one the file
states — `MeasureState::resolve_click` refuses to commit it on the click
that found it:

```text
pdfcer-diag measure-pick outcome=Promoted reason=derived-candidate-needs-confirm
```

That is `pdfce_FeatureRequests/README.md` rule 4's fuzzy-never-sneaky gate:
an inference is announced before it is acted on, and a second click on the
same point confirms it. It is deliberate, it is what
`canvas::snap::snap_commit_clicks` exists to encode, and a check that
treated it as a failure would be filing a defect against the feature.

So a pick is **one or two clicks**, and this check clicks again when it is
told to — modelling the operator, who does exactly that. Three other
responses were available and each was rejected:

| Response | Why not |
|---|---|
| aim the picks at open paper so no candidate is found | lucky rather than honest: nothing about a fixture guarantees a point is far from every endpoint, midpoint and axis, and a check that silently depended on that would start failing the day somebody changed `--pdf` |
| drive with **Alt** held, which refuses the snap | it would assert about a *non-default* configuration. Snapping on is the shipped behaviour, so a check that only ever exercised snapping off would stop covering the path the operator uses |
| loosen to "at least one `measure-pick` line" | it would pass against a build where nothing ever commits, which is the entire failure this check exists to catch |

The strictness is kept exactly where it was. Every click produces **one**
trace line, a `Promoted` line is followed by a click at the *same* point
which must then resolve (`resolve_click` compares the promoted point against
the new one, and the same screen pixel yields the same candidate, so it
converges or the two-click confirm is broken and this check says so), and
the three resolved picks must still read `committed=false, false, true`.
[`MAX_CLICKS_PER_PICK`] is the bound.

### What is deliberately not asserted here: the Tab cycle

`measure-snap-cycle index=N` reports the operator choosing *"the other
candidate"* between an endpoint and the midpoint a few pixels from it. It is
driven by <kbd>Tab</kbd>, and this check does not assert it. It is named
rather than omitted so the gap is on the record: **the snap cycle is covered
by unit test alone.** Nothing prevents driving it — synthetic keys reach the
window — so this is unwritten work rather than a limitation.

## Document evidence — that the engine accepted it

`add-dimension page=… n=1 epoch=… disclosures=…`. See §"link 6" above.

# Where the six clicks are aimed

The three ribbon clicks go to rectangles **the application itself declared
on the frame it drew them** ([`crate::coords::WindowFrame::declared_center`]).
The three canvas clicks go through [`crate::coords::CanvasMapping`], built
from the `canvas rect=`/`zoom=`/`page=` the application traced this run, so
the only spatial literals in this file are the three
[`DocPoint`](crate::coords::DocPoint) fractions in [`PICKS`] — which is the
one literal [`crate::checks`] rule 2 permits, because a document coordinate
is stable under every layout change the roadmap contemplates.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the page size could not be read from the fixture and no `--page-size` was
  given — without the page height there is no y-flip, and a wrong page
  height mirrors every click about the page centre, landing on the page and
  hit-testing something plausible;
* the mode segment, the Measure tab, or the Dimension group's controls were
  never declared — each names the specific surface that is missing, and the
  `ribbon.item.*` case names `report::band_item` and its call site;
* a measure tool was already armed before the click — `arm_measure`
  **toggles** on the same kind, so a click on an already-armed Linear
  correctly *disarms* it, and a check that did not notice would report the
  feature broken;
* the two controls already looked different before the click, so a
  difference afterwards could not be attributed to it;
* the canvas is not showing page 1, so the harness's one known page size
  does not describe the page it would be clicking on;
* a pick point does not map onto the canvas as currently laid out.
