//! # `app::status` — the status bar: the narrator on the left, the constant controls on the right
//!
//! `RIBBON_IA.md` §6 specifies this surface in one paragraph, and the
//! paragraph contains the whole design:
//!
//! > **Status bar** — Find toggle, actual size, fit width, fit page, zoom
//! > −/%/+, page ◀ n/N ▶, and a **new editable page-number box**. These are
//! > the controls a user touches constantly; they belong where they never
//! > disappear behind a tab change. The current render-diagnostics text
//! > moves behind a disclosure (see `DEFECTS.md` §5).
//!
//! Two halves, two arguments.
//!
//! ## The left half: the narrator, demoted
//!
//! `DEFECTS.md`'s "Not defects" table records the old shell opening with a
//! substitute-glyph census:
//!
//! > The first thing a user reads is the app talking about itself. Excellent
//! > information, wrong prominence — put it behind the disclosure triangle
//! > that is already there.
//!
//! So the render diagnostics — which glyphs were substituted, which images
//! were skipped, which content streams the file does not actually contain —
//! are still here, still complete, and **closed by default**. The word
//! "Render notes" stays visible so the report is discoverable; only the
//! report itself is one click away.
//!
//! That table's other entry that lands on this surface is the zoom anchor:
//! *"Zoom buttons pin the page's top-left, not the centre or the cursor."*
//! The − and + buttons below raise the same `ZoomIn`/`ZoomOut` actions the
//! ribbon and the keyboard raise, so they inherit that anchor exactly. It is
//! **not** fixed here, and it must not be fixed here: the anchor is
//! `crate::canvas`' scroll arithmetic (`zoom_anchor_offset`), and a status
//! bar that anchored zoom differently from the ribbon would be a second
//! zoom model. Recorded so the next reader knows the omission is a decision.
//!
//! ## ★ Beside the narrator: the two disclosure lines, which are not narration
//!
//! The left half carries four things, and only the first is the narrator.
//! The others look similar and are governed by different rules, so the
//! distinction is worth stating before the layout is:
//!
//! | line | what it is | drawn by |
//! |---|---|---|
//! | Render notes | **narration** — a census of what the last raster contained. Demoted behind a disclosure triangle, closed by default. | [`notes`] |
//! | Fill disclosure | **rule 4** — what a form fill *inferred*: an auto-size pdfcer chose, characters it could not encode. | [`fill_disclosure`] |
//! | Edit disclosure | **rule 4** — what a move or a delete had to *change about an object's form* to express the request: an `re` rectangle rewritten as four lines, an implicit subpath start materialised. | [`edit_disclosure`] |
//! | Worded decline | **not rule 4 at all** — a command that was invoked and *did not run*, because there was nothing for it to act on. | [`decline::show`] |
//!
//! Rows two and three are the same species of fact and are treated
//! identically. Each is:
//!
//! - **not behind the triangle.** The narrator was demoted because its
//!   prominence was wrong (`DEFECTS.md` §5). A disclosure the operator has to
//!   *open something* to find is a disclosure that did not happen, which is
//!   the opposite failure.
//! - **keyed on [`OpenDoc::edit_epoch`]**, so any later edit — including an
//!   undo — retires the sentence with no code remembering to clear it. State
//!   that must be cleared is state that will one day be shown against the
//!   wrong document.
//! - **incapable of changing the bar's height** (R128 — see below), because
//!   both arrive *without the operator asking for anything*: a drag ends, and
//!   a sentence appears on the next frame. If that grew the bar, the page
//!   would re-fit at the exact moment a gesture completed.
//! - **drawn through one function**, [`disclosure_line`], so the four small
//!   rules that together make the previous point true — bounded width, fixed
//!   row, elide-don't-wrap, full text on hover — are written once.
//!
//! The two can never be live at the same time: one edit bumps the epoch once
//! and records at most one kind of disclosure, so the mutual exclusion is a
//! property of the epoch rather than a rule anyone has to enforce. See
//! [`crate::app::actions::last_edit_disclosure`].
//!
//! **The edit disclosure closes `FEATURES.md`'s "edit-disclosure surface"
//! row.** `crate::app::actions::vector_edit` had traced these sentences since
//! stage S4 and its own header named the gap — *"tracing is not surfacing …
//! the status line is `app::status`'s to own, not this module's to invent"*.
//! This module now owns it. The trace is unchanged.
//!
//! ### ★ Row four is the same surface and a DIFFERENT store
//!
//! The worded decline reuses this half of the bar, [`disclosure_line`], its
//! own named region and the R128 fixed row — the *place* and the *discipline*.
//! It deliberately does **not** reuse the edit-epoch key, and the wording
//! diverges too (*"Nothing to zoom to"*, never *"About your last edit"*),
//! because a disclosure says *this happened, and here is the part you cannot
//! see* while a decline says *this did not happen*. One slot and one wording
//! for both would make a completed gesture and a refused one indistinguishable
//! in the same place, which is worse than the trace-only state it replaces.
//!
//! The short form of why the epoch is wrong here: **a decline changes no
//! document, so the epoch never moves** — an epoch-keyed decline would never
//! retire — and **a decline must be repeatable**, which an epoch key cannot
//! express because nothing changed between the two presses. The precedent it
//! is modelled on is [`page_box`]'s clamp note, retired by the operator's next
//! act. [`decline`]'s own header carries the full argument, the retirement
//! rule, and the one case it refuses to word (a ceiling-clamped region zoom,
//! which is a partial grant the zoom readout already reports honestly).
//!
//! ## The right half: the controls that must never move
//!
//! Everything on the right exists because it is reached constantly and
//! because a tab change must not take it away. They are **mirrors**, and
//! amendment P1a is what makes mirroring legal:
//!
//! > *the QAT and the status bar are shortcut surfaces, not tabs. A command
//! > may appear on exactly one tab and additionally on the QAT and/or the
//! > status bar.*
//!
//! So `Actual size · Fit width · Fit page` appear here *and* on View ▸ Zoom,
//! and `RibbonTab::groups()` remains the single source of truth for tab
//! ownership because this surface is outside its domain.
//!
//! ## ★ The editable page box is the point of the exercise
//!
//! `GUI_ROADMAP.md` 3.3 states the problem in one line: *"Reaching page 37
//! of 42 currently means the thumbnail rail or 36 keystrokes."* Type `37`,
//! press Enter, arrive. Four properties make that usable rather than
//! merely present, and each is implemented deliberately:
//!
//! 1. **Commit on Enter or focus loss, never per keystroke.** Someone typing
//!    `42` passes through `4`, and a box that navigated per keystroke would
//!    take them to page 4, re-render a CAD sheet, and then take them to page
//!    42 — with the intermediate render wasted and the operator's eye
//!    already moved. See [`page_box`].
//! 2. **Out of range clamps, and says so.** `99` in a 42-page document goes
//!    to page 42 *and reports that it did*
//!    ([`crate::text::status::page_clamped_note`]). A silent clamp is
//!    indistinguishable from a box that ignored what was typed, and an
//!    operator who cannot tell those apart stops trusting the control.
//! 3. **Non-numeric input is refused without discarding it.** The text stays
//!    in the box with a note beside it. Wiping an operator's typing to
//!    "helpfully" restore the current page destroys the evidence of what
//!    they meant.
//! 4. **★ It suppresses the unmodified keyboard bindings while focused, and
//!    that is defect D1 from the other end.** `crate::app::keyboard` guards
//!    those bindings with `ctx.text_edit_focused()` — *not*
//!    `egui_wants_keyboard_input()`, which means "any widget has focus" and
//!    cost the operator the Delete key and all keyboard page navigation from
//!    the first canvas click onward. `text_edit_focused()` resolves the
//!    focused id and asks whether a `TextEditState` exists **for that id**,
//!    so this control has to be a real [`egui::TextEdit`] with a stable id
//!    for the guard to see it. A `DragValue`, a custom painted field, or a
//!    label-plus-popup would all typecheck and all silently re-open D1 in
//!    the reverse direction: `PageDown` would step the page while the
//!    operator was halfway through typing a page number.
//!    `page_box::tests::typing_a_digit_into_the_page_box_does_not_also_step_the_page`
//!    is the regression test, and it asserts the failing condition is really
//!    present before asserting the fix — the shape the D1 post-mortem says
//!    the original test was missing.
//!
//! ## ★ The bar has a FIXED height, and this is measured rather than tidy
//!
//! `D:/dev/rag/egui/bottom_panel_height_change_retriggers_fit_to_viewport_zoom.md`,
//! pdfcer standing rule **R128**: *a panel whose size feeds a
//! fit-to-viewport computation has a fixed size.*
//!
//! The loop is real and it was measured. A content-driven status panel takes
//! space from the central panel; `FitMode::Page`/`FitMode::Width` recompute
//! their zoom from the canvas viewport **every frame they are active**
//! ([`crate::viewer::ViewState::apply_fit`]); so one extra status line on
//! frame N produces a smaller fit scale on frame N+1. On pdfcer that showed
//! up as a page that visibly shrank across three frames (230 % → 224 % →
//! 215 %) and, worse, as click coordinates that went stale between the frame
//! they were captured on and the next render. The canonical symptom is *"the
//! page jumped when I clicked an object"* — which reads as a selection bug
//! and gets investigated in the selection code, where nothing is wrong.
//!
//! Two defences, and this module carries both ends:
//!
//! - **The caller pins the panel.** [`HEIGHT_PTS`] exists to be passed to
//!   `egui::Panel::bottom(..).exact_size(..)`. Only `exact_size` closes the
//!   loop: `default_height` is a starting value that content still overrides,
//!   `min_height`/`max_height` bound a *range* the panel still varies inside,
//!   and `resizable(false)` only stops the operator dragging the edge.
//! - **The content cannot grow anyway.** [`show`] lays everything out inside
//!   one allocated row of [`ROW_HEIGHT_PTS`], and — the part that actually
//!   takes discipline — **opening the disclosure does not add a line.** The
//!   render notes are drawn *on the same row*, to the right of the triangle,
//!   elided if they are long, with the full text on hover. That is why
//!   §6 says "one line" and why there is no [`egui::CollapsingHeader`]
//!   anywhere in this file: a collapsing header's entire behaviour is to
//!   change its own height, which is the one thing this surface may not do.
//!   [`tests::the_bar_is_exactly_as_tall_open_as_closed`] pins it.
//!
//! ## ★ Two defects this surface surfaced, both since fixed
//!
//! Recorded because the *reasoning* is the durable part, not the bug.
//!
//! **`Actual size` did not produce actual size.** It raised
//! `Action::Fit(FitMode::None)`, which only stops the per-frame re-fit and
//! leaves `zoom` wherever it was — so a control whose tooltip promises
//! *"one PDF point per screen point"* pinned 73 % at 73 %. It was mirrored
//! here **as-is on purpose** rather than worked around: the status bar is a
//! *shortcut surface* for a tab command (P1a), and a mirror that behaved
//! differently from the control it mirrors would be a second zoom model,
//! which is worse than a shared defect. Fixed 2026-08-13 by
//! `Action::ZoomTo`, dispatched from `view.zoom_actual` and from here —
//! one change, both surfaces. Deliberately not `ZoomBy(1.0 / zoom)`, which
//! lands on the right number but routes a discrete command through the
//! wheel path's 150 ms settle.
//!
//! **`Ctrl+0` had two owners.** The manifest keymap bound it to
//! `view.zoom_actual` while `crate::app::keyboard` bound it to
//! `Fit(FitMode::Page)` and got there first, so this bar's Actual-size
//! tooltip had to advertise no chord at all. Fixed 2026-08-13, and fixed
//! *structurally*: `keyboard` no longer knows what a manifest chord means —
//! it spells the key, looks it up in the keymap, and returns a command id
//! that goes through the same dispatcher a ribbon click reaches. The
//! tooltip names its chord again, and `no_chord_has_two_owners` fails
//! naming the chord and both claimants if the conflict returns.
//!
//! ## ★ The Find toggle, which §6 lists first and which used to be absent
//!
//! It is drawn now. This section used to read:
//!
//! > **The Find toggle.** §6 lists it first, and it is absent. `Find` has no
//! > entry in `crate::shell::commands` … There is no find panel, no search
//! > over the page's text, and no action to raise. Under `PROJECT_PLAN.md`
//! > §3's no-placeholders invariant an unavailable capability renders
//! > **nothing** … It lands with the find panel.
//!
//! It has landed. `edit.find` is registered, `Ctrl+F` is bound to it in the
//! manifest keymap and parsed by `crate::app::keyboard::parse_chord`, the
//! dispatch arm toggles the bar, and `crate::find::bar` is the surface. So
//! the control appears — and it appears **here** rather than on the ribbon
//! because §6 puts it here, in the section headed *what deliberately does not
//! go on the ribbon*.
//!
//! Two details worth stating because both are decisions:
//!
//! - **It is a `selectable_label`, not a button**, and it shows whether the
//!   bar is open. The bar is a persistent surface an operator leaves up while
//!   working through hits, and a control that made no claim about state would
//!   leave them with no way to tell "closed" from "open behind something".
//!   That is the same argument the render-notes disclosure at the other end of
//!   this bar makes, and it is drawn the same way.
//! - **It writes `FindState` directly rather than raising an action.**
//!   Opening a bar touches no document, so there is nothing for the funnel to
//!   order or to log — the same reason `PdfcerApp::show_panel` mounts a panel
//!   during dispatch. What *does* go through the funnel is the search itself.
//!   The paragraph below on actions-not-mutations is unchanged and still
//!   binds every other control here.
//!
//! ## What is NOT drawn, and why that is not an oversight
//!
//! **The page controls, on a document with no pages.** `/Count 0` is legal
//! PDF. A page box over a document with no pages is a control whose every
//! input is out of range, so the group is dropped and the zoom controls stay.
//!
//! **The whole left half, before the first raster.** The render notes are a
//! property of a *drawn page*; there is nothing to disclose until a page has
//! been drawn, and `page_texture` is `None` only before the first render and
//! after a render failure (which the canvas already reports in words).
//!
//! ## Actions, not mutations
//!
//! Every control here pushes an [`Action`] and mutates nothing.
//! `crate::app::actions`' header states the invariant — *"No code path runs
//! from a widget to a document"* — and this module honours it including for
//! the page box, whose commit raises `Action::GoToPage` rather than touching
//! `view.page_index`. The only state this module writes is its own widget
//! state (the draft text, the note, the disclosure flag), which lives in
//! `egui`'s per-id store and describes the *control*, not the document.
//!
//! ### Why that state lives in `egui::Memory` when `crate::app::state`
//! argues against it
//!
//! `OpenDoc`'s docs record moving the canvas selection *off* `egui::Memory`,
//! because a selection is document-scoped and `Memory` outlives documents —
//! which forced a synthetic document identity, and an address is not an
//! identity. None of that applies here, for a reason rather than by luck:
//!
//! - The draft is a **text-editing buffer**. `egui` already keeps one for
//!   this very widget (`TextEditState`, keyed by the same id), so the draft
//!   sits beside its own cursor and selection rather than in a second place
//!   with a different lifetime.
//! - It is **discarded on focus loss**, always: the box shows the current
//!   page whenever it is not being edited and no note is outstanding. A
//!   value that cannot survive a click elsewhere cannot survive a document
//!   open either, so there is nothing to key on and no staleness to detect.
//! - Neither this module nor the parent may add a field: `app/mod.rs`,
//!   `app/state.rs` and `app/actions.rs` are owned elsewhere, and inventing
//!   a parallel owner for four bytes of widget state would be a worse
//!   structural change than using the store egui provides for exactly this.
//!
//! ## Where three subjects went, and the one question left behind
//!
//! This file has reached standing rule R2's 1,500-line ceiling three times,
//! and each split took out a subject with **its own state, its own vocabulary
//! and its own pure decision function** rather than a slice of whatever
//! happened to be at the bottom of the file:
//!
//! | module | answers | its own |
//! |---|---|---|
//! | [`page_box`] | *what did the operator mean by what they typed?* | draft + note state, `PageCommit`, `resolve`, defect D1's keyboard guard |
//! | [`decline`] | *what did a refused command owe the operator, and for how long?* | decline store, `Declined`, `still_true`, the speech-act argument |
//! | [`notes`] | *what did the renderer compromise on, and how is that one line?* | open/closed flag, `NoteEntry`, `notes_line`, the editorial rule about which counters are actionable |
//!
//! What is left here is the one question none of them answers: **how is the
//! bar laid out, and what does each group show?** A fixed row (R128), the
//! order the groups are added in and why that is the reverse of the reading
//! order, the two rule-4 disclosure lines and the single [`disclosure_line`]
//! they share, and the two clusters of stateless mirrors on the right.
//!
//! Each module's own header carries its argument in full — the commit rule
//! and D1 for the page box, the retirement rule and what is deliberately not
//! worded for the decline, the prominence argument for the notes.

/// Page navigation and the editable page-number box. See this module's
/// header for the seam, and that one's for the control.
// The four named zoom levels -- Actual size, Fit width, Fit height, Fit page.
// Split out under R2 on 2026-08-24; see its header for the layout rule.
mod disclosure;
mod fit;
/// **What the bar can afford when the window is narrow** — the shed rule, and
/// the reachability clause that makes shedding legitimate. Added 2026-08-26.
pub(super) mod fitting;
mod page_box;
/// ★★★ **How to get the application back** — the read-mode exit statement.
///
/// Added 2026-09-05 for `OPERATOR_REQUESTS.md` O115: *"I didn't see a way to
/// get back out of read mode."* Read mode hides the ribbon, and the only
/// control that turns it off is on the ribbon — so the mode hides its own exit.
///
/// It is on **this** bar because this bar is the one piece of chrome §2 of
/// `app::window` deliberately keeps, and because read mode composes with full
/// screen: in the combined state there is no ribbon *and* no title bar, so the
/// window title — which carries the same statement — is absent in exactly the
/// state with the least chrome left. Its header carries the full argument for
/// why two surfaces is not duplication here.
mod readmode;
/// **What is selected, said in words** — the readout that turns *"all I get is
/// the page selected"* into a diagnosis. Added 2026-08-27; see its header.
mod selected;

/// The worded decline — a command that was invoked and did not run.
///
/// Split out under R2 like [`page_box`], and along a seam of the same kind:
/// what is left here answers *"how is the bar laid out, and what does each
/// group show?"*, while that module answers *"what did a refused command owe
/// the operator, and how long does it owe it for?"* — its own store, its own
/// vocabulary, its own pure retirement predicate, and an argument about speech
/// acts that nothing else on this surface shares.
///
/// `pub(super)` rather than private, unlike [`page_box`]: `crate::app::dispatch`
/// is the choke point that records a decline and retires it, so the store has
/// to be reachable from a sibling of this module. Nothing outside `crate::app`
/// can see it, which is the right boundary — a decline is written by the one
/// dispatcher and read by the one bar.
pub(super) mod decline;

/// The narrator — the render-diagnostics disclosure and its one line.
///
/// The third module split out of this file under R2, and the seam is the one
/// this header draws in prose two sections above: the notes are **narration**
/// (a census of what a raster contained, demoted behind a triangle because its
/// prominence was wrong), and everything else on the left half is a fact about
/// the operator's own document or gesture, which must not be demoted at all.
/// The two change for different reasons and are argued from opposite
/// premises.
///
/// ★ It is `pub(crate)` rather than private since 2026-08-15, for exactly one
/// export: [`notes::findings`], the ordered, filtered list of what a raster
/// compromised on. The Render-diagnostics dialog
/// (`crate::dialogs::diagnostics`) shows the same facts with room for more than
/// one line, and the *editorial* rules behind that list — which counters are
/// actionable, which two are excluded, and in whose interest the order is —
/// belong to the narrator and must be stated once. The dialog joins nothing and
/// filters nothing; it lists what this module already decided.
pub(crate) mod notes;

use egui::{Align, Layout, Vec2};

use crate::app::actions::Action;
use crate::app::state::Status;
use crate::canvas::pick::PickFilter;
use crate::find::FindState;
use crate::text::find as t_find;

/// ★ The **Select** popup — what a click on the page may land on (O17).
///
/// The one control on this bar that is not a readout: everything else here
/// reports what is true about the view, and this changes what the pointer
/// does. Its header carries why that earns it both its own file and its own
/// position at the left edge of the fixed cluster.
pub(super) mod filter;
/// ★ The **maximum-zoom** popup, behind the zoom readout — O24, and the
/// operator's *"put the max zoom setting on the bar at the bottom"*.
///
/// Its header carries why the readout rather than a new control: the bar's
/// height and right-hand cluster are fixed, and a label that turns out to be
/// a button is already this surface's idiom.
pub(super) mod maxzoom;
/// The zoom controls and the maximum-zoom popup the readout opens.
///
/// Split out under R2 when the popup pushed this file over 1,500 lines;
/// its header carries why the seam is a real one rather than arbitrary.
pub(super) mod zoom;

// ---------------------------------------------------------------------------
// Geometry — see the ★ R128 section of the module docs
// ---------------------------------------------------------------------------

/// The exact outer height, in egui points, the status panel must be given.
///
/// **Pass this to `egui::Panel::bottom(..).exact_size(..)`, not to
/// `default_height`.** The difference is rule R128: `exact_size` pins the
/// panel's outer size so its content cannot perturb the central region at
/// all, and every other sizing API leaves the fit-to-viewport feedback loop
/// open. The module docs carry the measured case.
///
/// [`ROW_HEIGHT_PTS`] plus egui's own `Frame::side_top_panel` inner margin
/// (2 pt above and below) plus a little room for the panel's separator
/// stroke. Generous rather than tight: a bar whose content is clipped by one
/// point is a legibility defect, and the cost of the slack is four pixels of
/// canvas that never change size.
pub const HEIGHT_PTS: f32 = 30.0;

/// ★★★ **The panel's height for a given theme** — use this, not
/// [`HEIGHT_PTS`], and the difference is a defect that shipped.
///
/// # What was wrong with the constant
///
/// [`HEIGHT_PTS`] is `ROW_HEIGHT_PTS` (24) plus egui's frame margins — an
/// arithmetic that is correct **only if the bar's controls are 24 points
/// tall**. They are not. `egui_shell::theme::Metrics::control_height` is
/// **28** in the shipped preset, and a button adds its own padding on top, so
/// the zoom stepper and the Find toggle lay out at **30 points** inside a
/// panel whose content box is 26.
///
/// Measured, at both scales, on a real window:
///
/// ```text
/// ui_scale 1.00, 1600x1000 client:  status-bar  972.0 .. 1002.0   ★ 2 pt past the bottom
/// ui_scale 1.80, 1100x800  client:  status-bar  416.4 ..  446.4   ★ 2 pt past 444.4
/// ```
///
/// So the bottom two points of two controls were clipped off the window, at
/// every UI scale, since the theme's control height was raised.
///
/// ## ★★ Why no test saw it, which is the transferable part
///
/// [`tests::the_bar_is_exactly_as_tall_open_as_closed`] asserts exactly this
/// property — *"the bar's content overflowed its allocated row"* — and it
/// **passes**. It builds an `egui::Context::default()`, which carries egui's
/// own default spacing and **not this application's theme**. In that context
/// the controls really are under 24 points and the assertion is true.
///
/// That is R1's founding shape verbatim: *the only test of that function builds
/// a bare `egui::Context`, so the condition that breaks the real app cannot
/// occur in the harness.* The bar was measured in a world with no theme in it.
///
/// # Why a function and not a bigger constant
///
/// Because the number is a property of the **theme**, and this project ships
/// more than one preset. A constant large enough for the tallest preset would
/// waste canvas on the others and would silently become wrong again the next
/// time a preset raised its control height. Taking it from `Metrics` means the
/// two move together by construction.
///
/// ★ **R128 is untouched.** The rule is that the bar's height must not depend
/// on *what there is to show* — a disclosure opening, a document loading, a
/// note appearing. It may depend on the theme, which changes only when the
/// operator changes it, and which re-lays the whole application out anyway.
/// The panel frame's inner margin, top plus bottom, in points.
///
/// egui's `Frame::side_top_panel` insets its content by 2 points above and
/// below, so a panel of height `h` has `h - FRAME_MARGIN_PTS` to lay out in.
/// Named because both [`height_for`] and the test that guards it need the same
/// number, and a test that used a different one would be measuring a different
/// panel.
pub const FRAME_MARGIN_PTS: f32 = 4.0;

#[must_use]
pub fn height_for(theme: &egui_shell::theme::Theme) -> f32 {
    // A button is its control height plus egui's `button_padding.y` on each
    // side. Two points is `Theme::apply`'s setting; taken as a constant rather
    // than read back from the style because this is called before the frame's
    // `Ui` exists.
    const BUTTON_PADDING_Y: f32 = 2.0;
    // egui's `Frame::side_top_panel` inner margin, above and below, plus room
    // for the panel's separator stroke. The same slack [`HEIGHT_PTS`]'s doc
    // describes, which was right about the margins and wrong about the row.
    const FRAME_SLACK: f32 = FRAME_MARGIN_PTS + 2.0;
    let control = theme.metrics.control_height + BUTTON_PADDING_Y * 2.0;
    // `.max(ROW_HEIGHT_PTS)` so a theme with unusually short controls still
    // gets the row the layout allocates, which is what everything else in this
    // file is written against.
    control.max(ROW_HEIGHT_PTS) + FRAME_SLACK
}

/// The height of the single row every control is laid out inside.
///
/// [`show`] allocates exactly this, so the bar's content height is a
/// constant rather than a function of what there is to say — which is the
/// second half of the R128 defence and the reason the disclosure draws its
/// line *beside* the triangle rather than beneath it.
pub const ROW_HEIGHT_PTS: f32 = 24.0;

/// The panel must be taller than the row it contains, or the bar's own
/// content is clipped by the frame that is supposed to hold it.
///
/// Checked at **compile time** rather than in a test: the relationship
/// between the two constants is a property of the constants, and a test
/// would only re-discover at run time what the compiler can refuse outright.
const _: () = assert!(
    HEIGHT_PTS > ROW_HEIGHT_PTS,
    // ui-text-exempt: compile-error text, never displayed in the UI
    "the panel height must leave room for its own inner margin"
);

/// The **floor** under the zoom readout's reserve — no longer the whole story.
///
/// `8%` and `800%` are different widths, and without a reserve the − button
/// would step sideways every time the operator clicked +. This constant was
/// once the entire answer, and its own doc comment said so: *"wide enough for
/// four characters, which is the whole range [`crate::viewer::ZOOM_LADDER`]
/// can produce."*
///
/// ★★★ **That sentence stopped being true on 2026-08-22 and nothing
/// re-read it.** O24 made the ceiling a preference. `MAX_MAX_ZOOM_PERCENT` is
/// `1e12`, [`crate::text::status::zoom_percent`] formats with `{:.0}`, and so
/// the readout can be asked to draw `1000000000000%` — **fourteen characters
/// in a reserve sized for four.** The outside review of 2026-09-03 spotted the
/// staleness and put the figure at seven; seven was the old ceiling's width,
/// not this one's.
///
/// ★ Kept as a FLOOR rather than deleted, because it is still doing the
/// original job at the bottom of the range: `10%` measures narrower than four
/// characters, and letting the reserve shrink to it would move the − button
/// the other way. The reserve is now `max(this, the measured width of the
/// widest string the CURRENT ceiling can produce)` — see
/// [`status::zoom::readout_width`].
const ZOOM_READOUT_WIDTH_PTS: f32 = 46.0;

/// The share of the bar the render-notes line may occupy before eliding.
///
/// The notes are the *least* urgent thing on this surface (`DEFECTS.md`:
/// "excellent information, wrong prominence"), so on a narrow window they
/// yield to the navigation controls rather than squeezing them. The full
/// text is always available on hover, so nothing is lost — only deferred.
pub(super) const NOTES_WIDTH_FRACTION: f32 = 0.45;

// ---------------------------------------------------------------------------
// Named regions — see `crate::diag::ui_rect` for the contract and the naming
// rule ("a stable, lowercase, hyphenated noun for the thing an operator
// would point at"). These names are matched literally by `tools/ui-verify`,
// so renaming one silently un-aims whatever check was measuring it.
// ---------------------------------------------------------------------------

/// The strip the bar's content occupies.
const REGION_BAR: &str = "status-bar"; // ui-text-exempt: trace region name, never displayed

/// The last fill's rule-4 disclosure, when one is live for this revision.
///
/// Named as a region so `ui-verify` can assert it is **on screen and
/// legible** rather than merely constructed — which for a disclosure is the
/// whole of the requirement.
pub(super) const REGION_FILL_DISCLOSURE: &str = "status-group:fill-disclosure"; // ui-text-exempt: trace region name, never displayed

/// The last vector edit's rule-4 disclosure, when one is live for this
/// revision.
///
/// Named as a region for the same reason as its fill sibling: a disclosure's
/// whole requirement is that it is **on screen and legible**, and `ui-verify`
/// can only assert that about a rect the application published.
pub(super) const REGION_EDIT_DISCLOSURE: &str = "status-group:edit-disclosure"; // ui-text-exempt: trace region name, never displayed

/// See [`recovered_disclosure`].
pub(super) const REGION_RECOVERED: &str = "status-group:recovered"; // ui-text-exempt: trace region name, never displayed
/// The blend-space disclosure's rect, for `ui-verify`.
///
/// ★ A published region name is a cross-repo stability contract with the
/// harness: renaming it turns a check into a skip rather than a failure.
pub(super) const REGION_BLEND_SPACE: &str = "status-group:blend-space"; // ui-text-exempt: trace region name, never displayed
/// The "the picture is still being drawn" line (`OPERATOR_REQUESTS.md` O63).
///
/// ★ The one region in this group naming a **state** rather than an event, so a
/// check reading it is asking *"is the page behind right now"* and not *"did an
/// edit disclose something"*.
pub(super) const REGION_CATCHING_UP: &str = "status-group:catching-up"; // ui-text-exempt: trace region name, never displayed
/// ★★★ The "line weights are off, so this is not what will print" line —
/// `OPERATOR_REQUESTS.md` **O137**.
///
/// The **second** state region, and the one a driven check must be able to find
/// by name, because the whole safety argument for the feature rests on the
/// disclosure being **on screen and legible** rather than merely constructed.
/// A `line_weights` toggle whose disclosure was built into a zero-width rect
/// would be the feature shipping without the thing that makes it safe, and
/// nothing but a published rect can tell those apart.
pub(super) const REGION_LINE_WEIGHTS: &str = "status-group:line-weights"; // ui-text-exempt: trace region name, never displayed

/// `Actual size · Fit width · Fit page`.
const REGION_FIT: &str = "status-group:fit"; // ui-text-exempt: trace region name, never displayed

/// `−  ⟨percent⟩  +`.
const REGION_ZOOM: &str = "status-group:zoom"; // ui-text-exempt: trace region name, never displayed

/// Prefix for one row of the open maximum-zoom popup:
/// `status-maxzoom-row:<index>`.
///
/// Indexed rather than named, for the reason the filter's rows are: a label
/// is operator copy and gets reworded, an index is stable, and a harness
/// chooses positionally.
const REGION_MAXZOOM_ROW: &str = "status-maxzoom-row"; // ui-text-exempt: trace region name, never displayed

/// The Find toggle.
const REGION_FIND: &str = "status-group:find"; // ui-text-exempt: trace region name, never displayed

/// The selection-filter button — the CLOSED control, not its popup.
///
/// The popup publishes its own rows separately (see [`REGION_FILTER_ROW`]),
/// because a harness that can open a list but not choose from it can only
/// assert *the control exists*, which is the one claim that is also true of
/// every inert control.
const REGION_FILTER: &str = "status-group:filter"; // ui-text-exempt: trace region name, never displayed

/// The standing line shown when the filter has left nothing selectable.
///
/// A named region rather than a bare label, because the whole requirement for
/// this sentence is that it is **on screen and legible** at the moment the
/// canvas has stopped responding — and that can only be asserted about a rect
/// the application published.
const REGION_FILTER_EMPTY: &str = "status-group:filter-empty"; // ui-text-exempt: trace region name, never displayed

/// Prefix for one row of the open filter popup: `status-filter-row:<index>`.
///
/// ★ **Indexed, not named.** Labels are operator copy and get reworded; an
/// index is stable and a harness is choosing positionally anyway. The index is
/// the position in [`PickClass::ALL`], which is also the display order.
///
/// These regions exist only on the frames the popup is open, which is what an
/// `Area` laid out at paint time does — see
/// `D:/dev/rag/egui/a_combobox_popup_is_an_area_laid_out_at_paint_time_so_only_the_app_can_publish_its_entry_rects.md`.
const REGION_FILTER_ROW: &str = "status-filter-row"; // ui-text-exempt: trace region name, never displayed

/// The popup's **All** button.
///
/// Published so a driven check can reach a known filter state without knowing
/// which class the fixture's object belongs to — see [`filter::show`].
const REGION_FILTER_ALL: &str = "status-filter-all"; // ui-text-exempt: trace region name, never displayed

/// The popup's **None** button — the twin of [`REGION_FILTER_ALL`].
const REGION_FILTER_NONE: &str = "status-filter-none"; // ui-text-exempt: trace region name, never displayed

/// Trace slot for the bar's steady state, de-duplicated on the rendered line.
const STATUS_SLOT: &str = "status"; // ui-text-exempt: trace slot name, never displayed

// ---------------------------------------------------------------------------
// The bar
// ---------------------------------------------------------------------------

/// Draw the status bar.
///
/// Call it inside a bottom panel pinned to [`HEIGHT_PTS`]:
///
/// ```ignore
/// egui::Panel::bottom("status")
///     .exact_size(crate::app::status::HEIGHT_PTS)
///     .show(ui, |ui| crate::app::status::show(ui, &self.status, &mut actions));
/// ```
///
/// Composition order matters, and the rule is already written down in
/// `crate::app`'s header: *a full-width bar must be added **before** any side
/// panel, or it starts at the side panel's edge instead of spanning the
/// window. A status bar that does not span the window is not a status bar.*
/// So this belongs with the ribbon, above the docks, and the `CentralPanel`
/// stays last because it takes whatever is left.
///
/// Raises actions and mutates nothing — see the module docs.
pub fn show(
    ui: &mut egui::Ui,
    status: &Status,
    find: &mut FindState,
    filter: &mut PickFilter,
    // ★ The operator's configured maximum zoom, edited by the popup behind
    // the zoom readout. Threaded like `filter`, and persisted by the caller
    // for the same reason — see `app::frame`'s status-bar block.
    max_zoom_percent: &mut f32,
    wheel_paging: &mut crate::app::prefs::WheelPaging,
    actions: &mut Vec<Action>,
) {
    // ★ One allocated row, of a height that does not depend on what there is
    // to show. R128; see the module docs for the measurement.
    let row = Vec2::new(ui.available_width(), ROW_HEIGHT_PTS);
    let bar = ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Center), |ui| {
        // ★ Claim the whole row even when nothing is drawn into it.
        //
        // `allocate_ui_with_layout` advances its parent by the child's
        // *min_rect* — what the content actually used — not by the size that
        // was asked for (`egui-0.35.0/src/ui.rs:1330`). Without this line a
        // bar with no document, or with the disclosure closed, would consume
        // less height than one with them, and the R128 loop would be open
        // again through the one path the panel's `exact_size` does not cover:
        // a caller who forgot to use it. Two independent defences, and this
        // is the one that lives in the code being defended.
        ui.set_min_height(ROW_HEIGHT_PTS);

        // ★★★ **FIRST of everything, and BEFORE the no-document guard: how to
        // get the application back.**
        //
        // Read mode hides the ribbon and the docks, and the only control that
        // turns it off lives on the ribbon — so from the moment it is on, this
        // bar is the only piece of chrome left that can say how to leave. The
        // operator reported exactly that on 2026-09-05 (O115).
        //
        // Ahead of the page-drag caption, which the block below says outranks
        // the disclosures, which outrank the narrator. The rule that puts this
        // above all three generalises: **a sentence about how to reach the
        // interface outranks every sentence about the document**, because an
        // operator who cannot reach the interface cannot act on the others.
        //
        // ★ Above the `Status::Open` guard, deliberately. Read mode is per
        // WINDOW rather than per document (`app::window` §3), so the last file
        // can be closed while it is on — and a bar that explained the way out
        // only when a document happened to be open would go silent in the state
        // where the window has the least in it.
        readmode::show(ui);

        // With nothing open there is no page to number, no zoom to report
        // and no raster to have notes about. The bar still occupies its
        // height — that is the whole point of pinning it — but it draws no
        // control, because a control that cannot work is the placeholder the
        // project's invariants forbid.
        let Status::Open(doc) = status else {
            return;
        };

        // ★★ **First on the left while a page drag is in flight**, ahead of
        // everything else the bar has to say.
        //
        // Rule 4's disclosure half for the drag: the caret drawn into the page
        // list and the page view says *where* graphically, and this says the
        // same thing in page numbers and document names, off-canvas. A
        // hairline between two near-identical drawing sheets is precise and
        // not checkable — `panels::pages` reached that conclusion first, for
        // its own caret, and a drag that can now cross documents needs the
        // sentence more, not less, because *which document* is a fact no caret
        // can carry.
        //
        // ★ It also has to be here rather than only in the Pages panel,
        // because the panel can be **closed**. A drop onto the page view is a
        // complete gesture on its own — press in one document's page list,
        // spring a tab, release on the sheet — and the operator can perform
        // most of it with no page list on screen at all.
        //
        // First rather than last: a transient sentence about a gesture in
        // progress outranks four disclosures about things that have already
        // happened, and the left half of this bar yields right-to-left when it
        // runs out of room (see the cluster below), so a line added later
        // would be the one that got squeezed.
        //
        // Costs one `egui::Memory` lookup per frame when nothing is being
        // dragged, which is every frame but the handful the operator is
        // carrying something.
        if let Some(caption) = crate::pagedrag::caption(ui.ctx()) {
            ui.label(caption);
            ui.separator();
        }

        // ★ …and the same treatment for a CANVAS drag being constrained.
        //
        // `ui-conventions/drag-moves.md` D5's second clause: *the affordance
        // shows the constraint while it is active*, whose stated failure mode
        // is an operator who *"holds Shift, gets a result they did not expect,
        // and cannot tell whether the modifier did anything"*. The ghost shows
        // the object behaving; it cannot show that the KEY is why.
        //
        // Beside the page-drag caption rather than folded into it: they are the
        // same species of line — a transient caption about a gesture in
        // progress — and they cannot be live at once, because one drags pages
        // in a list and the other drags geometry on a sheet.
        //
        // It retires itself (`canvas::constrain::caption` compares a frame
        // stamp) so nothing here has to remember to clear it, and it cannot
        // change the bar's height: one label on the pinned row, exactly as its
        // neighbour.
        if let Some(caption) = crate::canvas::constrain::caption(ui.ctx()) {
            ui.label(caption);
            ui.separator();
        }

        // ★★★ FIRST on the left: what is selected.
        //
        // Ahead of the narrator because it is the answer to a question the
        // operator is actively asking — *what did I just click?* — where the
        // render notes are something pdfcer volunteers. When the bar runs out of
        // room the left is what yields, and within the left the volunteered
        // line should yield before the asked-for one.
        selected::show(ui, doc);

        // Left: the narrator, demoted behind a disclosure.
        notes::show(ui, doc);

        // ★ …and beside it, what the last fill INFERRED — which is not
        // demoted, because it is not narration.
        //
        // Rule 4's surviving half: an inference the operator **cannot see**
        // still owes an off-canvas report. `applied_autosize` (pdfcer chose
        // the point size) and `unencodable_chars` (characters replaced with
        // `?`) are the only two facts a fill produces that are **not
        // re-derivable from the saved document** — afterwards they look
        // exactly like the author's own decision.
        //
        // The Forms panel shows them too, and that was enough while the
        // panel was the only way to fill. It stopped being enough on
        // 2026-08-14, when filling arrived on the canvas: a fill can now
        // happen in **Read mode with the panel closed**, and the disclosure
        // would be reachable only by an operator who thought to switch
        // modes and open a panel to look for a message they were never told
        // existed. That is a silent inference, which is the one thing rule
        // 4 forbids outright.
        //
        // It is keyed on `edit_epoch`, so it says nothing about a document
        // that has moved on — an undo or any later edit retires it without
        // anything having to remember to.

        // ★ …and the same obligation for the verbs that move geometry.
        //
        // A move or a delete sometimes has to change how an object is
        // *written* in order to express what the operator asked for — an `re`
        // rectangle becomes four explicit lines when one corner moves on its
        // own, because a rectangle can only describe a box. The picture is
        // identical and the bytes are not recoverable by dragging the corner
        // back, so this is the same species of fact as an inferred auto-size:
        // something pdfcer decided that the saved document cannot afterwards be
        // asked about.
        //
        // `pdfcer-core` has always returned these sentences and
        // `crate::app::actions::vector_edit` has always traced them; until
        // 2026-08-14 that was the whole of it, and that function's own header
        // called it out — recorded, not disclosed. This is where they are
        // disclosed.
        //
        // Keyed on `edit_epoch` exactly as its neighbour is, and for the same
        // reason: an undo or any later edit retires the sentence without
        // anything having to remember to. The two can never both be live —
        // one edit bumps the epoch once and records at most one of them.
        disclosure::all(ui, doc);

        // ★ …and the opposite speech act, in the same place.
        //
        // The three lines above all say *something happened*. This one says
        // *nothing happened*: a command was invoked and declined, because
        // there was nothing for it to act on. Today that is zoom-to-selection
        // with no resolvable bounds and no canvas — `canvas::zoom` has
        // returned those outcomes and traced them from the start, and
        // `app::dispatch` dropped them on the floor until 2026-08-14.
        //
        // It is drawn here rather than folded into `edit_disclosure` because
        // it is a **different store**, not a different message: a decline
        // changes no document, so `edit_epoch` never moves, and an
        // epoch-keyed decline would still be on screen forty gestures later.
        // It retires by the operator's next act instead — `page_box`'s clamp
        // note's rule, not the disclosures'. See `decline`'s header.
        //
        // It can coexist with an edit disclosure (an edit, then a deselect,
        // then the chord), and that is bounded rather than unbounded: each
        // line takes a fraction of what *remains*, so the left half converges
        // and the right-to-left cluster opposite is what yields — the same
        // behaviour the render-notes line has always had.
        // ★ Before the decline note, because it outranks it: a decline
        // explains why one gesture did nothing, while this explains why
        // EVERY gesture will. An operator reading the bar because the
        // canvas stopped responding needs the general answer first.
        filter::empty_note(ui, *filter);

        decline::show(ui, doc);

        // Right: the controls that must never move.
        //
        // ★ Laid out RIGHT-TO-LEFT, so the group added FIRST is drawn
        // RIGHTMOST. The reading order on screen is therefore the reverse of
        // the call order below:
        //
        //     screen:  fit  │  zoom  │  page          (left → right)
        //     calls:   page │  zoom  │  fit           (first → last)
        //
        // The alternative — measuring the cluster and left-aligning it at a
        // computed offset — is the pattern `egui-shell`'s own dock notes call
        // out as fragile (`right − width` goes negative the moment the bar is
        // narrower than its content). A right-to-left layout cannot get that
        // wrong; it simply runs out of room, and the notes on the left are
        // what yields.
        // ★★★ **WHAT THE BAR CAN AFFORD** — added 2026-08-26, and it is the
        // fix for a defect the paragraph above this block could not have
        // prevented, because that paragraph is about *which layout* and this is
        // about *how much*.
        //
        // At `ui_scale = 1.80` in an 1100 x 800 window — 611 points wide — the
        // fixed cluster needs 666 points. A right-to-left layout does not clip
        // to its parent; it runs past the left edge into negative coordinates.
        // Find sat at x = -54 and the selection filter at x = -127, both
        // unreachable, with the left-hand notes drawn underneath the fit group.
        //
        // `fitting` decides, and its header carries the whole argument —
        // including the clause that makes shedding legitimate at all: nothing
        // it may drop is the operator's last route to that capability, checked
        // against the real command registry rather than asserted.
        //
        // ★ The widths come from **last frame's measured rects**, remembered in
        // `egui::Memory`. See `fitting`'s header on why that beats a
        // `min_width()` per group: the alternative is a second implementation
        // of egui's layout, and it would drift silently in the direction of a
        // bar that believes it fits.
        let widths = fitting_widths(&ui.ctx().clone());
        let shown = fitting::affordable(ui.available_width(), &widths);
        fitting::trace_shed(&shown);
        let mut measured = widths.clone();
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Unconditional — `fitting::affordable` always includes it, and the
            // call is written outside the loop so that stays true by
            // construction rather than by the list happening to start with it.
            let before = ui.available_width();
            page_box::group(ui, doc, wheel_paging, actions);
            measured.record(fitting::Group::Page, before - ui.available_width());

            for group in shown.iter().skip(1) {
                ui.separator();
                let before = ui.available_width();
                match group {
                    fitting::Group::Zoom => zoom::group(ui, doc, max_zoom_percent, actions),
                    fitting::Group::Fit => fit::group(ui, doc, actions),
                    // Drawn LEFTMOST but one of the right-hand cluster — which
                    // is where §6 lists it: "Find toggle, actual size, fit
                    // width, fit page, zoom, page". The call order here is the
                    // reverse of the reading order on screen; see the comment
                    // above this block.
                    fitting::Group::Find => find_group(ui, find),
                    // To Find's LEFT — the left end of the fixed cluster, which
                    // is the closest a right-to-left layout can put it to the
                    // canvas it governs. Everything to its right is about the
                    // VIEW (zoom, fit, which page); this is the only control on
                    // the bar that changes what the pointer does, so it sits at
                    // the boundary between the two rather than inside the view
                    // group.
                    fitting::Group::Filter => {
                        let _ = filter::show(ui, filter);
                    }
                    // Unreachable: `affordable` returns a prefix beginning with
                    // `Page`, and the loop skips it. Written as a no-op rather
                    // than `unreachable!()` because a panic in a status bar is
                    // a far worse outcome than a missing page number, and the
                    // prefix property has its own test.
                    fitting::Group::Page => {}
                }
                measured.record(*group, before - ui.available_width());
            }
        });
        // ★ Only the groups actually drawn are re-measured; a shed group keeps
        // its last known width, which is what lets the bar put it back when the
        // window widens again. Clearing it instead would make a shed group
        // "unmeasured", and an unmeasured group is shown unconditionally — so
        // the bar would flap between showing and hiding it every other frame.
        set_fitting_widths(&ui.ctx().clone(), &measured);
    });

    // The content strip, not the panel: a legibility check wants the pixels
    // the bar actually drew into. See `crate::diag::ui_rect` on why the
    // application measures this rather than the harness computing a fraction
    // of the window.
    crate::diag::ui_rect(REGION_BAR, bar.response.rect);

    if let Status::Open(doc) = status {
        crate::diag::trace_changed(STATUS_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                // ★ `wheel=` carries the O30 preference, because a driven
                // check that TOGGLES a persisted setting has to be able to
                // normalise it first. Without it, the second run of such a
                // check inherits the first run's choice and reports the
                // default as broken — which is exactly what happened on
                // 2026-08-24, and the RAG entry it repeated was already
                // written. A setting a check can change is a setting the
                // trace must state.
                "status page={} pages={} zoom={} fit={:?} wheel={}",
                doc.view.page_index,
                doc.pages.len(),
                doc.view.zoom_percent(),
                doc.view.fit,
                doc.prefs.wheel_paging.key(),
            )
        });
    }
}

// ---------------------------------------------------------------------------
// Left — the narrator
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Right — find
// ---------------------------------------------------------------------------

/// The Find toggle, and the whole of what this bar knows about searching.
///
/// A `selectable_label` showing whether the bar is open — see the star
/// section of the module docs for why it shows state at all, and why it
/// writes [`FindState`] instead of raising an [`Action`].
///
/// Drawn only with a document open (the caller has already returned
/// otherwise), because `crate::find::bar` draws nothing without one: a toggle
/// that produced no visible bar would be the placeholder P3 forbids, and the
/// registered command is gated on `doc.pages` for the same reason.
/// The `egui::Memory` slot the bar's measured group widths live in.
///
/// One id for the whole map rather than one per group: they are written
/// together at the end of a frame and read together at the start of the next,
/// and five separate slots would admit the state where three are from this
/// frame and two from the last.
fn fitting_id() -> egui::Id {
    egui::Id::new("status.fitting.widths") // ui-text-exempt: a memory key, never displayed
}

/// What each group of the fixed cluster occupied last frame.
///
/// Empty on the first frame of a session, which [`fitting::affordable`] treats
/// as *show everything* — see its docs on why that bootstrap is required rather
/// than merely tolerant.
fn fitting_widths(ctx: &egui::Context) -> fitting::Widths {
    ctx.data_mut(|d| d.get_temp::<fitting::Widths>(fitting_id()))
        .unwrap_or_default()
}

/// Remember this frame's measurements for the next one.
fn set_fitting_widths(ctx: &egui::Context, widths: &fitting::Widths) {
    ctx.data_mut(|d| d.insert_temp(fitting_id(), widths.clone()));
}

fn find_group(ui: &mut egui::Ui, find: &mut FindState) {
    let rect = ui
        .scope(|ui| {
            if ui
                .selectable_label(find.is_open(), t_find::toggle())
                .on_hover_text(t_find::toggle_tooltip())
                .clicked()
            {
                let open = find.toggle();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("find-toggled open={open} by=status-bar")
                });
            }
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_FIND, rect);
}

// ---------------------------------------------------------------------------
// Right — fit

/// Fixtures the bar's own tests and [`page_box`]'s tests both need.
///
/// A module of its own rather than helpers inside `mod tests`, because two
/// sibling test modules share them and `pub(super)` on a helper buried in one
/// of them would read as "the other module reaches into my tests" rather than
/// as "this is the shared harness". Visible to `crate::app::status` and its
/// descendants, and to nothing else.
#[cfg(test)]
pub(super) mod test_support {
    use super::{Action, PickFilter, Status, show};
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use egui::{Context, Event, Key, Modifiers, RawInput};

    /// An application status with the four-page fixture open.
    ///
    /// Opened through `crate::app::state::open_fixture`, which is the same
    /// three calls `PdfcerApp::open_path` makes in the same order — so what
    /// these tests drive is the real state machine rather than a hand-built
    /// approximation of it.
    pub(in crate::app::status) fn opened() -> Status {
        Status::Open(Box::new(open_fixture(FOUR_PAGES)))
    }

    /// Run one frame of the bar and return the actions it raised.
    pub(in crate::app::status) fn frame(
        ctx: &Context,
        status: &Status,
        input: RawInput,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        // A throwaway `FindState`: these tests are about the bar's own
        // controls, and the Find toggle writes its state directly rather than
        // raising an action, so nothing they assert can reach it.
        let mut find = crate::find::FindState::default();
        let mut filter = PickFilter::default();
        let mut max_zoom = crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT;
        let _ = ctx.run_ui(input, |ui| {
            show(
                ui,
                status,
                &mut find,
                &mut filter,
                &mut max_zoom,
                &mut crate::app::prefs::WheelPaging::default(),
                &mut actions,
            )
        });
        actions
    }

    /// Build a `RawInput` carrying one key press.
    pub(in crate::app::status) fn key_press(key: Key, modifiers: Modifiers) -> RawInput {
        RawInput {
            events: vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            modifiers,
            ..Default::default()
        }
    }

    /// One frame of the bar, measured: how tall it was, and how many shapes
    /// it painted.
    ///
    /// `None` when no measurement happened at all — the closure never ran, or
    /// it produced a non-finite height. That is `HANDOFF.md` §10's rule, and
    /// it is not theoretical here: `cargo test -p egui-shell` and `cargo test
    /// --workspace` compile `egui` with different features (no fonts vs
    /// `default_fonts`), so a layout assertion can be entirely vacuous under
    /// one of the two commands a developer runs. A helper that returned a bare
    /// `f32` would hand a vacuous run the same `NAN == NAN`-adjacent silence a
    /// real one gets.
    ///
    /// The shape count is the second half of the same discipline, and it is
    /// the half that matters for a *sentence*: a height comparison between two
    /// frames that both drew nothing is true and worthless. Counting the
    /// painted shapes is how a test proves the line reached the painter rather
    /// than merely reaching the data.
    ///
    /// Lives here rather than in `mod tests` because **three** R128 tests need
    /// it now — the fill line, the edit line and [`super::decline`]'s — and the
    /// third is in a sibling module. `pub(super)` on a helper buried inside one
    /// test module would read as "the other module reaches into my tests"
    /// rather than as "this is the shared harness".
    pub(in crate::app::status) fn bar_frame(
        ctx: &Context,
        status: &Status,
    ) -> Option<(f32, usize)> {
        let mut height = f32::NAN;
        let mut find = crate::find::FindState::default();
        let mut filter = PickFilter::default();
        let mut max_zoom = crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT;
        let output = ctx.run_ui(RawInput::default(), |ui| {
            let mut actions = Vec::new();
            height = ui
                .scope(|ui| {
                    show(
                        ui,
                        status,
                        &mut find,
                        &mut filter,
                        &mut max_zoom,
                        &mut crate::app::prefs::WheelPaging::default(),
                        &mut actions,
                    )
                })
                .response
                .rect
                .height();
        });
        height.is_finite().then_some((height, output.shapes.len()))
    }

    /// Two frames, reporting the second.
    ///
    /// egui settles over a pass: fonts are laid out lazily, widget galleys are
    /// cached on first sight, and animations start at their "from" value. A
    /// single frame therefore compares one state's *first* look against
    /// another state's *first* look, which is a comparison of two different
    /// things. Every caller wants the steady state.
    pub(in crate::app::status) fn settled_bar_frame(
        ctx: &Context,
        status: &Status,
    ) -> Option<(f32, usize)> {
        let _ = bar_frame(ctx, status);
        bar_frame(ctx, status)
    }
}

#[cfg(test)]
mod tests;
