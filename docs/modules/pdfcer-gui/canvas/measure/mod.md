# `canvas::measure` — the dimensioning tools

Phase 7. Placed under `canvas/` rather than at `tools/measure/`, following
the precedent this shell sets: [`crate::canvas::markup`] is the other
on-canvas
authoring tool, it lives here, and a measure tool is the same kind of thing
— a gesture that reads the page and raises an `Action`.

## The parts

| module | what it holds | egui? |
|---|---|---|
| [`pick`] | the pick state machines — linear, circular, two-line | **no** |
| [`circular`] | the radius/diameter tool: its pick, its **two endings**, its outlines | yes |
| [`scale`] | the scale-entry and dimension-group model — salvaged, **not yet reachable**; see the note on `MeasureKind` | **no** |
| [`state`] | the container built on tool entry, and what it discards | **no** |
| `draw` | what the tools show before a click commits, and the one projection every mark goes through | yes |
| this file | the canvas hosting: picks, overlay, disclosure | yes |

[`circular`] is the newest row and the only one that hosts a single tool.
**R2** forced it out of this file when the radius/diameter tool was armed
(1,617 lines), but the line count only says that *something* had to move;
that module's own header carries which subject was separable and why —
briefly, it is the only measure gesture that does not end itself, so the
machinery for *saying when it is over* has nothing corresponding to it in
the other two tools.

[`state`] is a third row rather than the two the salvage planned, and the
reason is **R2**: the old `measure_tool.rs` is 2,044 lines, and the planned
two-way split still leaves the pick half about twenty lines over the
1,500-line limit. It was cut once more at a seam the original had already
drawn for itself — a `// ---` banner separating the three tools' individual
pick machines from the single container that owns all of them — rather than
by shaving doc comments to fit a threshold, which is the incentive
`tools/gates/check-file-size.sh` says in its own header it exists to refuse.
That module's own docs carry the full argument.

**`pick`, `scale` and `state` never see an `egui` type**, and that is carried
across deliberately from the old shell's `measure_tool.rs`, whose header
makes the argument: every transition is unit-testable without a live frame.
It is also what let the whole file be salvaged rather than rewritten.

## This module owns no geometry

Every load-bearing computation is a call into the already-shipped
`pdfcer-core::dimension` / `pdfcer-core::vector` — the Taubin best-fit circle,
the axis-constrained projection, the measured length, the scale back-calc,
and `author_from_two_lines`. The rule the old shell stated and this one
keeps: **reuse, never reimplement**, so a dimension authored on the canvas
is byte-for-byte the one `pdfcer dimension-add` writes.

## Item notes

### `fn trace_pick`

A function rather than the `format!` written twice, because the circular arm
returns before the tail of [`click`] and a harness reading this channel must
not have to know which arm produced its line. Two spellings would drift on
the first field anyone added.

### `fn arming_another_measure_tool_discards_the_circle_fit`

`MeasureState::set_kind` owns the rule and has its own tests; this
asserts the *hosting* applies it, because `load` is what calls it and a
hosting that skipped the call would carry a fit set into the linear tool
— where it would sit invisible, unfinishable, and would reappear the
moment the operator came back.

### `mod pick`

The visibility is wide for one reason and it is the right one: a picked
point's ORIGIN is an operator-facing disclosure — `OPERATOR_REQUESTS.md`
O106 — and every operator-facing string in this crate lives in
`crate::text`, which `tools/gates/check-ui-strings.sh` enforces. The
alternative was a second enum in the text module mirroring this one, which
is the mirrored-type drift `canvas::target`'s header exists to refuse.

### `fn cycle_snap`

Tab's claimant while a measure tool is armed. Two candidates a few pixels
apart — an endpoint and the midpoint of the segment it ends — are
indistinguishable by pointing, so the operator needs a way to say *"the
other one"*. That is what the cycle index is, and
[`snap::next_snap_index`] is the salvaged rule for advancing it.

# Why it does not need the candidate list

It advances the index and lets [`snap::active_snap_candidate`] wrap it
against whatever list the next frame produces, rather than reading the list
here to bound it. That is deliberate: the list is rebuilt every frame from
the live pointer, so a bound taken now would be a bound on a list that no
longer exists by the time it is used. `active_snap_candidate` already takes
the index modulo the list it is given, which is the only place the two are
guaranteed to be the same list.

Returns `false` when no measure state exists, so Tab falls through to
whatever else wants it rather than being silently eaten by a tool that is
not armed.

### `fn active_group`

The half of [`load`] that has no side effects, for the two callers that must
not manufacture a state merely by asking: the ribbon's enabled-when
predicate ([`finishable`]) is evaluated on **every frame**, and the Finish
command's arm may be reached by a chord with no tool armed at all. `load`
would build a fresh `MeasureState` for either, which would then be written
back by the next `store` and leave the canvas holding a state for a tool
nobody armed.
The group the next dimension will join, for a surface that needs to name
it — today, the Set-scale dialog.

# Why `Option`, and why the caller does not get a default

`None` means the measure tool has never been entered this session, so there
is no state in `egui::Memory` and no group has been chosen. The caller could
substitute `DEFAULT_GROUP_ID` — and must not, silently: calibrating the
default group when the operator has been working in a named one is a wrong
answer that looks exactly like a right one, because both are "a group got a
scale".

`crate::app::dispatch`'s arm therefore falls back **deliberately and with a
trace line**, so a run where the fallback fired is distinguishable from one
where it did not.

### `fn set_active_group`

The write half of [`active_group`], and the control the ui-spec calls the
*group picker* (§2.6). `MeasureState::group` carries that meaning, and
`crate::dialogs::dimension_groups` is what writes to it. **Without a
writer** a second group can exist, carry its own scale, and be joinable by
nothing.

# Why it manufactures a state when there is none, and why not through `load`

It must **create** the state if there is none: the Manage-groups window can
be opened with no measure tool ever armed, and an operator who picks a group
there and then arms Linear expects their choice to have survived. Reading
and giving up would drop the choice silently.

It does not go through [`load`], though, and the difference is not cosmetic.
`load` takes a page and a kind and **synchronises to both** — including
`clear_gesture()` when the page differs. This function has neither to hand
and has no business discarding a pick in progress on behalf of a window that
is not the canvas. So it seeds a bare state at page 0 instead, which the
tool's own `load` corrects on the frame the tool is next armed; a fresh
state has no gesture, so the correction discards nothing.

# Why it is not an `Action`

It changes no document. It says where the *next* gesture's product will go,
which is application state with no undo log to order against and nothing to
alias — the same argument `canvas::markup::swatch` makes about the pen, one
value along. The dimension it eventually authors **is** an action, and
carries this group in it.

### `fn finishable`

# Why this is one function rather than one per tool

Because `measure.finish` is ONE command. The ribbon shows one control, the
operator presses it once, and it has to mean *"end whatever I am in the
middle of"* - so the question *"is there something to end?"* has to be asked
the same way. Two `enabled_when` conditions on one command would be a
control that is live for one tool and dead for another with nothing on
screen saying which.

The perimeter tool is the second open-ended gesture on this tab, which is
why asking the question once per tool does not work.

### `fn finish`

Routes to the armed tool's own completion path - never to a second one. Each
tool has exactly one function that builds its `DimensionKind`, and this is
the door the ribbon knocks on rather than a third place that could author a
slightly different shape.

Returns `false` when there is nothing to finish, so the dispatcher can say
which kind of nothing happened rather than tracing a success it did not
have.

### `fn take_completed_scale_line`

Returns `Some(points)` on the single frame after the two-point pick
completes, and `None` on every other frame — the length is cleared from the
state as it is read.

# Read-and-clear, because the alternative re-opens the dialog forever

`ScalePick::drawn_pdf_length` stays `Some` for as long as the pick holds a
completed line; that is what keeps the reference line drawn on the page
while the operator types. A caller that merely *observed* it would therefore
see it `Some` on every subsequent frame and re-open the Set-scale dialog
sixty times a second, discarding whatever had been typed into it each time.

Clearing here rather than asking the caller to remember is the same choice
`ScaleDialog::take_calibrate_request` makes: an edge that has to be reset by
discipline is one that eventually is not.

The whole pick is cleared, not just the length, so the tool is left ready
for another calibration rather than holding a line the dialog has already
consumed.

### `fn abandon`

Escape's claimant. It sits *below* the drag-in-flight rung and *above*
retiring the tool — see [`crate::canvas::tool::disarm_measure`], which
carries the argument for why those are two separate presses.

This block must stay attached to THIS function. A doc comment that drifts
up against a neighbouring item documents that item and leaves this one bare,
which is what `tools/gates/check-orphan-docs.py` exists to catch.

### `struct Pick`

A struct rather than seven parameters, and not merely to satisfy a lint: the
five that describe *where* the click landed — the document, the page, the
pointer, the decomposition and the mapping — are only meaningful together.
Any call site that had six of them and reached for the seventh from
somewhere else would be resolving a click against a page it did not come
from, which is the class of defect `canvas::mapping`'s header exists to make
unavailable.

### `fn click`

The whole of the tool's input. Called from `canvas::interact`'s `Click` arm
when [`crate::canvas::tool::CanvasTool::measure_kind`] says a measure tool
is armed — which is the same arm that would otherwise hit-test for a
selection, so a measure click and a selecting click are mutually exclusive
by construction rather than by a guard either could forget.

# The commit happens on the placing click, and there is no accept box

Every gesture ends in a **placing step**. Once the picks say what is
measured — a linear's two points, a closed perimeter, a finished circle
fit, a two-line pair — the authored dimension is held in
[`MeasureState::placing`] and follows the pointer: `draw` paints it with its
value text at the snapped hover point, through the same
`pdfcer_gui_base::measure::place::placed_at` the commit uses. The next
canvas click, whatever the tool, **is** the commit, and the value text lands
where it was clicked (O261). For an angular dimension only the arc radius
follows the click; the engine centres its text on the arc (request G064).

There is no accept box: the operator disliked *"a separate accept / reject
box somewhere on the screen"* (decision 024), and `MODES_AND_PANELS.md`
defaults application-initiated floating surfaces to **Never**. A misplaced
dimension is corrected by undo, as a misdrawn markup is. Escape, a tool
change or a page change drops a held dimension uncommitted.

`pending` is never set here. It stays on [`MeasureState`] because it is
salvaged state with its own tests.

# The circular tool is the exception, and it has two endings

Every sentence above is about a gesture whose *arity* ends it. The
radius/diameter tool has none — see [`MeasureKind::Circular`] — so the
operator ends it, in one of two ways:

| ending | where it is taken | why it exists |
|---|---|---|
| **double-click** on the canvas | this function, the `double` flag on [`Pick`] | what every drawing package's multi-pick tool uses; the standing *"make it work the way other programs do"* tie-breaker |
| **`measure.finish`** on the ribbon | [`finish`], through the dispatcher | discoverable without knowing the double-click, and reachable when the last pick sits somewhere awkward to double-click |

Both call [`circular::complete`], which moves the fit into the placing
step; neither commits. Neither is a floating accept box, so decision 024
stands.

**The first click of a double-click is still a pick**, and that is
deliberate: it toggles whatever it lands on, exactly as a single click
would, and then the second click finishes. The alternative — swallowing the
pair — would make the operator's last object need a separate click *and* a
double-click somewhere harmless. It is also the convention this canvas
already follows: [`crate::canvas::selection::SelectionState::click`] takes
the same flag and gives the *second* click its own meaning (descend a rung)
rather than repeating the first's.

A **triple** click ends the fit too. egui reports a release as a triple when
it falls within twice the double-click delay of the click before last, so an
operator who picks quickly and then double-clicks produces a triple;
`canvas::clicking` passes `double || triple` so it still ends the fit
instead of toggling the last point off.
