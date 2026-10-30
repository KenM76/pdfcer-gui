//! The Add validation evidence window: whether the signers' own certificates
//! go in, and, on a document certified with no changes permitted, the
//! operator's acceptance of the risk. The work happens in
//! `app::actions::evidence`, which opens the file picker.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::evidence as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "evidence.body";
/// The signers' certificates checkbox.
// ui-text-exempt: trace region name, never displayed
pub const REGION_OWN: &str = "evidence.own-certs";
/// The "add it anyway" checkbox, drawn only under a no-changes certification.
// ui-text-exempt: trace region name, never displayed
pub const REGION_ALLOW: &str = "evidence.allow";
/// The button that goes on to the picker.
// ui-text-exempt: trace region name, never displayed
pub const REGION_CHOOSE: &str = "evidence.commit";

/// The window's state.
pub struct EvidenceDialog {
    signatures: usize,
    /// Certified with `/P 1`: the warning and its checkbox are drawn.
    no_changes: bool,
    own_certs: bool,
    allow: bool,
    choose_requested: bool,
    close_requested: bool,
}

impl EvidenceDialog {
    fn open(doc: &OpenDoc) -> Option<Self> {
        let census = doc.session.signature_census();
        (census.signatures > 0).then(|| Self {
            signatures: census.signatures,
            no_changes: census.certifications > 0 && census.certification_permission == Some(1),
            own_certs: true,
            allow: false,
            choose_requested: false,
            close_requested: false,
        })
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "validation-evidence", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(520.0, 300.0),
            egui::vec2(380.0, 200.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.choose_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "evidence-options own={} allow={}",
                    u8::from(self.own_certs),
                    u8::from(self.allow)
                )
            });
            actions.push(Action::AddValidationEvidence {
                own_certs: self.own_certs,
                allow_no_changes: self.no_changes && self.allow,
            });
            return false;
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro(self.signatures));
        ui.add_space(10.0);
        let own = ui
            .checkbox(&mut self.own_certs, t::own_certs())
            .on_hover_text(t::own_certs_hover());
        crate::diag::ui_rect_visible(REGION_OWN, own.rect, ui.clip_rect());
        if self.no_changes {
            ui.add_space(10.0);
            ui.label(t::no_changes_certified());
            let allow = ui.checkbox(&mut self.allow, t::allow_anyway());
            crate::diag::ui_rect_visible(REGION_ALLOW, allow.rect, ui.clip_rect());
        }
        ui.add_space(12.0);
        ui.separator();
        let ready = !self.no_changes || self.allow;
        ui.horizontal(|ui| {
            let choose = ui
                .add_enabled(ready, egui::Button::new(t::choose_button()))
                .on_disabled_hover_text(t::needs_allow());
            crate::diag::ui_rect_visible(REGION_CHOOSE, choose.rect, ui.clip_rect());
            if choose.clicked() {
                self.choose_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// The window for the current document; `None` when none is open or it
/// carries no signature, which `app::actions::evidence` refuses with its
/// own sentence.
pub fn open_for(status: &Status) -> Option<EvidenceDialog> {
    match status {
        Status::Open(doc) => EvidenceDialog::open(doc),
        _ => None,
    }
}
