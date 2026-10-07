//! # `panels::properties::tool` — the armed tool's own settings, where a
//! property belongs
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/tool.md`.

use egui::Ui;
use pdfcer_gui_base::entry;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::measure::MeasureKind;
use crate::canvas::tool::CanvasTool;
use crate::text::tool as t;

/// The region the whole section publishes when it has drawn anything.
pub const REGION: &str = "properties.tool"; // ui-text-exempt: trace region name, never displayed
/// The region the text pen's controls publish.
pub const REGION_TEXT_PEN: &str = "properties.tool.text_pen"; // ui-text-exempt: trace region name, never displayed
/// The text pen's invisible switch.
pub const REGION_TEXT_PEN_INVISIBLE: &str = "properties.tool.text_pen_invisible"; // ui-text-exempt: trace region name, never displayed
/// The region the Select tool's three scale switches publish.
pub const REGION_SCALE_SWITCHES: &str = "properties.tool.scale_switches"; // ui-text-exempt: trace region name, never displayed
/// The *Scale line weight* switch's own rect.
pub const REGION_SCALE_STROKE: &str = "properties.tool.scale.stroke"; // ui-text-exempt: trace region name, never displayed
/// The *Keep the inner margins* switch's own rect.
pub const REGION_SCALE_INSETS: &str = "properties.tool.scale.insets"; // ui-text-exempt: trace region name, never displayed
/// The *Allow the artwork to distort* switch's own rect.
pub const REGION_SCALE_DISTORT: &str = "properties.tool.scale.distort"; // ui-text-exempt: trace region name, never displayed
/// The region the radius/diameter tool's picked-point list publishes.
pub const REGION_MEASURE_POINTS: &str = "properties.tool.measure_points"; // ui-text-exempt: trace region name, never displayed
/// The prefix of one picked point's row; its index in the set is appended.
pub const REGION_MEASURE_POINT_PREFIX: &str = "properties.tool.measure_point."; // ui-text-exempt: trace region name, never displayed

/// Which block of controls an armed tool brings with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    /// The text pen's face, size and colour.
    TextPen,
    /// The circular measure's removable pick list.
    MeasurePoints,
    /// The three resize modifiers.
    ScaleSwitches,
    /// The group the next ce dimension joins and, for the linear tool, its
    /// direction.
    DimensionTool,
}

/// The controls `tool` brings, if any.
///
/// Exhaustive with no `_` arm on the outer shape, so a new `CanvasTool` variant
/// has to be ruled on rather than silently inheriting *"no settings"*.
#[must_use]
pub fn block_for(tool: CanvasTool) -> Option<Block> {
    match tool {
        // `Select` is the RESTING state, and its options are the resize
        // switches. A panel that drew tool settings only when something *was*
        // armed would make this arm unreachable and those switches dead code,
        // which is why the block's slot is decided separately from whether a
        // tool is armed — see [`Slot`].
        CanvasTool::Select => Some(Block::ScaleSwitches),
        // **Add only, not Edit.** `TextEditKind::Add` writes a NEW run, so a
        // face, a size and a colour are exactly what it needs. `Edit` replaces
        // the words inside a run that already has all three, and pdfcer cannot
        // restyle a run it did not write — showing these controls there would
        // offer a change the commit silently discards.
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Add) => Some(Block::TextPen),
        CanvasTool::Measure(MeasureKind::Circular) => Some(Block::MeasurePoints),
        // Every tool that authors a ce dimension; `Scale` calibrates and
        // authors none.
        CanvasTool::Measure(
            MeasureKind::Linear
            | MeasureKind::TwoLine
            | MeasureKind::Perimeter
            | MeasureKind::PathLength
            | MeasureKind::Area,
        ) => Some(Block::DimensionTool),
        CanvasTool::TextEdit(_)
        | CanvasTool::Measure(MeasureKind::Scale)
        | CanvasTool::Node
        | CanvasTool::Hand
        | CanvasTool::Text
        | CanvasTool::Markup(_)
        | CanvasTool::TextAnnot(_)
        | CanvasTool::Form(_)
        | CanvasTool::Place(_)
        | CanvasTool::Snapshot
        | CanvasTool::InkPicker => None,
    }
}

/// Where in the panel a [`Block`] belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Above every selection-scoped section, at the top of the panel.
    ///
    /// For blocks that describe **the stroke about to be drawn**. Somebody who
    /// has armed the text pen has no selection and is about to ask *what size*;
    /// somebody mid-way through a circular fit is asking *which points*. In
    /// both cases the armed tool really is the more immediate subject, and in
    /// both cases nothing below is competing for the space.
    AboveTheSelection,
    /// Below everything, at the foot of the panel.
    ///
    /// For blocks that are a **standing preference** rather than a description
    /// of anything. The reading order this produces is the one `RIBBON_IA.md`
    /// §5.6 asks a properties surface for, with one clause added at the end:
    /// what you can change about this thing, then what is true of it, then how
    /// the next gesture will behave.
    BelowTheSelection,
}

/// Which [`Slot`] a block draws in.
#[must_use]
pub fn slot_of(block: Block) -> Slot {
    match block {
        Block::TextPen | Block::MeasurePoints | Block::DimensionTool => Slot::AboveTheSelection,
        Block::ScaleSwitches => Slot::BelowTheSelection,
    }
}

/// The armed tool's settings, drawn above the selection-scoped sections.
pub(super) fn armed_section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    section_in(ui, Slot::AboveTheSelection, doc, actions)
}

/// The standing preferences, drawn at the foot of the panel.
pub(super) fn preferences_section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    section_in(ui, Slot::BelowTheSelection, doc, actions)
}

/// Draw the armed tool's block if it belongs in `slot`, and say whether it did.
fn section_in(ui: &mut Ui, slot: Slot, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    let ctx = ui.ctx().clone();
    let tool = crate::canvas::tool::selected(&ctx);
    let Some(block) = block_for(tool) else {
        return false;
    };
    if slot_of(block) != slot {
        return false;
    }
    match (block, tool) {
        (Block::ScaleSwitches, _) => scale_switches(ui, &ctx),
        (Block::TextPen, _) => text_pen(ui, &ctx),
        // The circular tool authors a ce dimension too, so it gets the group
        // row above its pick list.
        (Block::MeasurePoints, _) => {
            super::tooldim::section(ui, doc, MeasureKind::Circular, actions);
            measure_points(ui, &ctx);
        }
        (Block::DimensionTool, CanvasTool::Measure(kind)) => {
            super::tooldim::section(ui, doc, kind, actions);
        }
        (Block::DimensionTool, _) => {}
    }
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    ui.separator();
    true
}

/// The text pen — face, size and colour for the next run of new text.
fn text_pen(ui: &mut Ui, ctx: &egui::Context) {
    use crate::canvas::textedit::pen;
    let mut current = pen::read(ctx);
    let before = current;

    ui.label(t::text_pen_heading());
    crate::diag::ui_rect_visible(REGION_TEXT_PEN, ui.min_rect(), ui.clip_rect());

    ui.horizontal_wrapped(|ui| {
        ui.label(t::text_pen_font_label());
        egui::ComboBox::from_id_salt("properties-text-pen-font")
            .selected_text(t::text_pen_font_name(current.face))
            .show_ui(ui, |ui| {
                for face in pen::FACES.iter().copied() {
                    ui.selectable_value(&mut current.face, face, t::text_pen_font_name(face));
                }
            });
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(t::text_pen_size_label());
        let (widget, refusal) = entry::drag_value(
            ui,
            &mut current.size_pt,
            entry::Kind::Length(entry::LengthUnit::Point),
        );
        refusal.show(ui.add(widget.range(pen::MIN_SIZE_PT..=pen::MAX_SIZE_PT).speed(0.5)));
    });
    ui.horizontal_wrapped(|ui| {
        ui.label(t::text_pen_colour_label());
        ui.color_edit_button_srgb(&mut current.colour);
    });
    let invisible = ui
        .checkbox(&mut current.invisible, t::text_pen_invisible_label())
        .on_hover_text(t::text_pen_invisible_hover());
    crate::diag::ui_rect_visible(REGION_TEXT_PEN_INVISIBLE, invisible.rect, ui.clip_rect());
    ui.label(egui::RichText::new(t::text_pen_note()).small().weak());

    // Written back only when it CHANGED. An unconditional `insert_temp` would
    // be harmless and would also make the trace line below fire sixty times a
    // second, which is the difference between a log a reader can use and one
    // they cannot.
    if current != before {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "text-pen face={:?} size={:.1} rgb={},{},{} invisible={}",
                current.face,
                current.size_pt,
                current.colour[0],
                current.colour[1],
                current.colour[2],
                u8::from(current.invisible)
            )
        });
        pen::store(ctx, current);
    }
}

/// The radius/diameter tool's pick set, listed and removable —
/// `OPERATOR_REQUESTS.md` **O107**.
fn measure_points(ui: &mut Ui, ctx: &egui::Context) {
    ui.label(t::measure_points_heading());
    crate::diag::ui_rect_visible(REGION_MEASURE_POINTS, ui.min_rect(), ui.clip_rect());

    let Some(st) = crate::canvas::measure::read(ctx) else {
        ui.label(
            egui::RichText::new(t::measure_points_empty())
                .small()
                .weak(),
        );
        return;
    };
    let points = st.circular.points();
    if points.is_empty() {
        ui.label(
            egui::RichText::new(t::measure_points_empty())
                .small()
                .weak(),
        );
        return;
    }

    let mut remove: Option<usize> = None;
    for (index, point) in points.iter().enumerate() {
        let label = t::measure_point_row(
            index + 1,
            t::measure_point_origin(point.origin),
            point.at.x,
            point.at.y,
        );
        let response = ui.add(egui::Button::new(egui::RichText::new(label).small()).frame(false));
        crate::diag::ui_rect_visible(
            &format!("{REGION_MEASURE_POINT_PREFIX}{index}"),
            response.rect,
            ui.clip_rect(),
        );
        // The row IS the remove control and it says so before it is pressed —
        // the gesture does not go through undo, because a pick set is
        // pre-commit state and never enters the document's history.
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
}

/// The Select tool's three resize modifiers — `OPERATOR_REQUESTS.md` **O51**.
fn scale_switches(ui: &mut Ui, ctx: &egui::Context) {
    let mut current = crate::canvas::scaling::read(ctx);
    let before = current;

    ui.label(t::scale_heading());
    crate::diag::ui_rect_visible(REGION_SCALE_SWITCHES, ui.min_rect(), ui.clip_rect());

    let stroke = ui.checkbox(&mut current.scale_stroke_width, t::scale_stroke_label());
    crate::diag::ui_rect_visible(REGION_SCALE_STROKE, stroke.rect, ui.clip_rect());
    // The `/RD` switch is spelled as an opt-OUT in the engine and in
    // `canvas::scaling`, and it is presented here as one too — *"keep"*, not
    // *"scale"*. An inverted label over an opt-out field is the single easiest
    // way to ship a control that does the opposite of what it says.
    let insets = ui.checkbox(&mut current.keep_rect_differences, t::scale_insets_label());
    crate::diag::ui_rect_visible(REGION_SCALE_INSETS, insets.rect, ui.clip_rect());
    let distort = ui.checkbox(&mut current.allow_distortion, t::scale_distort_label());
    crate::diag::ui_rect_visible(REGION_SCALE_DISTORT, distort.rect, ui.clip_rect());
    ui.label(egui::RichText::new(t::scale_note()).small().weak());

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every region is its own name and every one is under this section's
    /// prefix, so a driven check sweeping `properties.tool` finds all of them
    /// and none of anything else.
    #[test]
    fn every_region_is_its_own_name_under_the_sections_prefix() {
        let names = [
            REGION,
            REGION_TEXT_PEN,
            REGION_TEXT_PEN_INVISIBLE,
            REGION_SCALE_SWITCHES,
            REGION_SCALE_STROKE,
            REGION_SCALE_INSETS,
            REGION_SCALE_DISTORT,
            REGION_MEASURE_POINTS,
            REGION_MEASURE_POINT_PREFIX,
        ];
        let mut seen = std::collections::BTreeSet::new();
        for name in names {
            assert!(seen.insert(name), "{name} is declared twice");
            assert!(
                name.starts_with(REGION),
                "{name} is outside the section's region prefix"
            );
        }
    }

    /// **Every control the Tool panel held is reachable from a tool this
    /// section actually draws for.**
    #[test]
    fn each_moved_control_has_a_tool_that_reaches_it() {
        use crate::canvas::textedit::TextEditKind;
        assert_eq!(block_for(CanvasTool::Select), Some(Block::ScaleSwitches));
        assert_eq!(
            block_for(CanvasTool::TextEdit(TextEditKind::Add)),
            Some(Block::TextPen)
        );
        assert_eq!(
            block_for(CanvasTool::Measure(MeasureKind::Circular)),
            Some(Block::MeasurePoints)
        );
        // And the negative half, which is what catches an over-eager arm:
        // Edit-text has no pen (it cannot restyle a run it did not write) and
        // the scale tool authors no ce dimension.
        assert_eq!(block_for(CanvasTool::TextEdit(TextEditKind::Edit)), None);
        assert_eq!(
            block_for(CanvasTool::Measure(MeasureKind::Linear)),
            Some(Block::DimensionTool)
        );
        assert_eq!(block_for(CanvasTool::Measure(MeasureKind::Scale)), None);
        assert_eq!(block_for(CanvasTool::Hand), None);
    }

    /// **The four blocks are reachable from four DIFFERENT tools**, so no
    /// two of them can be shadowed by one arm.
    #[test]
    fn all_four_blocks_are_reachable() {
        use crate::canvas::textedit::TextEditKind;
        let every_tool = [
            CanvasTool::Select,
            CanvasTool::Node,
            CanvasTool::Hand,
            CanvasTool::Text,
            CanvasTool::TextEdit(TextEditKind::Add),
            CanvasTool::TextEdit(TextEditKind::Edit),
            CanvasTool::Measure(MeasureKind::Linear),
            CanvasTool::Measure(MeasureKind::Perimeter),
            CanvasTool::Measure(MeasureKind::Circular),
            CanvasTool::Measure(MeasureKind::Scale),
        ];
        let mut reached: Vec<Block> = every_tool.into_iter().filter_map(block_for).collect();
        reached.sort_by_key(|b| format!("{b:?}"));
        reached.dedup();
        assert_eq!(
            reached.len(),
            4,
            "one of the four control blocks is unreachable: {reached:?}"
        );
    }

    /// **The resting tool's block draws BELOW the selection, and the two
    /// authoring tools' blocks draw above it** — `OPERATOR_REQUESTS.md` O198.
    #[test]
    fn the_resting_tools_block_is_the_only_one_below_the_selection() {
        assert_eq!(slot_of(Block::ScaleSwitches), Slot::BelowTheSelection);
        assert_eq!(slot_of(Block::TextPen), Slot::AboveTheSelection);
        assert_eq!(slot_of(Block::MeasurePoints), Slot::AboveTheSelection);
        assert_eq!(slot_of(Block::DimensionTool), Slot::AboveTheSelection);
    }

    /// **Exactly one block occupies the foot of the panel.**
    #[test]
    fn one_block_and_only_one_draws_at_the_foot_of_the_panel() {
        let below: Vec<Block> = [
            Block::ScaleSwitches,
            Block::TextPen,
            Block::MeasurePoints,
            Block::DimensionTool,
        ]
        .into_iter()
        .filter(|b| slot_of(*b) == Slot::BelowTheSelection)
        .collect();
        assert_eq!(below, vec![Block::ScaleSwitches], "{below:?}");
    }
}
