//! # `canvas::trace` — what the canvas says on the `PDFCER_DIAG` channel
//!
//! Three lines, one subject: *where is the canvas, what is on it, and what did
//! the operator just do to the selection?*
//!
//! ## Why this is a module rather than three functions at the bottom of [`super`]
//!
//! R2's 1,500-line ceiling forces a seam somewhere, and this is a real one.
//! Every function here exists to serve a **consumer outside the process**:
//! `tools/ui-verify`, which drives the binary and reads its stderr. That gives
//! them a property nothing else in `canvas/` has: **their output shape is a
//! contract.** A field renamed here breaks a harness that does not compile
//! against this crate and will therefore not fail to build — it will fail to
//! find what it is looking for, at run time, in a check whose subject is
//! something else entirely.
//!
//! Keeping them together is what makes that contract reviewable in one place.
//! `PROJECT_PLAN.md` §4.3's three requirements are all discharged by the
//! functions below, and each one's doc comment carries the requirement it
//! answers and the failure mode it guards.
//!
//! ## The de-duplication slots, and why each line has its own
//!
//! [`crate::diag::trace_changed`] emits a line only when it differs from the
//! last one written to the same slot. The slots are separate because the lines
//! answer different questions on different timescales — the pointer moves
//! constantly while the layout does not — and sharing one would make each
//! silence the other.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/trace.md`.

use egui::{Rect, Vec2};

use crate::app::state::OpenDoc;
use crate::canvas::selection::SelectionState;
use crate::viewer;

// ---------------------------------------------------------------------------
// The names the diagnostic channel is keyed on
// ---------------------------------------------------------------------------
//
// These are the *vocabulary* of the contract this module's header describes,
// so they sit with the functions that spend them rather than with the wiring
// that happens to call those functions. `canvas/mod.rs` reaches them as
// `trace::LAYOUT_SLOT`, which reads as what it is: a name this module owns and
// a consumer outside the process is keyed on.

/// The de-duplication slot every canvas-layout line shares.
pub(super) const LAYOUT_SLOT: &str = "canvas"; // ui-text-exempt: trace slot name, never displayed

/// The de-duplication slot for the document-space pointer report.
///
/// Separate from [`LAYOUT_SLOT`]: the pointer moves constantly while the
/// layout does not, and sharing a slot would make each silence the other.
pub(super) const POINTER_SLOT: &str = "canvas-pointer"; // ui-text-exempt: trace slot name, never displayed

/// Named region: the page raster's own rect, in window logical points.
pub(super) const REGION_PAGE: &str = "page"; // ui-text-exempt: trace region name, never displayed

/// Named region: the scrollable viewport the page sits inside.
pub(super) const REGION_CANVAS_VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: trace region name, never displayed

/// Named region: **every page this frame drew, taken together** — the strip.
pub(super) const REGION_STRIP: &str = "canvas-strip"; // ui-text-exempt: trace region name, never displayed

/// Named region: the one-sentence message shown instead of a page.
pub(super) const REGION_PAGE_MESSAGE: &str = "canvas-message"; // ui-text-exempt: trace region name, never displayed

/// Trace slot for what the selection layer did — a click, a marquee, an
/// Escape, a Delete.
pub(super) const SELECTION_SLOT: &str = "canvas-selection"; // ui-text-exempt: trace slot name, never displayed

/// Trace slot for what the **text** selection did — a sweep, a word, a line, a
/// select-all, a clear.
pub(super) const TEXT_SELECTION_SLOT: &str = "canvas-text-selection"; // ui-text-exempt: trace slot name, never displayed

/// **Report what the text selection just became.**
///
/// # Why this line has to exist at all
///
/// **Dense linework and a flat wash are the same picture to a pixel oracle**,
/// which is the sharpest lesson this project has, and the trap is here in a
/// purer form. A text selection is a **translucent wash over glyphs**: a
/// screenshot of a page carrying a three-word selection and a screenshot of the
/// same page carrying none are very nearly the same image — at the low alpha
/// `overlay`'s `TEXT_SELECTION_ALPHA` is deliberately set to, over the linework
/// of a CAD sheet, they may be indistinguishable.
///
/// So the application says what it selected, in characters, and a harness can
/// prove the gesture happened rather than inferring it from a wash. `chars=` is
/// the number an assertion should be made on: it is `> 0` if and only if the
/// sweep found glyphs, and it is the length of the string a copy would put on
/// the clipboard — the same value, from the same field, so a trace and a
/// clipboard cannot disagree about what was selected.
///
/// # The fields
///
/// ```text
/// pdfcer-diag canvas-text-selection via=drag page=0 chars=27 quads=2
/// ```
///
/// * `via=` — `drag`, `word`, `line`, `extend`, `all` or `clear`. Which gesture
///   produced this, so a check can tell a double-click from a sweep that
///   happened to cover one word.
/// * `page=` — the page the range is on. A selection is single-page
///   (`canvas::textsel` §4), so this is a fact about the whole value.
/// * `chars=` — the byte length of the selected text. **Zero means cleared**,
///   which is a real event with a real cause and is traced rather than being a
///   silence a consumer has to interpret.
/// * `quads=` — how many line boxes the wash is drawn from. `quads=0` with
///   `chars>0` would be a selection that copies text and highlights nothing,
///   which is exactly the divergence the one-derivation rule exists to prevent
///   — so the pair is on the line together and a check can assert on both.
///
/// De-duplicated through [`crate::diag::trace_changed`], like every other line
/// here: a sweep moves the pointer sixty times a second and the intermediate
/// states are noise. **This is the same trap `ui-verify`'s `read_mode` check
/// documents** — a consumer that clicks the same word twice and expects two
/// lines will see one. `via=` is on the line partly for that reason: two
/// gestures producing the same range still differ if they differ in kind.
pub(super) fn text_selection(
    page: usize,
    selection: Option<&crate::canvas::textsel::TextSelection>,
    via: &str,
) {
    crate::diag::trace_changed(TEXT_SELECTION_SLOT, || {
        let (chars, quads) = selection.map_or((0, 0), |s| (s.len(), s.quads.len()));
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // Placed directly above the literal — see `layout`.
            "canvas-text-selection via={via} page={page} chars={chars} quads={quads}"
        )
    });
}

/// Report a selection-changing gesture on the `PDFCER_DIAG` channel.
///
/// De-duplicated on the rendered line, so a marquee dragged across a sheet
/// does not bury the events around it — the lesson `canvas-pointer` taught
/// when a stationary pointer emitted fifty identical lines in nine seconds.
/// The count and the level are on the line because they are what a harness
/// asserts on: *"the click landed"* is `sel=` moving, and *"the ladder
/// descended"* is `level=` moving.
///
/// ```text
/// pdfcer-diag canvas-selection via=click mod=false sel=1 level=Object first=leaf:37
/// ```
///
/// * `via=` — the gesture: `click`, `marquee`, `key`, `escape`, and the rest.
/// * `mod=` — whether the modifier (Shift) was held. Every caller passes
///   the operator's modifier and nothing else: a flag that happens to be a
///   `bool` at the call site — double-click, additive-marquee, a constant —
///   reads as the modifier here and is wrong in both directions for a check
///   asserting that Shift arrived.
/// * `sel=` — how many entries the selection holds.
/// * `level=` — which rung of the ladder: `Object`, `Part` or `Node`.
/// * `first=` — **`object:N`, `leaf:N` or `none`.** Which of the page's two
///   index spaces the first entry names, and its index in that space. See the
///   body for why a count and a rung could not answer the question this was
///   added for.
pub(super) fn selection_event(selection: &SelectionState, kind: &str, modifier: bool) {
    // **`first=` — which of the two index spaces the selection landed in.**
    //
    // `sel=` is a count and `level=` is a rung, and neither can answer the one
    // question form-XObject descent turns on: *did the click select the
    // page-sized form, or the object painted inside it?* Both produce
    // `sel=1 level=Object`, so without this field a driven check cannot tell
    // the defect from the fix — and this project's own stated worst outcome is
    // a check that passes while measuring nothing.
    //
    // Printed as `object:N`, `leaf:N` or `none`. The kind is spelled out
    // rather than implied by a second field, so a human reading a trace after
    // the fact cannot mistake `leaf 7` for `objects[7]` — they are different
    // things in the same document and the whole safety property of `TargetId`
    // is that they cannot be confused.
    //
    // Additive: `via=`, `mod=`, `sel=` and `level=` keep their names,
    // positions and meanings, so every existing consumer of this line is
    // unaffected. That is a deliberate constraint rather than luck — this
    // module's header calls the output shape a **contract** with a consumer
    // that does not compile against this crate, so a rename here fails at run
    // time inside a check whose subject is something else.
    //
    // The FIRST entry rather than a list: a multi-select can mix the two, and
    // the readout that matters is what a single click produced. A check that
    // needs the whole set reads `object_indices_on` / `leaf_indices_on`
    // through a unit test, where it can see them exactly.
    // **`part=` — WHICH chunk, so repeatability is measurable.**
    //
    // `level=Part` says the operator is inside something; it cannot say inside
    // *what*, and O215 ask 1 is entirely about whether the same click lands on
    // the same chunk twice. Printed as the index or `none`.
    //
    // This line is written through `trace_changed`, so a click that lands on
    // the chunk already selected writes nothing. A driven check measuring
    // repeatability therefore alternates — chunk A, chunk B, chunk A — and
    // reads three lines, rather than clicking one chunk twice and reading a
    // silence it cannot tell from a dead painter.
    crate::diag::trace_changed(SELECTION_SLOT, || {
        let part = selection
            .entries()
            .first()
            .and_then(|e| e.subpath)
            .map_or_else(
                || "none".to_owned(), // ui-text-exempt: diagnostic trace, never displayed
                |p| p.to_string(),
            );
        let first = selection.entries().first().map_or_else(
            || "none".to_owned(), // ui-text-exempt: diagnostic trace, never displayed
            |e| {
                let list = if e.object.is_leaf() { "leaf" } else { "object" };
                format!("{list}:{}", e.object.raw())
            },
        );
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // Placed directly above the literal — see `trace_layout`.
            "canvas-selection via={kind} mod={modifier} sel={} level={} first={first} part={part}",
            selection.len(),
            selection.level().traced(),
        )
    });
}

/// Report where the canvas is, at what magnification, on the `PDFCER_DIAG`
/// channel — **unconditionally**, not only when something happens.
///
/// # The deadlock this removes
///
/// `PROJECT_PLAN.md` §4.3 requirement 1: a canvas line traced only on pointer
/// events deadlocks the harness, which cannot aim until it clicks and cannot
/// click until it can aim. A line gated on `pressed || released || down ||
/// zoom` says nothing at all about a freshly opened document, which is none of
/// those — and without a canvas rect there is no document-to-window mapping,
/// and without that mapping there is no click that can be aimed.
///
/// The only workaround open to the harness is a *layout-probe* click at the
/// client-area centre (`ui-verify`'s `WindowFrame::layout_probe_point`, still
/// used by two checks for reasons of their own), and it is safe but not free:
/// it rests on the assumption that the centre of the client area is the canvas,
/// it fires a real OS click into a document before any assertion has been made,
/// and a check using it must count the events it produces so they are not
/// mistaken for the check's own. An application that simply says where its
/// canvas is owes the harness none of that.
///
/// # When this emits
///
/// Every frame builds the line; [`crate::diag::trace_changed`] emits it only
/// when it differs from the last one. So in practice:
///
/// * **once per document open** — the first frame of a new document finds an
///   empty gate (see [`crate::diag::reset_change_gates`], called from the open
///   path), so there is always a line before any input is delivered;
/// * **again on every layout change** — a window resize, a panel resize, a
///   zoom step, a fit-mode re-derivation, a page change, a scroll;
/// * **not at all** on the frames in between, which is what keeps a
///   several-minute driven run from burying its own evidence.
///
/// # The line, field by field
///
/// ```text
/// pdfcer-diag canvas rect=[[240.0 96.0] - [1560.0 968.0]] zoom=1.5000 page=0 pages=3 off=[0.0 0.0]
/// ```
///
/// * `rect=` — the **page raster's** rect in window logical points, printed
///   as `egui::Rect`'s own `Debug`. Not the viewport, not the panel: the
///   thing `viewer::screen_to_page` is the inverse of. `ui-verify`'s
///   `CanvasMapping` computes `window = rect.min + canvas_point * zoom`, so
///   handing it anything else would be a confidently wrong click.
/// * `zoom=` — logical points per PDF user-space unit, the same number
///   `viewer::screen_to_page` divides by. Four decimals because a fit scale
///   is rarely round and two would quantise a 1320 pt page by a whole point.
/// * `page=` — the 0-based page index `rect` shows. `ui-verify` refuses to
///   convert a document point against a mapping for a different page, and it
///   can only do that if the application says which page it drew.
/// * `pages=` — the document's page count, so a check that walked off the end
///   can tell "no such page" from "the application ignored the command".
/// * `off=` — the scroll offset the area settled on. Reported because
///   `ui-verify`'s `coords` module documents an **unverified assumption**
///   that `rect=` already accounts for scrolling, names the experiment that
///   would settle it, and holds a `scroll` correction at zero until someone
///   runs it. It cannot be run against a binary that does not report the
///   offset, so this field is what makes the assumption falsifiable.
///
/// # `sel=` — the selection size, and why it is only ever a real count
///
/// `ui-verify` reads `sel=` as a fallback when a click produced no event of its
/// own, so the field has to mean *the selection holds this many entries* and
/// nothing else. A `sel=0` published by a build with no hit test and no
/// selection set is a measurement of something that does not exist, and it
/// turns `delete_key_after_canvas_click` from an honest SKIP (*"the harness
/// cannot tell whether the click landed"*) into a FAIL blaming a subsystem
/// nobody has written.
///
/// ⇒ **A trace field lands in the same commit as the thing it counts**, or it
/// lies for as long as the gap lasts. It is counted **after** the frame's
/// gesture has been applied (see the call site), so a click and the `sel=` that
/// describes it appear on the same frame rather than one apart.
///
/// # `display=`, `visible=` and `drawn=` — appended at the END, deliberately
///
/// The first five fields keep their names, their order and their meaning,
/// because `ui-verify`'s `CanvasMapping` parses them and `rect=` is the
/// **acting page's** rect — the thing `viewer::screen_to_page` is the inverse
/// of, and the one a click has to be aimed against. Under a continuous mode
/// several pages are on screen and `rect=` names one of them; `page=` says
/// which.
///
/// The fields below answer what a multi-page strip makes askable, and they are
/// appended rather than inserted so no existing parser moves:
///
/// * `display=` — the page-display mode's id (`single`, `continuous`,
///   `facing`, `facing-continuous`). Without it a trace cannot distinguish
///   "one page is on screen because the operator chose Single" from "one page
///   is on screen because that is all that fits", and those need opposite
///   responses.
/// * `visible=` — how many pages this frame drew. The number a scroll check
///   watches move, and the number that says whether the strip is doing
///   anything at all.
/// * `drawn=` — how many of those had a raster. `drawn < visible` is the
///   honest statement that the renderer is behind, which is exactly what the
///   undrawn pages are saying on screen; `drawn == visible` is a settled
///   strip. A check that measured only `visible` could not tell a filled strip
///   from an empty one.
/// * `crop=` and `rot=` — **they close a whole class of harness defect.** The
///   alternative is a harness reading a page's size by scanning the PDF's first
///   `/MediaBox` with a regular expression and doing the document→canvas
///   conversion as a single `height - y` flip: correct for an upright page
///   whose crop origin is (0, 0) and silently wrong for every other page — the
///   same defect, in the harness, that O174 was in the renderer. An incremental
///   update that rewrites `/Rotate 0` to `/Rotate 270` defeats even a *correct*
///   regex, which finds the superseded one; every `--doc-point` in the suite
///   then aims at the wrong place, and a bounds check that believes the page is
///   792 pt wide makes the right-hand third of a 1224 pt canvas unreachable.
///
///   The application already knows the answer — it holds the parsed `Page`.
///   Tracing it makes the harness's mapping a *reading* rather than a *guess*,
///   and keeps `ui-verify` free of the `pdfcer-core` dependency its own header
///   argues against. `crop=` is `llx,lly,urx,ury` in PDF user space; `rot=` is
///   the page's effective `/Rotate` in degrees, already inherited down the page
///   tree. Both describe the page named by `page=`, i.e. the one `rect=` is
///   the rect of.
pub(super) fn layout(
    doc: &OpenDoc,
    image_rect: Rect,
    scroll_offset: Vec2,
    selected: usize,
    visible: usize,
    with_raster: usize,
) {
    // The acting page's own frame, appended below so a harness never has to
    // scan the PDF for it. See the `crop=` / `rot=` note above.
    let (crop, rotate) = doc.pages.get(doc.view.page_index).map_or(
        (
            pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 0.0, 0.0),
            0,
        ),
        |p| (p.crop_box, p.rotate),
    );
    crate::diag::trace_changed(LAYOUT_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // This comment sits directly above the literal, not above the
            // enclosing call: the gate's scope is the line, and rustfmt is
            // free to reflow a call's arguments out from under a comment
            // placed further up.
            "canvas rect={image_rect:?} zoom={:.4} page={} pages={} off={scroll_offset:?} sel={selected} display={} visible={} drawn={with_raster} crop={:.3},{:.3},{:.3},{:.3} rot={}",
            doc.view.zoom,
            doc.view.page_index,
            doc.pages.len(),
            doc.view.display.id(),
            visible,
            crop.llx,
            crop.lly,
            crop.urx,
            crop.ury,
            rotate,
        )
    });
}

/// Report the pointer's position in **document space** on the `PDFCER_DIAG`
/// channel.
pub(super) fn pointer(ui: &egui::Ui, doc: &OpenDoc, image_rect: Rect, extent: (f32, f32)) {
    if !crate::diag::enabled() {
        return;
    }
    let Some(screen) = ui.ctx().pointer_latest_pos() else {
        return;
    };
    let page = viewer::screen_to_page(screen, image_rect, extent, doc.view.zoom);
    let pdf = doc
        .current_page()
        .and_then(|p| viewer::canvas_to_pdf_space(page, p));
    crate::diag::trace_changed(POINTER_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            // Placed directly above the literal — see `trace_layout`.
            "canvas-pointer screen=({:.1},{:.1}) page=({:.2},{:.2}) pdf={} zoom={:.4}",
            screen.x,
            screen.y,
            page.x,
            page.y,
            pdf.map_or_else(|| "none".to_owned(), |p| format!("({:.2},{:.2})", p.x, p.y)),
            doc.view.zoom,
        )
    });
}

/// Report the view's **pan position** on the `PDFCER_DIAG` channel, in `f64`.
///
/// # Why [`layout`]'s `rect=` cannot answer this
///
/// `rect=` is an `egui::Rect`, so it is `f32`, and at a deep zoom the acting
/// page's rect holds a number around 10¹². An `f32`'s representable spacing
/// there is about 65,536 — so a 40-point pan does not change `rect=` at all,
/// on a build where the pan worked perfectly. A check that read `rect=` would
/// report *"the pan did nothing"* against a correct application, which is the
/// worst kind of harness failure: it aims the next reader at a file that is
/// fine.
///
/// This line carries the same quantity computed and printed in `f64`, and it
/// is the only thing in the trace that can distinguish a pan that was refused
/// from a pan whose result cannot be written down in single precision.
///
/// # The quantity
///
/// ```text
/// pdfcer-diag canvas-pos at=1234567890.5,987654321.0 tier=deep
/// ```
///
/// `at=` is **how far the view has been panned from the acting page's
/// top-left corner, in screen pixels** — the viewport's top-left minus the
/// page's origin on screen. Screen pixels rather than PDF units deliberately:
/// at a trillion percent a 40-pixel pan is 4 × 10⁻¹¹ user units, which needs
/// fifteen significant figures to see, where in screen pixels it is *40* and
/// a check can compare it against the drag it asked for.
///
/// `tier=` says which mechanism produced it — `scroll` for the `f32` scroll
/// offset that owns the position below the deep threshold, `deep` for the
/// `f64` [`crate::viewer::deep::DeepAnchor`] above it. A check that finds a
/// refused pan needs to know which of the two to go and read.
///
/// # Emission
///
/// Ungated, unlike [`layout`]. It is one short line on frames where the view
/// moved, and the change gate that [`layout`] uses keys on a formatted string
/// — which would suppress exactly the sub-threshold movements this exists to
/// measure if two consecutive positions rounded to the same text.
/// `paint=` — where the acting page's raster was actually DRAWN.
///
/// Below the pixmap ceiling this equals the page's own rect and carries
/// nothing new. Above it the raster covers a region rather than the page, and
/// the two part company — which is where `OPERATOR_REQUESTS.md` O24c lived:
/// the page's rect moved smoothly with the pan the whole time, so `rect=` was
/// innocent of the lurch the operator could plainly see. Only this field can
/// witness it.
/// `region=` and `ext=` — what the drawn pixels are a picture OF.
///
/// # Why these are here: so the harness can CHECK the placement
///
/// `region=` is the page-space rectangle of the raster that was actually
/// painted, read from the held texture's own key — **not** the region the
/// shell would like next. `ext=` is the page's extent in the same units.
///
/// Together with `rect=` on the `canvas` line they let `ui-verify` recompute
/// `render::region::region_on_screen` **independently** and compare it against
/// `paint=`. That is the difference between a test that restates the code and
/// one that can catch its reversal: if someone changes the placement back to
/// the *wanted* region — which is O24c, the page lurching backwards mid-pan —
/// the traced region still describes the pixels, the harness's recomputation
/// still says where they belong, and the two disagree by the grid step.
///
/// The cross-check is only valid on the `scroll` tier. Above the deep
/// threshold the placement comes from the `f64` anchor rather than from the
/// page's rect, and reconstructing it would need the anchor too — so the
/// check restricts itself and says so, rather than comparing against a
/// formula that does not apply. Both are `None` for a whole-page raster,
/// where the question does not arise.
/// `want=` — the region the shell wants NEXT, beside `region=` which is the one
/// the pixels on screen are a picture of.
///
/// # Why both, and what reading only one cost
///
/// They differ exactly while a new raster is in flight — and, under
/// `OPERATOR_REQUESTS.md` O25's defect, **for ever**: a pan changes `want` and
/// nothing asks for a render, so `region` stays put and the newly exposed area
/// is blank indefinitely.
///
/// A check written against `region` alone cannot see that. With the defect
/// present the held texture never changes, so its region never changes, and the
/// check reads *"the view did not move"* — indistinguishable from *"nothing was
/// exposed, so nothing was owed"*, which is a SKIP against a broken binary.
/// `panning_past_the_overscan_renders_the_new_area` asserts on both fields for
/// exactly that reason.
///
/// `want` is the shell's intent and moves the instant the view does; `region`
/// is what arrived. **The gap between them is the defect**, and it takes two
/// fields to measure a gap.
pub(super) fn position(
    at: (f64, f64),
    tier: &'static str,
    paint: (f32, f32),
    region: Option<pdfcer_core::page_tree::Rect>,
    want: Option<pdfcer_core::page_tree::Rect>,
    extent: (f32, f32),
) {
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            "canvas-pos at={:.3},{:.3} tier={tier} paint={:.3},{:.3} region={} want={} ext={:.3},{:.3}",
            at.0,
            at.1,
            paint.0,
            paint.1,
            region.map_or_else(
                || "none".to_owned(),
                // SCIENTIFIC, and with enough digits to survive the deep
                // tier. A fixed `{:.4}` cannot express a region 6e-8 pt tall:
                // at a trillion percent every field rounds to the same four
                // decimals, and the difference between them reads as a constant
                // 2.3e-3 — which looks exactly like the region hitting a floor.
                // It is not; it is the trace hitting one. Same lesson as this
                // function's own header: a measurement coarser than the thing
                // measured invents a defect.
                |r| format!("{:.9e},{:.9e},{:.9e},{:.9e}", r.llx, r.lly, r.urx, r.ury),
            ),
            // The same formatting for both, so a check can compare them as
            // text without either side having to parse.
            want.map_or_else(
                || "none".to_owned(),
                |r| format!("{:.9e},{:.9e},{:.9e},{:.9e}", r.llx, r.lly, r.urx, r.ury),
            ),
            extent.0,
            extent.1
        )
    });
}

/// The slot [`surface`] de-duplicates on.
pub(super) const SURFACE_SLOT: &str = "canvas-surface"; // ui-text-exempt: trace slot name, never displayed

/// **Which of the canvas's two interactive rectangles owned this frame's
/// gesture** — the only external evidence that O23's off-page half is live.
pub(super) fn surface(
    surface: super::pasteboard::Surface,
    on_page: bool,
    page_gesture: bool,
    paste_gesture: bool,
    page_popup: bool,
) {
    crate::diag::trace_changed(SURFACE_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-surface surface={} onpage={on_page} pagegesture={page_gesture} pastegesture={paste_gesture} pagepopup={page_popup}",
            match surface {
                super::pasteboard::Surface::Page => "page",
                super::pasteboard::Surface::Pasteboard => "pasteboard",
            }
        )
    });
}

/// The slot [`halo`] de-duplicates on.
pub(super) const HALO_SLOT: &str = "canvas-halo"; // ui-text-exempt: trace slot name, never displayed

/// **Which raster tier the current page is on, and how far past the sheet
/// it reaches** — the only external evidence that O23's "see" half is live.
pub(super) fn halo(
    tier: &str,
    known: bool,
    off_page: bool,
    box_of: Option<pdfcer_core::page_tree::Rect>,
) {
    // ui-text-exempt: trace field VALUES, never displayed in the UI.
    let offpage = if off_page { "on" } else { "off" };
    crate::diag::trace_changed(HALO_SLOT, || match box_of {
        Some(r) => format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-halo tier={tier} known={known} offpage={offpage} box={:.3},{:.3},{:.3},{:.3}",
            r.llx, r.lly, r.urx, r.ury
        ),
        None => format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-halo tier={tier} known={known} offpage={offpage} box=none"
        ),
    });
}

/// The slot [`pasteboard`] de-duplicates on.
pub(super) const PASTEBOARD_SLOT: &str = "canvas-pasteboard"; // ui-text-exempt: trace slot name, never displayed

/// **How far the layout reaches past the sheets** — the second half of
/// `View ▸ Display ▸ Off-page content`, and the half the operator described
/// first.
pub(super) fn pasteboard(overhang: egui::Vec2, off_page: bool) {
    // ui-text-exempt: trace field VALUES, never displayed in the UI.
    let offpage = if off_page { "on" } else { "off" };
    crate::diag::trace_changed(PASTEBOARD_SLOT, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-pasteboard offpage={offpage} overhang={:.3},{:.3}",
            overhang.x, overhang.y
        )
    });
}

/// The slot [`placed`] de-duplicates on.
pub(super) const PLACE_SLOT: &str = "canvas-place"; // ui-text-exempt: trace slot name, never displayed

/// **What the ranked offset decision asked the scroll area for, this frame.**
pub(super) fn placed(
    decision: &crate::canvas::offset::Decision,
    frames: u8,
    strip: egui::Vec2,
    row: egui::Rect,
    vp: egui::Vec2,
    overhang: egui::Vec2,
) {
    crate::diag::trace_changed(PLACE_SLOT, || {
        // Taken from the decision itself rather than passed in beside it, so
        // the name and the number on the line cannot drift apart at the call
        // site — the same reason `Decision::won` is the only constructor.
        let src = decision.source;
        let want = match decision.offset {
            // ui-text-exempt: trace field VALUES, never displayed in the UI.
            Some(w) => format!("{:.1},{:.1}", w.x, w.y),
            None => "none".to_owned(), // ui-text-exempt: trace field value
        };
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-place src={src} want={want} frames={frames} strip={:.1}x{:.1} \
             row=[{:.1},{:.1} {:.1}x{:.1}] vp={:.1}x{:.1} over={:.1}x{:.1}",
            strip.x,
            strip.y,
            row.min.x,
            row.min.y,
            row.width(),
            row.height(),
            vp.x,
            vp.y,
            overhang.x,
            overhang.y
        )
    });
}

/// The slot [`confined`] de-duplicates on.
pub(super) const CONFINED_SLOT: &str = "canvas-confined"; // ui-text-exempt: trace slot name, never displayed

/// **Whether the `f64` anchor had to be pulled back into the range the view
/// can actually place** — `OPERATOR_REQUESTS.md` **O186**, stage one.
pub(super) fn confined(x: bool, y: bool) {
    // ui-text-exempt: trace field VALUES, never displayed in the UI.
    let axes = match (x, y) {
        (true, true) => "xy",
        (true, false) => "x",
        (false, true) => "y",
        (false, false) => "none",
    };
    crate::diag::trace_changed(CONFINED_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("canvas-confined axes={axes}")
    });
}

/// Its own slot rather than sharing [`SELECTION_SLOT`]: a selection line
/// changes when the operator picks something, and this one changes when a
/// gesture starts and stops. Sharing would make each silence the other on
/// exactly the frames the other is about.
pub(super) const MOVE_GHOST_SLOT: &str = "canvas-move-ghost"; // ui-text-exempt: trace slot name, never displayed

/// **Whether the move ghost was drawn, and if not, why not** —
/// `OPERATOR_REQUESTS.md` **O215** ask 5.
pub(super) fn move_ghost(boxes: usize, part_rung: bool, suppressed: bool) {
    // ui-text-exempt: trace field VALUES, never displayed in the UI.
    let rung = if part_rung { "part" } else { "object" };
    // ui-text-exempt: trace field VALUES, never displayed in the UI.
    let why = if suppressed { "o63" } else { "no" };
    crate::diag::trace_changed(MOVE_GHOST_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("canvas-move-ghost boxes={boxes} rung={rung} suppressed={why}")
    });
}

/// The trace slot for the **travelling copy of the page's own pixels**.
pub(super) const RASTER_GHOST_SLOT: &str = "canvas-raster-ghost"; // ui-text-exempt: trace slot name, never displayed

/// Why [`raster_ghost`] drew what it drew — the `reason=` field's vocabulary.
#[derive(Clone, Copy)]
pub(super) enum RasterGhostReason {
    /// It drew. Read `drawn=` for how many.
    Drew,
    /// The real geometry is already travelling under a shape preview, so a
    /// second picture of it would state nothing. The one correct zero.
    GeometryTravels,
    /// The page has no picture yet, so there is nothing to take a copy of.
    NoRaster,
    /// Every held outline's source fell outside the rastered part of the page.
    OffRaster,
    /// Nothing is selected.
    NoSelection,
}

impl RasterGhostReason {
    // ui-text-exempt: trace field VALUES, never displayed in the UI.
    const fn as_field(self) -> &'static str {
        match self {
            Self::Drew => "none",
            Self::GeometryTravels => "geometry",
            Self::NoRaster => "no-raster",
            Self::OffRaster => "off-raster",
            Self::NoSelection => "no-selection",
        }
    }
}

/// **Whether the glyphs themselves travelled, and if not, why not** —
/// `OPERATOR_REQUESTS.md` **O215** ask 5.
pub(super) fn raster_ghost(drawn: usize, clipped: usize, reason: RasterGhostReason) {
    let reason = reason.as_field();
    crate::diag::trace_changed(RASTER_GHOST_SLOT, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("canvas-raster-ghost drawn={drawn} clipped={clipped} reason={reason}")
    });
}
