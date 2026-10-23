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
    for ink in &diag.inks {
        let mut shown = !preview.hides(ink);
        let row = ui
            .checkbox(&mut shown, ink)
            .on_hover_text(t::ink_hover(ink));
        crate::diag::ui_rect(
            &format!("{REGION_INK_PREFIX}{}", ink.replace(' ', "_")),
            row.rect,
        );
        if row.changed() {
            actions.push(Action::SetSeparations(Some(toggled(preview, ink, shown))));
        }
    }
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
