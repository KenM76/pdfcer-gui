//! # `dialogs::print::position` — the Position tab
//!
//! The offsets, the clip test and the arrow-key nudge are
//! [`pdfcer_gui_base::printposition`]; this file draws the tab over them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printposition.md`.

use egui::Ui;

pub(crate) use pdfcer_gui_base::printposition::*;

use crate::dialogs::print::PrintDialog;
use crate::dialogs::print::spooler::Job;
use crate::text::print as t;
use crate::units;

/// Publish one Position button's rectangle for `ui-verify`.
fn publish(ui: &Ui, name: &str, rect: egui::Rect) {
    crate::diag::ui_rect_visible(name, rect, ui.clip_rect());
}

/// **The body of the Position tab.**
pub(super) fn group(
    ui: &mut Ui,
    dialog: &mut PrintDialog,
    job: Option<&Job>,
    page_sizes: &[(f64, f64)],
) {
    let Some(job) = job else { return };
    let shown = dialog.preview_page.min(job.plans.len().saturating_sub(1));
    let (Some(&plan), Some(&size)) = (
        job.plans.get(shown),
        job.plans.get(shown).and_then(|p| page_sizes.get(p.index)),
    ) else {
        return;
    };
    let page = plan.index;
    let printable = job.device.printable_pt;

    ui.add_space(8.0);
    ui.separator();
    ui.label(t::position_heading());
    ui.label(
        egui::RichText::new(t::position_page_label(page + 1, size))
            .small()
            .weak(),
    );

    // The two typed entries. Displayed through millimetres every frame so the
    // number on screen is in the unit the readouts beside it are; converted
    // back only on `changed()`, so an unedited entry is never rewritten by its
    // own display rounding.
    let offset = dialog.page_positions.of(page);
    // `horizontal_wrapped`, like the two button rows below it: a plain
    // `ui.horizontal` lays out past the end of its column and reports a
    // `min_rect` that wide, which propagates outward as the body's content
    // width and raises a horizontal scrollbar the operator cannot dismiss.
    ui.horizontal_wrapped(|ui| {
        ui.label(t::position_across());
        let mut across_mm = units::mm_from_points(offset.dx_pt);
        if ui
            .add(
                egui::DragValue::new(&mut across_mm)
                    .speed(NUDGE_MM)
                    .fixed_decimals(1)
                    .suffix(t::position_mm_suffix()),
            )
            .changed()
        {
            dialog.page_positions.set_axis(
                page,
                Axes::Horizontally,
                units::points_from_mm(across_mm),
            );
        }
        ui.label(t::position_down());
        let mut down_mm = units::mm_from_points(offset.dy_pt);
        if ui
            .add(
                egui::DragValue::new(&mut down_mm)
                    .speed(NUDGE_MM)
                    .fixed_decimals(1)
                    .suffix(t::position_mm_suffix()),
            )
            .changed()
        {
            dialog
                .page_positions
                .set_axis(page, Axes::Vertically, units::points_from_mm(down_mm));
        }
    });
    ui.label(egui::RichText::new(t::position_frame()).small().weak());

    // Reset first, then the three centrings. Reset is what the operator reaches
    // for to undo an experiment, so it is where the eye lands.
    ui.horizontal_wrapped(|ui| {
        let moved = dialog.page_positions.is_moved(page);
        let reset = ui
            .add_enabled(moved, egui::Button::new(t::position_reset()))
            .on_disabled_hover_text(t::position_reset_unmoved());
        publish(ui, super::REGION_POSITION_RESET, reset.rect);
        if reset.clicked() {
            dialog.page_positions.reset(page);
        }
        let centre = ui
            .button(t::position_centre())
            .on_hover_text(t::position_centre_tooltip());
        publish(ui, super::REGION_POSITION_CENTRE, centre.rect);
        if centre.clicked() {
            dialog
                .page_positions
                .centre(page, Axes::Both, plan.placement, size, printable);
        }
        let across = ui.button(t::position_centre_horizontally());
        publish(ui, super::REGION_POSITION_CENTRE_H, across.rect);
        if across.clicked() {
            dialog
                .page_positions
                .centre(page, Axes::Horizontally, plan.placement, size, printable);
        }
        let down = ui.button(t::position_centre_vertically());
        publish(ui, super::REGION_POSITION_CENTRE_V, down.rect);
        if down.clicked() {
            dialog
                .page_positions
                .centre(page, Axes::Vertically, plan.placement, size, printable);
        }
    });

    // The job-wide reset, with its scope visible before it is pressed.
    ui.horizontal_wrapped(|ui| {
        let any = dialog.page_positions.any();
        let all = ui
            .add_enabled(any, egui::Button::new(t::position_reset_all()))
            .on_disabled_hover_text(t::position_reset_all_none());
        publish(ui, super::REGION_POSITION_RESET_ALL, all.rect);
        if all.clicked() {
            dialog.page_positions.reset_all();
        }
        let scope = if any {
            t::position_moved_count(dialog.page_positions.moved_count())
        } else {
            t::position_reset_all_none().to_owned()
        };
        ui.label(egui::RichText::new(scope).small().weak());
    });

    // The disclosure, OFF-CANVAS and never in the warning colour — rule 4.
    // It states geometry (*the page extends past the printable area*) and never
    // loss, because on a 1:1 CAD drawing the overhang is usually empty paper
    // and the ink verdict beside the preview is the surface entitled to make a
    // claim about content. Two surfaces making overlapping claims about one
    // risk is how a dialog comes to contradict itself.
    ui.label(
        egui::RichText::new(cropped(plan.placement, size, printable).line())
            .small()
            .weak(),
    );
    ui.label(egui::RichText::new(t::position_drag_hint()).small().weak());
}
