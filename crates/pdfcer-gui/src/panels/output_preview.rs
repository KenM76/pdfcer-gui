//! # `panels::output_preview` — the page as its printing inks, plate by plate
//!
//! Acrobat's Output Preview, separations half. A switch puts the canvas under
//! a [`SeparationPreview`]; while it is on, the page's plates — read from the
//! texture the engine drew under that preview — are listed with a show box
//! each. Hiding a plate changes only the canvas. Disclosure stays off-canvas:
//! the RGB-forced-to-CMYK note and unmatched names here, the "not how it will
//! print" line in the status bar.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/output_preview.md`.

use pdfcer_render::SeparationPreview;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::inkhover::{Hover, Tints};
use crate::canvas::inkpick::Found;
use crate::panels::PanelsState;
use crate::text::panels::output_preview as t;

/// The panel body.
pub const REGION_BODY: &str = "panel:output-preview"; // ui-text-exempt: trace region name, never displayed
/// The on/off switch.
pub const REGION_SWITCH: &str = "output-preview.on"; // ui-text-exempt: trace region name, never displayed
/// One ink's show box, suffixed with the ink's name, spaces as `_`.
pub const REGION_INK_PREFIX: &str = "output-preview.ink:"; // ui-text-exempt: trace region name, never displayed
/// The "show all" button.
pub const REGION_SHOW_ALL: &str = "output-preview.show-all"; // ui-text-exempt: trace region name, never displayed

/// Draw the Output Preview panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, _state: &mut PanelsState, actions: &mut Vec<Action>) {
    crate::diag::ui_rect(REGION_BODY, ui.max_rect());
    let mut on = doc.separations.is_some();
    let switch = ui
        .checkbox(&mut on, t::switch())
        .on_hover_text(t::switch_hover());
    crate::diag::ui_rect(REGION_SWITCH, switch.rect);
    if switch.changed() {
        actions.push(Action::SetSeparations(on.then(SeparationPreview::new)));
    }
    let Some(preview) = doc.separations.as_ref() else {
        trace("on=0 inks=- hidden=-"); // ui-text-exempt: diag trace value
        ui.label(egui::RichText::new(t::off_hint()).weak());
        return;
    };
    // Only a texture drawn under THIS preview describes its plates; any other
    // is the frame before the switch or the click, and is waited out.
    let Some(diag) = doc
        .page_texture
        .as_ref()
        .filter(|tex| tex.key.drawn_under(Some(preview)))
        .map(|tex| &tex.diagnostics)
    else {
        ui.label(egui::RichText::new(t::waiting()).weak());
        return;
    };
    trace(&format!(
        "on=1 inks={} hidden={}", // ui-text-exempt: diag trace value
        token(&diag.inks),
        token(&preview.hidden)
    ));
    if diag.inks.is_empty() {
        ui.label(t::no_inks());
        return;
    }
    ui.separator();
    let hover = crate::canvas::inkhover::current(ui.ctx(), doc);
    let probe = hover.as_ref().and_then(|h| match &h.found {
        Found::Probe(p) => Some(Tints::of(p)),
        _ => None,
    });
    ink_rows(ui, preview, &diag.inks, probe, actions);
    coverage(ui, hover.as_ref(), probe);
    let all = ui.add_enabled(!preview.hidden.is_empty(), egui::Button::new(t::show_all()));
    crate::diag::ui_rect(REGION_SHOW_ALL, all.rect);
    if all.clicked() {
        actions.push(Action::SetSeparations(Some(SeparationPreview::new())));
    }
    if diag.inks_forced_subtractive > 0 {
        ui.label(egui::RichText::new(t::forced_subtractive()).small().weak());
    }
    if !diag.inks_unmatched.is_empty() {
        ui.label(
            egui::RichText::new(t::unmatched(&diag.inks_unmatched))
                .small()
                .weak(),
        );
    }
}

/// One row per plate: its show switch and, under the pointer, its tint.
fn ink_rows(
    ui: &mut egui::Ui,
    preview: &SeparationPreview,
    inks: &[String],
    probe: Option<Tints<'_>>,
    actions: &mut Vec<Action>,
) {
    for ink in inks {
        let mut shown = !preview.hides(ink);
        let tint = probe.and_then(|p| p.tint(ink));
        let row = ui
            .horizontal(|ui| {
                let row = ui.checkbox(&mut shown, ink);
                if let Some(tint) = tint {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(t::percent(tint));
                    });
                }
                row
            })
            .inner
            .on_hover_text(t::ink_hover(ink));
        crate::diag::ui_rect(
            &format!("{REGION_INK_PREFIX}{}", ink.replace(' ', "_")),
            row.rect,
        );
        if row.changed() {
            actions.push(Action::SetSeparations(Some(toggled(preview, ink, shown))));
        }
    }
}

/// The line under the ink list: where the pointer reading was taken and its
/// total area coverage, or why there is none.
fn coverage(ui: &mut egui::Ui, hover: Option<&Hover>, probe: Option<Tints<'_>>) {
    let Some(hover) = hover else {
        trace_coverage("none"); // ui-text-exempt: diag trace value
        ui.label(egui::RichText::new(t::hover_hint()).small().weak());
        return;
    };
    let (x, y) = hover.at;
    let line = match (&hover.found, probe) {
        (Found::Reading, _) => {
            trace_coverage("reading"); // ui-text-exempt: diag trace value
            t::hover_reading().to_owned()
        }
        (Found::Failed(why), _) => {
            trace_coverage("failed"); // ui-text-exempt: diag trace value
            t::hover_failed(why)
        }
        (Found::Probe(p), _) if p.source == pdfcer_render::InkProbeSource::OutOfRange => {
            trace_coverage("off_page"); // ui-text-exempt: diag trace value
            t::hover_off_page().to_owned()
        }
        (Found::Probe(_), Some(tints)) if tints.total().is_some() => {
            trace_coverage(&reading_token(hover, tints));
            ui.label(egui::RichText::new(t::hover_at(x, y)).small().weak());
            let total = tints.total().unwrap_or_default();
            ui.label(t::total_coverage(total))
                .on_hover_text(t::total_coverage_hover());
            return;
        }
        (Found::Probe(_), _) => {
            trace_coverage("no_inks"); // ui-text-exempt: diag trace value
            t::hover_no_inks().to_owned()
        }
    };
    ui.label(egui::RichText::new(line).small().weak());
}

/// `page= x= y= inks=Cyan:1.000|… total=` for a reading with tints.
fn reading_token(hover: &Hover, tints: Tints<'_>) -> String {
    let mut inks: Vec<String> = pdfcer_render::PROCESS_INKS
        .iter()
        .filter_map(|ink| tints.tint(ink).map(|t| format!("{ink}:{t:.3}")))
        .collect();
    inks.extend(
        tints
            .spots
            .iter()
            .map(|(name, t)| format!("{}:{t:.3}", name.replace(' ', "_"))),
    );
    format!(
        "page={} x={:.1} y={:.1} inks={} total={:.3}", // ui-text-exempt: diag trace value
        hover.page,
        hover.at.0,
        hover.at.1,
        inks.join("|"),
        tints.total().unwrap_or_default()
    )
}

/// The coverage line's state, on change only.
fn trace_coverage(state: &str) {
    crate::diag::trace_on_change("output-preview-coverage", || state.to_owned());
}

/// `preview` with `ink` shown (`show`) or hidden.
fn toggled(preview: &SeparationPreview, ink: &str, show: bool) -> SeparationPreview {
    let hidden = preview
        .hidden
        .iter()
        .filter(|h| h.as_str() != ink)
        .cloned()
        .chain((!show).then(|| ink.to_owned()));
    SeparationPreview::new().with_hidden(hidden)
}

/// Ink names as one trace token: `|`-joined, spaces as `_`, `-` for none.
fn token(names: &[String]) -> String {
    if names.is_empty() {
        return "-".to_owned();
    }
    names.join("|").replace(' ', "_")
}

/// The panel's state line, on change only.
fn trace(state: &str) {
    crate::diag::trace_on_change("output-preview", || state.to_owned());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_an_ink_hides_then_shows_it() {
        let p = toggled(&SeparationPreview::new(), "Magenta", false);
        assert!(p.hides("Magenta"));
        let p = toggled(&p, "Magenta", true);
        assert!(p.hidden.is_empty());
    }

    #[test]
    fn hiding_twice_does_not_list_an_ink_twice() {
        let p = toggled(&SeparationPreview::new(), "Cyan", false);
        let p = toggled(&p, "Cyan", false);
        assert_eq!(p.hidden, vec!["Cyan".to_owned()]);
    }
}
