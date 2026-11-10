//! # `panels::properties::toolregion` — the snapshot tool's Region block
//!
//! While the snapshot tool is armed, Tool Options shows what the box is (the
//! chip) and the commands that act on it: copy, delete, redact inside or
//! outside, crop. Each button raises its registered command, so the ribbon,
//! the menus and this block run one handler. Without a box only the
//! instruction is drawn.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/toolregion.md`.

use egui::Ui;
use pdfcer_gui_base::snapshotbox::SnapshotBox;

use crate::app::actions::Action;
use crate::app::dispatch::region::{CROP, DELETE, REDACT, REDACT_OUTSIDE};
use crate::app::state::OpenDoc;
use crate::text::region as t;

/// The block.
pub const REGION_BLOCK: &str = "properties.tool.region"; // ui-text-exempt: trace region name, never displayed
/// The chip line.
pub const REGION_CHIP: &str = "properties.tool.region.chip"; // ui-text-exempt: trace region name, never displayed
/// The Inside only / Touching choice.
pub const REGION_TAKES: &str = "properties.tool.region.takes"; // ui-text-exempt: trace region name, never displayed
/// The Inside / Outside choice.
pub const REGION_REDACT_SIDE: &str = "properties.tool.region.redact_side"; // ui-text-exempt: trace region name, never displayed
/// The Inside choice of the two.
pub const REGION_REDACT_INSIDE: &str = "properties.tool.region.redact_side.inside"; // ui-text-exempt: trace region name, never displayed
/// The Outside choice of the two.
pub const REGION_REDACT_OUTSIDE: &str = "properties.tool.region.redact_side.outside"; // ui-text-exempt: trace region name, never displayed
/// The prefix of each command button; the command id is appended.
pub const REGION_BUTTON_PREFIX: &str = "properties.tool.region.button."; // ui-text-exempt: trace region name, never displayed

const OUTSIDE_KEY: &str = "pdfcer-region-redact-outside"; // ui-text-exempt: internal memory id, never displayed
const COUNT_KEY: &str = "pdfcer-region-count"; // ui-text-exempt: internal memory id, never displayed

/// What the count was measured on: the box, the revision and the choice.
type CountKey = (usize, [u64; 4], u64, bool);

/// The region's object count, measured once per box, revision and choice
/// rather than every frame: a hit test on a dense drawing is not free.
fn object_count(
    ctx: &egui::Context,
    doc: &OpenDoc,
    region: SnapshotBox,
    touching: bool,
) -> Option<usize> {
    if region.page != doc.view.page_index {
        return None;
    }
    let r = region.rect;
    let key: CountKey = (
        region.page,
        [
            r.llx.to_bits(),
            r.lly.to_bits(),
            r.urx.to_bits(),
            r.ury.to_bits(),
        ],
        doc.edit_epoch,
        touching,
    );
    let id = egui::Id::new(COUNT_KEY);
    if let Some((held, n)) = ctx.data(|d| d.get_temp::<(CountKey, usize)>(id))
        && held == key
    {
        return Some(n);
    }
    let n = crate::canvas::region::take(doc, touching)?.selection.len();
    ctx.data_mut(|d| d.insert_temp(id, (key, n)));
    Some(n)
}

/// Draw the block.
pub(super) fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) {
    let ctx = ui.ctx().clone();
    ui.label(t::heading());
    let Some(region) = doc.snapshot else {
        ui.label(t::none_yet());
        crate::diag::ui_rect_visible(REGION_BLOCK, ui.min_rect(), ui.clip_rect());
        return;
    };
    let mut touching = crate::canvas::region::touching(&ctx);
    let r = region.rect;
    let count = object_count(&ctx, doc, region, touching);
    let chip = ui.label(t::chip(region.page, r.urx - r.llx, r.ury - r.lly, count));
    crate::diag::ui_rect_visible(REGION_CHIP, chip.rect, ui.clip_rect());

    let copy = crate::text::commands::edit_copy_snapshot_text();
    button(
        ui,
        "edit.copy_snapshot_text",
        copy.label,
        copy.tooltip,
        actions,
    );
    let outlines = crate::text::commands::edit_copy_snapshot_outlines();
    button(
        ui,
        "edit.copy_snapshot_outlines",
        outlines.label,
        outlines.tooltip,
        actions,
    );

    if !crate::canvas::tool::capabilities(&ctx).edit_content {
        crate::diag::ui_rect_visible(REGION_BLOCK, ui.min_rect(), ui.clip_rect());
        return;
    }
    ui.add_space(4.0);
    let takes = ui
        .horizontal(|ui| {
            ui.label(t::selects_label());
            ui.radio_value(&mut touching, false, t::selects_inside());
            ui.radio_value(&mut touching, true, t::selects_touching());
        })
        .response;
    crate::diag::ui_rect_visible(REGION_TAKES, takes.rect, ui.clip_rect());
    if touching != crate::canvas::region::touching(&ctx) {
        crate::canvas::region::set_touching(&ctx, touching);
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("region-takes touching={touching}"));
    }
    ui.label(egui::RichText::new(t::selects_note()).small().weak());
    let delete = crate::text::commands::edit_delete_in_region();
    button(ui, DELETE, delete.label, delete.tooltip, actions);

    ui.add_space(4.0);
    redact_row(ui, &ctx, actions);
    let crop = crate::text::commands::pages_crop_to_region();
    button(ui, CROP, crop.label, crop.tooltip, actions);
    crate::diag::ui_rect_visible(REGION_BLOCK, ui.min_rect(), ui.clip_rect());
}

/// The Inside region | Outside region choice and its one button. Outside has
/// no shortcut and always asks first, in its own window.
fn redact_row(ui: &mut Ui, ctx: &egui::Context, actions: &mut Vec<Action>) {
    let id = egui::Id::new(OUTSIDE_KEY);
    let mut outside = ctx.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let side = ui
        .horizontal(|ui| {
            ui.label(t::redact_label());
            let inside = ui.selectable_value(&mut outside, false, t::redact_inside());
            crate::diag::ui_rect_visible(REGION_REDACT_INSIDE, inside.rect, ui.clip_rect());
            let away = ui.selectable_value(&mut outside, true, t::redact_outside());
            crate::diag::ui_rect_visible(REGION_REDACT_OUTSIDE, away.rect, ui.clip_rect());
        })
        .response;
    crate::diag::ui_rect_visible(REGION_REDACT_SIDE, side.rect, ui.clip_rect());
    ctx.data_mut(|d| d.insert_temp(id, outside));
    let (command, words) = if outside {
        (
            REDACT_OUTSIDE,
            crate::text::commands::edit_redact_outside_region(),
        )
    } else {
        (REDACT, crate::text::commands::edit_redact_region())
    };
    button(
        ui,
        command,
        t::redact_button(outside),
        words.tooltip,
        actions,
    );
}

/// One command button, published as `{REGION_BUTTON_PREFIX}{id}`.
fn button(ui: &mut Ui, id: &str, label: &str, tooltip: &str, actions: &mut Vec<Action>) {
    let pressed = ui.button(label).on_hover_text(tooltip);
    crate::diag::ui_rect_visible(
        &format!("{REGION_BUTTON_PREFIX}{id}"),
        pressed.rect,
        ui.clip_rect(),
    );
    if pressed.clicked() {
        actions.push(Action::Command(id.to_owned()));
    }
}
