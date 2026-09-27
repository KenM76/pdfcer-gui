# `app::status` — the status bar: the narrator on the left, the constant controls on the right

`RIBBON_IA.md` §6 specifies this surface in one paragraph, and the
paragraph contains the whole design:

> **Status bar** — Find toggle, actual size, fit width, fit page, zoom
> −/%/+, page ◀ n/N ▶, and a **new editable page-number box**. These are
> the controls a user touches constantly; they belong where they never
> disappear behind a tab change. The current render-diagnostics text
> moves behind a disclosure (see `DEFECTS.md` §5).

Two halves, two arguments.

## The left half: the narrator, demoted

`DEFECTS.md`'s "Not defects" table records the old shell opening with a
substitute-glyph census:

> The first thing a user reads is the app talking about itself. Excellent
> information, wrong prominence — put it behind the disclosure triangle
> that is already there.

So the render diagnostics — which glyphs were substituted, which images
were skipped, which content streams the file does not actually contain —
are still here, still complete, and **closed by default**. The word
"Render notes" stays visible so the report is discoverable; only the
report itself is one click away.

That table's other entry that lands on this surface is the zoom anchor:
*"Zoom buttons pin the page's top-left, not the centre or the cursor."*
The − and + buttons below raise the same `ZoomIn`/`ZoomOut` actions the
ribbon and the keyboard raise, so they inherit that anchor exactly. It is
**not** fixed here, and it must not be fixed here: the anchor is
`crate::canvas`' scroll arithmetic (`zoom_anchor_offset`), and a status
bar that anchored zoom differently from the ribbon would be a second
zoom model. Recorded so the next reader knows the omission is a decision.

## Beside the narrator: the two disclosure lines, which are not narration

The left half carries four things, and only the first is the narrator.
The others look similar and are governed by different rules, so the
distinction is worth stating before the layout is:

| line | what it is | drawn by |
|---|---|---|
| Render notes | **narration** — a census of what the last raster contained. Demoted behind a disclosure triangle, closed by default. | [`notes`] |
| Fill disclosure | **rule 4** — what a form fill *inferred*: an auto-size pdfcer chose, characters it could not encode. | [`fill_disclosure`] |
| Edit disclosure | **rule 4** — what a move or a delete had to *change about an object's form* to express the request: an `re` rectangle rewritten as four lines, an implicit subpath start materialised. | [`edit_disclosure`] |
| Worded decline | **not rule 4 at all** — a command that was invoked and *did not run*, because there was nothing for it to act on. | [`decline::show`] |

Rows two and three are the same species of fact and are treated
identically. Each is:

- **not behind the triangle.** The narrator was demoted because its
  prominence was wrong (`DEFECTS.md` §5). A disclosure the operator has to
  *open something* to find is a disclosure that did not happen, which is
  the opposite failure.
- **keyed on [`OpenDoc::edit_epoch`]**, so any later edit — including an
  undo — retires the sentence with no code remembering to clear it. State
  that must be cleared is state that will one day be shown against the
  wrong document.
- **incapable of changing the bar's height** (R128 — see below), because
  both arrive *without the operator asking for anything*: a drag ends, and
  a sentence appears on the next frame. If that grew the bar, the page
  would re-fit at the exact moment a gesture completed.
- **drawn through one function**, [`disclosure_line`], so the four small
  rules that together make the previous point true — bounded width, fixed
  row, elide-don't-wrap, full text on hover — are written once.

The two can never be live at the same time: one edit bumps the epoch once
and records at most one kind of disclosure, so the mutual exclusion is a
property of the epoch rather than a rule anyone has to enforce. See
[`crate::app::actions::last_edit_disclosure`].

**The edit disclosure closes `FEATURES.md`'s "edit-disclosure surface"
row.** `crate::app::actions::vector_edit` had traced these sentences since
stage S4 and its own header named the gap — *"tracing is not surfacing …
the status line is `app::status`'s to own, not this module's to invent"*.
This module now owns it. The trace is unchanged.

### Row four is the same surface and a DIFFERENT store

The worded decline reuses this half of the bar, [`disclosure_line`], its
own named region and the R128 fixed row — the *place* and the *discipline*.
It deliberately does **not** reuse the edit-epoch key, and the wording
diverges too (*"Nothing to zoom to"*, never *"About your last edit"*),
because a disclosure says *this happened, and here is the part you cannot
see* while a decline says *this did not happen*. One slot and one wording
for both would make a completed gesture and a refused one indistinguishable
in the same place, which is worse than the trace-only state it replaces.

The short form of why the epoch is wrong here: **a decline changes no
document, so the epoch never moves** — an epoch-keyed decline would never
retire — and **a decline must be repeatable**, which an epoch key cannot
express because nothing changed between the two presses. The precedent it
is modelled on is [`page_box`]'s clamp note, retired by the operator's next
act. [`decline`]'s own header carries the full argument, the retirement
rule, and the one case it refuses to word (a ceiling-clamped region zoom,
which is a partial grant the zoom readout already reports honestly).

## The right half: the controls that must never move

Everything on the right exists because it is reached constantly and
because a tab change must not take it away. They are **mirrors**, and
amendment P1a is what makes mirroring legal:

> *the QAT and the status bar are shortcut surfaces, not tabs. A command
> may appear on exactly one tab and additionally on the QAT and/or the
> status bar.*

So `Actual size · Fit width · Fit page` appear here *and* on View ▸ Zoom,
and `RibbonTab::groups()` remains the single source of truth for tab
ownership because this surface is outside its domain.

## The editable page box is the point of the exercise

`GUI_ROADMAP.md` 3.3 states the problem in one line: *"Reaching page 37
of 42 currently means the thumbnail rail or 36 keystrokes."* Type `37`,
press Enter, arrive. Four properties make that usable rather than
merely present, and each is implemented deliberately:

1. **Commit on Enter or focus loss, never per keystroke.** Someone typing
   `42` passes through `4`, and a box that navigated per keystroke would
   take them to page 4, re-render a CAD sheet, and then take them to page
   42 — with the intermediate render wasted and the operator's eye
   already moved. See [`page_box`].
2. **Out of range clamps, and says so.** `99` in a 42-page document goes
   to page 42 *and reports that it did*
   ([`crate::text::status::page_clamped_note`]). A silent clamp is
   indistinguishable from a box that ignored what was typed, and an
   operator who cannot tell those apart stops trusting the control.
3. **Non-numeric input is refused without discarding it.** The text stays
   in the box with a note beside it. Wiping an operator's typing to
   "helpfully" restore the current page destroys the evidence of what
   they meant.
4. **It suppresses the unmodified keyboard bindings while focused, and
   that is defect D1 from the other end.** `crate::app::keyboard` guards
   those bindings with `ctx.text_edit_focused()` — *not*
   `egui_wants_keyboard_input()`, which means "any widget has focus" and
   cost the operator the Delete key and all keyboard page navigation from
   the first canvas click onward. `text_edit_focused()` resolves the
   focused id and asks whether a `TextEditState` exists **for that id**,
   so this control has to be a real [`egui::TextEdit`] with a stable id
   for the guard to see it. A `DragValue`, a custom painted field, or a
   label-plus-popup would all typecheck and all silently re-open D1 in
   the reverse direction: `PageDown` would step the page while the
   operator was halfway through typing a page number.
   `page_box::tests::typing_a_digit_into_the_page_box_does_not_also_step_the_page`
   is the regression test, and it asserts the failing condition is really
   present before asserting the fix — the shape the D1 post-mortem says
   the original test was missing.

## The bar has a FIXED height, and this is measured rather than tidy

`D:/dev/rag/egui/bottom_panel_height_change_retriggers_fit_to_viewport_zoom.md`,
pdfcer standing rule **R128**: *a panel whose size feeds a
fit-to-viewport computation has a fixed size.*

The loop is real and it was measured. A content-driven status panel takes
space from the central panel; `FitMode::Page`/`FitMode::Width` recompute
their zoom from the canvas viewport **every frame they are active**
([`crate::viewer::ViewState::apply_fit`]); so one extra status line on
frame N produces a smaller fit scale on frame N+1. On pdfcer that showed
up as a page that visibly shrank across three frames (230 % → 224 % →
215 %) and, worse, as click coordinates that went stale between the frame
they were captured on and the next render. The canonical symptom is *"the
page jumped when I clicked an object"* — which reads as a selection bug
and gets investigated in the selection code, where nothing is wrong.

Two defences, and this module carries both ends:

- **The caller pins the panel.** [`HEIGHT_PTS`] exists to be passed to
  `egui::Panel::bottom(..).exact_size(..)`. Only `exact_size` closes the
  loop: `default_height` is a starting value that content still overrides,
  `min_height`/`max_height` bound a *range* the panel still varies inside,
  and `resizable(false)` only stops the operator dragging the edge.
- **The content cannot grow anyway.** [`show`] lays everything out inside
  one allocated row of [`ROW_HEIGHT_PTS`], and — the part that actually
  takes discipline — **opening the disclosure does not add a line.** The
  render notes are drawn *on the same row*, to the right of the triangle,
  elided if they are long, with the full text on hover. That is why
  §6 says "one line" and why there is no [`egui::CollapsingHeader`]
  anywhere in this file: a collapsing header's entire behaviour is to
  change its own height, which is the one thing this surface may not do.
  [`tests::the_bar_is_exactly_as_tall_open_as_closed`] pins it.

## Two rules the mirrors on this bar are held to

**A mirror behaves exactly as the control it mirrors.** The status bar is
a *shortcut surface* for a tab command (P1a), so `Actual size` dispatches
the tab command's own action and works nothing around locally: a mirror
that behaved differently would be a second zoom model, which is worse than
a shared defect. The action is `Action::ZoomTo`, not
`Action::Fit(FitMode::None)` — which only stops the per-frame re-fit and
leaves `zoom` wherever it was, so a control promising *"one PDF point per
screen point"* would pin 73 % at 73 % — and deliberately not
`ZoomBy(1.0 / zoom)`, which lands on the right number but routes a
discrete command through the wheel path's 150 ms settle.

**A chord has exactly one owner.** `crate::app::keyboard` does not know
what a manifest chord means: it spells the key, looks it up in the keymap,
and returns a command id that goes through the same dispatcher a ribbon
click reaches. That is what lets a tooltip here name its chord at all —
two claimants on `Ctrl+0` means the Actual-size tooltip can honestly
advertise none. `no_chord_has_two_owners` fails naming the chord and both
claimants if a conflict appears.

## The Find toggle, which §6 lists first

`edit.find` is registered, `Ctrl+F` is bound to it in the manifest keymap
and parsed by `crate::app::keyboard::parse_chord`, the dispatch arm toggles
the bar, and `crate::find::bar` is the surface. The control appears
**here** rather than on the ribbon because §6 puts it here, in the section
headed *what deliberately does not go on the ribbon*.

Two details worth stating because both are decisions:

- **It is a `selectable_label`, not a button**, and it shows whether the
  bar is open. The bar is a persistent surface an operator leaves up while
  working through hits, and a control that made no claim about state would
  leave them with no way to tell "closed" from "open behind something".
  That is the same argument the render-notes disclosure at the other end of
  this bar makes, and it is drawn the same way.
- **It writes `FindState` directly rather than raising an action.**
  Opening a bar touches no document, so there is nothing for the funnel to
  order or to log — the same reason `PdfcerApp::show_panel` mounts a panel
  during dispatch. What *does* go through the funnel is the search itself.
  The paragraph below on actions-not-mutations is unchanged and still
  binds every other control here.

## What is NOT drawn, and why that is not an oversight

**The page controls, on a document with no pages.** `/Count 0` is legal
PDF. A page box over a document with no pages is a control whose every
input is out of range, so the group is dropped and the zoom controls stay.

**The whole left half, before the first raster.** The render notes are a
property of a *drawn page*; there is nothing to disclose until a page has
been drawn, and `page_texture` is `None` only before the first render and
after a render failure (which the canvas already reports in words).

## Actions, not mutations

Every control here pushes an [`Action`] and mutates nothing.
`crate::app::actions`' header states the invariant — *"No code path runs
from a widget to a document"* — and this module honours it including for
the page box, whose commit raises `Action::GoToPage` rather than touching
`view.page_index`. The only state this module writes is its own widget
state (the draft text, the note, the disclosure flag), which lives in
`egui`'s per-id store and describes the *control*, not the document.

### Why that state lives in `egui::Memory` when `crate::app::state`
argues against it

`OpenDoc`'s docs record moving the canvas selection *off* `egui::Memory`,
because a selection is document-scoped and `Memory` outlives documents —
which forced a synthetic document identity, and an address is not an
identity. None of that applies here, for a reason rather than by luck:

- The draft is a **text-editing buffer**. `egui` already keeps one for
  this very widget (`TextEditState`, keyed by the same id), so the draft
  sits beside its own cursor and selection rather than in a second place
  with a different lifetime.
- It is **discarded on focus loss**, always: the box shows the current
  page whenever it is not being edited and no note is outstanding. A
  value that cannot survive a click elsewhere cannot survive a document
  open either, so there is nothing to key on and no staleness to detect.
- Neither this module nor the parent may add a field: `app/mod.rs`,
  `opendoc.rs` and `app/actions.rs` are owned elsewhere, and inventing
  a parallel owner for four bytes of widget state would be a worse
  structural change than using the store egui provides for exactly this.

## Where three subjects live, and the one question left here

Three subjects have modules of their own, and each was taken out along a
seam rather than sliced off the bottom of the file: each has **its own
state, its own vocabulary and its own pure decision function**.

| module | answers | its own |
|---|---|---|
| [`page_box`] | *what did the operator mean by what they typed?* | draft + note state, `PageCommit`, `resolve`, defect D1's keyboard guard |
| [`decline`] | *what did a refused command owe the operator, and for how long?* | decline store, `Declined`, `still_true`, the speech-act argument |
| [`notes`] | *what did the renderer compromise on, and how is that one line?* | open/closed flag, `NoteEntry`, `notes_line`, the editorial rule about which counters are actionable |

What is left here is the one question none of them answers: **how is the
bar laid out, and what does each group show?** A fixed row (R128), the
order the groups are added in and why that is the reverse of the reading
order, the two rule-4 disclosure lines and the single [`disclosure_line`]
they share, and the two clusters of stateless mirrors on the right.

Each module's own header carries its argument in full — the commit rule
and D1 for the page box, the retirement rule and what is deliberately not
worded for the decline, the prominence argument for the notes.

## Item notes

### `mod ocrlayer`

The canvas may not be marked to say *this page carries no recognised
text* — R8b — so the one state where the mode is on and draws nothing is
stated here or nowhere. See its header.

### `mod readmode`

`OPERATOR_REQUESTS.md` O115, the operator: *"I didn't see a way to get back
out of read mode."* Read mode hides the ribbon, and the only
control that turns it off is on the ribbon — so the mode hides its own exit.

It is on **this** bar because this bar is the one piece of chrome §2 of
`app::window` deliberately keeps, and because read mode composes with full
screen: in the combined state there is no ribbon *and* no title bar, so the
window title — which carries the same statement — is absent in exactly the
state with the least chrome left. Its header carries the full argument for
why two surfaces is not duplication here.

### `const _`

Checked at **compile time** rather than in a test: the relationship
between the two constants is a property of the constants, and a test
would only re-discover at run time what the compiler can refuse outright.

### `const ZOOM_READOUT_WIDTH_PTS`

`8%` and `800%` are different widths, and without a reserve the − button
would step sideways every time the operator clicked +. Four characters is
**not** the width to reserve: O24 made the ceiling a preference,
`MAX_MAX_ZOOM_PERCENT` is `1e12` and
[`crate::text::status::zoom_percent`] formats with `{:.0}`, so the readout
can be asked to draw `1000000000000%` — fourteen characters.

Kept as a FLOOR rather than deleted, because it is still doing the
original job at the bottom of the range: `10%` measures narrower than four
characters, and letting the reserve shrink to it would move the − button
the other way. The reserve is now `max(this, the measured width of the
widest string the CURRENT ceiling can produce)` — see
[`status::zoom::readout_width`].

### `const REGION_MAXZOOM_ROW`

Indexed rather than named, for the reason the filter's rows are: a label
is operator copy and gets reworded, an index is stable, and a harness
chooses positionally.

### `const REGION_FILTER`

The popup publishes its own rows separately (see [`REGION_FILTER_ROW`]),
because a harness that can open a list but not choose from it can only
assert *the control exists*, which is the one claim that is also true of
every inert control.

### `const REGION_FILTER_EMPTY`

A named region rather than a bare label, because the whole requirement for
this sentence is that it is **on screen and legible** at the moment the
canvas has stopped responding — and that can only be asserted about a rect
the application published.

### `const REGION_FILTER_ROW`

**Indexed, not named.** Labels are operator copy and get reworded; an
index is stable and a harness is choosing positionally anyway. The index is
the position in [`PickClass::ALL`], which is also the display order.

These regions exist only on the frames the popup is open, which is what an
`Area` laid out at paint time does — see
`D:/dev/rag/egui/a_combobox_popup_is_an_area_laid_out_at_paint_time_so_only_the_app_can_publish_its_entry_rects.md`.

### `fn fitting_id`

A `selectable_label` showing whether the bar is open — see the module
docs for why it shows state at all, and why it writes [`FindState`]
instead of raising an [`Action`].

Drawn only with a document open (the caller has already returned
otherwise), because `crate::find::bar` draws nothing without one: a toggle
that produced no visible bar would be the placeholder P3 forbids, and the
registered command is gated on `doc.pages` for the same reason.
The `egui::Memory` slot the bar's measured group widths live in.

One id for the whole map rather than one per group: they are written
together at the end of a frame and read together at the start of the next,
and five separate slots would admit the state where three are from this
frame and two from the last.

### `fn fitting_widths`

Empty on the first frame of a session, which [`fitting::affordable`] treats
as *show everything* — see its docs on why that bootstrap is required rather
than merely tolerant.

### `mod decline`

Split out under R2 like [`page_box`], and along a seam of the same kind:
what is left here answers *"how is the bar laid out, and what does each
group show?"*, while that module answers *"what did a refused command owe
the operator, and how long does it owe it for?"* — its own store, its own
vocabulary, its own pure retirement predicate, and an argument about speech
acts that nothing else on this surface shares.

`pub(super)` rather than private, unlike [`page_box`]: `crate::app::dispatch`
is the choke point that records a decline and retires it, so the store has
to be reachable from a sibling of this module. Nothing outside `crate::app`
can see it, which is the right boundary — a decline is written by the one
dispatcher and read by the one bar.

### `mod notes`

It is `pub(crate)` rather than private for exactly one export:
[`notes::findings`], the ordered, filtered list of what a raster
compromised on. The Render-diagnostics dialog
(`crate::dialogs::diagnostics`) shows the same facts with room for more than
one line, and the *editorial* rules behind that list — which counters are
actionable, which two are excluded, and in whose interest the order is —
belong to the narrator and must be stated once. The dialog joins nothing and
filters nothing; it lists what this module already decided.

### `mod filter`

The one control on this bar that is not a readout: everything else here
reports what is true about the view, and this changes what the pointer
does. Its header carries why that earns it both its own file and its own
position at the left edge of the fixed cluster.

### `mod maxzoom`

Its header carries why the readout rather than a new control: the bar's
height and right-hand cluster are fixed, and a label that turns out to be
a button is already this surface's idiom.

### `mod rasterstop`

The only line on this bar that needs **no store and no retirement rule**: it
is a pure function of the current frame's state, so it appears at the ceiling
and is gone the moment he zooms out, turns the page, or edits it. Its header
carries why that follows from what the sentence means rather than being a
shortcut, and why [`decline`]'s partial-grant ruling does not cover it.

### `const HEIGHT_PTS`

**Pass this to `egui::Panel::bottom(..).exact_size(..)`, not to
`default_height`.** The difference is rule R128: `exact_size` pins the
panel's outer size so its content cannot perturb the central region at
all, and every other sizing API leaves the fit-to-viewport feedback loop
open. The module docs carry the measured case.

[`ROW_HEIGHT_PTS`] plus egui's own `Frame::side_top_panel` inner margin
(2 pt above and below) plus a little room for the panel's separator
stroke. Generous rather than tight: a bar whose content is clipped by one
point is a legibility defect, and the cost of the slack is four pixels of
canvas that never change size.

### `const ROW_HEIGHT_PTS`

[`show`] allocates exactly this, so the bar's content height is a
constant rather than a function of what there is to say — which is the
second half of the R128 defence and the reason the disclosure draws its
line *beside* the triangle rather than beneath it.

### `const NOTES_WIDTH_FRACTION`

The notes are the *least* urgent thing on this surface (`DEFECTS.md`:
"excellent information, wrong prominence"), so on a narrow window they
yield to the navigation controls rather than squeezing them. The full
text is always available on hover, so nothing is lost — only deferred.

### `const REGION_FILL_DISCLOSURE`

Named as a region so `ui-verify` can assert it is **on screen and
legible** rather than merely constructed — which for a disclosure is the
whole of the requirement.

### `const REGION_EDIT_DISCLOSURE`

Named as a region for the same reason as its fill sibling: a disclosure's
whole requirement is that it is **on screen and legible**, and `ui-verify`
can only assert that about a rect the application published.

### `const REGION_LOAD_ANOMALIES`

The **third** region here that is about the FILE rather than about a gesture,
and the one a driven check most needs by name: the engine's notice makes the
whole tolerant-loading Pass conditional on the shell disclosing what was
decided, so a census built into a zero-width rect would be the loader
shipping without the thing that makes it honest. Only a published rect can
tell "on screen and legible" from "constructed".

Distinct from [`REGION_RECOVERED`] on purpose — the two conditions are
disjoint and can be live together; see [`anomalies`]' header for the table.

### `const REGION_CATCHING_UP`

The one region in this group naming a **state** rather than an event, so a
check reading it is asking *"is the page behind right now"* and not *"did an
edit disclose something"*.

### `const REGION_LINE_WEIGHTS`

The **second** state region, and the one a driven check must be able to find
by name, because the whole safety argument for the feature rests on the
disclosure being **on screen and legible** rather than merely constructed.
A `line_weights` toggle whose disclosure was built into a zero-width rect
would be the feature shipping without the thing that makes it safe, and
nothing but a published rect can tell those apart.

### `mod test_support`

A module of its own rather than helpers inside `mod tests`, because two
sibling test modules share them and `pub(super)` on a helper buried in one
of them would read as "the other module reaches into my tests" rather than
as "this is the shared harness". Visible to `crate::app::status` and its
descendants, and to nothing else.

### `fn opened`

Opened through `crate::app::state::open_fixture`, which is the same
three calls `PdfcerApp::open_path` makes in the same order — so what
these tests drive is the real state machine rather than a hand-built
approximation of it.

### `fn bar_frame`

`None` when no measurement happened at all — the closure never ran, or
it produced a non-finite height. **A measurement that did not happen
must not read as a measurement**, and that is not theoretical here:
`cargo test -p egui-shell` and `cargo test --workspace` compile `egui`
with different features (no fonts vs `default_fonts`), so a layout
assertion can be entirely vacuous under one of the two commands a
developer runs. A helper that returned a bare `f32` would hand a
vacuous run the same `NAN == NAN`-adjacent silence a real one gets.

The shape count is the second half of the same discipline, and it is
the half that matters for a *sentence*: a height comparison between two
frames that both drew nothing is true and worthless. Counting the
painted shapes is how a test proves the line reached the painter rather
than merely reaching the data.

Lives here rather than in `mod tests` because **three** R128 tests need
it — the fill line, the edit line and [`super::decline`]'s — and the
third is in a sibling module. `pub(super)` on a helper buried inside one
test module would read as "the other module reaches into my tests"
rather than as "this is the shared harness".

### `fn settled_bar_frame`

egui settles over a pass: fonts are laid out lazily, widget galleys are
cached on first sight, and animations start at their "from" value. A
single frame therefore compares one state's *first* look against
another state's *first* look, which is a comparison of two different
things. Every caller wants the steady state.
