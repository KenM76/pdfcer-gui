//! # `dialogs::unembed` — the confirmation before font programs come OUT of a
//! document
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/unembed.md`.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::unembed as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "unembed.body";
/// The Remove button — the one control that changes anything.
// ui-text-exempt: trace region name, never displayed
pub const REGION_REMOVE: &str = "unembed.commit";

/// The window, and the plan it is showing.
pub struct UnembedDialog {
    /// What the engine says would happen, computed once when the window opens.
    ///
    /// ★ Once, not per frame, for [`crate::dialogs::embed`]'s reason applied to
    /// a cheaper computation: `unembed_preview` walks every font-bearing
    /// surface in the document to build an inventory, and nothing it reads can
    /// change while this window is open.
    plan: pdfcer_core::font_unembed::UnembedPlan,
    /// The request the plan came from, so the commit sends the identical one.
    request: pdfcer_core::font_unembed::UnembedRequest,
    /// The blocked rows worth drawing, by index into `plan.blocked`.
    ///
    /// ★★ NOT the whole list, and the filter is the opposite of the embed
    /// window's. Under `AllRemovable` the engine reports *"every other font in
    /// the document, including the ones that are simply not embedded"* — and
    /// says outright that a shorter list *"is not actionable, which is the
    /// divergence from Acrobat this whole feature is built around"*.
    ///
    /// ★ So the full list is the engine's deliberate position and this filter
    /// removes exactly one class from it: a font with **no embedded program at
    /// all**, which is not a refusal in this window's terms — there was nothing
    /// to remove. Every real refusal is drawn.
    shown: Vec<usize>,
    /// How many signatures the document carries, for the disclosure.
    signatures: usize,
    remove_requested: bool,
    close_requested: bool,
}

impl UnembedDialog {
    /// Build the plan and open, or answer `None` when there is nothing to show.
    #[must_use]
    pub fn open(doc: &OpenDoc) -> Option<Self> {
        // ★ `all_removable`, with the default subset-tag policy of `Strip`. A
        // §9.6.4 tag asserts *"this file holds part of that face"*, and once
        // the program is gone that assertion is false — so stripping it is the
        // correct default rather than a convenience, and `keeping_subset_tag`
        // exists for a caller who is round-tripping rather than publishing.
        let request = pdfcer_core::font_unembed::UnembedRequest::all_removable();
        let plan = doc.session.unembed_preview(&request);
        let shown: Vec<usize> = plan
            .blocked
            .iter()
            .enumerate()
            .filter(|(_, blocked)| {
                !matches!(
                    blocked.blocker,
                    pdfcer_core::font_unembed::UnembedBlocker::NotRemovable(
                        pdfcer_core::fontinfo::Removability::NotEmbedded
                    )
                )
            })
            .map(|(index, _)| index)
            .collect();
        if plan.targets.is_empty() && shown.is_empty() {
            return None;
        }
        // ★★★ The window's own plan counts, traced when it opens.
        //
        // Not decoration. A window with `targets=0` draws a greyed button and a
        // list of refusals, which is CORRECT for a document whose embedded
        // fonts are all identity-encoded - and from outside it is
        // indistinguishable from a button that is broken. A driven check
        // without this line reports the fixture as a defect in the program,
        // which is the failure mode this project has named as "blaming the
        // feature for the fixture".
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "unembed-fonts-opened targets={} blocked={} shown={} bytes={}",
                plan.targets.len(),
                plan.blocked.len(),
                shown.len(),
                plan.bytes_reclaimable()
            )
        });
        Some(Self {
            plan,
            request,
            shown,
            signatures: doc.session.signature_census().signatures,
            remove_requested: false,
            close_requested: false,
        })
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "unembed-fonts", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(560.0, 580.0),
            egui::vec2(380.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.remove_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "unembed-fonts-requested targets={} blocked={} shown={} bytes={}",
                    self.plan.targets.len(),
                    self.plan.blocked.len(),
                    self.shown.len(),
                    self.plan.bytes_reclaimable()
                )
            });
            actions.push(Action::UnembedFonts {
                request: Box::new(self.request.clone()),
            });
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        // ★★★ The two document-level warnings FIRST, above the list, and both
        // are conditional. They are the only sentences here about the file as a
        // whole rather than about one font, and an operator scanning twenty
        // rows should meet them before the rows rather than under them.
        if let Some(line) = t::pdfa_line(&self.plan.pdfa) {
            ui.label(line);
            ui.add_space(4.0);
        }
        if self.signatures > 0 {
            ui.label(t::signature_line());
            ui.add_space(4.0);
        }
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .max_height((ui.available_height() - 84.0).max(160.0))
            .show(ui, |ui| {
                if !self.plan.targets.is_empty() {
                    ui.label(t::will_remove(self.plan.targets.len()));
                    for target in &self.plan.targets {
                        let face = target.base_font.as_deref().unwrap_or_default();
                        ui.small(t::remove_row(
                            face,
                            target.stored_bytes,
                            target.program_freed,
                            target.rename.as_deref(),
                        ));
                    }
                    ui.add_space(8.0);
                }
                if !self.shown.is_empty() {
                    ui.label(t::cannot_remove(self.shown.len()));
                    for &index in &self.shown {
                        let blocked = &self.plan.blocked[index];
                        let face = blocked.base_font.as_deref().unwrap_or_default();
                        ui.small(t::blocked_row(face, &blocked.blocker));
                    }
                    ui.add_space(8.0);
                }
                if !self.plan.unmatched.is_empty() {
                    ui.small(t::unmatched(&self.plan.unmatched));
                }
            });

        ui.add_space(8.0);
        // ★★★ The size note, LAST and outside the scroll area, so it cannot be
        // scrolled past. It is the sentence that contradicts the operator's own
        // reason for opening this window, and it is the one the engine's docs
        // say must never be omitted.
        if !self.plan.targets.is_empty() {
            ui.small(t::size_note(self.plan.bytes_reclaimable()));
            ui.add_space(4.0);
        }
        ui.separator();
        ui.horizontal(|ui| {
            let can = !self.plan.targets.is_empty();
            let remove = ui.add_enabled(can, egui::Button::new(t::remove_button()));
            crate::diag::ui_rect_visible(REGION_REMOVE, remove.rect, ui.clip_rect());
            let remove = if can {
                remove
            } else {
                // R9's temporarily-unavailable case: greyed, explained on
                // hover, and the window still opens because the blocked list is
                // then the whole answer the operator came for.
                remove.on_disabled_hover_text(t::nothing_to_remove())
            };
            if remove.clicked() {
                self.remove_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// Open it for the current document, or answer `None`.
#[must_use]
pub fn open_for(status: &Status) -> Option<UnembedDialog> {
    match status {
        Status::Open(doc) => UnembedDialog::open(doc),
        _ => None,
    }
}
