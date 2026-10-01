//! The signing strip: a fixed-height bar over the page reading *Signed N of M*
//! with a *Next* button that brings the next unsigned signature box into view.
//!
//! Contract: drawn exactly when the canvas draws the red signature tags (the
//! same document gates and tool), counted per field rather than per widget,
//! and it never signs anything itself — Next only scrolls; the operator still
//! clicks the box. Design and rationale:
//! `docs/modules/pdfcer-gui/app/signstrip.md`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use eframe::egui;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::canvas::forms::boxes::FieldTarget;
use crate::text::handsign as t;

/// The strip's height, in points; fixed so the page fit never follows it.
pub const HEIGHT_PTS: f32 = 26.0;
/// Regions published for ui-verify.
pub const REGION: &str = "signstrip"; // ui-text-exempt: trace region name, never displayed
pub const REGION_NEXT: &str = "signstrip.next"; // ui-text-exempt: trace region name, never displayed
/// The reveal's reason token in the trace.
const WHY_NEXT: &str = "sign-next"; // ui-text-exempt: diagnostic token, never displayed
/// Where the last Next went: the document and its index in the unsigned list.
const LAST_KEY: &str = "pdfcer-sign-strip-last"; // ui-text-exempt: internal memory id, never displayed

/// Signature fields signed so far, out of all of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub signed: usize,
    pub total: usize,
}

/// Count distinct fields: a field with two widgets is one signature.
pub fn progress(unsigned: &[FieldTarget], is_signed: impl Fn(&str) -> bool) -> Progress {
    let fields: BTreeSet<&str> = unsigned.iter().map(|t| t.field.as_str()).collect();
    Progress {
        signed: fields.iter().filter(|f| is_signed(f)).count(),
        total: fields.len(),
    }
}

/// The index of the next box still to sign after `after`, wrapping to the
/// start; `None` when every box is signed.
pub fn next(
    unsigned: &[FieldTarget],
    is_signed: impl Fn(&str) -> bool,
    after: Option<usize>,
) -> Option<usize> {
    let open = |i: &usize| !is_signed(&unsigned[*i].field);
    let start = after.map_or(0, |a| a + 1);
    (start..unsigned.len())
        .find(open)
        .or_else(|| (0..unsigned.len()).find(open))
}

impl PdfcerApp {
    /// Draw the strip under the ribbon while the open document has signature
    /// boxes the canvas offers to sign.
    pub(super) fn sign_strip(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let Status::Open(doc) = &self.status else {
            return;
        };
        let ctx = ui.ctx().clone();
        if !crate::canvas::forms::offers_signing(doc, crate::canvas::tool::active(&ctx)) {
            return;
        }
        let placed = crate::canvas::forms::placed(&ctx, doc);
        let unsigned = &placed.unsigned;
        if unsigned.is_empty() {
            return;
        }
        let signed = |f: &str| doc.hand_signed.is_signed(f);
        let tally = progress(unsigned, signed);
        crate::diag::trace_changed("sign-strip", || {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("sign-strip signed={} total={}", tally.signed, tally.total)
        });
        let id = egui::Id::new(LAST_KEY);
        let last = ctx
            .data(|d| d.get_temp::<(PathBuf, usize)>(id))
            .filter(|(path, _)| *path == doc.path)
            .map(|(_, i)| i);
        let mut go: Option<usize> = None;
        egui::Panel::top("sign-strip") // ui-text-exempt: internal panel id, never displayed
            .exact_size(HEIGHT_PTS)
            .show(ui, |ui| {
                crate::diag::ui_rect(REGION, ui.max_rect());
                // The button is allocated before the sentence so a long
                // sentence truncates instead of pushing it off the edge.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let done = tally.signed == tally.total;
                    if !done {
                        let response = ui.button(t::next_box()).on_hover_text(t::next_box_hint());
                        crate::diag::ui_rect_visible(REGION_NEXT, response.rect, ui.clip_rect());
                        if response.clicked() {
                            go = next(unsigned, signed, last);
                        }
                    }
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let line = if done {
                            t::all_signed(tally.total)
                        } else {
                            t::signed_of(tally.signed, tally.total)
                        };
                        ui.add(egui::Label::new(line).truncate())
                            .on_hover_text(t::strip_hint());
                    });
                });
            });
        let Some(index) = go else {
            return;
        };
        let target = &unsigned[index];
        ctx.data_mut(|d| d.insert_temp(id, (doc.path.clone(), index)));
        // ui-text-exempt: diagnostic trace, never displayed. No field name.
        crate::diag::trace(|| format!("sign-next page={} index={index}", target.page));
        if doc.view.page_index != target.page {
            actions.push(Action::GoToPage(target.page));
        }
        if let Some(page) = doc.pages.get(target.page) {
            let (min, max) = crate::canvas::minreveal::fracs_for_canvas_rect(target.rect, page);
            actions.push(Action::RevealRect {
                page: target.page,
                min,
                max,
                why: WHY_NEXT,
            });
        }
    }
}

#[cfg(test)]
mod tests;
