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
  `app/state.rs` and `app/actions.rs` are owned elsewhere, and inventing
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
