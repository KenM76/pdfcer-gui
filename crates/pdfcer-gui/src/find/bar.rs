//! # `find::bar` — the Find overlay's controls, and the keys they own
//!
//! One compact box **floating over the top-right of the page**, which is
//! where Acrobat Reader, Chrome's PDF viewer and Edge's all put theirs:
//!
//! ```text
//!                          ┌────────────────────────────────────────────┐
//!                          │ Find [ total       ] ⏴ ⏵ 3 of 47  Options ⏷  × │
//!                          └────────────────────────────────────────────┘
//! ```
//!
//! [`super`]'s header carries the placement note. The short version is that an
//! overlay consumes **no layout space**, so opening Find does not resize the
//! canvas and therefore does not re-fit the page under the operator's eyes.
//! That is not a theoretical advantage: a docked version of this bar was built
//! first and driven, and pressing Ctrl+F under Fit page took the zoom from
//! 85 % to 81 % and back again on close — a page that jumps every time you go
//! looking for a word on it.
//!
//! ## ★ The four search options live in a menu, not on the bar
//!
//! Match case, Whole word, Wildcards and the whole-word **rule** are behind
//! the `Options` button. Three reasons, in order of weight:
//!
//! 1. **An overlay has to be narrow**, because it covers the page. Laid out in
//!    a row, those four controls are wider than the search field, the step
//!    buttons and the readout put together; the box would span most of the
//!    window and hide the thing being searched for.
//! 2. **It is what the reference product does.** Reader's Ctrl+F box is a
//!    field, two arrows and a settings dropdown holding *Whole words only* and
//!    *Case-Sensitive*. An operator arriving from Acrobat finds the options
//!    where they left them.
//! 3. **The rule chooser can then appear and disappear without moving
//!    anything.** It is meaningful only while Whole word is on (see
//!    [`options_menu`]), and inside a menu its arrival costs a row of a popup
//!    rather than shifting every control to its right — which, on a bar the
//!    operator is aiming at, is the difference between a tidy layout and a
//!    mis-click.
//!
//! ## ★ The width is fixed, and nothing on the bar may move
//!
//! The box is anchored by its **top-right corner** to the canvas viewport, so
//! its left edge is `right − width`: any change of width moves every control
//! on it. The search field, the readout and the buttons therefore all have
//! reserved widths ([`FIELD_WIDTH_PTS`], [`READOUT_WIDTH_PTS`]) and the
//! options are in a menu — so `3 of 47` becoming `No matches`, or a search
//! finding nothing at all, cannot slide the ⏴ ⏵ buttons out from under the
//! pointer between two clicks. That is the same reason the status bar reserves
//! a width for its zoom readout.
//!
//! The height is fixed too, at [`ROW_HEIGHT_PTS`], but for a much weaker
//! reason: layout tidiness. **R128 does not reach this surface.** That rule is
//! *a panel whose size feeds a fit-to-viewport computation has a fixed size*,
//! and an `egui::Area` feeds no such computation because it consumes none of
//! the layout. Docking the bar is what would have made R128 bind, and that is
//! one of the reasons it is not docked.
//!
//! ## ★ The three keys, and the one that is shared
//!
//! | key | while the field has focus | otherwise |
//! |---|---|---|
//! | Enter | search, or step to the next hit | belongs to whatever has focus |
//! | Shift+Enter | search, or step to the previous hit | as above |
//! | **Escape** | close the bar | **belongs to the canvas** |
//!
//! Escape is the interesting one, because three surfaces want it: a canvas
//! drag in flight wants to abandon itself, the selection ladder wants to
//! ascend a rung, and this bar wants to close. There is no arbitration code,
//! and there does not need to be — `crate::canvas::interact` already reads
//! Escape as `!ctx.text_edit_focused() && …`, so while the operator is typing
//! here the canvas is not offered the key at all. This file takes it only
//! under the same condition, from the other side.
//!
//! The consequence is worth stating because it looks like a gap: **Escape does
//! not close the bar after the operator has clicked on the page.** That is
//! deliberate. At that moment Escape is the selection ladder's, and a bar that
//! stole it would cost the operator the rung they were working in — the same
//! one-press-one-effect rule `canvas::interact` applies between the gesture
//! machine and the ladder. The close button and Ctrl+F are the routes out from
//! there, and both are visible.
//!
//! ## ★ What Enter does depends on whether the answer is still current
//!
//! [`super::FindState::readout`] is the single test, and [`enter_intent`] is
//! the whole decision as a pure function of it:
//!
//! | readout | Enter | Shift+Enter |
//! |---|---|---|
//! | `Idle` — nothing searched for what is in the box | **search** | **search** |
//! | `Stale` — the document has been edited since | **search** | **search** |
//! | `At` — there are hits | step **next** | step **previous** |
//! | `Empty` — searched, nothing found | **nothing** | **nothing** |
//!
//! The last row is the one that needs defending. Re-running a search that just
//! returned nothing would re-extract the whole document's text — **350 ms on
//! the benchmark drawing, measured**; see [`super`]'s cost section — to
//! produce the same empty answer, and an operator leaning on Enter would do it
//! once per press. Nothing has changed since the search ran; if something had,
//! the readout would be `Stale` and the first row would apply.
//!
//! ## Actions, not mutations
//!
//! Every commit leaves here as an [`Action::Find`]. The two exceptions are the
//! ones `crate::app::status`'s page box already takes and for the same reason:
//! the **text buffer** and the **option flags** are widget state, they describe
//! the control rather than the document, and deferring a keystroke to after
//! the frame would make typing lag by a frame.
//!
//! ## Where the strings are
//!
//! [`crate::text::find`], all of them. Nothing here is a literal an operator
//! can read; `tools/gates/check-ui-strings.sh` is the mechanical half of that
//! rule and [`crate::text`]'s header is the reason for it.

use egui::{Align, Align2, Layout, Pos2, Rect, Vec2};

use crate::app::actions::Action;
use crate::app::state::Status;
use crate::find::{FindOptions, FindRequest, FindState, Readout, Step};
use crate::text::find as t;
use pdfcer_core::edit::WordBoundary;

// ---------------------------------------------------------------------------
// Geometry — see the "the width is fixed" section of the module docs
// ---------------------------------------------------------------------------

/// How wide the overlay's content row is, in egui points.
///
/// **Fixed, and load-bearing.** The box is anchored by its top-right corner,
/// so its left edge is `right − width`; a width that varied with the readout's
/// text would move every control on the bar every time a search ran.
///
/// Deliberately a little generous: a row that overflows its allocation *wraps*
/// in egui, and a wrapped Find bar is two rows tall with its close button
/// underneath its own search field.
pub const BAR_WIDTH_PTS: f32 = 460.0;

/// The height of the single row every control is laid out inside.
///
/// Layout tidiness rather than R128 — see the module docs on why that rule
/// does not reach a surface which consumes no layout. What it does buy is that
/// the box does not change shape as the readout changes, which matters for the
/// same reason the width does.
pub const ROW_HEIGHT_PTS: f32 = 24.0;

/// The gap between the overlay and the canvas viewport's top-right corner.
///
/// Enough that the box reads as floating *over* the page rather than as
/// something welded to the edge of the window, and enough that its shadow has
/// somewhere to fall.
const MARGIN_PTS: f32 = 12.0;

/// How wide the search field is.
///
/// Wide enough for a part number or a short phrase — the two things drawing
/// reviewers actually search for. Reserved rather than proportional, for the
/// reason the module docs give: everything to its right is positioned from it.
const FIELD_WIDTH_PTS: f32 = 190.0;

/// How wide the position readout is.
///
/// `3 of 47`, `No matches` and `Document changed` are three very different
/// widths, and without a reserve every search would shove its neighbours
/// sideways — and, because the box is right-anchored, would move the search
/// field the operator is typing into. Sized for the longest of the three at
/// the default text size; anything longer elides, with the whole string on
/// hover.
const READOUT_WIDTH_PTS: f32 = 110.0;

// ---------------------------------------------------------------------------
// Named regions — see `crate::diag::ui_rect` for the contract and the naming
// rule. These names are matched literally by `tools/ui-verify`, so renaming
// one silently un-aims whatever check was measuring it.
// ---------------------------------------------------------------------------

/// The whole overlay, including its frame.
///
/// ★ Published so `ui-verify` can reach the bar at all. A check that wants to
/// assert *"Ctrl+F produced a find bar, and its text is legible"* has exactly
/// two honest sources for **where to look** — the application measures the
/// rect on the frame it reports, or the harness hard-codes a fraction of the
/// window and goes stale the first time a panel moves. This is the first
/// source, and for a *floating* surface it is the only one: an overlay's
/// position depends on the canvas viewport, which depends on the dock, so no
/// constant a harness could hold would survive opening a panel.
const REGION_BAR: &str = "find-bar"; // ui-text-exempt: trace region name, never displayed

/// The search field itself.
const REGION_FIELD: &str = "find-field"; // ui-text-exempt: trace region name, never displayed

/// The step buttons and the position readout.
const REGION_POSITION: &str = "find-position"; // ui-text-exempt: trace region name, never displayed

/// The options menu button.
const REGION_OPTIONS: &str = "find-options"; // ui-text-exempt: trace region name, never displayed

/// The OCR offer's second row, when it is drawn.
///
/// Published so `ui-verify` can assert on the offer's **presence and absence**
/// rather than on a screenshot. That matters more here than for the other
/// regions: the offer is one line of muted text and a small button on a
/// floating box over a drawing sheet, which is exactly the kind of thing a
/// pixel oracle cannot distinguish from the frame before it — `HANDOFF.md` §2's
/// defect 8, again. A declared rect is a claim the application makes about
/// itself, and the absence of one is the harness's evidence that the offer was
/// not drawn.
const REGION_OCR_OFFER: &str = "find-ocr-offer"; // ui-text-exempt: trace region name, never displayed

/// Trace slot for the bar's steady state, de-duplicated on the rendered line.
const FIND_SLOT: &str = "find-bar"; // ui-text-exempt: trace slot name, never displayed

// ---------------------------------------------------------------------------
// Widget ids
// ---------------------------------------------------------------------------

/// The overlay's `egui::Area` id.
const AREA_ID: &str = "pdfcer-find-bar"; // ui-text-exempt: widget id, never displayed

/// The search field's id.
///
/// ★ **Stable and explicit, because defect D1 depends on it.**
/// `crate::app::keyboard::collect` guards its unmodified bindings with
/// `ctx.text_edit_focused()`, which resolves the focused id and asks whether a
/// `TextEditState` exists *for that id*. The field therefore has to be a real
/// [`egui::TextEdit`] with an id that does not move between frames, or
/// `PageDown` would step the page while the operator was halfway through
/// typing a search term. It also has to be stable for
/// [`super::FindState::take_focus_request`] to be able to focus it.
const FIELD_ID: &str = "pdfcer-find-field"; // ui-text-exempt: widget id, never displayed

// ---------------------------------------------------------------------------
// The overlay
// ---------------------------------------------------------------------------

/// Draw the Find overlay, if it is open and there is a document to search.
///
/// Call it from `PdfcerApp::ui` **after** the canvas and **before** the modal
/// dialogs. Both halves are ordering decisions:
///
/// - **after the canvas**, because the box is positioned from the canvas
///   viewport's own rect, which `crate::canvas::show` records through
///   `zoom::remember_frame` as the last thing it does. Drawing first would
///   position this frame's box from last frame's layout, which is visible as a
///   one-frame lag every time a dock is resized;
/// - **before the dialogs**, because a modal takes the frame and must be on
///   top of everything, this included.
///
/// # Two states draw nothing at all, and neither is an oversight
///
/// - **Closed.** No area, no widgets, no hit-test region over the page.
/// - **Open with no document.** `edit.find` is gated on `doc.pages`, so the
///   bar cannot be *opened* without one; but a document can be closed while it
///   is open, and a search box over nothing is a control whose every input is
///   refused. The flag survives, so reopening a document brings the bar back
///   exactly as the operator left it — the same courtesy the recent list
///   extends, for the same reason.
pub fn show(ui: &mut egui::Ui, state: &mut FindState, status: &Status, actions: &mut Vec<Action>) {
    if !state.is_open() {
        return;
    }
    let Status::Open(doc) = status else {
        return;
    };
    let epoch = doc.edit_epoch;
    let ctx = ui.ctx().clone();
    let host = host_rect(&ctx);

    let area = egui::Area::new(egui::Id::new(AREA_ID))
        // `Middle` rather than `Foreground`: the box floats over the page and
        // the docks, and is floated over in turn by its own options menu, by
        // tooltips and by a modal — all of which egui puts in higher orders.
        // Claiming `Foreground` here would put the Find bar over its own popup.
        .order(egui::Order::Middle)
        // ★ Anchored by its RIGHT-top corner, and that is a fix rather than a
        // preference — found by driving the binary and reading the trace.
        //
        // With a LEFT-top pivot the position is `right − width`, and egui does
        // not know an `Area`'s width until it has laid it out once. So on the
        // frame Ctrl+F was first pressed the box appeared **108 points to the
        // left** of where it belonged and snapped into place on the next
        // frame: two `ui-rect name=find-bar` lines back to back, same size,
        // different origin. Visible as a flinch every time the bar opened.
        //
        // A right-top pivot makes the corner this design actually cares about
        // — the one MARGIN_PTS inside the canvas's top-right — the thing egui
        // is given, so it is exact from the first frame whatever the measured
        // width turns out to be. `default_width` closes the same gap for the
        // constraint below, which would otherwise solve against a width of
        // zero on that first frame.
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(anchor_right_top(host))
        .default_width(BAR_WIDTH_PTS)
        // A canvas viewport narrower than the box — reachable with both docks
        // open on a minimum-size window — must not push the close button off
        // the edge the operator is reaching for.
        .constrain_to(host);

    // ★ The OCR offer's condition, evaluated HERE and nowhere else.
    //
    // Two questions, and the order is the whole affordability argument:
    //
    // 1. `readout == Empty` — a search has been committed and matched nothing.
    //    Free: it is a comparison against state the bar already holds.
    // 2. the page has no extractable text at all — a `PageTextCache` read
    //    (`OpenDoc::page_has_extractable_text`), which on a cache miss is one
    //    page extraction.
    //
    // Asking (2) only after (1) is what keeps this off the frame budget. The
    // bar draws on every frame it is open and this module's header records that
    // nothing here may search on a keystroke; a per-frame page extraction would
    // be the same defect one size smaller. By the time (1) holds, the operator
    // has just paid a WHOLE-DOCUMENT extraction for the search itself — so the
    // page extraction is strictly cheaper than the gesture that caused it, and
    // it is charged to that gesture rather than to the act of opening the bar.
    //
    // ★ And (2) is not a refinement of (1). It is the *actual* trigger — the
    // operator's rule is that the offer means "this document is images", never
    // "this search had no matches". (1) is here because the offer is drawn in
    // the place the empty readout occupies and there is nowhere else on a
    // fixed-width bar for it to go; (2) is what makes it correct. A build that
    // dropped (2) would offer to OCR a text PDF every time somebody mistyped a
    // part number.
    let offer_ocr = offer_ocr(state.readout(epoch), || doc.page_has_extractable_text());
    // ★★★ THE OTHER REASON A SEARCH FINDS NOTHING.
    //
    // `pdfcer-core` v0.11.0's note, in its own words: *"a zero-result search is
    // not proof the word is absent"*. Two situations produce an identical empty
    // result — the needle is not there, or the document's text was never
    // recoverable as Unicode so no needle could have matched. The second does
    // not look broken, because the text **renders perfectly**.
    //
    // Drawn only on an EMPTY readout: a search that found things has already
    // answered the operator's question, and a caveat under a successful result
    // is the nagging the operator objected to. See `find::Results::
    // unsearchable_fonts` for why this is owed at all under rule 4.
    let unsearchable = if matches!(state.readout(epoch), Readout::Empty) {
        state.unsearchable_fonts(epoch)
    } else {
        0
    };

    let response = area
        .show(&ctx, |ui| {
            // `Frame::popup` is the theme's own floating-surface frame — fill,
            // stroke, rounding and shadow all read from `Style`. A hand-built
            // frame here would be a second set of colours outside the theme
            // module, which is exactly what `check-theme-colors.sh` exists to
            // prevent.
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                body(ui, state, epoch, actions);
                if unsearchable > 0 {
                    unsearchable_note(ui, unsearchable);
                }
                if offer_ocr {
                    ocr_offer(ui, actions);
                }
            });
        })
        .response;

    crate::diag::ui_rect(REGION_BAR, response.rect);
}

/// The rect the overlay is positioned inside — the **canvas viewport**, not
/// the window.
///
/// Read from `crate::canvas::zoom::last_frame`, which is the canvas's own
/// record of where it drew. That matters as soon as a dock is open: anchoring
/// to the window's top-right would put the box over the right-hand panel
/// rather than over the page, and would move it every time a splitter was
/// dragged even though the page had not moved.
///
/// The fallback is the whole screen rect, and it is reachable rather than
/// defensive: a document with no pages, or one whose current page will not
/// rasterize, makes `canvas::show` return before it records a frame — and
/// both of those documents still have text worth searching. The box then sits
/// at the window's top-right, over the sentence explaining why there is no
/// page, which is the best available answer.
fn host_rect(ctx: &egui::Context) -> Rect {
    // `content_rect`, not `viewport_rect`: the former is what egui considers
    // safe to draw content into (it subtracts an OS status bar or a display
    // notch), and a Find box tucked under a notch is a Find box the operator
    // cannot close.
    crate::canvas::zoom::last_frame(ctx).map_or_else(|| ctx.content_rect(), |f| f.viewport_rect)
}

/// **The point the overlay's right-top corner is pinned to** — [`MARGIN_PTS`]
/// inside `host`'s own top-right corner.
///
/// A free function, and taking no width, because that is the whole point of
/// the right-top pivot: the placement is a corner, not a corner minus a
/// measurement. See the pivot's comment in [`show`] for the frame-one defect
/// that made it one.
///
/// Clamped into `host` on both axes so that a host smaller than the margin
/// cannot produce a point outside the canvas. `constrain_to` would pull the
/// box back anyway; this keeps the constraint a safety net rather than the
/// thing deciding the layout.
#[must_use]
fn anchor_right_top(host: Rect) -> Pos2 {
    Pos2::new(
        (host.right() - MARGIN_PTS).max(host.left()),
        (host.top() + MARGIN_PTS).min(host.bottom()),
    )
}

/// Everything on the row.
///
/// Split from [`show`] so the placement and the layout are separately
/// readable, and so the row can be driven by a test without an `Area` — what
/// those tests are about is the controls and the keys, not where the box sits.
fn body(ui: &mut egui::Ui, state: &mut FindState, epoch: u64, actions: &mut Vec<Action>) {
    let row = Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS);
    ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Center), |ui| {
        // Claim the whole row even if the content uses less of it.
        // `allocate_ui_with_layout` advances its parent by the child's
        // *min_rect* — what the content actually used — so without these the
        // box would breathe as the readout changed, and a right-anchored box
        // that breathes moves its own search field.
        ui.set_min_size(row);
        ui.set_max_size(row);

        field(ui, state, epoch, actions);
        position(ui, state, epoch, actions);

        // The options menu and the close button, hard right, in that order
        // from the right edge inwards. A right-to-left layout over whatever is
        // left cannot get this wrong; measuring the row and placing a button
        // at `width − button` goes negative the moment the content is wider
        // than the row, which `egui-shell`'s dock notes record as the fragile
        // pattern.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .button(t::close())
                .on_hover_text(t::close_tooltip())
                .clicked()
            {
                state.close();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "find-closed by=button".to_owned()
                });
            }
            options(ui, state, actions);
        });
    });

    // `is_open()` rather than a literal `true`: a control drawn on this very
    // row may have closed the bar during the frame — the close button does,
    // and so does Escape — and a line reading `open=true` on the frame the bar
    // closed would be the last thing in the trace and would be false.
    crate::diag::trace_changed(FIND_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-bar open={} query={:?} readout={:?}",
            state.is_open(),
            state.query(),
            state.readout(epoch),
        )
    });
}

// ---------------------------------------------------------------------------
// The field
// ---------------------------------------------------------------------------

/// The label, the search box, and the three keys the box owns.
fn field(ui: &mut egui::Ui, state: &mut FindState, epoch: u64, actions: &mut Vec<Action>) {
    let readout = state.readout(epoch);
    let rect = ui
        .scope(|ui| {
            ui.label(t::field_label());

            // Read the keys BEFORE the widget is built, so what is examined is
            // the frame's raw input rather than whatever survived the
            // `TextEdit` consuming it. egui's single-line `TextEdit` responds
            // to Enter and Escape by surrendering focus, which is why both are
            // recognised through `lost_focus()` below rather than through
            // `has_focus()` alone.
            let (enter, escape, shift) = ui.input(|i| {
                (
                    i.key_pressed(egui::Key::Enter),
                    i.key_pressed(egui::Key::Escape),
                    i.modifiers.shift,
                )
            });

            let focus_wanted = state.take_focus_request();
            let response = ui.add_sized(
                Vec2::new(FIELD_WIDTH_PTS, ROW_HEIGHT_PTS),
                egui::TextEdit::singleline(state.query_mut())
                    .id(egui::Id::new(FIELD_ID))
                    .hint_text(t::field_label()),
            );
            let response = response.on_hover_text(t::field_tooltip());
            if focus_wanted {
                response.request_focus();
            }

            let had_focus = response.has_focus() || response.lost_focus();
            if had_focus && escape {
                // See the module docs: this bar takes Escape ONLY while the
                // field has focus, which is exactly the condition under which
                // `canvas::interact` has already declined it.
                state.close();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "find-closed by=escape".to_owned()
                });
                return;
            }
            if response.lost_focus() && enter {
                // Give the focus straight back, so a run of Enters walks the
                // hits instead of the first one dropping the operator out of
                // the box.
                response.request_focus();
                if let Some(request) = enter_intent(readout, shift) {
                    actions.push(Action::Find(request));
                }
            }
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_FIELD, rect);
}

/// ★ **What Enter means**, as a pure function of the readout and the shift
/// key.
///
/// The table is in this module's header; the argument for the `Empty` row —
/// the only one that returns `None` — is there too, and it is about cost: a
/// re-search of a query that just matched nothing re-extracts the whole
/// document's text to produce the same answer, and nothing has changed since
/// it did.
///
/// `Stale` searches rather than steps, which is the mechanism by which the
/// bar's own "press Enter to search again" tooltip is true.
#[must_use]
fn enter_intent(readout: Readout, shift: bool) -> Option<FindRequest> {
    match readout {
        Readout::Idle | Readout::Stale => Some(FindRequest::Search),
        Readout::At { .. } => Some(FindRequest::Step(if shift {
            Step::Previous
        } else {
            Step::Next
        })),
        Readout::Empty => None,
    }
}

// ---------------------------------------------------------------------------
// The OCR offer
// ---------------------------------------------------------------------------

/// ★ **Whether to offer OCR**, as a pure function of the readout and the page.
///
/// `page_has_text` is a closure rather than a `bool` so that the caller's
/// answer is **not computed unless it is needed** — the short-circuit is the
/// affordability argument, and passing an already-evaluated `bool` would make
/// the call site pay for a page extraction on every frame the bar is open while
/// this function still looked correct. That is the shape of `HANDOFF.md` §2's
/// defect 9: right work, wrong moment, invisible to every test.
///
/// # The rule, and the trap inside it
///
/// | readout | page has text | offer |
/// |---|---|---|
/// | `Empty` | no | **yes** — there is nothing here for any search to have found |
/// | `Empty` | yes | no — an ordinary empty result; the words are there, that one is not |
/// | `At` / `Idle` / `Stale` | either | no |
///
/// The second row is the operator's stated rule and the whole reason this is a
/// function with a test rather than an `if` in the layout: *"the document is
/// images"* is not *"this search had no matches"*, and a build that collapsed
/// them would offer to recognise a text PDF every time somebody mistyped a part
/// number. [`tests::an_ordinary_empty_result_on_a_text_page_offers_nothing`] is
/// the falsifying case.
///
/// The third row is not merely "nothing to offer". A `Stale` readout means the
/// document has been edited since the search ran, so the *page* answer is about
/// a revision the hit list does not describe; and `Idle` means nothing has been
/// asked at all, where an offer would be the bar volunteering an opinion about a
/// document the operator has not yet questioned.
#[must_use]
fn offer_ocr(readout: Readout, page_has_text: impl FnOnce() -> bool) -> bool {
    matches!(readout, Readout::Empty) && !page_has_text()
}

/// The second row: what is true of the page, and the way out of it.
///
/// Drawn **below** the control row rather than inside it, and that is a
/// consequence of this module's fixed-width rule rather than a layout
/// preference. The box is anchored by its top-right corner, so anything added
/// to the row would move the search field the operator is typing into; a row
/// added underneath grows the box downwards, over the page, and moves nothing.
/// An `egui::Area` consumes no layout, so the extra height costs the canvas
/// nothing either — which is the same property that made the bar an overlay in
/// the first place (see the module header's 85 %-to-81 % measurement).
///
/// It appears and disappears with the condition rather than being greyed. P3
/// permits greying only for a *temporarily* unavailable capability that is
/// always explained on hover, and "this page happens to have text on it" is not
/// a state an operator can act their way out of — a permanently dead control
/// explaining a fact about the document is the placeholder the rule forbids.
fn ocr_offer(ui: &mut egui::Ui, actions: &mut Vec<Action>) {
    ui.add_space(4.0);
    ui.separator();
    let rect = ui
        .scope(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_min_size(Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS));
                    // The muted role, because this is a statement about the
                    // document rather than a control. Not `.strong()`:
                    // `DEFECTS.md` D11 records that role as unusable in this
                    // theme.
                    let theme = egui_shell::theme::Theme::of(ui.ctx());
                    ui.label(
                        egui::RichText::new(crate::text::ocr::offer())
                            .color(theme.palette.text_muted),
                    );
                    if ui
                        .button(crate::text::ocr::offer_action())
                        .on_hover_text(crate::text::ocr::offer_tooltip())
                        .clicked()
                    {
                        // ★ Raised as a COMMAND, not as a new action variant.
                        //
                        // The offer is a second route to `file.ocr` and must
                        // not become a second implementation of it: routing it
                        // through the command means the ribbon control and this
                        // button reach one dispatch arm, one dialog and one set
                        // of guards. `app/mod.rs`'s rule that dispatch arms
                        // route rather than compute is what makes that free.
                        actions.push(Action::Command(OCR_COMMAND.to_owned()));
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed in the UI
                            "find-ocr-offer accepted=true".to_owned()
                        });
                    }
                },
            );
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_OCR_OFFER, rect);
}

/// The command the offer raises.
///
/// Named here rather than typed inline so that
/// [`tests::the_offer_raises_the_registered_recognise_command`] can assert it
/// against the registry — an id that is merely spelled at a call site is an id
/// that goes stale silently, which is `HANDOFF.md` §5's whole subject.
const OCR_COMMAND: &str = "file.ocr"; // ui-text-exempt: a command id, never displayed

// ---------------------------------------------------------------------------
// Stepping and the readout
// ---------------------------------------------------------------------------

/// `⏴ ⏵  3 of 47`.
///
/// The two buttons are **enabled only when there is something to step**, and
/// each explains its greyed state on hover — P3 permits greying only for
/// *temporarily* unavailable and only when it is always explained. Both
/// conditions hold here: an empty or stale result set is a state that ends the
/// moment the operator presses Enter, and
/// [`crate::text::find::step_unavailable_tooltip`] says so.
fn position(ui: &mut egui::Ui, state: &FindState, epoch: u64, actions: &mut Vec<Action>) {
    let readout = state.readout(epoch);
    let steppable = matches!(readout, Readout::At { .. });
    let rect = ui
        .scope(|ui| {
            if ui
                .add_enabled(steppable, egui::Button::new(t::previous()))
                .on_hover_text(t::previous_tooltip())
                .on_disabled_hover_text(t::step_unavailable_tooltip())
                .clicked()
            {
                actions.push(Action::Find(FindRequest::Step(Step::Previous)));
            }
            if ui
                .add_enabled(steppable, egui::Button::new(t::next()))
                .on_hover_text(t::next_tooltip())
                .on_disabled_hover_text(t::step_unavailable_tooltip())
                .clicked()
            {
                actions.push(Action::Find(FindRequest::Step(Step::Next)));
            }

            // A reserved slot, drawn even when it is empty, so that running a
            // search cannot move the box's own left edge. See the module docs.
            let (text, hover) = readout_text(readout);
            ui.allocate_ui_with_layout(
                Vec2::new(READOUT_WIDTH_PTS, ROW_HEIGHT_PTS),
                Layout::left_to_right(Align::Center),
                |ui| {
                    let label = ui.add(egui::Label::new(&text).truncate());
                    if !hover.is_empty() {
                        label.on_hover_text(hover);
                    }
                },
            );
        })
        .response
        .rect;
    crate::diag::ui_rect(REGION_POSITION, rect);
}

/// The readout's text and its hover text, or two empty strings for
/// [`Readout::Idle`].
///
/// Split out as a pure function so the four sentences can be asserted without
/// a frame, and so the "Idle draws nothing" case is a value rather than a
/// branch somebody has to notice in the layout code.
#[must_use]
fn readout_text(readout: Readout) -> (String, &'static str) {
    match readout {
        // Deliberately blank rather than `0 of 0`. Nothing has been asked, so
        // there is nothing to answer.
        Readout::Idle => (String::new(), ""),
        Readout::Empty => (t::no_matches().to_owned(), t::no_matches_tooltip()),
        Readout::Stale => (t::stale().to_owned(), t::stale_tooltip()),
        Readout::At { current, total } => (t::position(current, total), t::position_tooltip()),
    }
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// The `Options` menu button.
///
/// ★ **Changing an option re-runs the search**, when and only when a search
/// has already been run for what is in the box. Both halves matter:
///
/// - re-running is what makes the control *do* something — a "Whole word"
///   checkbox that left the old hit list on screen would be an inert control,
///   which is the shape defect D1 took;
/// - only after a search, because otherwise ticking a box on a bar the
///   operator has not yet used would run a whole-document text extraction for
///   a query they have not finished typing.
///
/// The test is [`super::FindState::answered`] *before* the change is applied —
/// "the bar is currently showing an answer to what is in it", which is exactly
/// the state in which leaving the old answer up would be wrong.
fn options(ui: &mut egui::Ui, state: &mut FindState, actions: &mut Vec<Action>) {
    let mut options = state.options();
    let before = options;

    let rect = ui
        .menu_button(t::options(), |ui| options_menu(ui, &mut options))
        .response
        .on_hover_text(t::options_tooltip())
        .rect;
    crate::diag::ui_rect(REGION_OPTIONS, rect);

    if options == before {
        return;
    }
    // Asked BEFORE the new options are stored: afterwards the answer is
    // `false` by construction, because the stored results were computed under
    // the old options and would no longer match.
    let was_showing_an_answer = state.answered();
    state.set_options(options);
    if was_showing_an_answer && !state.query().is_empty() {
        actions.push(Action::Find(FindRequest::Search));
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-options case={} whole={} wildcards={} boundary={:?} research={was_showing_an_answer}",
            options.case_sensitive, options.whole_word, options.wildcards, options.word_boundary,
        )
    });
}

/// The contents of the `Options` menu.
///
/// A free function taking `&mut FindOptions` so the whole menu is testable
/// without a popup: what matters about it is which controls appear under which
/// conditions, and that is a property of the options value.
///
/// ★ **The word-rule chooser exists only while it means something.**
/// P3: an unavailable capability renders nothing, and greying is for
/// *temporarily* unavailable with an explanation. A rule chooser beside an
/// unticked *Whole word* is neither — it is a control that would change a value
/// nothing reads, which is worse than a greyed one because it looks like it
/// works. So it appears with the option and disappears with it, and
/// [`crate::text::find::whole_word_tooltip`] warns the operator that it will.
fn options_menu(ui: &mut egui::Ui, options: &mut FindOptions) {
    ui.checkbox(&mut options.case_sensitive, t::match_case())
        .on_hover_text(t::match_case_tooltip());
    ui.checkbox(&mut options.whole_word, t::whole_word())
        .on_hover_text(t::whole_word_tooltip());

    if options.whole_word {
        ui.separator();
        ui.label(t::word_rule())
            .on_hover_text(t::word_rule_tooltip());
        for rule in FindOptions::WORD_RULES {
            ui.radio_value(&mut options.word_boundary, *rule, word_rule_label(*rule))
                .on_hover_text(word_rule_tooltip(*rule));
        }
        ui.separator();
    }

    ui.checkbox(&mut options.wildcards, t::wildcards())
        .on_hover_text(t::wildcards_tooltip());
}

/// The label for one whole-word rule.
///
/// A free function rather than a method on [`WordBoundary`] because that type
/// belongs to `pdfcer-core` and its operator-facing wording belongs to this
/// crate's catalog. `pub(crate)` so [`super`]'s test can assert that every rule
/// the chooser offers has one.
#[must_use]
pub(crate) fn word_rule_label(rule: WordBoundary) -> &'static str {
    match rule {
        WordBoundary::NonSpace => t::word_rule_non_space(),
        WordBoundary::NonSpaceOrDash => t::word_rule_non_space_or_dash(),
        // `Alphanumeric`, plus any variant a future `pdfcer-core` adds:
        // `WordBoundary` is `#[non_exhaustive]`, so a wildcard arm is required
        // and naming `Alphanumeric` beside it would be an unreachable pattern.
        // It falls back to the DEFAULT's label rather than to a blank, because
        // a new variant arriving from a core upgrade must not produce an empty
        // row in a menu. `super::tests::every_word_rule_the_chooser_offers_has_a_label`
        // is what keeps `FindOptions::WORD_RULES` honest.
        _ => t::word_rule_alphanumeric(),
    }
}

/// The hover text for one whole-word rule. See [`word_rule_label`].
#[must_use]
fn word_rule_tooltip(rule: WordBoundary) -> &'static str {
    match rule {
        WordBoundary::NonSpace => t::word_rule_non_space_tooltip(),
        WordBoundary::NonSpaceOrDash => t::word_rule_non_space_or_dash_tooltip(),
        _ => t::word_rule_alphanumeric_tooltip(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, RawInput};

    // =======================================================================
    // ★ The OCR offer
    // =======================================================================

    /// ★ **A page with no text at all, after a search that found nothing.**
    ///
    /// The one combination that offers OCR, and the operator's actual rule
    /// stated as a case: there is nothing on this page for any search to have
    /// matched, so the empty result is a fact about the *document*.
    #[test]
    fn a_page_with_no_text_at_all_offers_recognition() {
        assert!(offer_ocr(Readout::Empty, || false));
    }

    /// ★★ **THE FALSIFYING CASE.** An ordinary empty result offers nothing.
    ///
    /// This is the assertion the whole feature turns on, and it is the one a
    /// plausible wrong implementation fails. Offering OCR on any zero-hit
    /// search is one character simpler to write, passes
    /// [`Self::a_page_with_no_text_at_all_offers_recognition`] perfectly, and
    /// would put *"this page has no text on it"* under every mistyped part
    /// number on a drawing full of text.
    ///
    /// The operator named this trap in the specification rather than leaving it
    /// to be discovered: *"the trigger is 'this document is images', NOT 'this
    /// search had no matches'"*, and `FEATURES.md` records that the two "must
    /// not be collapsed."
    #[test]
    fn an_ordinary_empty_result_on_a_text_page_offers_nothing() {
        assert!(
            !offer_ocr(Readout::Empty, || true),
            "a search for a word that is simply not on a page full of text is an ordinary \
             empty result; offering to recognise it would be nonsense"
        );
    }

    /// ★ **The page is not even asked about unless the search came back empty.**
    ///
    /// The short-circuit, asserted rather than assumed — and it is a
    /// correctness property, not an optimisation. `page_has_extractable_text`
    /// costs one page extraction on a cache miss, and the bar draws on every
    /// frame it is open; a version that evaluated the closure first would put
    /// that extraction on the frame budget while looking identical in every
    /// other test here. That is `HANDOFF.md` §2's defect 9 exactly: the right
    /// work, charged at the wrong moment, invisible to a suite that only asks
    /// whether it happened.
    #[test]
    fn the_page_is_not_extracted_unless_the_search_found_nothing() {
        for readout in [
            Readout::Idle,
            Readout::Stale,
            Readout::At {
                current: 1,
                total: 4,
            },
        ] {
            let mut asked = false;
            let offer = offer_ocr(readout, || {
                asked = true;
                false
            });
            assert!(!offer, "{readout:?} must not offer recognition");
            assert!(
                !asked,
                "{readout:?} asked the page for its text; that is a page extraction charged to \
                 a frame the operator did not ask anything on"
            );
        }
    }

    /// The offer raises the command the ribbon registers, not a spelling of it.
    ///
    /// An id written at a call site and nowhere else goes stale in silence —
    /// `HANDOFF.md` §5's whole subject — and the symptom here would be a button
    /// that traces `command-unimplemented` and does nothing.
    #[test]
    fn the_offer_raises_the_registered_recognise_command() {
        let mut registry = egui_shell::commands::CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        assert!(
            registry.get(OCR_COMMAND).is_some(),
            "`{OCR_COMMAND}` is not registered, so the offer's button would reach the \
             dispatcher's fall-through arm and do nothing"
        );
    }

    // =======================================================================
    // ★ Placement
    // =======================================================================

    /// ★ **The box is pinned inside the canvas viewport's top-right corner**,
    /// not the window's.
    ///
    /// The distinction is invisible until a dock is open, and then it is the
    /// whole difference between a find bar over the page and a find bar over
    /// the Objects panel. Asserted against a host rect deliberately offset
    /// from the origin, so an implementation that forgot `host.right()` or
    /// `host.top()` and used the screen's still fails.
    #[test]
    fn the_overlay_is_pinned_inside_the_hosts_top_right_corner() {
        let host = Rect::from_min_max(Pos2::new(200.0, 90.0), Pos2::new(1000.0, 700.0));
        let at = anchor_right_top(host);

        assert!(host.contains(at), "the pivot must be on the canvas: {at:?}");
        assert!(at.y > host.top(), "it must not sit on the canvas's edge");
        assert!(at.y < host.top() + 40.0, "…nor halfway down the page");
        assert!(at.x < host.right(), "…nor flush against the right edge");
        assert!(
            at.x - BAR_WIDTH_PTS > host.left(),
            "an ordinary canvas must be wide enough for the whole box, or the \
             constraint rather than this function is deciding the layout"
        );
    }

    /// A host far narrower than the box still yields a pivot on the canvas.
    ///
    /// Reachable: the canvas viewport shrinks with every dock the operator
    /// opens, and `MIN_WINDOW_SIZE` is 640 points wide before any of them.
    /// egui's `constrain_to` is what pulls the *box* back in that case; what
    /// is asserted here is that it is not being handed a nonsense point to
    /// start from.
    #[test]
    fn a_narrow_host_still_yields_a_pivot_on_the_canvas() {
        for narrow in [
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(200.0, 400.0)),
            // Degenerate: a window dragged to nothing, which egui reports
            // before it clamps.
            Rect::from_min_max(Pos2::new(50.0, 50.0), Pos2::new(54.0, 54.0)),
        ] {
            let at = anchor_right_top(narrow);
            assert!(
                narrow.contains(at),
                "{at:?} is outside {narrow:?}; the pivot must never leave the canvas"
            );
        }
    }

    // =======================================================================
    // ★ What Enter means
    // =======================================================================

    /// ★ **Enter searches when the answer is not current and steps when it
    /// is.**
    ///
    /// The whole of the bar's key behaviour, asserted without a frame. The
    /// interesting rows are `Stale` — which must search rather than step,
    /// because stepping through geometry the module has already declared
    /// untrustworthy is exactly what the staleness rule exists to prevent —
    /// and `Empty`, which must do nothing.
    #[test]
    fn enter_searches_when_there_is_no_current_answer_and_steps_when_there_is() {
        assert_eq!(
            enter_intent(Readout::Idle, false),
            Some(FindRequest::Search)
        );
        assert_eq!(enter_intent(Readout::Idle, true), Some(FindRequest::Search));
        assert_eq!(
            enter_intent(Readout::Stale, false),
            Some(FindRequest::Search),
            "a stale bar must search again, not step through hits it has disowned"
        );
        assert_eq!(
            enter_intent(
                Readout::At {
                    current: 1,
                    total: 9
                },
                false
            ),
            Some(FindRequest::Step(Step::Next))
        );
        assert_eq!(
            enter_intent(
                Readout::At {
                    current: 1,
                    total: 9
                },
                true
            ),
            Some(FindRequest::Step(Step::Previous)),
            "Shift+Enter goes backwards"
        );
    }

    /// ★ **Enter on a fruitless search does nothing at all.**
    ///
    /// A search is a whole-document text extraction — 350 ms on the benchmark
    /// drawing, measured. Re-running one that just matched nothing, once per
    /// keypress, is how a viewer becomes unusable on the files it exists for
    /// — and it would produce the same answer, because if anything had changed
    /// the readout would be `Stale`.
    #[test]
    fn enter_does_not_re_run_a_search_that_found_nothing() {
        assert_eq!(enter_intent(Readout::Empty, false), None);
        assert_eq!(enter_intent(Readout::Empty, true), None);
    }

    // =======================================================================
    // The readout's four sentences
    // =======================================================================

    /// The four states produce four different readouts, and only one of them
    /// is blank.
    #[test]
    fn the_four_readouts_are_four_different_sentences() {
        let idle = readout_text(Readout::Idle).0;
        let empty = readout_text(Readout::Empty).0;
        let stale = readout_text(Readout::Stale).0;
        let at = readout_text(Readout::At {
            current: 3,
            total: 47,
        })
        .0;

        assert!(idle.is_empty(), "nothing asked, nothing answered");
        assert!(!empty.is_empty() && !stale.is_empty() && !at.is_empty());
        assert_ne!(empty, stale, "`no matches` and `stale` are different facts");
        assert_ne!(empty, at);
        assert_eq!(at, "3 of 47");
    }

    /// Every readout that says something also explains itself on hover.
    ///
    /// The blank one does not, and must not: a hover target with no text is
    /// worse than none.
    #[test]
    fn every_readout_that_says_something_explains_itself() {
        for readout in [
            Readout::Empty,
            Readout::Stale,
            Readout::At {
                current: 1,
                total: 1,
            },
        ] {
            let (text, hover) = readout_text(readout);
            assert!(!text.is_empty());
            assert!(
                !hover.is_empty(),
                "{readout:?} says something and explains nothing"
            );
        }
        assert!(readout_text(Readout::Idle).1.is_empty());
    }

    // =======================================================================
    // The options menu
    // =======================================================================

    /// ★ **The word-rule chooser appears with Whole word and not before.**
    ///
    /// P3, applied to the one control on this surface whose availability is
    /// conditional. Driven through a real `Ui` so what is asserted is what the
    /// menu actually builds, and counted by *widgets that were laid out*
    /// rather than by reading the branch — a test that read the branch would
    /// pass on a build where the `if` had been inverted and the label moved.
    #[test]
    fn the_word_rule_chooser_appears_only_with_whole_word() {
        let ctx = Context::default();
        let widgets = |whole_word: bool| {
            let mut options = FindOptions {
                whole_word,
                ..FindOptions::default()
            };
            let mut height = 0.0_f32;
            let _ = ctx.run_ui(RawInput::default(), |ui| {
                height = ui
                    .scope(|ui| options_menu(ui, &mut options))
                    .response
                    .rect
                    .height();
            });
            height
        };
        let plain = widgets(false);
        let with_rule = widgets(true);
        assert!(
            with_rule > plain + 20.0,
            "switching Whole word on must add the rule chooser to the menu \
             ({plain} pt -> {with_rule} pt)"
        );
    }

    // =======================================================================
    // Legibility — the labels that are glyphs
    // =======================================================================

    /// ★ **Every glyph the bar draws exists in the bundled font set.**
    ///
    /// `⏴`, `⏵` and `×` are the entire visible text of three controls. A
    /// codepoint egui's bundled fonts (Ubuntu-Light + NotoEmoji +
    /// emoji-icon-font) cannot draw renders as a tofu box, which is defect
    /// D2's shape — an invisible label — on a control the operator has to hit.
    ///
    /// The status bar has the identical test and it has already paid for
    /// itself once: that catalog was written with `◀ ▶ ▸ ▾`, **all four of
    /// which are missing**, and they would have shipped as tofu on the two
    /// controls an operator touches most. This test is not a duplicate of it —
    /// it cannot see this file's strings, and this file cannot see the status
    /// bar's.
    #[test]
    fn every_glyph_the_find_bar_draws_has_a_glyph() {
        let ctx = Context::default();
        let labels: Vec<String> = vec![
            t::field_label().to_owned(),
            t::previous().to_owned(),
            t::next().to_owned(),
            t::close().to_owned(),
            t::options().to_owned(),
            t::position(3, 47),
            t::no_matches().to_owned(),
            t::stale().to_owned(),
            t::match_case().to_owned(),
            t::whole_word().to_owned(),
            t::wildcards().to_owned(),
            t::word_rule().to_owned(),
            t::word_rule_alphanumeric().to_owned(),
            t::word_rule_non_space().to_owned(),
            t::word_rule_non_space_or_dash().to_owned(),
            t::toggle().to_owned(),
        ];

        let mut missing = Vec::new();
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let font = egui::FontId::proportional(14.0);
            ui.ctx().fonts_mut(|f| {
                for label in &labels {
                    for c in label.chars() {
                        if !f.has_glyph(&font, c) {
                            missing.push((label.clone(), c));
                        }
                    }
                }
            });
        });

        assert!(
            missing.is_empty(),
            "these labels contain codepoints the bundled fonts cannot draw, so they would \
             render as tofu boxes: {missing:?}"
        );
    }

    // =======================================================================
    // The bar as a whole
    // =======================================================================

    /// A bar showing the answer to a search for `query` that found `hits`
    /// hits, all on page 0.
    ///
    /// Built by writing `super`'s private fields directly, which a child
    /// module may do. The alternative — a constructor on `FindState` that only
    /// tests call — would be a second way to assemble a result set, and the
    /// currency key is exactly the thing that must have one.
    pub(super) fn searched(query: &str, hits: usize) -> FindState {
        let mut state = FindState::default();
        state.open();
        state.query_mut().push_str(query);
        state.results = Some(crate::find::Results {
            query: query.to_owned(),
            options: FindOptions::default(),
            epoch: 0,
            hits: (0..hits)
                .map(|_| crate::find::Hit {
                    page: 0,
                    canvas: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(10.0, 10.0))),
                    text: query.to_owned(),
                })
                .collect(),
            current: 0,
            unsearchable_fonts: 0,
        });
        state
    }

    /// Run one frame of the row and return the actions it raised.
    fn frame(ctx: &Context, state: &mut FindState, epoch: u64, input: RawInput) -> Vec<Action> {
        let mut actions = Vec::new();
        let _ = ctx.run_ui(input, |ui| body(ui, state, epoch, &mut actions));
        actions
    }

    /// ★ **The box is exactly the same size whatever the readout says.**
    ///
    /// The property the module docs argue for: the overlay is anchored by its
    /// top-right corner, so a box that changed width would move the search
    /// field the operator is typing into, and the ⏴ ⏵ buttons out from under
    /// a pointer aimed between two clicks.
    ///
    /// Four readouts, four very different strings, one size.
    #[test]
    fn the_box_is_the_same_size_whatever_the_readout_says() {
        let ctx = Context::default();
        let size = |state: &mut FindState, epoch: u64| {
            let mut got = Vec2::ZERO;
            let _ = ctx.run_ui(RawInput::default(), |ui| {
                let mut actions = Vec::new();
                got = ui
                    .scope(|ui| body(ui, state, epoch, &mut actions))
                    .response
                    .rect
                    .size();
            });
            got
        };

        let mut idle = FindState::default();
        idle.open();
        let baseline = size(&mut idle, 0);
        assert!(
            (baseline.x - BAR_WIDTH_PTS).abs() < 1.0,
            "the row must occupy exactly its reserved width, got {baseline:?}"
        );

        for (label, mut state) in [
            ("empty", searched("zzz", 0)),
            ("hits", searched("total", 47)),
        ] {
            let got = size(&mut state, 0);
            assert!(
                (got.x - baseline.x).abs() < 0.01 && (got.y - baseline.y).abs() < 0.01,
                "the `{label}` readout resized the box ({baseline:?} -> {got:?}); a \
                 right-anchored box that resizes moves every control on it"
            );
        }

        // …and a stale one, which is the longest string of the three.
        let mut stale = searched("total", 47);
        let got = size(&mut stale, 1);
        assert!(
            (got.x - baseline.x).abs() < 0.01 && (got.y - baseline.y).abs() < 0.01,
            "the stale readout resized the box ({baseline:?} -> {got:?})"
        );
    }

    /// The step buttons raise nothing until there is something to step.
    ///
    /// Driven through a real frame rather than by calling the arm, so what is
    /// under test is the wiring — the failure this catches is a button that
    /// draws, is enabled, and reports nothing, which is the shape three of the
    /// old shell's panels shipped in.
    #[test]
    fn the_step_buttons_are_inert_until_there_is_something_to_step() {
        let ctx = Context::default();
        let mut state = FindState::default();
        state.open();

        assert_eq!(state.readout(0), Readout::Idle);
        let actions = frame(&ctx, &mut state, 0, RawInput::default());
        assert!(
            actions.is_empty(),
            "an untouched bar must raise nothing at all"
        );
    }

    /// ★ **Typing raises nothing.**
    ///
    /// The cost rule, from the other end: a search is a whole-document text
    /// extraction — 350 ms on the benchmark drawing — so a bar that raised one
    /// per keystroke would spend 1.4 seconds of blocked UI thread on the word
    /// `part`. This is what "never searches on a keystroke" means in a test.
    #[test]
    fn typing_raises_no_search() {
        let ctx = Context::default();
        let mut state = FindState::default();
        state.open();
        state.query_mut().push_str("tot");

        let input = RawInput {
            events: vec![egui::Event::Text("a".to_owned())],
            ..Default::default()
        };
        let actions = frame(&ctx, &mut state, 0, input);
        assert!(
            actions.is_empty(),
            "a keystroke must not reach the engine; a search is a whole-document \
             text extraction"
        );
    }

    /// Closing the bar is a widget state change and raises no action.
    ///
    /// Closing touches no document, so it does not go through the funnel — the
    /// same rule that keeps `show_panel` out of the action list.
    #[test]
    fn closing_the_bar_is_not_a_document_action() {
        let mut state = FindState::default();
        state.open();
        state.close();
        assert!(!state.is_open());
    }
}

/// **Say that part of this document could never have matched.**
///
/// # ★★★ Off-canvas, and that is the whole design
///
/// Rule 4 as narrowed by pdfcer's decision 059: an inference the operator cannot
/// see still owes them a report, **and the report does not go on the page.** No
/// badge over the offending run, no tint, no dashed outline, nothing drawn into
/// the page view at all. Applied content renders exactly as saved content
/// renders; the disclosure lives in a status line, a results panel, or — here —
/// the bar's own second row, which already exists for the OCR offer.
///
/// That is not fastidiousness. A provisional styling layer is a **second
/// rendering path for the same content**, and two paths drift. The operator's
/// own words on the old shell: *"the nagging and red flagging made for a lot of
/// extra bugs in the visibility when editing."*
///
/// # Why it sits beside the OCR offer rather than replacing it
///
/// They answer different questions and can be true at once. The OCR offer fires
/// when **this page** has no extractable text at all — a scan. This fires when
/// the **document** contains a font whose text is unreachable, which is
/// perfectly compatible with the current page being ordinary searchable text.
/// A file with a Type 3 titleblock on page 1 and normal text everywhere else
/// produces this note and no OCR offer, which is exactly right.
fn unsearchable_note(ui: &mut egui::Ui, fonts: u64) {
    ui.allocate_ui_with_layout(
        Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_size(Vec2::new(BAR_WIDTH_PTS, ROW_HEIGHT_PTS));
            // Muted, for the reason `ocr_offer` gives: a statement about the
            // document, not a control. Not `.strong()` — `DEFECTS.md` D11
            // records that role as unusable in this theme.
            let theme = egui_shell::theme::Theme::of(ui.ctx());
            let text = if fonts == 1 {
                crate::text::find::unsearchable_one().to_owned()
            } else {
                crate::text::find::unsearchable_many(fonts)
            };
            ui.add(
                egui::Label::new(egui::RichText::new(text).color(theme.palette.text_muted))
                    .truncate(),
            )
            .on_hover_text(crate::text::find::unsearchable_tooltip());
        },
    );
}

/// ★★★ The unsearchable note answers a DIFFERENT question from the OCR offer,
/// and both can be true at once.
///
/// Written when the note landed, because the two are drawn in the same row and
/// the obvious mistake is to make one an `else` of the other. They are not
/// alternatives:
///
/// | | OCR offer | unsearchable note |
/// |---|---|---|
/// | scope | **this page** | the **whole document** |
/// | fires when | the page has no extractable text at all — a scan | some font's text is unreachable, anywhere |
///
/// A drawing with a Type 3 titleblock on page 1 and ordinary text everywhere
/// else produces the note and **no** OCR offer, and that is correct: the page
/// in front of the operator is searchable, and the document still contains
/// something no search will ever reach.
#[cfg(test)]
mod unsearchable_tests {
    use super::tests::searched;
    use super::*;

    /// **Nothing is said when nothing is wrong.** The guard that keeps this
    /// from becoming the nagging the operator objected to in the old shell.
    #[test]
    fn a_document_with_no_unreachable_fonts_says_nothing() {
        let state = crate::find::FindState::default();
        assert_eq!(
            state.unsearchable_fonts(0),
            0,
            "a bar that has run no search has nothing to disclose"
        );
    }

    /// **The two disclosures are independent**, asserted as a truth table so
    /// that anyone rewriting the row as an if/else has to delete a case.
    #[test]
    fn the_ocr_offer_and_the_note_are_not_alternatives() {
        // OCR offer depends only on the readout and whether the PAGE has text.
        assert!(
            offer_ocr(Readout::Empty, || false),
            "an empty result on a page with no text is the OCR case"
        );
        assert!(
            !offer_ocr(Readout::Empty, || true),
            "an empty result on a page WITH text is not the OCR case — and it is exactly the case the unsearchable note exists for"
        );
        // …and the note depends on neither of those, only on the document's
        // font diagnostics. The combination in the second assertion above is
        // the one a file with a Type 3 titleblock produces.
    }

    /// ★★ **A sentence about one search cannot outlive that search.**
    ///
    /// Three ways a result stops describing what the bar is showing, and all
    /// three must silence the note: the operator edits the query, the operator
    /// changes an option, or the DOCUMENT is edited (a new epoch). The last is
    /// the one worth having a test for — an edit does not touch the query, so a
    /// naive implementation keeps a stale sentence on screen indefinitely while
    /// the bar beside it has already gone blank.
    #[test]
    fn a_result_that_no_longer_describes_the_bar_discloses_nothing() {
        let mut state = searched("alpha", 0);
        if let Some(r) = state.results.as_mut() {
            r.unsearchable_fonts = 2;
        }
        assert_eq!(
            state.unsearchable_fonts(0),
            2,
            "the current search DOES have something to say — checked first, so the assertions below cannot pass by the accessor simply always returning zero"
        );

        // The document was edited: same query, new epoch.
        assert_eq!(
            state.unsearchable_fonts(1),
            0,
            "an edited document invalidates the hit list, and it invalidates the sentence beside it for the same reason"
        );

        // The operator typed on.
        state.query_mut().push('b');
        assert_eq!(state.unsearchable_fonts(0), 0);
    }
}
