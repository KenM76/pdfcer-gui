//! # `panels::properties::tool` — the armed tool's own settings, where a
//! property belongs
//!
//! `OPERATOR_REQUESTS.md` **O123**, and this module is the whole of his
//! argument made real:
//!
//! > *"I never understood why there is a tool dock when everything can be in
//! > object and properties."*
//!
//! ## ★★★ Nothing here is new. Every control was in the Tool panel yesterday
//!
//! `SHELL_LAYOUT_PROPOSAL.md` §3 said a one-line tool strip *"deletes the armed
//! options block"* and ranked the proposal last for it. That objection was
//! correct about what a 28 pt row can hold and wrong about where the controls
//! belong. They are not the tool's; they are **properties of what is about to
//! be drawn**, which is the same category of thing as the properties of what is
//! already drawn — and this panel is the surface that owns that category.
//!
//! | control | was | is now |
//! |---|---|---|
//! | text pen **font** picker | `panels::tool::armed::options` | [`text_pen`] |
//! | text pen **size** | same | [`text_pen`] |
//! | text pen **colour** swatch | same | [`text_pen`] |
//! | the pen's disclosure note | same | [`text_pen`] |
//! | the circular measure's **pick list**, one removable row per point | `panels::tool::armed::measure_points` | [`measure_points`] |
//! | *Scale line weight* | `panels::tool::armed::scale_switches` | [`scale_switches`] |
//! | *Keep the inner margins* | same | [`scale_switches`] |
//! | *Allow the artwork to distort* | same | [`scale_switches`] |
//! | the switches' note | same | [`scale_switches`] |
//!
//! Every string is the one that was already written
//! ([`crate::text::tool`]); every store is the one that was already read
//! (`canvas::textedit::pen`, `canvas::measure`, `canvas::scaling`). This is a
//! move, and it is written as one so that a diff shows a move.
//!
//! ## ★★ Where it sits in the panel, and why FIRST
//!
//! At the top of the body, above every selection-scoped section. Two reasons:
//!
//! 1. **It is where the operator's eye already goes.** These controls sat in
//!    the top-right corner of the window, in the Tool panel's own stack. The
//!    panel changed; the corner did not.
//! 2. **An armed tool is the more immediate subject.** When somebody has armed
//!    the text pen, the question they are about to ask is *what size?*, not
//!    *what is that path's line width?* — and when nothing is armed but Select,
//!    this section is three switches that state how the next resize behaves,
//!    which is still a statement about the next gesture.
//!
//! ## ★★★ It is deliberately NOT part of `something_drew`
//!
//! `OPERATOR_REQUESTS.md` **O75** collapses the *This document* section
//! whenever a selection-scoped section has spoken. This section is **not**
//! selection-scoped: [`scale_switches`] draws whenever the Select tool is armed,
//! which is most of the time and has nothing to do with what is selected.
//! Folding it into that predicate would collapse the document section for ever
//! and suppress *"nothing is selected"* for ever, which is O75 answered
//! backwards.
//!
//! ## The reachability rule this module inherits, and why it is written twice
//!
//! `panels::tool`'s own header recorded it: the three scale switches were first
//! written into a branch `CanvasTool::Select` **cannot reach**, and *"an option
//! row added there is dead code that compiles, reads correctly, and draws
//! nothing … Every unit test in the chain passed. Nothing tested that the
//! control is on screen."* The check that caught it drove the real binary.
//!
//! ⇒ So every region here publishes through [`crate::diag::ui_rect_visible`]
//! rather than `ui_rect`, and one per **switch** rather than one per block. A
//! rect proves layout; only a rect measured against the clip in force proves
//! the operator could reach it — and this panel is a `ScrollArea`, so a control
//! scrolled past the fold has a perfectly healthy rectangle.

use egui::Ui;

use crate::canvas::measure::MeasureKind;
use crate::canvas::tool::CanvasTool;
use crate::text::tool as t;

/// The region the whole section publishes when it has drawn anything.
pub const REGION: &str = "properties.tool"; // ui-text-exempt: trace region name, never displayed
/// The region the text pen's controls publish.
pub const REGION_TEXT_PEN: &str = "properties.tool.text_pen"; // ui-text-exempt: trace region name, never displayed
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
///
/// ★ Per ROW rather than one rect for the list, because the whole capability
/// `OPERATOR_REQUESTS.md` O107 asks for is *removing a particular point*, and a
/// check that could only find "the list" could not press one.
pub const REGION_MEASURE_POINT_PREFIX: &str = "properties.tool.measure_point."; // ui-text-exempt: trace region name, never displayed

/// Which block of controls an armed tool brings with it.
///
/// # ★★★ A real function, not a `match` buried in a draw call
///
/// The mapping *tool → controls* is the whole of what this module decides, and
/// it is the thing that broke last time: the three scale switches were written
/// into a branch `CanvasTool::Select` **cannot reach**, and *"an option row
/// added there is dead code that compiles, reads correctly, and draws
/// nothing."* Every unit test in that chain passed, because none of them could
/// ask the question — the decision lived inside a function that needed a `Ui`.
///
/// It does not any more. [`section`] dispatches on this and nothing else, so
/// `each_moved_control_has_a_tool_that_reaches_it` is asserting the shipped
/// decision rather than a copy of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    /// The text pen's face, size and colour.
    TextPen,
    /// The circular measure's removable pick list.
    MeasurePoints,
    /// The three resize modifiers.
    ScaleSwitches,
}

/// The controls `tool` brings, if any.
///
/// Exhaustive with no `_` arm on the outer shape, so a new `CanvasTool` variant
/// has to be ruled on rather than silently inheriting *"no settings"*.
#[must_use]
pub fn block_for(tool: CanvasTool) -> Option<Block> {
    match tool {
        // ★★★ `Select` is the RESTING state, and its options are the resize
        // switches. This is the arm whose absence made those switches dead code
        // the first time they were written: the old panel drew the armed block
        // only when something *was* armed, so a `Select` arm inside it could
        // never be reached.
        CanvasTool::Select => Some(Block::ScaleSwitches),
        // ★ **Add only, not Edit.** `TextEditKind::Add` writes a NEW run, so a
        // face, a size and a colour are exactly what it needs. `Edit` replaces
        // the words inside a run that already has all three, and pdfcer cannot
        // restyle a run it did not write — showing these controls there would
        // offer a change the commit silently discards.
        CanvasTool::TextEdit(crate::canvas::textedit::TextEditKind::Add) => Some(Block::TextPen),
        CanvasTool::Measure(MeasureKind::Circular) => Some(Block::MeasurePoints),
        CanvasTool::TextEdit(_)
        | CanvasTool::Measure(_)
        | CanvasTool::Node
        | CanvasTool::Hand
        | CanvasTool::Text
        | CanvasTool::Markup(_)
        | CanvasTool::TextAnnot(_)
        | CanvasTool::Form(_)
        | CanvasTool::Place(_) => None,
    }
}

/// Draw whichever of the armed tool's settings apply, and say whether anything
/// was drawn.
///
/// Returns `false` when the armed tool has no settings — which is most of them,
/// and is the honest shape rather than a heading with nothing under it (R9).
pub(super) fn section(ui: &mut Ui) -> bool {
    let ctx = ui.ctx().clone();
    let Some(block) = block_for(crate::canvas::tool::selected(&ctx)) else {
        return false;
    };
    match block {
        Block::ScaleSwitches => scale_switches(ui, &ctx),
        Block::TextPen => text_pen(ui, &ctx),
        Block::MeasurePoints => measure_points(ui, &ctx),
    }
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
    ui.separator();
    true
}

/// The text pen — face, size and colour for the next run of new text.
///
/// Moved from `panels::tool::armed::options`, unchanged. The two notes that
/// travelled with it, because both are the kind of thing a move loses:
///
/// * The faces come from `pen::FACES`, **not** a hand-written list. Its own
///   test asserts all fourteen are offered exactly once — a list that quietly
///   held thirteen would be a face an operator could never reach, with no error
///   anywhere.
/// * The size's range is the **store's** bounds rather than two local literals,
///   for `dialogs::settings`' reason: a control narrower than what the value
///   accepts silently rewrites a setting the operator never touched.
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
        ui.add(
            egui::DragValue::new(&mut current.size_pt)
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

    // ★ Written back only when it CHANGED. An unconditional `insert_temp` would
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

/// The radius/diameter tool's pick set, listed and removable —
/// `OPERATOR_REQUESTS.md` **O107**.
///
/// > *"we should be able to unselect points/clicked locations, and it should
/// > have a box in the side panel showing what is part of our selection …
/// > clicking on a point or location listed should allow us to remove it."*
///
/// The canvas markers say *where* the points are. They cannot say **how many**,
/// and on a dense CAD sheet a marker sitting on a junction is not
/// distinguishable from the junction. The list is the only surface that answers
/// *"what is actually in this fit?"*
///
/// ★★ **The removal is applied AFTER the loop, never inside it.** `st` is a
/// copy read out of `egui::Memory` and `points` borrows it; a removal that
/// mutated mid-iteration would shift every row below the one pressed while the
/// loop was still drawing them — the classic one-frame mis-aim, where the
/// operator presses row 3, row 4 slides up under the pointer, and the next
/// frame's press lands on something they did not choose.
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
///
/// The order is by how often it is wanted: stroke width first (the one he asked
/// for by name), insets second, and the distortion escape last, because it is
/// the one that makes the result imperfect and a control that degrades the
/// output belongs after the two that do not.
///
/// ★ **Always drawn, never greyed** while Select is armed — live with nothing
/// selected and with a form field selected. An operator sets a modifier
/// *before* the gesture it modifies; greying them until an annotation happens
/// to be selected would hide the control exactly when somebody is deciding how
/// to resize.
///
/// ★★★ **One published rect per switch**, not one for the block. A driven check
/// aiming at "the options row" and then guessing which line is the second
/// checkbox would be encoding a layout, and it goes wrong silently — by ticking
/// the wrong switch — the day a label wraps to two lines at a narrower dock.
fn scale_switches(ui: &mut Ui, ctx: &egui::Context) {
    let mut current = crate::canvas::scaling::read(ctx);
    let before = current;

    ui.label(t::scale_heading());
    crate::diag::ui_rect_visible(REGION_SCALE_SWITCHES, ui.min_rect(), ui.clip_rect());

    let stroke = ui.checkbox(&mut current.scale_stroke_width, t::scale_stroke_label());
    crate::diag::ui_rect_visible(REGION_SCALE_STROKE, stroke.rect, ui.clip_rect());
    // ★ The `/RD` switch is spelled as an opt-OUT in the engine and in
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

    /// ★★★ **Every control the Tool panel held is reachable from a tool this
    /// section actually draws for.**
    ///
    /// Asserted against [`block_for`] — **the function [`section`] dispatches
    /// on**, not a copy of its `match`. That distinction is the test: a mirror
    /// in this module would go on passing while the shipped decision drifted,
    /// which is the shape of the defect that let three scale switches compile,
    /// read correctly and draw nothing.
    ///
    /// This is the unit half of the reachability claim. The other half is the
    /// driven check, because a branch that runs is still not a control on
    /// screen.
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
        // the linear measure has no pick set.
        assert_eq!(block_for(CanvasTool::TextEdit(TextEditKind::Edit)), None);
        assert_eq!(block_for(CanvasTool::Measure(MeasureKind::Linear)), None);
        assert_eq!(block_for(CanvasTool::Hand), None);
    }

    /// ★★ **The three blocks are reachable from three DIFFERENT tools**, so no
    /// two of them can be shadowed by one arm.
    ///
    /// The failure this catches is subtle and has happened here before: an arm
    /// written above another that would have matched, leaving the second
    /// unreachable with nothing to show for it. Sweeping every tool the
    /// application can arm and collecting the blocks that come back is the only
    /// way to see it.
    #[test]
    fn all_three_blocks_are_reachable() {
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
            3,
            "one of the three moved control blocks is unreachable: {reached:?}"
        );
    }
}
