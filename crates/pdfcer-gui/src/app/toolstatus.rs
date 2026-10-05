//! # `app::toolstatus` — the one line that replaced the Tool panel
//!
//! `OPERATOR_REQUESTS.md` **O123**, verbatim:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/toolstatus.md`.

use egui::Ui;

use crate::app::state::OpenDoc;
use crate::canvas::measure::MeasureKind;
use crate::canvas::tool::CanvasTool;
use crate::shell::menus::MenuHost;
use crate::text::tool as t;
use crate::text::toolstatus as ts;

/// The height the right dock reserves for the strip, in points.
pub const BANNER_HEIGHT_PTS: f32 = 26.0;

/// The region the strip publishes when it has drawn.
pub const REGION: &str = "toolstatus"; // ui-text-exempt: trace region name, never displayed
/// The region the *Put this tool down* button publishes.
pub const REGION_PUT_DOWN: &str = "toolstatus.put_down"; // ui-text-exempt: trace region name, never displayed

/// Draw the strip into the banner `Ui` the dock reserved.
pub fn banner(ui: &mut Ui, doc: Option<&OpenDoc>, host: Option<&MenuHost<'_>>) {
    let Some(doc) = doc else {
        return;
    };
    let ctx = ui.ctx().clone();
    let armed = crate::canvas::tool::selected(&ctx);
    let (primary, secondary) = sentence(&ctx, doc, armed);
    let line = match name_of(armed, host) {
        Some(name) => ts::status_line(name, &primary),
        None => primary.clone(),
    };
    // The hover carries what the line could not: the second sentence of the
    // stages that have one, then the note saying where the controls went.
    let hover = match &secondary {
        Some(extra) => format!("{line}\n\n{extra}"),
        None => line.clone(),
    };

    // **The button is allocated BEFORE the sentence, and that order is the
    // whole of the layout.**
    //
    // `Label::truncate` fills the width it is given, so a sentence laid out
    // first takes the entire strip and the button is pushed off the end —
    // present in the tree, rectangle and all, and unreachable. That is this
    // project's recorded failure shape, and it would have arrived in the
    // surface built to replace the panel it arrived in last time.
    //
    // A right-to-left outer layout places the button against the right edge and
    // hands the remainder back; the inner left-to-right layout puts the
    // sentence in that remainder, reading normally.
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if armed != CanvasTool::Select {
            put_down(ui, &ctx);
        }
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            // `truncate`, never `wrap`. The strip's height is a constant the
            // dock has already taken off the side; a second line would be drawn
            // over the first stack's tab bar and clipped away, which reads as a
            // rendering fault. The full text is on hover, which is the same
            // elide-and-defer discipline [`crate::app::status::disclosure`]
            // applies to the bar's single row and for the same reason.
            let response = ui
                .add(egui::Label::new(egui::RichText::new(&line).small()).truncate())
                .on_hover_text(&hover)
                .on_hover_text(ts::status_tooltip());
            crate::diag::ui_rect_visible(REGION, response.rect, ui.clip_rect());
        });
    });
}

/// The *Put this tool down* button, moved verbatim from the armed block.
fn put_down(ui: &mut Ui, ctx: &egui::Context) {
    let response = ui
        .button(t::put_down_button())
        .on_hover_text(t::put_down_hint());
    crate::diag::ui_rect_visible(REGION_PUT_DOWN, response.rect, ui.clip_rect());
    if response.clicked() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "tool-panel-put-down".to_owned()
        });
        crate::canvas::tool::select(ctx, CanvasTool::Select);
    }
}

/// The armed tool's name, read from the command registry.
fn name_of<'a>(tool: CanvasTool, host: Option<&'a MenuHost<'_>>) -> Option<&'a str> {
    host?.label(command_for(tool)?)
}

/// The one sentence, plus whatever the old stage said on a **second** line.
fn sentence(ctx: &egui::Context, doc: &OpenDoc, tool: CanvasTool) -> (String, Option<String>) {
    match tool {
        // The resting state, and the one sentence in the application that says
        // what a plain drag MEANS in this mode — it marquees objects in Edit
        // and sweeps text in Read, decided by `canvas::textsel::takes_the_press`
        // reading the mode. Block A of the old panel carried it; nothing else
        // ever has.
        CanvasTool::Select => {
            let caps = crate::canvas::tool::capabilities(ctx);
            let line = if caps.edit_content {
                t::pointer_edit()
            } else {
                t::pointer_reading()
            };
            (line.to_owned(), None)
        }
        CanvasTool::Node => (
            t::node_instruction().to_owned(),
            Some(t::node_shift().to_owned()),
        ),
        CanvasTool::Hand => (
            t::hand_instruction().to_owned(),
            Some(t::hand_borrow().to_owned()),
        ),
        CanvasTool::Text => {
            // The second sentence is rendered only where it is TRUE. In Read
            // and Review the select tool already swept text, so arming this
            // takes nothing away and the sentence would be describing a change
            // that did not happen. Absent rather than reworded — R9 applied to
            // a sentence, and carried across unchanged from the armed block.
            let extra = crate::canvas::tool::capabilities(ctx)
                .edit_content
                .then(|| t::text_select_takes_the_press().to_owned());
            (t::text_select_instruction().to_owned(), extra)
        }
        CanvasTool::Form(kind) => (
            t::form_instruction().to_owned(),
            Some(t::form_kind_hint(kind).to_owned()),
        ),
        // O66 — the ONLY surface that states this gesture and its way out,
        // because the window that asked for the placement has hidden itself.
        CanvasTool::Place(_) => (crate::text::placing::armed_instruction().to_owned(), None),
        CanvasTool::Snapshot => (
            t::snapshot_instruction().to_owned(),
            Some(t::snapshot_stays().to_owned()),
        ),
        CanvasTool::InkPicker => crate::canvas::inkpick::sentence(ctx, doc),
        CanvasTool::TextAnnot(kind) => (
            t::text_annot_instruction(kind).to_owned(),
            Some(t::text_annot_release().to_owned()),
        ),
        CanvasTool::TextEdit(kind) => {
            // Live when there is a caret, the instruction before there is one.
            let live = matches!(crate::canvas::textedit::read(ctx), Some(d) if d.kind == kind);
            let line = if live {
                t::text_edit_live().to_owned()
            } else {
                t::text_edit_instruction(kind).to_owned()
            };
            (line, None)
        }
        CanvasTool::Markup(kind) => {
            let line = match crate::canvas::markup::vertex::read(ctx) {
                Some(run) if run.kind == kind && run.in_progress() => {
                    t::vertices_placed(run.vertices.len())
                }
                _ => t::markup_instruction(kind).to_owned(),
            };
            (line, None)
        }
        CanvasTool::Measure(MeasureKind::Perimeter) => (perimeter_stage(ctx, doc), None),
        CanvasTool::Measure(MeasureKind::Area) => (area_stage(ctx, doc), None),
        CanvasTool::Measure(MeasureKind::Circular) => (circular_stage(ctx, doc), None),
        CanvasTool::Measure(kind) => (t::measure_instruction(kind).to_owned(), None),
    }
}

/// The perimeter tool's live sentence: the instruction before the first click,
/// the running total after it.
fn perimeter_stage(ctx: &egui::Context, doc: &OpenDoc) -> String {
    let instruction = || t::measure_instruction(MeasureKind::Perimeter).to_owned();
    let Some(st) = crate::canvas::measure::read(ctx) else {
        return instruction();
    };
    let picked = st.perimeter.points().len();
    if picked == 0 {
        return instruction();
    }
    let model = doc.session.dimension_model();
    let Some(group) = model.group(st.group) else {
        return instruction();
    };
    let shown = pdfcer_core::dimension::format_measurement(
        st.perimeter.length_points(),
        group.scale,
        group.format,
    );
    t::measure_perimeter_live(picked, &shown.text)
}

/// The area tool's live sentence: the instruction before three corners, the
/// enclosed area from then on.
fn area_stage(ctx: &egui::Context, doc: &OpenDoc) -> String {
    let instruction = || t::measure_instruction(MeasureKind::Area).to_owned();
    let Some(st) = crate::canvas::measure::read(ctx) else {
        return instruction();
    };
    let picked = st.perimeter.points().len();
    if picked < 3 {
        return instruction();
    }
    let model = doc.session.dimension_model();
    let Some(group) = model.group(st.group) else {
        return instruction();
    };
    let shown = pdfcer_core::dimension::format_area_measurement(
        st.perimeter.area_points(None),
        group.scale,
        group.format,
    );
    t::measure_area_live(picked, &shown.text)
}

/// The radius/diameter tool's live sentence: the instruction before the first
/// click, the count and the current fit after it.
fn circular_stage(ctx: &egui::Context, doc: &OpenDoc) -> String {
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

/// The command that arms `tool`, if one does.
fn command_for(tool: CanvasTool) -> Option<&'static str> {
    match tool {
        // ui-text-exempt: command ids, never displayed
        CanvasTool::Select => Some("view.tool_select"),
        CanvasTool::Node => Some("view.tool_node"),
        CanvasTool::Hand => Some("view.tool_hand"),
        CanvasTool::Snapshot => Some("view.tool_snapshot"),
        CanvasTool::InkPicker => Some("tools.ink_picker"),
        CanvasTool::Text => Some("view.tool_text"),
        CanvasTool::Markup(kind) => Some(crate::shell::commands::markup_command(kind)),
        // Each kind names its own command, which is what lets the strip show
        // the armed field type. The mapping lives on the kind rather than here
        // so the two cannot drift.
        CanvasTool::Form(kind) => Some(kind.command_id()),
        // **None** — a placement is armed from inside a dialog and has no
        // ribbon control to name, so the NAME is absent. The sentence is not;
        // see this module's O66 section. Written as its own arm rather than
        // folded into a `_` so that a second `PlaceKind` has to be ruled on
        // rather than inheriting this silently.
        CanvasTool::Place(_) => None,
        // The empty string is `MeasureKind::Scale`'s id, and it is not a
        // command — that kind is armed from inside the Set-scale window and
        // deliberately maps to nothing.
        CanvasTool::Measure(kind) => {
            let id = crate::shell::commands::measure_command(kind);
            (!id.is_empty()).then_some(id)
        }
        CanvasTool::TextAnnot(kind) => Some(kind.command()),
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Edit) => Some("edit.text"),
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Add) => Some("edit.add_text"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two regions are distinct and neither collides with a panel's.
    #[test]
    fn the_regions_are_their_own_names() {
        assert_ne!(REGION, REGION_PUT_DOWN);
        assert!(REGION_PUT_DOWN.starts_with(REGION));
        assert!(!REGION.starts_with("panel:"));
    }

    /// The reserved height is inside the band `egui-shell` will honour at a
    /// realistic window, so the strip cannot silently resolve to nothing.
    #[test]
    fn the_reserved_height_survives_the_shells_clamp() {
        let resolved = egui_shell::dock::banner::resolve_height(BANNER_HEIGHT_PTS, 800.0);
        assert!(
            (resolved - BANNER_HEIGHT_PTS).abs() < f32::EPSILON,
            "the dock would clamp the strip to {resolved} pt"
        );
    }

    /// **Every tool that had a name has one still, and the two that never
    /// did still do not.**
    #[test]
    fn only_the_two_dialog_armed_tools_have_no_command() {
        use crate::canvas::textedit::TextEditKind;
        let named = [
            CanvasTool::Select,
            CanvasTool::Node,
            CanvasTool::Hand,
            CanvasTool::Snapshot,
            CanvasTool::InkPicker,
            CanvasTool::Text,
            CanvasTool::TextEdit(TextEditKind::Add),
            CanvasTool::TextEdit(TextEditKind::Edit),
            CanvasTool::Measure(MeasureKind::Linear),
            CanvasTool::Measure(MeasureKind::Perimeter),
            CanvasTool::Measure(MeasureKind::Area),
            CanvasTool::Measure(MeasureKind::Circular),
        ];
        for tool in named {
            assert!(
                command_for(tool).is_some_and(|id| !id.is_empty()),
                "{tool:?} lost the command that names it"
            );
        }
        assert_eq!(
            command_for(CanvasTool::Measure(MeasureKind::Scale)),
            None,
            "the scale kind is armed from inside a window and names no command"
        );
    }
}
