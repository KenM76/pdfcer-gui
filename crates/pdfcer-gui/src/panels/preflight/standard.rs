//! # `panels::preflight::standard` — a standard's verdict, above the findings
//!
//! For the PDF/X, PDF/A and PDF/UA profiles, the engine's answer to "does
//! this file conform?" — conforms, does not, or cannot tell — with the counts
//! behind it. PDF/X adds Convert to PDF/X, which opens File ▸ Print's window
//! through its command, so it is greyed for the same reason the ribbon's is.

use super::job::Standard;
use crate::app::actions::Action;
use crate::text::panels::preflight as t;

const REGION_VERDICT: &str = "preflight.verdict"; // ui-text-exempt: diagnostic region name
const REGION_CONVERT: &str = "preflight.convert_pdfx"; // ui-text-exempt: diagnostic region name

/// Trace the verdict once, when the run lands.
pub(super) fn trace(s: &Standard) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "preflight-verdict standard={} verdict={} failed={} warned={} person={} unchecked={}",
            s.claim.replace(' ', "_"),
            s.verdict.unwrap_or("none"), // ui-text-exempt: a trace token
            s.failed,
            s.warned,
            s.needs_person,
            s.not_checked
                .map_or_else(|| "-".to_owned(), |n| n.to_string()) // ui-text-exempt: a trace token
        )
    });
}

/// Draw the verdict; `convert_off` is why Convert to PDF/X is greyed.
pub(super) fn view(
    ui: &mut egui::Ui,
    s: &Standard,
    convert_off: Option<&str>,
    actions: &mut Vec<Action>,
) {
    let r = ui.label(t::standard_verdict(&s.claim, s.verdict));
    crate::diag::ui_rect(REGION_VERDICT, r.rect);
    ui.label(t::standard_reasons(
        s.failed,
        s.warned,
        s.needs_person,
        s.not_checked,
    ));
    match s.verdict {
        None => {
            ui.label(egui::RichText::new(t::standard_ua_no_verdict()).weak());
        }
        Some("unconfirmed") => {
            ui.label(egui::RichText::new(t::standard_unconfirmed()).weak());
        }
        _ => {}
    }
    if s.pdfx.is_some() && s.verdict != Some("conforms") {
        let b = match convert_off {
            None => ui
                .button(t::convert_pdfx())
                .on_hover_text(t::convert_pdfx_hover()),
            Some(why) => ui
                .add_enabled(false, egui::Button::new(t::convert_pdfx()))
                .on_disabled_hover_text(why),
        };
        crate::diag::ui_rect(REGION_CONVERT, b.rect);
        if b.clicked() {
            actions.push(Action::Command("file.convert_to_pdfx".to_owned())); // ui-text-exempt: a command id
        }
    }
    ui.add_space(4.0);
}
