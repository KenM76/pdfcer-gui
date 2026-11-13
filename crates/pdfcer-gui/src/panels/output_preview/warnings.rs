//! # `panels::output_preview::warnings` — the ink coverage warnings
//!
//! *Total area coverage* and *Rich black*, each a switch and a threshold, as
//! in Acrobat's Output Preview. [`crate::canvas::coverage`] holds the rules
//! and paints them; this section sets them and says what the part of the page
//! on screen flagged, beside each rule's overlay colour.
//!
//! Traces `output-preview-warn tac=<n|-> rich=<n|-> opacity=<percent>` on change, then
//! `found=<checking|refused>` or `found_tac=<0|1|-> found_rich=<0|1|->`.

use crate::app::state::OpenDoc;
use crate::canvas::coverage::{self, RICH_COLOUR, Status, TAC_COLOUR};
use crate::text::panels::output_preview as t;

/// The total area coverage switch.
pub const REGION_TAC: &str = "output-preview.tac"; // ui-text-exempt: trace region name, never displayed
/// The rich black switch.
pub const REGION_RICH: &str = "output-preview.rich-black"; // ui-text-exempt: trace region name, never displayed
/// The warning opacity field.
pub const REGION_OPACITY: &str = "output-preview.warn-opacity"; // ui-text-exempt: trace region name, never displayed

/// Each rule's threshold, kept while the rule is off.
#[derive(Clone, Copy)]
pub struct WarnUi {
    total: u16,
    rich_black: u16,
    /// The overlay's opacity, percent.
    opacity: u8,
}

impl Default for WarnUi {
    /// Acrobat's defaults.
    fn default() -> Self {
        Self {
            total: 300,
            rich_black: 95,
            opacity: 100,
        }
    }
}

/// One rule's row.
struct Row<'a> {
    label: &'a str,
    hover: &'a str,
    region: &'a str,
    colour: egui::Color32,
    max: u16,
}

/// The warnings section, under the simulation profile.
pub(super) fn section(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut WarnUi) {
    ui.separator();
    ui.label(t::warn_heading());
    let ctx = ui.ctx().clone();
    let before = coverage::warnings(&ctx);
    let status = coverage::status(&ctx, doc);
    let mut next = before;
    let tac = Row {
        label: t::warn_total(),
        hover: t::warn_total_hover(),
        region: REGION_TAC,
        colour: TAC_COLOUR,
        // Four process plates and any spots; Acrobat's own field stops at 400.
        max: 400,
    };
    let found = |pick: fn(&coverage::Found) -> Option<bool>| match &status {
        Status::Done(f) => pick(f).map_or(Shown::Off, Shown::Found),
        Status::Checking => Shown::Checking,
        Status::Refused => Shown::Refused,
    };
    row(
        ui,
        &tac,
        &mut next.total,
        &mut state.total,
        found(|f| f.total),
    );
    let rich = Row {
        label: t::warn_rich(),
        hover: t::warn_rich_hover(),
        region: REGION_RICH,
        colour: RICH_COLOUR,
        max: 100,
    };
    row(
        ui,
        &rich,
        &mut next.rich_black,
        &mut state.rich_black,
        found(|f| f.rich_black),
    );
    if next != before {
        coverage::set(&ctx, next);
    }
    if next.any() {
        opacity(ui, &mut state.opacity);
    }
    crate::diag::trace_on_change("output-preview-warn", || {
        let n = |v: Option<u16>| v.map_or_else(|| "-".to_owned(), |v| v.to_string());
        let b = |v: Option<bool>| v.map_or("-", |v| if v { "1" } else { "0" });
        let found = match &status {
            Status::Checking => "found=checking".to_owned(),
            Status::Refused => "found=refused".to_owned(),
            Status::Done(f) => format!("found_tac={} found_rich={}", b(f.total), b(f.rich_black)),
        };
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "tac={} rich={} opacity={} {found}",
            n(next.total),
            n(next.rich_black),
            state.opacity
        )
    });
}

/// The warning opacity field, shown while a rule is on.
fn opacity(ui: &mut egui::Ui, percent: &mut u8) {
    let ctx = ui.ctx().clone();
    ui.horizontal(|ui| {
        ui.label(t::warn_opacity());
        let r = ui
            .add(egui::DragValue::new(percent).range(10..=100).suffix("%"))
            .on_hover_text(t::warn_opacity_hover());
        crate::diag::ui_rect(REGION_OPACITY, r.rect);
    });
    coverage::set_opacity(&ctx, *percent);
}

/// What a rule's row says after its threshold.
#[derive(Clone, Copy)]
enum Shown {
    Off,
    Checking,
    Refused,
    Found(bool),
}

fn row(ui: &mut egui::Ui, spec: &Row<'_>, on: &mut Option<u16>, kept: &mut u16, shown: Shown) {
    ui.horizontal(|ui| {
        let mut checked = on.is_some();
        let r = ui
            .checkbox(&mut checked, spec.label)
            .on_hover_text(spec.hover);
        crate::diag::ui_rect(spec.region, r.rect);
        ui.add(egui::DragValue::new(kept).range(0..=spec.max).suffix("%"))
            .on_hover_text(spec.hover);
        let (swatch, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
        ui.painter().rect_filled(swatch, 1.0, spec.colour);
        *on = checked.then_some(*kept);
        if on.is_none() {
            return;
        }
        let words = match shown {
            Shown::Off => return,
            Shown::Checking => t::warn_checking(),
            Shown::Refused => t::warn_refused(),
            Shown::Found(true) => t::warn_found(),
            Shown::Found(false) => t::warn_none(),
        };
        ui.label(egui::RichText::new(words).small().weak());
    });
}
