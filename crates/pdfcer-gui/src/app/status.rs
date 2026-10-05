//! # `app::status` — the status bar: the narrator on the left, the constant controls on the right
//!
//! `RIBBON_IA.md` §6 specifies this surface in one paragraph, and the
//! paragraph contains the whole design:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status.md`.

/// **What the file contradicted itself about** — the one reading of
/// `Document::load_anomalies()` that both the bar's census line and the
/// Document-properties list are drawn from. See its header for why the
/// derivation is shared and why it carries no `edit_epoch` key.
pub use pdfcer_gui_base::anomalycensus as anomalies;
#[cfg(test)]
mod anomalies_tests;
mod disclosure;
// The four named zoom levels -- Actual size, Fit width, Fit height, Fit page.
// See its header for the layout rule.
mod fit;
/// **What the bar can afford when the window is narrow** — the shed rule, and
/// the reachability clause that makes shedding legitimate.
pub use pdfcer_gui_base::statusfitting as fitting;
#[cfg(test)]
mod fitting_tests;
/// **Where the OCR blend sits, and whether the page has anything to blend.**
mod ocrlayer;
/// Page navigation and the editable page-number box. See this module's
/// header for the seam, and that one's for the control.
mod page_box;
/// **How to get the application back** — the read-mode exit statement.
mod readmode;
/// **What is selected, said in words** — the readout that turns *"all I get is
/// the page selected"* into a diagnosis. See its header.
mod selected;

/// The worded decline — a command that was invoked and did not run.
pub(super) mod decline;

/// The narrator — the render-diagnostics disclosure and its one line.
pub(crate) mod notes;

use egui::{Align, Layout, Vec2};

use crate::app::actions::Action;
use crate::app::state::Status;
use crate::canvas::pick::PickFilter;
use crate::find::FindState;
use crate::text::find as t_find;

/// The **Select** popup — what a click on the page may land on (O17).
pub(super) mod filter;
/// The **maximum-zoom** popup, behind the zoom readout — O24, and the
/// operator's *"put the max zoom setting on the bar at the bottom"*.
pub(super) mod maxzoom;
/// **Why zooming in stopped** — O186's fourth clause, the one sentence that
/// keeps the learned raster ceiling from being a control that silently stops
/// responding.
pub(super) mod rasterstop;
/// The zoom controls and the maximum-zoom popup the readout opens.
///
/// Split out under R2 when the popup pushed this file over 1,500 lines;
/// its header carries why the seam is a real one rather than arbitrary.
pub(super) mod zoom;

// ---------------------------------------------------------------------------
// Geometry — see the R128 section of the module docs
// ---------------------------------------------------------------------------

/// The exact outer height, in egui points, the status panel must be given.
pub const HEIGHT_PTS: f32 = 30.0;

/// **The panel's height for a given theme** — use this, not [`HEIGHT_PTS`].
///
/// # Why the constant is not enough on its own
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
/// ui_scale 1.00, 1600x1000 client:  status-bar  972.0 .. 1002.0   2 pt past the bottom
/// ui_scale 1.80, 1100x800  client:  status-bar  416.4 ..  446.4   2 pt past 444.4
/// ```
///
/// Size the panel from the constant and the bottom two points of two controls
/// fall off the window, at every UI scale.
///
/// ## Why no unit test sees it, which is the transferable part
///
/// [`tests::the_bar_is_exactly_as_tall_open_as_closed`] asserts exactly this
/// property — *"the bar's content overflowed its allocated row"* — and it
/// **passes either way**. It builds an `egui::Context::default()`, which
/// carries egui's own default spacing and **not this application's theme**. In
/// that context the controls really are under 24 points and the assertion is
/// true.
///
/// That is R1's founding shape verbatim: *the only test of that function builds
/// a bare `egui::Context`, so the condition that breaks the real app cannot
/// occur in the harness.* A bar measured in a world with no theme in it is not
/// measured.
///
/// # Why a function and not a bigger constant
///
/// Because the number is a property of the **theme**, and this project ships
/// more than one preset. A constant large enough for the tallest preset would
/// waste canvas on the others and would silently become wrong again the next
/// time a preset raised its control height. Taking it from `Metrics` means the
/// two move together by construction.
///
/// **R128 is untouched.** The rule is that the bar's height must not depend
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
pub const ROW_HEIGHT_PTS: f32 = 24.0;

/// The panel must be taller than the row it contains, or the bar's own
/// content is clipped by the frame that is supposed to hold it.
const _: () = assert!(
    HEIGHT_PTS > ROW_HEIGHT_PTS,
    // ui-text-exempt: compile-error text, never displayed in the UI
    "the panel height must leave room for its own inner margin"
);

/// The **floor** under the zoom readout's reserve, and only the floor.
const ZOOM_READOUT_WIDTH_PTS: f32 = 46.0;

/// The share of the bar the render-notes line may occupy before eliding.
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
pub(super) const REGION_FILL_DISCLOSURE: &str = "status-group:fill-disclosure"; // ui-text-exempt: trace region name, never displayed

/// The last vector edit's rule-4 disclosure, when one is live for this
/// revision.
pub(super) const REGION_EDIT_DISCLOSURE: &str = "status-group:edit-disclosure"; // ui-text-exempt: trace region name, never displayed

/// See [`recovered_disclosure`].
pub(super) const REGION_RECOVERED: &str = "status-group:recovered"; // ui-text-exempt: trace region name, never displayed
/// The *"this file contradicted itself and pdfcer decided"* line.
pub(super) const REGION_LOAD_ANOMALIES: &str = "status-group:load-anomalies"; // ui-text-exempt: trace region name, never displayed
/// The blend-space disclosure's rect, for `ui-verify`.
///
/// A published region name is a cross-repo stability contract with the
/// harness: renaming it turns a check into a skip rather than a failure.
pub(super) const REGION_BLEND_SPACE: &str = "status-group:blend-space"; // ui-text-exempt: trace region name, never displayed
/// The "some spot inks are drawn as process colour" line.
pub(super) const REGION_SPOTS_FLATTENED: &str = "status-group:spots-flattened"; // ui-text-exempt: trace region name, never displayed
/// The "the picture is still being drawn" line (`OPERATOR_REQUESTS.md` O63).
pub(super) const REGION_CATCHING_UP: &str = "status-group:catching-up"; // ui-text-exempt: trace region name, never displayed
/// The "your typing is shown in a stand-in font" line, while a draft is open.
pub(super) const REGION_PREVIEW_FALLBACK: &str = "status-group:preview-fallback"; // ui-text-exempt: trace region name, never displayed
/// The "the pasted lines were joined" / "Tab was typed as spaces" line.
pub(super) const REGION_DRAFT_NOTE: &str = "status-group:draft-note"; // ui-text-exempt: trace region name, never displayed
/// The "line weights are off, so this is not what will print" line —
/// `OPERATOR_REQUESTS.md` **O137**.
pub(super) const REGION_LINE_WEIGHTS: &str = "status-group:line-weights"; // ui-text-exempt: trace region name, never displayed
/// The "tiny details are being skipped" line, while `view.skip_tiny_details` is on.
pub(super) const REGION_TINY_DETAILS: &str = "status-group:tiny-details"; // ui-text-exempt: trace region name, never displayed

/// `Actual size · Fit width · Fit page`.
const REGION_FIT: &str = "status-group:fit"; // ui-text-exempt: trace region name, never displayed

/// `−  ⟨percent⟩  +`.
const REGION_ZOOM: &str = "status-group:zoom"; // ui-text-exempt: trace region name, never displayed

/// Prefix for one row of the open maximum-zoom popup:
/// `status-maxzoom-row:<index>`.
const REGION_MAXZOOM_ROW: &str = "status-maxzoom-row"; // ui-text-exempt: trace region name, never displayed

/// The Find toggle.
const REGION_FIND: &str = "status-group:find"; // ui-text-exempt: trace region name, never displayed

/// The selection-filter button — the CLOSED control, not its popup.
const REGION_FILTER: &str = "status-group:filter"; // ui-text-exempt: trace region name, never displayed

/// The standing line shown when the filter has left nothing selectable.
const REGION_FILTER_EMPTY: &str = "status-group:filter-empty"; // ui-text-exempt: trace region name, never displayed

/// Prefix for one row of the open filter popup: `status-filter-row:<index>`.
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
#[allow(clippy::too_many_arguments)] // each is a separate owner's state the bar edits
pub fn show(
    ui: &mut egui::Ui,
    status: &Status,
    // The registry, so a decline offers a remedy only this build registers.
    commands: &egui_shell::CommandRegistry,
    find: &mut FindState,
    filter: &mut PickFilter,
    // The operator's configured maximum zoom, edited by the popup behind
    // the zoom readout. Threaded like `filter`, and persisted by the caller
    // for the same reason — see `app::frame`'s status-bar block.
    max_zoom_percent: &mut f32,
    wheel_paging: &mut crate::app::prefs::WheelPaging,
    remote: &mut crate::app::remote::Link,
    actions: &mut Vec<Action>,
) {
    // One allocated row, of a height that does not depend on what there is
    // to show. R128; see the module docs for the measurement.
    let row = Vec2::new(ui.available_width(), ROW_HEIGHT_PTS);
    let bar = ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Center), |ui| {
        // Claim the whole row even when nothing is drawn into it.
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

        // **FIRST of everything, and BEFORE the no-document guard: how to
        // get the application back.**
        //
        // Read mode hides the ribbon and the docks, and the only control that
        // turns it off lives on the ribbon — so from the moment it is on, this
        // bar is the only piece of chrome left that can say how to leave. The
        // operator reported exactly that (O115).
        //
        // Ahead of the page-drag caption, which the block below says outranks
        // the disclosures, which outrank the narrator. The rule that puts this
        // above all three generalises: **a sentence about how to reach the
        // interface outranks every sentence about the document**, because an
        // operator who cannot reach the interface cannot act on the others.
        //
        // Above the `Status::Open` guard, deliberately. Read mode is per
        // WINDOW rather than per document (`app::window` §3), so the last file
        // can be closed while it is on — and a bar that explained the way out
        // only when a document happened to be open would go silent in the state
        // where the window has the least in it.
        readmode::show(ui);
        remote.status_item(ui);

        // With nothing open there is no page to number, no zoom to report
        // and no raster to have notes about. The bar still occupies its
        // height — that is the whole point of pinning it — but it draws no
        // control, because a control that cannot work is the placeholder the
        // project's invariants forbid.
        let Status::Open(doc) = status else {
            return;
        };

        // **First on the left while a page drag is in flight**, ahead of
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
        // It also has to be here rather than only in the Pages panel,
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
        if let Some(caption) =
            crate::pagedrag::phase(ui.ctx()).map(|p| crate::text::doctabs::drag_caption(&p))
        {
            ui.label(caption);
            ui.separator();
        }

        // …and the same treatment for a CANVAS drag being constrained.
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

        // FIRST on the left: what is selected.
        //
        // Ahead of the narrator because it is the answer to a question the
        // operator is actively asking — *what did I just click?* — where the
        // render notes are something pdfcer volunteers. When the bar runs out of
        // room the left is what yields, and within the left the volunteered
        // line should yield before the asked-for one.
        // The left half is capped at what the never-shed groups leave: past
        // it, a line or a decline's button is drawn under the zoom group and a
        // click on it lands on the zoom.
        let budget =
            (ui.available_width() - fitting::floor_width(&fitting_widths(ui.ctx()))).max(0.0);
        ui.scope(|ui| {
            ui.set_max_width(budget);
            selected::show(ui, doc);

            // Left: the narrator, demoted behind a disclosure.
            notes::show(ui, doc);

            // …and beside it, what the last fill INFERRED — which is not
            // demoted, because it is not narration.
            //
            // Rule 4's surviving half: an inference the operator **cannot see**
            // still owes an off-canvas report. `applied_autosize` (pdfcer chose
            // the point size) and `unencodable_chars` (characters replaced with
            // `?`) are the only two facts a fill produces that are **not
            // re-derivable from the saved document** — afterwards they look
            // exactly like the author's own decision.
            //
            // The Forms panel shows them too, and that is not enough now that
            // filling also happens on the canvas: a fill can happen in **Read
            // mode with the panel closed**, and the disclosure would then be
            // reachable only by an operator who thought to switch modes and open
            // a panel to look for a message they were never told existed. That is
            // a silent inference, which is the one thing rule 4 forbids
            // outright.
            //
            // It is keyed on `edit_epoch`, so it says nothing about a document
            // that has moved on — an undo or any later edit retires it without
            // anything having to remember to.

            // …and the same obligation for the verbs that move geometry.
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
            // `pdfcer-core` returns these sentences and
            // `crate::app::actions::vector_edit` traces them. Tracing is
            // recording, not disclosing — that function's own header says so —
            // and this is where they are disclosed.
            //
            // Keyed on `edit_epoch` exactly as its neighbour is, and for the same
            // reason: an undo or any later edit retires the sentence without
            // anything having to remember to. The two can never both be live —
            // one edit bumps the epoch once and records at most one of them.
            disclosure::all(ui, doc);

            // …and the opposite speech act, in the same place.
            //
            // The three lines above all say *something happened*. This one says
            // *nothing happened*: a command was invoked and declined, because
            // there was nothing for it to act on. Today that is zoom-to-selection
            // with no resolvable bounds and no canvas — `canvas::zoom` returns
            // those outcomes and traces them, and this is where the dispatcher
            // turns one into a sentence instead of dropping it.
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
            // Before the decline note, because it outranks it: a decline
            // explains why one gesture did nothing, while this explains why
            // EVERY gesture will. An operator reading the bar because the
            // canvas stopped responding needs the general answer first.
            filter::empty_note(ui, *filter);

            // …and the same species of fact about one DIRECTION — O186.
            //
            // *"Why will every zoom-in gesture do nothing?"*, which is the empty
            // filter's question narrowed to one axis, and it belongs here for the
            // same reason: between the general answer above and the
            // single-gesture answer below. An operator reading this bar because a
            // control stopped responding wants them in that order.
            //
            // It is deliberately NOT part of `decline::show`. The clamped region
            // zoom that module declines to word is a partial grant the zoom readout
            // already explains; this one the readout cannot explain, because the
            // number it shows did not move. That argument is in `rasterstop`'s
            // header, and beside `decline`'s own ruling, because those are the two
            // places the next reader will look.
            rasterstop::show(ui, doc);

            // …and the same species of fact about one MODE.
            //
            // Below `rasterstop` because that one explains why a gesture the
            // operator just made did nothing, and this one explains why a mode he
            // turned on some time ago is showing nothing — the first is the
            // answer to a question he is asking right now, the second to one he
            // may not have thought to ask yet.
            ocrlayer::show(ui, doc);

            decline::show(ui, doc, commands, actions);
        });

        // Right: the controls that must never move.
        //
        // Laid out RIGHT-TO-LEFT, so the group added FIRST is drawn
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
        // **WHAT THE BAR CAN AFFORD** — the paragraph above is about *which
        // layout*, and this is about *how much*, which no choice of layout
        // settles.
        //
        // At `ui_scale = 1.80` in an 1100 x 800 window — 611 points wide — the
        // fixed cluster needs 666 points. A right-to-left layout does not clip
        // to its parent; it runs past the left edge into negative coordinates,
        // which puts Find at x = -54 and the selection filter at x = -127,
        // both unreachable, with the left-hand notes drawn underneath the fit
        // group.
        //
        // `fitting` decides, and its header carries the whole argument —
        // including the clause that makes shedding legitimate at all: nothing
        // it may drop is the operator's last route to that capability, checked
        // against the real command registry rather than asserted.
        //
        // The widths come from **last frame's measured rects**, remembered in
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
        // Only the groups actually drawn are re-measured; a shed group keeps
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
                // `wheel=` carries the O30 preference, because a driven
                // check that TOGGLES a persisted setting has to be able to
                // normalise it first. Without it the second run of such a
                // check inherits the first run's choice and reports the
                // default as broken. A setting a check can change is a
                // setting the trace must state.
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
fn fitting_id() -> egui::Id {
    egui::Id::new("status.fitting.widths") // ui-text-exempt: a memory key, never displayed
}

/// What each group of the fixed cluster occupied last frame.
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
#[cfg(test)]
pub(super) mod test_support {
    use super::{Action, PickFilter, Status, show};
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use egui::{Context, Event, Key, Modifiers, RawInput};

    /// An application status with the four-page fixture open.
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
                &egui_shell::CommandRegistry::new(),
                &mut find,
                &mut filter,
                &mut max_zoom,
                &mut crate::app::prefs::WheelPaging::default(),
                &mut crate::app::remote::Link::default(),
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
                        &egui_shell::CommandRegistry::new(),
                        &mut find,
                        &mut filter,
                        &mut max_zoom,
                        &mut crate::app::prefs::WheelPaging::default(),
                        &mut crate::app::remote::Link::default(),
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
