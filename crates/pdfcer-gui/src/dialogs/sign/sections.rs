//! # `dialogs::sign::sections` — the five sections of the Sign form
//!
//! Split out of [`super`] under **R2** on 2026-09-06, when `Pass 10.12`'s
//! certification option and `Pass 10.13`'s pre-placed-field list took
//! `dialogs/sign.rs` past the 1,500-line ceiling.
//!
//! ## The seam, and why it is a real one rather than a cut at a line number
//!
//! [`super`] holds the **window**: its state machine ([`super::Phase`]), the
//! ordering rule that every side effect happens *after* the draw closure, the
//! two-step by which an outcome comes back, and the guards that decide whether
//! the confirm control may exist at all. None of that is about what any
//! particular field looks like.
//!
//! This module holds the **form**: five `&mut self` methods that draw one
//! section each, in the order §1 of [`super`]'s header argues for — the
//! certificate first and everything else only once an identity is open.
//!
//! ⇒ The test is whether a change to one file wants a change to the other, and
//! for the two years of edits this window has had, it does not: adding a field
//! is entirely here, and changing when a section is *reachable* is entirely
//! there.
//!
//! ## ★★ Why the fields stay private and this is a child module
//!
//! Rust's privacy is *"visible to the defining module and its descendants"*, so
//! `dialogs::sign::sections` can read [`super::SignDialog`]'s private fields
//! without any of them becoming `pub(super)` or `pub(crate)`. That is the whole
//! reason for the nesting: a sibling module would have forced the passphrase
//! and the loaded identity — the two fields this window's `Debug` impl goes out
//! of its way to hide — to be widened to `pub(super)` in order to draw a text
//! box. **A layout split must not widen the visibility of key material.**

use egui_shell::theme::Theme;
use pdfcer_core::sign::apply::MdpPermission;

use super::{
    FIELD_WIDTH, Place, REGION_BOX_WHERE, REGION_CERTIFY, REGION_CHOOSE_CERTIFICATE,
    REGION_CONFIRM, REGION_EXISTING, REGION_IDENTITY, REGION_OPEN_CERTIFICATE, REGION_PAGE,
    REGION_PASSPHRASE, REGION_PLACE_BOX, SignDialog, field_region, file_name_of,
};
use crate::dialogs::sign::Destination;
use crate::text::protect as tp;
use crate::text::sign as t;

impl SignDialog {
    /// **The certificate: choose it, unlock it, and read back what it says.**
    pub(super) fn certificate_section(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(t::certificate_heading());
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let choose = ui.button(t::choose_certificate());
            crate::diag::ui_rect(REGION_CHOOSE_CERTIFICATE, choose.rect);
            if choose.clicked() {
                self.pick_requested = true;
            }
            ui.label(
                self.certificate
                    .as_deref()
                    .map_or_else(|| t::certificate_none_chosen().to_owned(), file_name_of),
            );
        });
        ui.add_space(6.0);
        ui.label(t::passphrase_label());
        let field = ui.add(
            egui::TextEdit::singleline(&mut self.passphrase)
                .password(true)
                .desired_width(FIELD_WIDTH),
        );
        crate::diag::ui_rect(REGION_PASSPHRASE, field.rect);
        ui.add_space(2.0);
        // ★★★ The promise about behaviour, where the secret is typed rather
        // than in a footnote. `crate::text::sign::passphrase_note` names the
        // two mechanisms that make it true.
        ui.label(
            egui::RichText::new(t::passphrase_note())
                .color(theme.palette.text_muted)
                .small(),
        );
        ui.add_space(6.0);
        // Enabled only once there is a file to open — a button that reports
        // "choose a certificate first" is a button that knew.
        let open = ui.add_enabled(
            self.certificate.is_some(),
            egui::Button::new(t::open_certificate()),
        );
        crate::diag::ui_rect(REGION_OPEN_CERTIFICATE, open.rect);
        if open.clicked() {
            self.open_certificate_requested = true;
        }

        if let Some(error) = &self.identity_error {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(error.clone()).color(theme.palette.danger));
        }

        let Some(identity) = &self.identity else {
            return;
        };
        let report = identity.report();
        ui.add_space(10.0);
        let heading = ui.label(t::identity_heading());
        crate::diag::ui_rect(REGION_IDENTITY, heading.rect);
        ui.add_space(4.0);
        ui.label(t::identity_subject(&report.subject));
        if let Some(friendly) = &report.friendly_name {
            ui.label(t::identity_friendly_name(friendly));
        }
        ui.label(t::identity_key(&report.key, report.chain_length));
        // ★★ Both directions, always. A line that appears only when something
        // is wrong is a line nobody learns to look for — and the wrong
        // direction here (a container with no MAC) is the one fact on this
        // window an operator could act on and would not otherwise be told.
        let integrity = t::identity_integrity(report.mac.as_deref());
        if report.mac.is_some() {
            ui.label(integrity);
        } else {
            ui.label(egui::RichText::new(integrity).color(theme.palette.danger));
        }
        if report.unrelated_certificates > 0 {
            ui.label(
                egui::RichText::new(t::identity_unrelated(report.unrelated_certificates))
                    .color(theme.palette.text_muted),
            );
        }
    }

    /// **What the signature will say** — `/Reason`, `/Location`, and the time.
    pub(super) fn details_section(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(t::details_heading());
        ui.add_space(6.0);
        ui.label(t::reason_label());
        ui.add(
            egui::TextEdit::singleline(&mut self.reason)
                .hint_text(t::reason_hint())
                .desired_width(FIELD_WIDTH),
        );
        ui.add_space(4.0);
        ui.label(t::location_label());
        ui.add(
            egui::TextEdit::singleline(&mut self.location)
                .hint_text(t::location_hint())
                .desired_width(FIELD_WIDTH),
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(t::authored_note())
                .color(theme.palette.text_muted)
                .small(),
        );
        ui.add_space(4.0);
        // The absence explained on screen — see `crate::text::sign`'s header.
        ui.label(
            egui::RichText::new(t::name_comes_from_the_certificate())
                .color(theme.palette.text_muted)
                .small(),
        );
        ui.add_space(6.0);
        if let Some(stamp) = &self.signing_time {
            ui.label(t::signing_time(stamp));
        }
    }

    /// **What kind of signature** — approval, or certifying as the author.
    ///
    /// `Pass 10.12`. §2d of [`crate::sign`]'s header argues why this is a radio
    /// pair here rather than a second ribbon command.
    ///
    /// ★★★ **The certifying option is ABSENT, not greyed, on a document that
    /// cannot carry one** — and the sentence explaining why is drawn in its
    /// place. R9's *explained* branch: both of the engine's certification
    /// refusals are states of the document that are knowable when this window
    /// opens, so meeting one by pressing rather than by reading is a failure
    /// this surface can avoid outright.
    ///
    /// ⚠ And the sentence ends *"you can still add an ordinary signature"*,
    /// which is the half that stops the note being read as a refusal of the
    /// whole window.
    pub(super) fn kind_section(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(t::kind_heading());
        ui.add_space(6.0);
        ui.radio_value(&mut self.certify, false, t::kind_approval());
        match self.standing.may_certify() {
            Ok(()) => {
                let certify = ui.radio_value(&mut self.certify, true, t::kind_certify());
                crate::diag::ui_rect(REGION_CERTIFY, certify.rect);
                ui.add_space(4.0);
                // Drawn whichever arm is selected, on this window's standing
                // rule: what an option does must be readable before it is
                // chosen, not after.
                ui.label(
                    egui::RichText::new(t::kind_certify_note())
                        .color(theme.palette.text_muted)
                        .small(),
                );
                if self.certify {
                    ui.add_space(8.0);
                    ui.label(t::mdp_heading());
                    ui.add_space(4.0);
                    for level in [
                        MdpPermission::NoChanges,
                        MdpPermission::FormFillAndSign,
                        MdpPermission::FormFillSignAnnotate,
                    ] {
                        ui.radio_value(&mut self.mdp, level, t::mdp_level(level));
                    }
                }
            }
            Err(bar) => {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(t::certify_unavailable(bar))
                        .color(theme.palette.text_muted),
                );
                // ★ The flag is cleared as well as the control removed. A
                // document that gains a signature under the window would
                // otherwise leave a `certify` set by a radio that is no longer
                // on screen, and `commit` would send a certification the
                // operator can no longer see he asked for. Belt and braces with
                // `commit`'s own re-ask, deliberately: this is the state, that
                // is the request, and neither should depend on the other being
                // right.
                self.certify = false;
            }
        }
    }

    /// **On the page** — nothing drawn by default; a box this shell places; or
    /// a box the document's author already placed (`Pass 10.13`).
    pub(super) fn placement_section(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(t::placement_heading());
        ui.add_space(6.0);
        ui.radio_value(&mut self.place, Place::Nothing, t::placement_invisible());
        let box_radio = ui.radio_value(&mut self.place, Place::Box, t::placement_visible());
        crate::diag::ui_rect(REGION_PLACE_BOX, box_radio.rect);

        // ★★★ `Pass 10.13`. The option is drawn even when the document holds
        // no box — disabled, with `no_existing_fields()` beneath it — which is
        // the OPPOSITE of this window's usual absent-not-greyed rule, and the
        // exception is argued at that string: the operator was told by the
        // sender that there is a box, and "the option is missing" and "the box
        // you were promised is not in this file" are the same picture and
        // completely different facts.
        let fields = &self.standing.empty_fields;
        let any = fields.iter().any(crate::sign::SigField::selectable);
        let existing = ui.add_enabled(
            any,
            egui::RadioButton::new(
                self.place == Place::Existing,
                t::placement_existing(fields.len()),
            ),
        );
        crate::diag::ui_rect(REGION_EXISTING, existing.rect);
        if existing.clicked() && any {
            self.place = Place::Existing;
        }
        if !any {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(t::no_existing_fields())
                    .color(theme.palette.text_muted)
                    .small(),
            );
        }

        match self.place {
            // ★★★ R8b Rule 4: what the box contains, said whichever arm is
            // selected — a note that appeared only once *draw a box* was picked
            // would arrive one press too late. ★ The sentence was CORRECTED on
            // 2026-09-06 when the pin moved; see `text::sign::placement_note`.
            Place::Nothing | Place::Box => {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(t::placement_note()).color(
                    if self.place == Place::Box {
                        theme.palette.danger
                    } else {
                        theme.palette.text_muted
                    },
                ));
                if self.place == Place::Box {
                    ui.add_space(4.0);
                    let where_line = ui.label(
                        egui::RichText::new(t::placement_where()).color(theme.palette.text_muted),
                    );
                    crate::diag::ui_rect(REGION_BOX_WHERE, where_line.rect);
                    ui.add_space(4.0);
                    // ★ A one-page document gets no chooser — a control with
                    // one possible value is a label pretending to be a choice.
                    if self.standing.pages > 1 {
                        ui.horizontal(|ui| {
                            ui.label(t::page_label());
                            // 1-based on screen, 0-based in the request. The
                            // engine takes an index; an operator counts pages.
                            let mut shown = self.page + 1;
                            let drag = ui.add(
                                egui::DragValue::new(&mut shown).range(1..=self.standing.pages),
                            );
                            crate::diag::ui_rect(REGION_PAGE, drag.rect);
                            self.page = shown.saturating_sub(1);
                        });
                    }
                }
            }
            // ★★★ THE PLACEMENT CONTROLS RETIRE, AND THE SENTENCE SAYS WHY.
            // The engine refuses `--visible`/`--page` alongside a field name by
            // name; this shell makes the combination unrepresentable, so
            // `placement_note`, `placement_where` and the page chooser are all
            // simply gone — and a control that vanishes without explanation is
            // indistinguishable from one that broke.
            Place::Existing => {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(t::placement_field_note()).color(theme.palette.text_muted),
                );
                ui.add_space(6.0);
                self.field_list(ui, theme);
            }
        }
    }

    /// **The pre-placed signature fields, one row each, with what the author
    /// attached to them.**
    ///
    /// ★★★ The two disclosures below a row — the `/Lock` and the `/SV` — are
    /// drawn **here, before the press**, and that is the whole point of reading
    /// them out of the document rather than waiting for `SignReport`. Signing a
    /// locked box freezes fields the author nominated; a consequence the
    /// operator learns about after the file is written is a consequence he did
    /// not consent to.
    fn field_list(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        // Cloned rather than borrowed, because the loop below assigns to
        // `self.field`. Three small strings per field, once per frame, on a
        // list that is almost always one item long — the alternative is an
        // index dance that makes the disclosure order harder to read than the
        // disclosure itself.
        let fields = self.standing.empty_fields.clone();
        for (index, field) in fields.iter().enumerate() {
            ui.add_space(4.0);
            let usable = field.selectable();
            let row = ui.add_enabled(
                usable,
                egui::RadioButton::new(
                    usable && self.place == Place::Existing && self.field == index,
                    t::field_row(&field.name, field.page),
                ),
            );
            crate::diag::ui_rect(&field_region(index), row.rect);
            if row.clicked() && usable {
                self.field = index;
            }
            ui.indent(field_region(index), |ui| {
                if let Some(bar) = field.unusable {
                    ui.label(
                        egui::RichText::new(t::field_unusable(bar)).color(theme.palette.danger),
                    );
                    return;
                }
                // ★★ The LOCK is `danger`-coloured and the others are muted,
                // and the split is by consequence rather than by severity: the
                // lock changes what the operator can do to his own document
                // afterwards, and the other two describe what he will see.
                if let Some(action) = &field.locks {
                    ui.label(
                        egui::RichText::new(t::field_locks(action)).color(theme.palette.danger),
                    );
                }
                if field.constrained {
                    ui.label(
                        egui::RichText::new(t::field_constrained())
                            .color(theme.palette.text_muted)
                            .small(),
                    );
                }
                if field.invisible {
                    ui.label(
                        egui::RichText::new(t::field_invisible())
                            .color(theme.palette.text_muted)
                            .small(),
                    );
                }
            });
        }
    }

    /// **Where the signed document goes.** Drawn only when there is an
    /// original to replace.
    pub(super) fn destination_section(&mut self, ui: &mut egui::Ui) {
        if !self.can_replace_original() {
            return;
        }
        let name = file_name_of(&self.source);
        ui.label(tp::destination_heading());
        ui.add_space(2.0);
        let mut choice = self.destination;
        ui.radio_value(
            &mut choice,
            Destination::NewFile,
            tp::destination_new_file(),
        )
        .on_hover_text(tp::destination_new_file_tooltip());
        ui.radio_value(
            &mut choice,
            Destination::ReplaceOriginal,
            tp::destination_replace(&name),
        )
        .on_hover_text(tp::destination_replace_tooltip());
        self.choose_destination(choice);
        ui.add_space(6.0);
        // Asked for only while it applies, for the reason above.
        if self.destination == Destination::ReplaceOriginal {
            ui.checkbox(
                &mut self.overwrite_acknowledged,
                tp::overwrite_acknowledgement_checkbox(&name),
            );
        }
    }

    /// The confirm control, and the sentence that explains it when it is
    /// greyed.
    pub(super) fn confirm_row(&mut self, ui: &mut egui::Ui) {
        // ★ The label IS the consequence, and the consequence depends on the
        // destination: an ellipsis promises the picker, and naming the file
        // promises there will be no further question before it is replaced.
        // Promising one with a punctuation mark and not asking it would be a
        // lie the operator acts on.
        let label = match self.destination {
            Destination::NewFile => t::confirm_button().to_owned(),
            Destination::ReplaceOriginal => t::confirm_button_replace(&file_name_of(&self.source)),
        };
        let ready = self.ready_to_confirm();
        let confirm = ui.add_enabled(ready, egui::Button::new(label));
        // Declared only while it is live, so its absence from a trace is
        // evidence the gates are closed rather than evidence a click missed.
        if ready {
            crate::diag::ui_rect(REGION_CONFIRM, confirm.rect);
        }
        let clicked = confirm.clicked();
        // ★★ The `if !ready` shape and the borrow order are copied from
        // `dialogs::redact`: `on_disabled_hover_text` CONSUMES the response, so
        // `.rect` and `.clicked()` are read first.
        if !ready {
            confirm.on_disabled_hover_text(self.disabled_reason());
        }
        if clicked {
            self.confirm_requested = true;
        }
    }
}
