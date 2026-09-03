//! # `canvas::painting` — everything the canvas draws, once everything is
//! decided
//!
//! One function, [`draw`], lifted out of [`super::interact`] on 2026-08-19 when
//! that file crossed R2's 1,500-line ceiling for the first time.
//!
//! ## ★ Why this is a seam and not a size
//!
//! `tools/gates/check-file-size.sh`'s own header refuses a split made to fit a
//! number: *"Split the module along its seams — one subject per file."* The
//! seam was already written into `interact`, as the numbered sections of its
//! own body:
//!
//! > 1 the pointer · 2 what a press would land on · 3 advance the gesture ·
//! > 4 the decomposition · 5 apply the gesture · 5b the right-click · 6 keys ·
//! > 7 re-resolve · **8 draw**
//!
//! Sections 1 to 7 answer *what happened and what does it mean*. This answers
//! *what does that look like*: it reads values the first seven produced, writes
//! nothing but pixels, raises no `Action`, and makes no decision.
//!
//! What stayed behind in section 8 is everything that is **not** painting — the
//! typing loop, the keyboard-ownership check and the cursor icon — which had
//! ended up under the same heading because they run at the same moment, not
//! because they are the same subject.
//!
//! ## ★★ The layer order IS this module's content
//!
//! Every position in the sequence is an argument, and each one travelled here
//! with the code rather than being summarised:
//!
//! | layer | why it is where it is |
//! |---|---|
//! | **grid** | under everything: the only thing here about the *paper* rather than about something the operator has selected, searched for or is dragging |
//! | **find highlights** | a wash answering *where is the text I asked about*, under the outline, which is a statement about what a verb would act on |
//! | **selection outlines**, **grips** | |
//! | **marquee** | |
//! | **guides** | on TOP of the selection — a guide is a line the operator aligns to, and an outline a few points across does not hide a hairline crossing it. The reverse order would hide the guide behind the very object being aligned to it |
//! | **move ghost**, **resize ghost** | over the real outline, and both stay visible: the pair is what states the change |
//! | **markup band**, **freehand trail**, **vertex run**, **measure preview** | last, over everything: while a gesture is in flight the shape IS the cursor, and anything drawn over it obscures the one thing being aimed |
//!
//! **Re-ordering any two of these is a behaviour change.**

use egui::{Rect, Ui};

use super::{grid, guides, handles, markup, measure, overlay};
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::markup::pen::Pen;
use crate::canvas::measure::Resolved;
use crate::canvas::selection::SelectionState;
use crate::canvas::tool::CanvasTool;

/// Everything the painting pass needs, and nothing else.
///
/// A struct rather than sixteen parameters, and the grouping is a statement:
/// **every member is a product of the decision half.** It also removes the
/// failure a long parameter list invites — five of them are `Option`s and
/// several are adjacent, so a swap would compile.
pub(super) struct Frame<'a> {
    /// The page on screen.
    pub page_index: usize,
    /// The clip rectangle every painter in this pass is bounded by.
    pub clip: Rect,
    /// The frame's screen ⟷ canvas mapping.
    pub map: &'a PageMapping,
    /// The selection as it stands **after** the gesture was applied.
    pub selection: &'a SelectionState,
    /// The rubber-band, if one is in flight.
    pub marquee: Option<Rect>,
    /// The move ghost's canvas-space displacement, if a move would commit.
    pub ghost: Option<egui::Vec2>,
    /// ★★★ **The selection's own geometry at its new position**, in page space
    /// (`OPERATOR_REQUESTS.md` O63).
    ///
    /// Borrowed rather than owned, because a preview of a CAD object can carry
    /// thousands of segments and this struct is built fresh every frame.
    ///
    /// `None` on every rung `canvas::shapes` cannot draw honestly — a text run,
    /// an image, a form XObject, a page that will not decompose, a selection
    /// past the cap — in which case [`Self::ghost`] is the whole answer, exactly
    /// as it was before this field existed.
    pub shape_preview: Option<&'a crate::canvas::shapes::ShapePreview>,
    /// The handle being dragged, if one is — its anchor, its side and where it
    /// now sits in canvas space.
    ///
    /// ★ Carried so the drawn handle follows the pointer during the drag. The
    /// decomposition still holds its OLD position — nothing is committed until
    /// the release — so a painter that read the model alone would draw the
    /// handle sitting still while the operator dragged it, which is the "the
    /// gesture does nothing" symptom this project keeps finding, in its most
    /// literal form.
    pub handle_drag: Option<(usize, pdfcer_core::vector::Handle, egui::Pos2)>,
    /// Where a dragged markup annotation would land, in canvas space.
    ///
    /// ★ Separate from [`Self::ghost`] even though both are one rectangle's
    /// worth of preview: that one is a *displacement* applied to the content
    /// selection's outlines, and this is an absolute rectangle for a selection
    /// that has no content outlines at all. They can never both be `Some`, and
    /// sharing the field would make the painter's question — *which kind of
    /// thing is this about?* — answerable only by re-inspecting the selection.
    pub annot_ghost: Option<egui::Rect>,
    /// The resize ghost's grip and factors, if a resize would commit.
    pub resize_ghost: Option<(handles::Grip, (f32, f32))>,
    /// A ce dimension being dragged to a new placement, as the **page-space**
    /// segments it would be drawn as on release.
    ///
    /// ★ Page space, not canvas space, and that is the one thing to know about
    /// this field. Every other preview here is a screen or canvas figure that
    /// the gesture computed directly. This one is produced by
    /// `measure::pick::dimension_preview_segments` - the *same* function a
    /// committed dimension is previewed from - which works in the document's
    /// own coordinates, and it has to, because the geometry it derives (where
    /// the dimension line runs, where each extension line reaches from) is
    /// defined in the page. Projecting happens once, at the painter, through
    /// the same two-hop bridge `canvas::measure` uses.
    pub dimension_preview: Option<&'a [(pdfcer_core::vector::Point, pdfcer_core::vector::Point)]>,
    /// What a perimeter corner being dragged is snapping to, if anything.
    ///
    /// ★ `ui-conventions/drag-moves.md` D6: *"a snap is an inference. It is
    /// announced by an indicator at the target while the drag is live — never
    /// applied silently."* Without the marker the corner simply arrives
    /// somewhere the operator did not put it, and there is nothing on screen to
    /// say why.
    ///
    /// It is the **same candidate** `dimdrag::drag_vertex` computed and the
    /// release commits, carried here rather than re-queried — one derivation,
    /// which is `measure::Resolved`'s founding rule and the reason a snap
    /// marker once described a point four days' worth of clicks did not land
    /// on.
    pub vertex_snap: Option<pdfcer_core::vector::snap::SnapCandidate>,
    /// The angle a rotate drag has turned through, in **screen** space and
    /// un-negated, or `None`.
    ///
    /// `Some` only when `rotating::drag` has established that a release would
    /// commit — the same honesty contract the move and resize ghosts are held
    /// to. A preview of a gesture that will be refused is *"a lie with a low
    /// alpha"*.
    pub rotate_ghost: Option<f32>,
    /// The markup band, if one would commit.
    pub band: Option<markup::band::Preview>,
    /// The lines a text-following highlight would cover, in canvas space.
    ///
    /// ★★ A separate slot from [`Self::band`] and never `Some` beside it: the
    /// two are the same gesture taking different geometry, and the drag decides
    /// which on every frame. Folding them into one value would mean the painter
    /// asking *"is this two points or a list?"* — a question the type can
    /// answer for free.
    pub text_marks: Option<Vec<egui::Rect>>,
    /// The freehand trail, already simplified, in canvas space.
    pub ink_trail: Option<Vec<egui::Pos2>>,
    /// The armed tool — read by the previews that draw whenever a tool is
    /// armed rather than only during a gesture.
    pub active_tool: CanvasTool,
    /// The markup pen, so a preview is drawn in the colour it will author.
    pub pen: Pen,
    /// The pointer, in screen space, if it is over the window at all.
    pub screen_pos: Option<egui::Pos2>,
    /// The find results, for the highlight wash.
    pub find: &'a crate::find::FindState,
    /// The text sweep's own selection, if there is one.
    pub text_selection: Option<&'a crate::canvas::textsel::TextSelection>,
    /// What the measure tool would snap to under the pointer.
    pub measure_hover: Option<Resolved>,
}

/// Paint the canvas.
///
/// Takes `ui` and `doc` alongside [`Frame`] because both are borrows the caller
/// still owns and neither is a *product* of the decision half — putting them in
/// the struct would make it a bag rather than a grouping.
pub(super) fn draw(
    ui: &Ui,
    ctx: &egui::Context,
    doc: &OpenDoc,
    pages: &[crate::canvas::strip::PageView],
    f: &Frame<'_>,
) {
    let Frame {
        page_index,
        clip,
        map,
        selection,
        marquee,
        ghost,
        annot_ghost,
        resize_ghost,
        handle_drag,
        active_tool,
        pen,
        screen_pos,
        ..
    } = f;
    let (page_index, clip, map, selection) = (*page_index, *clip, *map, *selection);
    let (marquee, ghost, resize_ghost) = (*marquee, *ghost, *resize_ghost);
    let (active_tool, pen, screen_pos) = (*active_tool, *pen, *screen_pos);
    let find = f.find;
    let text_selection = f.text_selection;
    let measure_hover = f.measure_hover;
    let text_marks = f.text_marks.as_deref();
    let ctx = ctx.clone();

    // ---- 8. draw --------------------------------------------------------
    let painter = ui.painter().with_clip_rect(clip);
    // ★ The grid goes UNDER everything, including the find wash. It is the
    // only thing painted here that is about the *paper* rather than about
    // something the operator has selected, searched for or is dragging, so
    // anything drawn over it is a statement about the drawing and must win.
    // Draws nothing at all with the toggle off. See `rulers`' header §2 for
    // why it is per page rather than across the viewport.
    if doc.view.grid {
        grid::draw(ui, doc, pages, clip);
    }
    // ★ The find highlights go on FIRST, under everything else.
    //
    // They are a wash over page content — an answer to "where is the text I
    // asked about" — while the selection outline is a statement about what a
    // verb would act on. Painting the wash over the outline would dim the
    // control feedback with a hint; painting it under leaves both readable.
    //
    // `page_highlights` yields nothing at all when the results are not current
    // — a stale epoch, a query the operator has edited, a closed bar — so an
    // edit stops the highlights by supplying an empty iterator rather than by
    // a check here. That is what keeps rule 4: this file cannot paint a mark
    // over content the search no longer describes, because it is never handed
    // one. See `crate::find`'s staleness section.
    //
    // ★ **Once per drawn page, each through its own map** — the one place the
    // canvas is legitimately about pages other than the one being acted on. A
    // search describes the whole document, so under a continuous mode its hits
    // are on several of the pages on screen at once, and painting them all
    // through the acting page's map would stack every page's highlights onto
    // one page. That is the failure this feature was most likely to ship
    // silently: the hits are found, the wash is drawn, and it is drawn in the
    // wrong place — which looks like a highlight bug rather than a mapping one.
    //
    // The loop reduces to exactly the previous call under `Single`, where
    // `pages` holds one entry and it is the acting page.
    for view in pages {
        overlay::draw_find_hits(
            &painter,
            ui.visuals(),
            &view.map,
            find.page_highlights(view.page, doc.edit_epoch),
        );
    }
    // ★ The text selection's wash, in the same layer as the find wash and for
    // the same reason: both are statements about *characters on the page*
    // rather than about a control, so they belong under anything that describes
    // a verb's operand. They cannot in fact both be on screen over the same
    // glyphs and matter — Find is a query and this is a sweep — but the
    // ordering is stated rather than left to chance, because the day they
    // overlap the reader has to be able to see both.
    //
    // Per drawn page, through that page's own map, exactly as the find wash is:
    // the selection is single-page, so all but one iteration is handed an empty
    // slice — but painting through the *acting* page's map instead would put a
    // continuous-strip selection on the wrong sheet, which is the failure the
    // find wash's own comment records as the one most likely to ship silently.
    for view in pages {
        overlay::draw_text_selection(
            &painter,
            ui.visuals(),
            &view.map,
            text_selection
                .as_ref()
                .map_or(&[][..], |s| s.highlights(view.page, doc.edit_epoch)),
        );
    }
    // ★★★ **ONE `GripSet`, ASKED ONCE, HANDED TO THE PAINTER — rule H7.**
    //
    // `pressing::grabbable` is the function `pressing::look` asks to decide
    // what the press lands on, and it is the function asked here to decide what
    // is drawn. Two calls, one decision procedure, no second predicate anywhere
    // for the two to drift apart on.
    //
    // Before 2026-08-28 the painter re-derived the equivalent condition
    // locally, which was survivable while every kind offered the same set. It
    // stopped being survivable when three kinds started offering three
    // different sets — a markup turns and scales, a ce dimension turns and does
    // not scale, a form field's box scales and does not turn. See
    // `overlay::draw_grips`' header for what each disagreement would look like
    // from a chair.
    //
    // ★ It is cheap: `grabbable` is three `Option` probes over values already
    // resolved for this frame, and the dimension probe short-circuits unless an
    // annotation is selected at all.
    let offer = crate::canvas::pressing::grabbable(&ctx, doc, map, selection);
    overlay::draw_selection(&painter, ui.visuals(), map, selection, offer);
    draw_anchors(
        &painter,
        ui,
        doc,
        map,
        selection,
        page_index,
        *handle_drag,
        active_tool,
    );
    if let Some(rect) = marquee {
        overlay::draw_marquee(&painter, ui.visuals(), map, rect);
    }
    // The ghost sits ON TOP of the real outline, and both stay visible: the
    // pair is what states the displacement. `ghost` is `Some` only when
    // `moving::drag` has already established that the release will commit — a
    // preview of a move that will be refused is the thing rule 4 and the
    // no-placeholders invariant both forbid.
    // The guides sit on TOP of the selection, and the order is the point: a
    // guide is a line the operator has to see while they align something to
    // it, and a selection outline is a box a few points across that a hairline
    // crossing it does not hide. The reverse order would hide a guide behind
    // exactly the object the operator is aligning to it.
    guides::draw(ui, doc, pages, clip);
    if let Some(delta) = ghost {
        overlay::draw_move_ghost(
            &painter,
            ui.visuals(),
            map,
            selection,
            delta,
            // ★ O69: no ghost box at an inner rung either. See the parameter.
            offer.outline,
        );
    }
    // The annotation ghost, on the same layer and under the same contract:
    // `annotdrag::drag` returns `Some` only for a selection whose release will
    // commit, so a locked annotation or a ce dimension draws none.
    if let Some(rect) = annot_ghost {
        overlay::draw_annot_ghost(&painter, ui.visuals(), map, *rect);
    }
    // ★ The resize ghost, on the same layer and under the same contract: it is
    // `Some` only when `resizing::drag` has established that a release would
    // commit, so a preview of a refused gesture is never drawn. The anchor is
    // re-read from the same `grip_box` the drag measured against rather than
    // carried on the value, because it is a pure function of the selection and
    // carrying it would be a second copy that could go stale between the frame
    // that computed it and the frame that paints.
    if let Some((grip, factors)) = resize_ghost
        && let Some(bounds) = overlay::grip_box(map, selection)
    {
        overlay::draw_resize_ghost(
            &painter,
            ui.visuals(),
            map,
            selection,
            // ★ `pivot`, not `anchor` — the SAME point `canvas::resizing`
            // commits about. `anchor` is where the grip is; the pivot is the
            // opposite corner, which is what stays still. Using the wrong one
            // here would preview a shape growing away from the operator's hand
            // and then commit one growing towards it, so the object would jump
            // by its own size on release.
            grip.pivot(bounds),
            factors,
        );
    }
    // ★ …and the rotate ghost, on the same layer and under the same contract.
    //
    // The centre is re-read from the same `grip_box` the drag measured against
    // rather than carried on the value, for the reason the resize ghost gives
    // one block up: it is a pure function of the selection, and carrying it
    // would be a second copy that could go stale between the frame that
    // computed it and the frame that paints.
    // ★★★ …and the box it is re-read from is `grabbable`'s, NOT
    // `overlay::grip_box`'s — 2026-08-28, and this line is one of the two the
    // whole annotation rotation hangs on.
    //
    // `grip_box` derives its answer from the selection's cached **content**
    // outlines, which `select_annot` clears: an annotation is not content and
    // has nothing decomposed to cache. So over a selected markup or dimension
    // it answers `None`, this `let` fails, and **no ghost is drawn at all** —
    // the operator drags the handle and sees nothing move until they let go,
    // which reads as a gesture that is not tracking.
    //
    // `pressing::grabbable` is the same function the drag itself measures
    // against (`canvas::rotating::Frame::bounds`) and the same one the hit test
    // and the painter above ask. One box; the preview, the pivot and the commit
    // cannot disagree about where the centre is.
    if let Some(radians) = f.rotate_ghost
        && let Some(bounds) = offer.bounds
    {
        overlay::draw_rotate_ghost(
            &painter,
            ui.visuals(),
            map,
            selection,
            crate::canvas::handles::Grip::Rotate.pivot(bounds),
            radians,
        );
    }
    // ★★ THE PERIMETER'S VERTEX HANDLES.
    //
    // Drawn whenever a perimeter ce dimension is selected, and drawn BEFORE the
    // drag rather than only during it - because a handle that appears once you
    // are already dragging is a handle nobody discovers. The operator asked to
    // *"edit the endpoints of the lines to adjust the shape"*, and a shape whose
    // corners are not marked gives them nothing to aim at.
    //
    // ★ The one rule that matters here is `dimdrag::vertex_at`'s: the drawn
    // square is the promise and the live target is slightly larger. Never the
    // reverse - a target smaller than its picture is the operator missing
    // something they can plainly see, which is `handles::grip_at`'s standing
    // convention and the reason both numbers live next to each other in one
    // module rather than one here and one there.
    //
    // Note this is NOT content marking. Rule 4 forbids styling applied content
    // as provisional; a selection handle is the cursor, and it is drawn for the
    // selected object only, and it disappears the moment the selection does.
    for (index, centre) in crate::canvas::dimdrag::vertices(doc, selection)
        .into_iter()
        .enumerate()
    {
        let screen = map.to_screen(centre);
        let handle = egui::Rect::from_center_size(
            screen,
            egui::Vec2::splat(crate::canvas::dimdrag::VERTEX_HANDLE_PT),
        );
        // ★★ PUBLISHED, so a driven check can AIM at a corner.
        //
        // The same argument `SELECTION_OUTLINE_REGION` makes: where a handle is
        // sits at the end of a page -> canvas -> screen conversion, and it is a
        // fact only the application knows. A harness that guessed "somewhere
        // near the corner I clicked" would land on the page instead, which
        // starts a marquee - and the check would then pass while exercising a
        // completely different gesture.
        //
        // Indexed, because *which* corner moved is the whole assertion: a drag
        // that reshapes the wrong vertex is a defect that looks exactly like a
        // working one until you compare the geometry.
        crate::diag::ui_rect(
            &format!("{}.{index}", crate::canvas::dimdrag::VERTEX_REGION),
            handle,
        );
        painter.rect_filled(handle, 0.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(
            handle,
            0.0,
            egui::Stroke::new(1.0, ui.visuals().selection.stroke.color),
            egui::StrokeKind::Middle,
        );
    }

    // ★★★ **THE SHAPE ITSELF, FOLLOWING THE POINTER** — O63.
    //
    // **Ken, 2026-08-30:** *"if I moved the end of a line, it didn't show me the
    // shape change of the line, it just had a perimeter box around it … there
    // isn't a real preview like there is in inkscape."*
    //
    // Drawn ABOVE the bounding ghost and below the snap marker. The order is the
    // reading order of the three: the outline says *which* thing is moving, the
    // shape says *what it will look like*, and the snap marker says *what it
    // will line up with*. An operator reads them outward from the object.
    //
    // ★★ Rule 4: this is the cursor, not the document. It is a pre-commit
    // affordance — the same category as the rubber band and the snap indicator,
    // both explicitly permitted — it is derived from the transform the release
    // will commit, and it disappears the moment the real thing is rendered.
    // Nothing already applied to the page is marked, tinted or outlined by it.
    //
    // ★★★ AND IT OUTLIVES THE GESTURE — O63's third piece.
    //
    // `f.shape_preview` is the in-flight value and is `None` the moment the
    // pointer is released. `doc.held_preview_to_draw()` is the same geometry
    // kept alive until the page raster carries the edit, because the raster
    // underneath still shows the object where it STARTED for one to two seconds
    // on the operator's own drawing — so dropping the preview at release makes
    // the object appear to snap back and then jump forward.
    //
    // ★ The in-flight value wins when both are present. A new gesture describes
    // the document better than a hold from the previous one, and `hold_preview`
    // replaces rather than accumulates, so the overlap is at most one frame.
    if let Some(preview) = f.shape_preview.or_else(|| doc.held_preview_to_draw())
        && let Some(page) = doc.pages.get(page_index)
    {
        crate::canvas::shapes::draw(
            &painter,
            preview,
            page,
            map,
            ui.visuals().selection.stroke.color,
            // ★ The scale is DERIVED by mapping a unit page vector, not read
            // off the mapping's private zoom. `coords`' standing rule is that a
            // coordinate is produced by exactly one conversion in exactly one
            // place, and a stroke width is a coordinate — asking the mapping to
            // convert a length is the same act as asking it to convert a point.
            map.page_vec_to_screen(egui::vec2(1.0, 0.0)).x.abs(),
        );
    }
    // ★ The ce-dimension placement preview, on the same layer and under the
    // same honesty contract as the two ghosts above: it is `Some` only when
    // `dimdrag::drag` has established that a release would commit, and it is
    // derived from the SAME placement the commit writes - literally the same
    // pair of `f64`s. So the operator cannot be shown one standoff and given
    // another.
    //
    // Drawn in the selection stroke rather than a colour of its own. The
    // dimension being dragged is the selected object, this is that object
    // following the pointer, and a third colour on the canvas would be a fourth
    // thing to learn for no information gained.
    if let Some(segments) = f.dimension_preview
        && let Some(page) = doc.pages.get(page_index)
    {
        let stroke = egui::Stroke::new(1.5, ui.visuals().selection.stroke.color);
        for (a, b) in segments {
            let (Some(sa), Some(sb)) = (
                crate::canvas::measure::page_to_screen(*a, page, map),
                crate::canvas::measure::page_to_screen(*b, page, map),
            ) else {
                continue;
            };
            painter.line_segment([sa, sb], stroke);
        }
    }
    // ★★ …and the snap marker for that corner, over the preview it belongs to.
    //
    // Drawn from `snap::snap_marker_shapes` — the same glyph set, in the same
    // theme role, at the same zoom-invariant size the measure tools use — so an
    // operator who has learned that a square means *endpoint* while placing a
    // perimeter reads the same square while correcting one. A second marker
    // vocabulary for the same inference would be a second thing to learn for no
    // information gained.
    //
    // Rule 4: this is a pre-commit affordance, the cursor describing what is
    // about to happen. It disappears on release, and what replaces it is the
    // dimension itself, rendered with no marking of any kind.
    if let Some(candidate) = f.vertex_snap
        && let Some(page) = doc.pages.get(page_index)
        && let Some(screen) = crate::canvas::measure::page_to_screen(candidate.point, page, map)
    {
        let colour = crate::canvas::snap::snap_indicator_tint(ui.ctx())
            .unwrap_or_else(|| ui.visuals().selection.stroke.color);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // The marker's screen position beside the pointer's, exactly as
            // `measure-snap-marker` carries them, and for the invariant that is
            // true by the definition of snapping: a marker is never further
            // from the pointer than the snap tolerance. That is the assertion
            // that catches a coordinate hop applied twice, which is the defect
            // this codebase has now met three times.
            let p = ui.ctx().pointer_latest_pos();
            format!(
                "vertex-snap-marker kind={:?} marker={:.1},{:.1} dx={:.1} dy={:.1} tol={:.2}",
                candidate.kind,
                screen.x,
                screen.y,
                p.map_or(f32::NAN, |q| screen.x - q.x),
                p.map_or(f32::NAN, |q| screen.y - q.y),
                map.snap_tolerance(),
            )
        });
        painter.extend(crate::canvas::snap::snap_marker_shapes(
            screen,
            candidate.kind,
            colour,
            crate::canvas::measure::SNAP_MARKER_PT,
        ));
    }
    // Last, and over everything: the band IS the cursor for as long as it
    // exists, and a guide or an outline drawn over the shape being authored
    // would obscure the one thing the operator is aiming.
    if let Some(band) = f.band {
        markup::band::draw_preview(&painter, map, band, pen);
    }
    // ★★★ The text-following highlight's preview — one wash per line the drag
    // crosses. `OPERATOR_REQUESTS.md` O54.
    //
    // Drawn with the SAME wash the area band uses, deliberately: they are one
    // feature reached by one tool, and a preview that changed colour depending
    // on whether the pointer had found text would read as two.
    if let Some(marks) = text_marks {
        markup::band::draw_text_marks(&painter, map, marks, pen);
    }
    // …and the freehand trail, on the same argument and in the same layer: while
    // the button is down the stroke IS the cursor, and it is drawn from the
    // simplified point list the release will author rather than from the raw
    // input, so the mark does not visibly change shape at the moment it commits.
    if let Some(trail) = &f.ink_trail {
        markup::ink::draw_preview(&painter, map, trail, pen);
    }
    // ★ …and the vertex run, which is drawn on EVERY frame the tool is armed
    // rather than only while a gesture is in flight — because for this family
    // there is no "in flight" the frame can see. A run between clicks is a
    // pointer that is not down, so a preview gated on a gesture would appear only
    // during the instant of a click and the operator would be placing vertices
    // into a canvas that never showed them.
    //
    // It takes the frame's `map` and the pointer, and it draws three things: the
    // committed run, the rubber segment to the pointer, and — for a Polygon
    // alone — the closing segment back to the first vertex, which is the single
    // visible difference between the two tools before the commit. See
    // `markup::vertex::preview`.
    if let Some(kind) = active_tool.markup_kind().filter(|k| k.is_vertex()) {
        markup::vertex::preview(
            ui,
            doc.current_page(),
            page_index,
            kind,
            map,
            screen_pos.map(|p| map.to_page(p)),
            pen,
        );
    }
    // …and the measure preview, on the same argument: while a pick is in
    // progress the preview IS the cursor, and it describes what the next click
    // will commit.
    //
    // ★ It takes the frame's `map`, and the comment here used to say it did not
    // need one *"because it converts through the renderer's own page
    // transform"*. That was the defect: the renderer's transform at scale 1.0
    // lands in **canvas** space — page top-left origin, no zoom — and the
    // painter speaks screen, so every mark the measure preview drew was offset
    // by wherever the page sat in the window and drawn at 100 % whatever the
    // magnification. See `measure::page_to_screen`, which is now the one place
    // both hops happen.
    if let Some(kind) = active_tool.measure_kind() {
        measure::preview(
            ui,
            measure::Preview {
                doc,
                page_index,
                kind,
                map,
                hover: measure_hover,
            },
        );
    }
    // ★ …and the caret, which is the same argument once more: while a draft is
    // in flight the caret IS the cursor, and it describes where the next
    // keystroke lands.
    //
    // It draws a caret and an extent bracket and **no glyphs** — see
    // `textedit::preview`, which carries the argument for why a better ghost is
    // the wrong fix for `DEFECTS.md` D4a rather than a deferred one.
    if active_tool.text_edit_kind().is_some() {
        crate::canvas::textedit::paint::preview(
            ui,
            &ctx,
            &crate::canvas::textedit::Preview {
                doc,
                page_index,
                map,
            },
        );
    }

    // ★ **The keystrokes**, read raw and consumed here.
    //
    // After the gesture machine and before the cursor, which is the only place
    // it can be: it needs `actions` (Enter commits) and it must not run on a
    // frame the canvas does not own the keyboard for.
    //
    // `!ctx.text_edit_focused()` is the guard, and it is `DEFECTS.md` **D1**'s
    // predicate rather than `egui_wants_keyboard_input()` — for the identical
    // reason `app::keyboard` and `canvas::tool::space_held` use it. The wrong
    // one is true whenever *any* widget has focus, and the canvas takes focus on
    // click, so a build using it would stop accepting characters the moment the
    // operator clicked the page they are trying to type on. The right one asks
    // whether a **text field** has it — the page-number box, a Properties value
    // — which is the only case where a character is not ours.
}

/// Mark the entered object's anchors when the operator is inside one.
///
/// # ★ Why this is a function here rather than three lines at the call site
///
/// Because it is the **only** place in the paint pass that needs the object
/// model, and reaching for it costs a `Ref` into the document's decomposition
/// cache. Keeping that borrow inside one short function is what guarantees it is
/// released before the rest of the frame — the same discipline
/// `app::cache::page_objects`' own docs set out, and the reason
/// `canvas::interact` has a comment about dropping its `Ref` explicitly.
///
/// It draws nothing at the Object rung. An object's anchors are not the
/// operator's subject there — the object is — and painting thousands of hollow
/// squares over a selection they are about to *move as a whole* would be noise
/// with a rendering cost.
#[allow(clippy::too_many_arguments)]
fn draw_anchors(
    painter: &egui::Painter,
    ui: &Ui,
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    page_index: usize,
    drag: Option<(usize, pdfcer_core::vector::Handle, egui::Pos2)>,
    tool: crate::canvas::tool::CanvasTool,
) {
    use crate::canvas::selection::SelectionLevel;

    // ★★ **Or the Node tool is armed**, as of 2026-08-19.
    //
    // Before that, anchors drew only after a two-double-click descent, so the
    // operator who wanted to move an end point had to already know a rung
    // ladder existed in order to discover it. Arming the white arrow now shows
    // them on the first click — see `SelectionState::click_direct`, which puts
    // the selection at the Part rung the moment a shape is clicked.
    //
    // The rung check STAYS beside the tool check rather than being replaced by
    // it: an operator who descended by double-clicking with the Select tool has
    // done the thing the marks describe, and taking them away because a
    // different tool is armed would punish the route that worked.
    // ★★★ **Or View ▸ Show points is on**, as of 2026-08-28 — a third disjunct
    // beside the two above, and added the same way and for the same reason the
    // second was.
    //
    // The command was registered, drawn and inert for the life of the project
    // behind a reason that said *"there is nothing for it to show"*. That was
    // true on 2026-08-15 and stopped being true four days later, when the
    // multi-node move landed with `overlay::draw_anchors` and with the
    // enumeration this function already calls. Re-derived on 2026-08-28 as one
    // of six stale blockers in eleven.
    //
    // ★★ What it gates and what it deliberately does NOT. It gates the draw at
    // its existing scope — the entered object, at the Part rung or the Node
    // rung. Its tooltip promises *"the editable points of every part of the
    // object you are working inside"*, which is what the existing scope
    // already means, and **widening it to every object on the page is a
    // separate decision that is the operator's**: `MAX_UNSELECTED_ANCHORS` is
    // 400 and has already fired blank once on his own SW41177, so "show all
    // the points" on a CAD sheet is a question about what to do with five
    // thousand of them rather than a flag.
    //
    // ⇒ Wiring the toggle to the honest scope is not a placeholder. It is the
    // difference between an operator having to *know a rung ladder exists* and
    // being able to ask.
    if !tool.is_node()
        && !doc.view.show_points
        && !matches!(
            selection.level(),
            SelectionLevel::Part | SelectionLevel::Node
        )
    {
        // ★ Silent, and it is the ONE return in this function that is. Past
        // this point somebody has asked for points — the node tool is armed, or
        // Show points is on, or the operator descended a rung — and a request
        // that produces nothing owes an account of why (see `declined`). Above
        // it nobody asked, and this is the branch taken on essentially every
        // frame the program runs; a line here would become the trace's largest
        // single contributor and would say only *"the operator is not editing
        // nodes"*.
        return;
    }
    // ★★★ **Why every return below this line writes a reason.**
    //
    // `overlay::draw_anchors`' census answers *how many points were there*.
    // These answer the question before it — *did the enumeration get far enough
    // to have a count at all* — and without them the two are the same silence.
    //
    // The four driven checks that read anchors (`tool_row`'s two, `multi_node`,
    // `bezier_handle`) all begin by asking whether `canvas-anchors` appeared,
    // and every one of them has to guess when it did not. On the sweep of
    // 2026-08-29 two guessed *"the program is broken"* and two guessed *"the
    // aim is wrong"*, on the same fixture at the same `--doc-point`. A reason
    // token settles that in the trace instead of in four checks' prose:
    //
    // | reason | what it means | what it is about |
    // |---|---|---|
    // | `nothing-selected` | Show points is on and the selection is empty | the driver: click something first |
    // | `not-entered` | a rung above Part, with nothing asking to see points | the driver, or the tool never armed |
    // | `other-page` | the entered object is on a page this call is not painting | neither; continuous view, normal |
    // | `leaf-in-form-xobject` | the target is inside a form XObject, whose geometry is not writable | the aim: pick a page-level object |
    // | `no-page` / `no-provider` | the document or its decomposition is not available this frame | the program, or a load still in flight |
    //
    // ★ Deliberately NOT named `canvas-anchors …`.
    // `tools/gates/check-trace-names.py` compares FIRST TOKENS, and a harness
    // asking `last("canvas-anchors")` must never be handed one of these by
    // accident — that caller reads `total=`, `selected=` and
    // `unselected_drawn=`, none of which is here. The suffix is the whole guard,
    // and it is the same convention the gate enforces against `vector_edit`'s
    // funnel labels.
    fn declined(reason: &'static str) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("canvas-anchors-declined reason={reason}")
        });
    }
    // ★★★ **The Object rung is reachable through `show_points` and through
    // nothing else**, and getting this wrong shipped an inert toggle for about
    // ten minutes.
    //
    // `entered_object()` answers `None` at the Object rung *by construction* —
    // "entered" means the operator descended. So the first wiring of
    // `view.show_points` added a disjunct to the guard above and then fell out
    // here on every ordinary selection: the toggle switched on, the trace said
    // `view-chrome ShowPoints on=true`, and **not one anchor was drawn**.
    //
    // That is the exact defect the command was being wired to remove — a
    // control an operator can see, aim at and press with no effect — and it was
    // caught by asking *"what does this actually change on screen?"* rather
    // than by any test. Nothing in the suite could have: the toggle's own
    // assertions are that it registers, renders pressed and reaches
    // `ViewState`, and all three passed.
    //
    // ⇒ With the toggle on, the **selected** object is the subject. That is
    // what its tooltip promises — *"the editable points of every part of the
    // object you are working inside"* — and it is bounded by the same
    // `MAX_UNSELECTED_ANCHORS` cap the descent path already lives under.
    let entered = match selection.entered_object() {
        Some(entered) => entered,
        None if doc.view.show_points => {
            let Some(first) = selection.outlines().first().map(|(sel, _)| *sel) else {
                return declined("nothing-selected");
            };
            first
        }
        None => return declined("not-entered"),
    };
    if entered.page != page_index {
        return declined("other-page");
    }
    // ★★★ **A target inside a form XObject draws its anchors too, as of
    // 2026-09-01** — `OPERATOR_REQUESTS.md` O70.
    //
    // This declined with `leaf-in-form-xobject`, and the reason was exactly
    // right at the time: *"anchor dots are grab targets for a node drag, and a
    // leaf's geometry cannot be written, so drawing them would offer a gesture
    // that must then refuse — the placeholder failure R9 forbids, in its most
    // misleading form."*
    //
    // Both halves of that changed together, which is the only order in which
    // either should have: `pdfcer-core` Pass 188.0 shipped the node verbs for a
    // leaf, and `provider::geometry` answers where its anchors are. The dots
    // are now grab targets for a gesture that commits.
    let subject = entered.object;
    let Some(page) = doc.pages.get(page_index) else {
        return declined("no-page");
    };
    let Some(provider) = doc.page_objects() else {
        return declined("no-provider");
    };
    // ★★ **The entered SUBPATH's anchors, not the object's** — and this is the
    // difference between a usable feature and a decoration.
    //
    // The first version drew the whole object's, with a 400-anchor cap above
    // which the unselected ones were suppressed. Driving it against
    // `SW41177.pdf` produced `canvas-anchors total=4972`: one object on this
    // operator's own drawing carries five thousand anchors, so the cap fired,
    // nothing unselected drew, and the operator had **no way to see where any
    // anchor was** — on precisely the documents the feature exists for.
    //
    // A cap that suppresses the answer on the documents that need it is not a
    // performance guard, it is the feature not working. The right scope was
    // there all along and is also the semantically correct one: the operator
    // descended into a *subpath*, and its anchors are what they may pick. A
    // subpath is tens of anchors where an object is thousands, so the cap
    // becomes a backstop rather than the normal case.
    //
    // `subpath_node_points` returns OBJECT-scoped indices, which is what the
    // selection and `move_nodes` both speak — the offset arithmetic lives in
    // the provider, in one place, exactly so callers like this one cannot get
    // it subtly wrong.
    //
    // Converted to canvas space HERE, by the one function entitled to do it,
    // and the `Ref` is dropped before anything is painted.
    let anchors = match entered.subpath {
        Some(subpath) => provider.subpath_node_points_of(subject, subpath),
        // No part entered yet — the Node rung is unreachable from here, and the
        // object's whole anchor list is the honest answer to "what could you
        // descend into". The cap still applies and still fires on a CAD object,
        // which is correct: at the Part rung the operator's subject is the
        // subpath, and five thousand dots would be noise rather than an answer.
        None => provider.object_node_points_of(subject),
    };
    // ★★ **The cap is disclosed when it fires, since 2026-08-28.**
    //
    // `overlay::draw_anchors` draws nothing unselected past
    // `MAX_UNSELECTED_ANCHORS`, which is correct — five thousand dots is noise
    // rather than an answer — and it means **Show points does nothing visible
    // on exactly the drawings this program is for**. A 5,000-node CAD path
    // toggled on and off looks identical, and an operator would report the
    // control as broken.
    //
    // Rule 4's half that survives: *an inference the operator cannot see still
    // owes an off-canvas report.* The canvas is not marked; the status bar
    // says the number and why.
    //
    // ★ Only when the operator ASKED — `show_points` on — and not on the
    // descent path, where the cap has always fired silently and where the
    // operator's subject is the subpath they entered rather than the whole
    // object.
    //
    // ★★★ **…AT EVERY RUNG, since 2026-08-31** — `OPERATOR_REQUESTS.md` O69.
    // The note above said "not on the descent path, where the cap has always
    // fired silently" — which described the defect and treated it as a
    // decision. The Points tool puts the selection at the PART rung, so the
    // one route the operator was reporting was the one route excluded: he
    // armed the tool, clicked a dense contour, and got no dots and no
    // sentence. A limit reported as an absence reads as a broken program.
    //
    // ★ Two sentences, because the remedy differs by rung. At the Object rung
    // "descend into a part" is right; at the Part rung there is nothing below
    // a subpath and the remedy is to zoom, which the viewport cull shipped in
    // the same commit made true.
    //
    // ★★ `show_points` is no longer required at an inner rung. It gates
    // whether the operator ASKED to see an object's points, which is the right
    // question for the Object rung and the wrong one for the Points tool —
    // arming that tool IS the ask.
    let capped = anchors.len() > crate::canvas::overlay::MAX_UNSELECTED_ANCHORS;
    let at_object_rung = selection.entered_object().is_none();
    if capped && (doc.view.show_points || !at_object_rung) {
        let note = if at_object_rung {
            crate::text::status::too_many_anchors(
                anchors.len(),
                crate::canvas::overlay::MAX_UNSELECTED_ANCHORS,
            )
        } else {
            crate::text::status::too_many_anchors_in_part(
                anchors.len(),
                crate::canvas::overlay::MAX_UNSELECTED_ANCHORS,
            )
        };
        crate::app::actions::record_note(doc.edit_epoch, note);
    }
    let points: Vec<(usize, egui::Pos2)> = anchors
        .into_iter()
        .filter_map(|(index, p)| {
            crate::viewer::pdf_space_to_canvas(egui::pos2(p.x as f32, p.y as f32), page)
                .map(|c| (index, c))
        })
        .collect();
    drop(provider);

    let selected: std::collections::BTreeSet<usize> = selection
        .selected_nodes_on(page_index, entered.object)
        .into_iter()
        .collect();
    overlay::draw_anchors(painter, ui.visuals(), map, &points, &selected);

    // ★★ The handles, and the in-flight one moved to the pointer.
    //
    // Read from the same provider borrow's data, converted the same way, and
    // then OVERRIDDEN for the handle being dragged — because the decomposition
    // still holds its pre-drag position and will until the release commits.
    let Some(provider) = doc.page_objects() else {
        return;
    };
    let mut handles = crate::canvas::handledrag::visible(selection, &provider, page, page_index);
    drop(provider);
    if let Some((node, side, at)) = drag {
        for h in &mut handles {
            if h.0 == node && h.1 == side {
                h.2 = at;
            }
        }
    }
    overlay::draw_handles(painter, ui.visuals(), map, &handles, &points);
}
