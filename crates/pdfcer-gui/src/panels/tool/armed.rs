//! # `panels::tool::armed` — what is armed, how far through the gesture is,
//! and how to put it down
//!
//! The frame every armed tool shares, and the per-family bodies inside it.
//!
//! ## ★ The frame, and why its order is not interchangeable
//!
//! | # | row | kind | present |
//! |---|---|---|---|
//! | 1 | **Identity** — the tool's name, its chord, its ribbon home | STATUS | always |
//! | 2 | **Stage** — one fixed slot: the instruction before a gesture, the live stage during one | STATUS | always |
//! | 3 | **Options** | OPTIONS | per tool; several families have none |
//! | 3b | **Points in this measurement** — the circular fit's set, one removable row each | LIST | the radius/diameter tool only |
//! | 4 | **Put the tool down** | verb | always |
//!
//! **Identity first** because it is the literal complaint — *"no side bar area
//! showing what tool is active"* — and because it is the only row true in every
//! state of every tool.
//!
//! **Stage second, and it is the volatile row.** A row that changes must sit
//! where its change cannot move anything the operator is aiming at. Put an
//! option control above it and every option shifts vertically each time a
//! vertex lands — a control that moves is a control you cannot aim at, which is
//! this project's layout defect wearing yet another set of clothes.
//!
//! **The stage is one fixed slot that is never empty and is never a
//! placeholder.** It holds the instruction when idle and the stage when live —
//! same slot, same height, no *"nothing in progress"* line. That single choice
//! is what makes the armed frame R9-clean without a single greyed control.
//!
//! ## ★★ The identity row reads the REGISTRY, never a string of its own
//!
//! `MenuHost::label` and `MenuHost::chord`. A second copy of a label compiles,
//! reads identically the day it is written, and drifts the first time either is
//! reworded — invisibly, because nothing renders both at once. `NO_SURFACE.md`
//! §1 records that failure with a colour and, worse, records the test that
//! failed to catch it: it *"asserted the literal triple against a function
//! returning the literal triple. Two copies of one constant cannot disagree."*
//!
//! The chord comes from the operator's own keymap for the same reason. A panel
//! that hard-coded `Ctrl+E` would be telling somebody to press a key their
//! manifest may not bind — and a chord that does not work reads as the
//! *feature* not working, which is exactly the report that produced this panel.
//!
//! ## What is deliberately absent from the armed frame
//!
//! - **The markup pen's colour and width.** They belong here. `Panel::show`
//!   cannot reach the pen — see [`super`]'s closing section — and a swatch that
//!   accepts a click and discards it is the control `panels::properties`
//!   refused to ship.
//! - **Any sibling kind.** Arming Rectangle shows Rectangle. An "or try
//!   Ellipse" row is the second ribbon.
//! - **Anything about a PLACED annotation.** This panel is about the *next*
//!   gesture; `panels::properties` and the Format tab are about the placed
//!   thing, and `RIBBON_IA.md` §5.5 draws that line explicitly.

use egui::Ui;

use crate::canvas::tool::CanvasTool;
use crate::shell::menus::MenuHost;
use crate::text::tool as t;

/// Draw the armed block.
pub(super) fn block(
    ui: &mut Ui,
    ctx: &egui::Context,
    doc: &crate::app::state::OpenDoc,
    tool: CanvasTool,
    host: Option<&MenuHost<'_>>,
) {
    ui.label(t::armed_heading());
    crate::diag::ui_rect(super::REGION_ARMED, ui.min_rect());

    identity(ui, tool, host);
    ui.add_space(4.0);
    stage(ui, ctx, doc, tool);
    ui.add_space(6.0);
    options(ui, ctx, tool);
    measure_points(ui, ctx, tool);
    put_down(ui, ctx);
}

/// Row 3 — the armed tool's options, for the tools that have any.
///
/// # ★ Exactly one family has options today, and the emptiness is the design
///
/// `super`'s header states it: the hand tool, the text sweep and **Edit text**
/// have none at all, and a panel shaped around options would render nothing for
/// them. This function is therefore mostly a `match` that does nothing, and
/// that is the honest shape — the alternative is a heading with nothing under
/// it, which is R9's placeholder.
///
/// **The markup pen is still absent**, and its absence is a plumbing fact
/// rather than a design one: `canvas::markup::pen::Pen` is a field on
/// `PdfcerApp` and a panel body cannot reach it. `canvas::textedit::pen` lives
/// in `egui::Memory` precisely so this one could be built today — see its
/// header, which argues the difference.
fn options(ui: &mut Ui, ctx: &egui::Context, tool: CanvasTool) {
    // ★★★ **`CanvasTool::Select` CANNOT REACH HERE**, and a branch for it was
    // written here on 2026-08-28 and was dead.
    //
    // This function is called only from [`block`], and `super::body` calls
    // `block` only in its `else` arm — the one entered when something IS armed.
    // Select is this panel's **idle** state; its options are drawn beside the
    // tool list, where `super::body` now calls [`scale_switches`] directly.
    //
    // ⇒ Recorded rather than silently deleted, because the mistake is one a
    // reader of this file alone cannot see: `options` takes a `CanvasTool` and
    // matches on it, so handling every variant looks obviously right from in
    // here. The constraint lives one file away, in a branch this function has
    // no way to mention.
    // ★ **Add only, not Edit**, and the distinction is the point of the whole
    // module. `TextEditKind::Add` writes a NEW run, so a face, a size and a
    // colour are exactly what it needs. `TextEditKind::Edit` replaces the words
    // inside a run that already has all three, and pdfcer cannot restyle a run
    // it did not write — showing these controls there would offer a change the
    // commit silently discards.
    if tool != CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Add) {
        return;
    }
    use crate::canvas::textedit::pen;
    let mut current = pen::read(ctx);
    let before = current;

    ui.label(t::text_pen_heading());
    crate::diag::ui_rect(super::REGION_TEXT_PEN, ui.min_rect());

    ui.horizontal_wrapped(|ui| {
        ui.label(t::text_pen_font_label());
        egui::ComboBox::from_id_salt("tool-text-pen-font")
            .selected_text(t::text_pen_font_name(current.face))
            .show_ui(ui, |ui| {
                // ★ `pen::FACES`, not a hand-written list. Its own test asserts
                // all fourteen are offered exactly once — a list that quietly
                // held thirteen would be a face an operator could never reach,
                // with no error anywhere.
                for face in pen::FACES.iter().copied() {
                    ui.selectable_value(&mut current.face, face, t::text_pen_font_name(face));
                }
            });
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(t::text_pen_size_label());
        ui.add(
            egui::DragValue::new(&mut current.size_pt)
                // The store's own bounds, not a local pair of literals — the
                // rule `dialogs::settings`' sliders follow, and for the same
                // reason: a control narrower than what the value accepts
                // silently rewrites a setting the operator never touched.
                .range(pen::MIN_SIZE_PT..=pen::MAX_SIZE_PT)
                .speed(0.5)
                .suffix(t::text_pen_size_suffix()),
        );
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(t::text_pen_colour_label());
        ui.color_edit_button_srgb(&mut current.colour);
    });
    ui.label(egui::RichText::new(t::text_pen_note()).small().weak());
    ui.add_space(6.0);

    // ★ Written back only when it CHANGED. `insert_temp` on every frame would
    // be harmless and would also make the trace line below fire sixty times a
    // second, which is the difference between a log a reader can use and one
    // they cannot.
    if current != before {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-pen face={:?} size={:.1} rgb={},{},{}",
                current.face,
                current.size_pt,
                current.colour[0],
                current.colour[1],
                current.colour[2]
            )
        });
        pen::store(ctx, current);
    }
}

/// Row 3b — **the radius/diameter tool's pick set, listed and removable.**
///
/// # ★★★ Why a list, when the points are drawn on the canvas already
///
/// `OPERATOR_REQUESTS.md` O107, in the operator's words: *"we should be able to
/// unselect points/clicked locations, and it should have a box in the side
/// panel showing what is part of our selection … clicking on a point or
/// location listed should allow us to remove it."*
///
/// The canvas markers say *where* the points are. They cannot say **how many**,
/// and on a dense CAD sheet — five thousand strokes on one A1 — a marker sitting
/// on top of a junction is not distinguishable from the junction. An operator
/// who has clicked five times and sees four markers has no way to find out
/// which fact is wrong. The list is the only surface that answers *"what is
/// actually in this fit?"*, and that question is the one behind the whole of
/// O105: he could not see that a single click had contributed six thousand
/// points.
///
/// # ★★ The row IS the remove control, and it says so before it is pressed
///
/// He asked for that literally, and it is also the conventional shape for a
/// short authored list. What it needs is disclosure, because the gesture does
/// **not** go through undo — a pick set is pre-commit state and never enters the
/// document's history. So every row carries
/// [`t::measure_point_remove_hint`] on hover, and the cost of a mistake is one
/// click on the canvas to put the point back.
///
/// # ★ It draws nothing at all when no circular tool is armed
///
/// R9. Not a greyed list, not an empty box with a heading: greying is reserved
/// for a *temporarily* unavailable control, and a pick list for a tool that is
/// not armed is not that. When the tool IS armed and the set is empty, one
/// sentence stands in for the list — an empty section under a heading reads as
/// a surface that failed to draw.
fn measure_points(ui: &mut Ui, ctx: &egui::Context, tool: CanvasTool) {
    use crate::canvas::measure::MeasureKind;

    if tool.measure_kind() != Some(MeasureKind::Circular) {
        return;
    }

    ui.label(t::measure_points_heading());
    crate::diag::ui_rect(super::REGION_MEASURE_POINTS, ui.min_rect());

    let Some(st) = crate::canvas::measure::read(ctx) else {
        ui.label(
            egui::RichText::new(t::measure_points_empty())
                .small()
                .weak(),
        );
        ui.add_space(6.0);
        return;
    };
    let points = st.circular.points();
    if points.is_empty() {
        ui.label(
            egui::RichText::new(t::measure_points_empty())
                .small()
                .weak(),
        );
        ui.add_space(6.0);
        return;
    }

    // ★★ The removal is applied AFTER the loop, never inside it.
    //
    // `st` is a copy read out of `egui::Memory`, and `points` borrows it. A
    // removal that mutated mid-iteration would shift every row below the one
    // pressed while the loop was still drawing them, which is the classic
    // one-frame mis-aim: the operator presses row 3, row 4 slides up under the
    // pointer, and the NEXT frame's press lands on something they did not
    // choose. Recording the index and acting once is what keeps a row's
    // position stable for the whole frame it is drawn in.
    let mut remove: Option<usize> = None;
    for (index, point) in points.iter().enumerate() {
        let label = t::measure_point_row(
            index + 1,
            t::measure_point_origin(point.origin),
            point.at.x,
            point.at.y,
        );
        let response = ui.add(egui::Button::new(egui::RichText::new(label).small()).frame(false));
        crate::diag::ui_rect(
            &format!("{}{index}", super::REGION_MEASURE_POINT_PREFIX),
            response.rect,
        );
        if response
            .on_hover_text(t::measure_point_remove_hint())
            .clicked()
        {
            remove = Some(index);
        }
    }
    if let Some(index) = remove {
        crate::canvas::measure::circular::remove_point(ctx, index);
    }
    ui.add_space(6.0);
}

/// The Select tool's option row — the three switches of `OPERATOR_REQUESTS.md`
/// **O51**.
///
/// ## ★★★ Why three, when Inkscape's parity set is one
///
/// Inkscape offers four toggles and only *Scale stroke width* has a PDF
/// equivalent — rounded corners, gradients and patterns have no annotation
/// analogue. The other two here come from PDF's own shape:
///
/// * **`/RD`, the inset distances**, which have no Inkscape counterpart and
///   scale by default. An inset **is** a length in the space being scaled;
///   leaving it fixed while `/Rect` doubles changes the proportions.
/// * **Allow the artwork to distort**, which exists because *no per-axis stroke
///   width exists* in PDF or in SVG. Under a non-uniform scale a carried
///   appearance's stroke becomes anisotropic and no `/BS /W` describes it, so
///   pdfcer refuses rather than silently producing an oval border — which is
///   what the parity reference does.
///
/// ## ★★ The order, which is by how often it is wanted
///
/// Stroke width first: it is the one he asked for by name and the one every
/// comparable program puts first. Insets second — a real switch nobody reaches
/// for weekly. The distortion escape last, because it is the one that makes the
/// result imperfect, and a control that degrades the output belongs after the
/// two that do not. The destructive-last rule from the context menus, applied
/// to a different kind of cost.
///
/// ## ★ Always drawn, never greyed
///
/// They are live with nothing selected and with a form field selected, and that
/// is deliberate: an operator sets a modifier **before** the gesture it
/// modifies. Greying them until an annotation happens to be selected would hide
/// the control exactly when somebody is deciding how to resize.
pub(super) fn scale_switches(ui: &mut Ui, ctx: &egui::Context) {
    let mut current = crate::canvas::scaling::read(ctx);
    let before = current;

    ui.label(t::scale_heading());
    crate::diag::ui_rect(super::REGION_SCALE_SWITCHES, ui.min_rect());

    // ★★★ **One published rect per switch**, not one for the block.
    //
    // A driven check aiming at "the options row" and then guessing which line
    // is the second checkbox would be encoding a layout — the same mistake
    // `field_menu` refuses to make about popup rows, and it goes wrong the same
    // way: silently, by ticking the wrong switch, the day a label wraps to two
    // lines at a narrower dock width.
    //
    // ⇒ `ui_rect` after each `checkbox`, from the response's own rect, so the
    // harness clicks what the operator clicks.
    let stroke = ui.checkbox(&mut current.scale_stroke_width, t::scale_stroke_label());
    crate::diag::ui_rect(super::REGION_SCALE_STROKE, stroke.rect);
    // ★ The `/RD` switch is spelled as an opt-OUT in the engine and in
    // `canvas::scaling`, and it is presented here as one too — *"keep"*, not
    // *"scale"*. An inverted label over an opt-out field is the single easiest
    // way to ship a control that does the opposite of what it says, and the
    // temptation is real because "scale the margins" reads better.
    let insets = ui.checkbox(&mut current.keep_rect_differences, t::scale_insets_label());
    crate::diag::ui_rect(super::REGION_SCALE_INSETS, insets.rect);
    let distort = ui.checkbox(&mut current.allow_distortion, t::scale_distort_label());
    crate::diag::ui_rect(super::REGION_SCALE_DISTORT, distort.rect);
    ui.label(egui::RichText::new(t::scale_note()).small().weak());
    ui.add_space(6.0);

    // ★ Written back only when it CHANGED — the text pen's rule above, for its
    // reason: an unconditional `insert_temp` is harmless and would make the
    // trace line fire sixty times a second.
    if current != before {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "resize-modifiers stroke={} keep_rd={} distort={}",
                current.scale_stroke_width, current.keep_rect_differences, current.allow_distortion
            )
        });
        crate::canvas::scaling::store(ctx, current);
    }
}

/// Row 1 — what is armed, and where it came from.
///
/// Falls back to nothing at all when the command is not registered or there is
/// no validated manifest, for [`super::idle`]'s reason: a name with no control
/// behind it is worse than no name.
fn identity(ui: &mut Ui, tool: CanvasTool, host: Option<&MenuHost<'_>>) {
    let Some(id) = command_for(tool) else {
        return;
    };
    let Some(host) = host else {
        return;
    };
    let Some(label) = host.label(id) else {
        return;
    };
    ui.label(label);
    let tab = tab_for(tool);
    ui.label(
        egui::RichText::new(t::row_home(tab, host.chord(id).as_deref()))
            .small()
            .weak(),
    );
}

/// Row 2 — the fixed slot: the instruction, or the stage of a live gesture.
///
/// # ★ One slot, two contents, never absent
///
/// The whole of this panel's stability. An operator's eye settles on this line
/// and the controls below it must not move when the line's *content* changes —
/// so the instruction and the live stage share it rather than the stage
/// appearing beneath the instruction.
fn stage(ui: &mut Ui, ctx: &egui::Context, doc: &crate::app::state::OpenDoc, tool: CanvasTool) {
    let text = match tool {
        CanvasTool::Select => return,
        // ★ The Tool panel teaches the armed gesture, and for a form field the
        // gesture is the whole feature: click for a default size, or drag for
        // an exact one. Saying so here is what stops an operator clicking once,
        // getting a standard-sized box, and concluding that dragging is not
        // offered.
        // ★★★ **The instruction has to live HERE while a placement is armed**,
        // because the window that asked for it is off screen —
        // `OPERATOR_REQUESTS.md` O66.
        //
        // Every other arm in this match is a convenience: the ribbon control is
        // still pressed, the tooltip is still hoverable, and the panel is
        // repeating what the operator can find elsewhere. This one is the ONLY
        // place the gesture and the way out are stated, because
        // `dialogs::placing` hides the requesting dialog for exactly as long as
        // the placement is pending. Deleting this line would strand an operator
        // who has forgotten what they armed.
        CanvasTool::Place(_) => {
            ui.label(egui::RichText::new(crate::text::placing::armed_instruction()).small());
            return;
        }
        CanvasTool::Form(kind) => {
            ui.label(egui::RichText::new(t::form_instruction()).small());
            ui.label(egui::RichText::new(t::form_kind_hint(kind)).small().weak());
            return;
        }
        CanvasTool::Node => {
            ui.label(egui::RichText::new(t::node_instruction()).small());
            ui.label(egui::RichText::new(t::node_shift()).small().weak());
            return;
        }
        CanvasTool::Hand => {
            ui.label(egui::RichText::new(t::hand_instruction()).small());
            ui.label(egui::RichText::new(t::hand_borrow()).small().weak());
            return;
        }
        CanvasTool::Text => {
            ui.label(egui::RichText::new(t::text_select_instruction()).small());
            // ★ Rendered only where it is TRUE. In Read and Review the select
            // tool already swept text, so arming this takes nothing away and
            // the sentence would be describing a change that did not happen.
            // Absent rather than reworded — R9 applied to a sentence.
            if crate::canvas::tool::capabilities(ctx).edit_content {
                ui.label(
                    egui::RichText::new(t::text_select_takes_the_press())
                        .small()
                        .weak(),
                );
            }
            return;
        }
        CanvasTool::Markup(kind) => {
            // The live count for a run of clicks, the instruction otherwise.
            // `vertex::read` answers `None` when nothing is in progress, which
            // is the idle case and gets the instruction — one slot, two
            // contents.
            match crate::canvas::markup::vertex::read(ctx) {
                Some(run) if run.kind == kind && run.in_progress() => {
                    t::vertices_placed(run.vertices.len())
                }
                _ => t::markup_instruction(kind).to_owned(),
            }
        }
        CanvasTool::TextAnnot(kind) => {
            ui.label(egui::RichText::new(t::text_annot_instruction(kind)).small());
            // ★★ The sentence that stops a working tool reading as broken.
            //
            // `CanvasTool` was split for exactly this: *"A markup band authors
            // on release, from geometry alone. These cannot: releasing produces
            // an empty box, and an empty box is not an annotation."* An
            // operator who drags one of these out, lets go, and sees nothing
            // land has met a release that authored nothing — which is the same
            // failure shape as the text-editing complaint that produced this
            // whole panel.
            ui.label(egui::RichText::new(t::text_annot_release()).small().weak());
            return;
        }
        CanvasTool::TextEdit(kind) => {
            // Live when there is a caret, the instruction before there is one.
            match crate::canvas::textedit::read(ctx) {
                Some(draft) if draft.kind == kind => t::text_edit_live().to_owned(),
                _ => t::text_edit_instruction(kind).to_owned(),
            }
        }
        // ★★ The perimeter tool reports its RUNNING TOTAL, and it is the one
        // measure tool that has to.
        //
        // Every other gesture here has a fixed arity, so the operator can see
        // how far through it they are by counting their own clicks. A perimeter
        // has no arity: after eight vertices around a building footprint there
        // is nothing on screen that says what has been measured so far, and the
        // operator asked for this number by name - *"it adds the distance of
        // all the segments together"*.
        //
        // ★ Formatted through the AUTHORING GROUP's scale and number format,
        // never as raw points. The whole of what he asked for is that this tool
        // behaves *"the same as the other dimensioning tools"*, and a live
        // readout in points beside a committed dimension in metres would be two
        // numbers for one measurement. `format_measurement` is the engine's own
        // function - the same one the committed label goes through - so the
        // running total and the final label cannot disagree about scale, unit,
        // precision or decimal marker.
        //
        // The RAW-page-units case is not hidden: `format_measurement` reports
        // it, and the group panel already discloses "no scale set". Showing
        // "735.37 pt" while tracing is honest, and it is exactly what the
        // committed dimension will print.
        CanvasTool::Measure(crate::canvas::measure::MeasureKind::Perimeter) => {
            perimeter_stage(ctx, doc)
        }
        // ★★★ The radius/diameter tool reports the SIZE IT HAS SO FAR, and
        // that is the operator's ask rather than a nicety.
        //
        // `OPERATOR_REQUESTS.md` O105: *"selecting more points around a hole
        // doesn't always get it to narrow down to the size of the hole."* An
        // operator adding points to a fit is watching a number converge, and
        // there was no number to watch — the circle was drawn on the canvas and
        // its value appeared only once the dimension had been placed. So the
        // tool could not be steered: every correction was a commit and an undo.
        CanvasTool::Measure(crate::canvas::measure::MeasureKind::Circular) => {
            circular_stage(ctx, doc)
        }
        CanvasTool::Measure(kind) => t::measure_instruction(kind).to_owned(),
    };
    ui.label(egui::RichText::new(text).small());
}

/// The perimeter tool's live sentence: the instruction before the first click,
/// the running total after it.
///
/// # Why the total is formatted through the group rather than shown in points
///
/// Because the operator's whole ask was that this tool behave *"the same as the
/// other dimensioning tools"*, and the other tools' output is read against the
/// group's scale. A live readout in points beside a committed dimension in
/// metres would be two numbers for one measurement, and the operator would have
/// to know which was which.
///
/// [`pdfcer_core::dimension::format_measurement`] is the ENGINE's own function -
/// the same one the committed label goes through - so the running total and the
/// final label cannot disagree about scale, unit, precision, fraction style or
/// decimal marker. Re-deriving any of that here would be a second formatter,
/// which is the failure `dialogs::insert_image` records at length.
///
/// Falls back to the instruction when the group cannot be read. That is the
/// honest answer rather than a number in the wrong units: a total whose scale
/// is unknown is not a total.
fn perimeter_stage(ctx: &egui::Context, doc: &crate::app::state::OpenDoc) -> String {
    let Some(st) = crate::canvas::measure::read(ctx) else {
        return t::measure_instruction(crate::canvas::measure::MeasureKind::Perimeter).to_owned();
    };
    let picked = st.perimeter.points().len();
    if picked == 0 {
        return t::measure_instruction(crate::canvas::measure::MeasureKind::Perimeter).to_owned();
    }
    let model = doc.session.dimension_model();
    let Some(group) = model.group(st.group) else {
        return t::measure_instruction(crate::canvas::measure::MeasureKind::Perimeter).to_owned();
    };
    let shown = pdfcer_core::dimension::format_measurement(
        st.perimeter.length_points(),
        group.scale,
        group.format,
    );
    t::measure_perimeter_live(picked, &shown.text)
}

/// The radius/diameter tool's live sentence: the instruction before the first
/// click, the count and the current fit after it.
///
/// # ★★ The measurement goes through the ENGINE's formatter, like every other
///
/// [`pdfcer_core::dimension::format_measurement`] and the authoring group's own
/// scale, exactly as `perimeter_stage` does and for the identical reason: a
/// live readout in points beside a committed dimension in millimetres would be
/// two numbers for one measurement, and the operator would have to know which
/// was which.
///
/// ★ **Radius or diameter follows the pick set's own display toggle**, so the
/// number the panel shows is the number the placed dimension will show. A panel
/// that always reported the radius would disagree with a committed diameter
/// label by a factor of two, silently, and the operator would have no way to
/// tell which of the two was the tool being wrong.
///
/// Falls back to the count-only sentence when the group cannot be read. That is
/// the honest answer rather than a number in unknown units: a measurement whose
/// scale is unknown is not a measurement.
fn circular_stage(ctx: &egui::Context, doc: &crate::app::state::OpenDoc) -> String {
    use crate::canvas::measure::MeasureKind;

    let Some(st) = crate::canvas::measure::read(ctx) else {
        return t::measure_instruction(MeasureKind::Circular).to_owned();
    };
    let picked = st.circular.point_count();
    if picked == 0 {
        return t::measure_instruction(MeasureKind::Circular).to_owned();
    }
    let Some(fit) = st.circular.fit() else {
        return t::measure_circular_needs_more(picked);
    };
    let model = doc.session.dimension_model();
    let Some(group) = model.group(st.group) else {
        return t::measure_circular_needs_more(picked);
    };
    let value = if st.circular.show_diameter {
        fit.radius * 2.0
    } else {
        fit.radius
    };
    let shown = pdfcer_core::dimension::format_measurement(value, group.scale, group.format);
    t::measure_circular_live(picked, &shown.text)
}

/// Row 4 — put the tool down.
///
/// # ★ It is NOT a Close button, and the distinction is one an operator can
/// lose a panel to
///
/// [`super::dimension_groups`]' rule stands: a panel has no Close button,
/// because the dock tab carries one. This is a different verb — it retires the
/// **tool** and leaves the panel exactly where it was — and it says so in its
/// own label rather than relying on position. Two controls a click apart that
/// both read as *closing something* is how somebody shuts a surface they
/// wanted.
///
/// # ★ It writes the armed tool DIRECTLY, and that is the house idiom rather
/// than an exception
///
/// The armed tool is not document state. It lives in `egui::Memory` beside the
/// gesture machine, `canvas::tool`'s own header argues why, and **every other
/// retirement path in the crate writes it the same way**: `disarm_markup`,
/// `disarm_measure` and `retire_forbidden` are all one `select(ctx,
/// CanvasTool::Select)`. The Dimension-groups panel writes the measure tool's
/// authoring group from a panel body on the identical argument — *"it changes
/// no document; it says where the next gesture's product will go."*
///
/// So this is not the funnel being bypassed. `crate::app::actions`' invariant
/// is about **document** state, and there is none here: putting a tool down
/// contributes nothing to the undo log and has nothing to order against.
/// Routing it through an `Action` would add a variant `apply` could only answer
/// by writing the same memory slot, which is the funnel pointing the wrong way
/// — the argument `crate::dialogs`' header makes about printing.
///
/// Returns to `Select` rather than to `Hand`, matching every other retirement
/// path: `Select` is the enum's `#[default]`, and a control that silently
/// swapped in a *different* tool would be a second surprise on top of the one
/// the operator asked for.
fn put_down(ui: &mut Ui, ctx: &egui::Context) {
    let response = ui.button(t::put_down_button());
    // ★ The hint names the key, and the key is `Escape` — which this build
    // handles in `canvas::keys` rather than through the manifest keymap, so
    // `MenuHost::chord` would answer `None` for it. Written here rather than
    // derived, and that is a deliberate exception to this panel's
    // read-the-registry rule with a narrow justification: Escape is not a
    // *binding*, it is a rung on a ladder (`canvas::keys`), and a keymap
    // lookup for it would be asking the wrong question rather than getting an
    // unlucky answer.
    let response = response.on_hover_text(t::put_down_hint());
    if response.clicked() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "tool-panel-put-down".to_owned()
        });
        crate::canvas::tool::select(ctx, CanvasTool::Select);
    }
}

/// The command that arms `tool`, if one does.
///
/// # ★ Derived from the existing id maps, never written a second time
///
/// `shell::commands::markup_command` and `measure_for_command`'s inverse are
/// the single binding between an id and a kind, exactly as
/// `Panel::from_command_id` is for panels. Re-listing them here would be a
/// second table to keep in step, and the failure when it drifted would be an
/// identity row naming the wrong tool — which is the one thing this panel
/// exists to get right.
fn command_for(tool: CanvasTool) -> Option<&'static str> {
    match tool {
        // ui-text-exempt: command ids, never displayed
        CanvasTool::Select => Some("view.tool_select"),
        CanvasTool::Node => Some("view.tool_node"),
        CanvasTool::Hand => Some("view.tool_hand"),
        CanvasTool::Text => Some("view.tool_text"),
        CanvasTool::Markup(kind) => Some(crate::shell::commands::markup_command(kind)),
        // ★ Each kind names its own command, which is what lets the Tool panel
        // show the armed field type as pressed on the ribbon. The mapping lives
        // on the kind rather than here so the two cannot drift.
        CanvasTool::Form(kind) => Some(kind.command_id()),
        // ★ **None, for `MeasureKind::Scale`'s reason spelled out again** — a
        // placement is armed from inside a dialog and has no ribbon control to
        // name, so the identity row is absent rather than blank. Written as its
        // own arm rather than folded into a `_` so that a second `PlaceKind`
        // has to be ruled on rather than inheriting this silently.
        CanvasTool::Place(_) => None,
        // ★ The empty string is `MeasureKind::Scale`'s id, and it is not a
        // command — that kind is armed from inside the Set-scale window and
        // deliberately maps to nothing. `MenuHost::label` would answer `None`
        // for it anyway, but filtering here says the reason: there is no
        // ribbon control to name, so the identity row is absent rather than
        // blank.
        CanvasTool::Measure(kind) => {
            let id = crate::shell::commands::measure_command(kind);
            (!id.is_empty()).then_some(id)
        }
        // ★ `TextAnnotKind::command` rather than a table here. It is the
        // single binding between one of these kinds and its id — the same
        // shape `markup_command` and `measure_command` have — and
        // `TextAnnotKind::from_command` is its inverse, which is what the
        // dispatcher uses. A third spelling in this file would be the second
        // table this function's own doc comment refuses.
        CanvasTool::TextAnnot(kind) => Some(kind.command()),
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Edit) => Some("edit.text"),
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Add) => Some("edit.add_text"),
    }
}

/// The ribbon tab `tool`'s command lives on.
///
/// The half of the identity row that teaches the ribbon. It is a `match` on the
/// tool rather than a lookup from the manifest because a manifest lookup would
/// answer *"which tab declares this item"* — which is the same answer today and
/// would silently follow a customized manifest that moved the control, telling
/// an operator to look somewhere their ribbon does not have. This says where
/// `RIBBON_IA.md` puts it, which is the promise the built-in shell keeps.
fn tab_for(tool: CanvasTool) -> &'static str {
    match tool {
        CanvasTool::Select | CanvasTool::Node | CanvasTool::Hand | CanvasTool::Text => {
            crate::text::ribbon::tab_view()
        }
        CanvasTool::Markup(_) | CanvasTool::TextAnnot(_) => crate::text::ribbon::tab_markup(),
        CanvasTool::Measure(_) => crate::text::ribbon::tab_measure(),
        // ★ A placement is armed from inside a dialog, so there is no tab an
        // operator could go to and press it again. `Edit` is named anyway
        // because that is where the command that OPENED the window lives
        // (`edit.insert_image`), which is the honest answer to "where did this
        // come from" — and `command_for` returns `None` for the same tool, so
        // the identity row is absent and this string is only reached by the
        // sentence that names the tab in prose.
        CanvasTool::TextEdit(_) | CanvasTool::Form(_) | CanvasTool::Place(_) => {
            crate::text::ribbon::tab_edit()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::markup::MarkupKind;
    use crate::canvas::textedit::TextEditKind;

    /// ★ Every armable tool names a command, and `Select` names none.
    ///
    /// ★★ **Rewritten 2026-08-19, and the sentence it replaces is the finding.**
    ///
    /// It used to read: *"`Select` is the unarmed state … so a command here
    /// would be a claim that there is a control called Select, which there is
    /// not."* True at the time, and the reason it was true is the defect: this
    /// shell had a Hand toggle and a Text toggle and **no Select control**, so
    /// the two read as unrelated switches rather than as members of a set, and
    /// there was nowhere for a third and fourth to join.
    ///
    /// A tool palette is the most conventional object in this product class.
    /// Not having one is the invention — which is exactly what the operator
    /// said on 2026-08-19: *"a lot of ideas are getting invented instead of just
    /// using the … most common method expected."*
    ///
    /// So every tool names a command now, including Select, and the assertion
    /// is the stronger one: **no tool is unnameable.**
    #[test]
    fn every_tool_names_its_command() {
        for tool in [
            CanvasTool::Select,
            CanvasTool::Node,
            CanvasTool::Hand,
            CanvasTool::Text,
            CanvasTool::Markup(MarkupKind::Rectangle),
            CanvasTool::Markup(MarkupKind::Cloud),
            CanvasTool::TextEdit(TextEditKind::Edit),
            CanvasTool::TextEdit(TextEditKind::Add),
        ] {
            assert!(
                command_for(tool).is_some(),
                "{tool:?} is armable and names no command, so the identity row would be \
                 blank for a tool the operator has in their hand"
            );
        }
    }

    /// The two text tools name **different** commands.
    ///
    /// The pair the operator confuses. An identity row that named the same
    /// command for both would make the panel unable to tell him which one he
    /// had armed — which is the state he is already in, and the state this
    /// panel exists to end.
    #[test]
    fn the_two_text_tools_are_told_apart() {
        assert_ne!(
            command_for(CanvasTool::TextEdit(TextEditKind::Edit)),
            command_for(CanvasTool::TextEdit(TextEditKind::Add))
        );
    }

    /// Every markup kind's command is the one `shell::commands` owns.
    ///
    /// Asserted as a **relation** to the id map rather than against literals,
    /// which is the whole point: two copies of one constant cannot disagree,
    /// so a test written against literals would pass on a build where this
    /// module had its own stale table.
    #[test]
    fn the_markup_identity_reads_the_id_map() {
        for kind in MarkupKind::ALL.iter().copied() {
            assert_eq!(
                command_for(CanvasTool::Markup(kind)),
                Some(crate::shell::commands::markup_command(kind))
            );
        }
    }
}
