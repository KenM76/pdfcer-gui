//! # `dialogs::sign::create_id` — the Sign window's *Create a digital ID* form
//!
//! Contract: opened by the button beside *Choose certificate…* and drawn in
//! the certificate section. *Create and save…* generates the ID with
//! `create_self_signed_id` on a worker thread (an RSA key takes seconds), and
//! only then asks where to save it, so every refusal arrives before the save
//! dialog. The saved `.pfx` becomes the window's chosen certificate, its
//! password the passphrase, and the identity is opened at once. The SHA-256
//! fingerprint is shown, and the certificate alone can be saved as `.cer` to
//! share. A child of `dialogs::sign` so it can set the window's private
//! certificate fields without widening them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/sign/create_id.md`.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

use egui_shell::theme::Theme;
use pdfcer_core::sign::digital_id::{
    DigitalId, DigitalIdSpec, IdError, IdKeyAlgorithm, IdUsage, create_self_signed_id,
};

use super::{FIELD_WIDTH, SignDialog, file_name_of};
use crate::app::files::{self, Picked};
use crate::text::digital_id as t;

const REGION_OPEN: &str = "sign-create-id"; // ui-text-exempt: trace region name, never displayed
const REGION_NAME: &str = "sign-create-name"; // ui-text-exempt: trace region name, never displayed
const REGION_PASSWORD: &str = "sign-create-password"; // ui-text-exempt: trace region name, never displayed
const REGION_CONFIRM: &str = "sign-create-confirm"; // ui-text-exempt: trace region name, never displayed
const REGION_GO: &str = "sign-create-go"; // ui-text-exempt: trace region name, never displayed
const REGION_ERROR: &str = "sign-create-error"; // ui-text-exempt: trace region name, never displayed
const REGION_FINGERPRINT: &str = "sign-create-fingerprint"; // ui-text-exempt: trace region name, never displayed
const REGION_SHARE: &str = "sign-create-share"; // ui-text-exempt: trace region name, never displayed
const REGION_DONE: &str = "sign-create-done"; // ui-text-exempt: trace region name, never displayed

/// The keys offered, in the order shown; the first is the default.
const KEYS: [IdKeyAlgorithm; 3] = [
    IdKeyAlgorithm::Rsa2048,
    IdKeyAlgorithm::Rsa3072,
    IdKeyAlgorithm::EcdsaP256,
];

/// The form's state while it is open.
pub(super) struct CreateId {
    name: String,
    organisation: String,
    email: String,
    country: String,
    key: IdKeyAlgorithm,
    encrypt: bool,
    years: u16,
    password: String,
    confirm: String,
    stage: Stage,
    go_requested: bool,
    share_requested: bool,
    close_requested: bool,
}

/// Where one creation has got to.
enum Stage {
    /// Being filled in; the refusal from the last attempt, if any.
    Filling(Option<String>),
    /// The worker is generating the key.
    Working(Receiver<Result<DigitalId, IdError>>),
    /// Saved, opened, and shown.
    Done(Made),
}

/// What a finished creation shows.
struct Made {
    file: String,
    certificate: Vec<u8>,
    fingerprint: String,
    validity: String,
    shared: Option<String>,
}

impl Default for CreateId {
    fn default() -> Self {
        Self {
            name: String::new(),
            organisation: String::new(),
            email: String::new(),
            country: String::new(),
            key: KEYS[0],
            encrypt: false,
            years: 5,
            password: String::new(),
            confirm: String::new(),
            stage: Stage::Filling(None),
            go_requested: false,
            share_requested: false,
            close_requested: false,
        }
    }
}

impl SignDialog {
    /// The *Create a digital ID…* button, drawn beside the certificate picker
    /// while the form is closed.
    pub(super) fn create_id_button(&mut self, ui: &mut egui::Ui) {
        if self.create.is_some() {
            return;
        }
        let button = ui
            .button(t::create_button())
            .on_hover_text(t::create_hover());
        crate::diag::ui_rect(REGION_OPEN, button.rect);
        if button.clicked() {
            self.create = Some(CreateId::default());
        }
    }

    /// The form, below the certificate picker, while it is open.
    pub(super) fn create_id_section(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let Some(form) = &mut self.create else {
            return;
        };
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.label(
                egui::RichText::new(t::heading())
                    .strong()
                    .color(theme.palette.text),
            );
            ui.add_space(4.0);
            match &form.stage {
                Stage::Done(_) => form.done_view(ui, theme),
                _ => form.filling_view(ui, theme),
            }
        });
    }

    /// Everything with a side effect, after the window's draw closure.
    pub(super) fn create_id_after(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.create.take() else {
            return;
        };
        if std::mem::take(&mut form.go_requested) && !matches!(form.stage, Stage::Working(_)) {
            form.start(ctx);
        }
        if let Stage::Working(rx) = &form.stage {
            match rx.try_recv() {
                Ok(Ok(id)) => self.adopt(&mut form, &id),
                Ok(Err(error)) => {
                    trace_refused(refusal_token(&error));
                    form.stage = Stage::Filling(Some(t::refusal(&error)));
                }
                Err(TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                }
                Err(TryRecvError::Disconnected) => {
                    trace_refused("worker-gone");
                    form.stage =
                        Stage::Filling(Some(t::refusal(&IdError::KeyOperation(String::new()))));
                }
            }
        }
        if std::mem::take(&mut form.share_requested) {
            form.share(&self.source);
        }
        if !std::mem::take(&mut form.close_requested) {
            self.create = Some(form);
        }
    }

    /// Ask where the new ID goes, write it, and make it the chosen
    /// certificate with its identity open.
    fn adopt(&mut self, form: &mut CreateId, id: &DigitalId) {
        let suggested = suggested(&self.source, &form.name, "pfx"); // ui-text-exempt: a file extension, never displayed
        let Picked::Path(path) = files::pick_new_digital_id_target(&suggested) else {
            form.stage = Stage::Filling(None);
            return;
        };
        if let Err(e) = std::fs::write(&path, &id.pfx) {
            trace_refused("write");
            form.stage = Stage::Filling(Some(t::write_failed(&e.to_string())));
            return;
        }
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed. The fingerprint
            // is public by design; the path and the password are not traced.
            format!(
                "digital-id-created key={} years={} encrypt={} bytes={} fingerprint={}",
                key_token(form.key),
                form.years,
                u8::from(form.encrypt),
                id.pfx.len(),
                hex(&id.sha256_fingerprint, "")
            )
        });
        self.certificate = Some(path.clone());
        self.passphrase = std::mem::take(&mut form.password);
        form.confirm.clear();
        self.identity = None;
        self.identity_error = None;
        self.open_identity();
        form.stage = Stage::Done(Made {
            file: file_name_of(&path),
            certificate: id.certificate.clone(),
            fingerprint: fingerprint_lines(&id.sha256_fingerprint),
            validity: t::validity(&id.key_label, &id.valid_until),
            shared: None,
        });
    }
}

impl CreateId {
    /// Check the passwords agree, then start the worker.
    fn start(&mut self, ctx: &egui::Context) {
        if self.password != self.confirm {
            trace_refused("mismatch");
            self.stage = Stage::Filling(Some(t::mismatch().to_owned()));
            return;
        }
        let Some(now) = crate::app::clock::unix_now() else {
            trace_refused("clock");
            self.stage = Stage::Filling(Some(t::clock_unusable().to_owned()));
            return;
        };
        let spec = self.spec(now);
        let password = self.password.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let repaint = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(create_self_signed_id(&spec, &password));
            repaint.request_repaint();
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "digital-id-requested key={} years={} encrypt={}",
                key_token(self.key),
                self.years,
                u8::from(self.encrypt)
            )
        });
        self.stage = Stage::Working(rx);
    }

    /// The engine's spec for what the form holds, starting at `now`.
    fn spec(&self, now: u64) -> DigitalIdSpec {
        let mut spec = DigitalIdSpec::new(self.name.trim(), now);
        spec.organization = self.organisation.trim().to_owned();
        spec.email = self.email.trim().to_owned();
        spec.country = self.country.trim().to_owned();
        spec.key = self.key;
        spec.usage = if self.encrypt {
            IdUsage::SigningAndEncryption
        } else {
            IdUsage::Signing
        };
        spec.valid_years = self.years;
        spec
    }

    /// Ask where the bare certificate goes, and write it.
    fn share(&mut self, source: &Path) {
        let Stage::Done(made) = &mut self.stage else {
            return;
        };
        let suggested = suggested(source, &self.name, "cer"); // ui-text-exempt: a file extension, never displayed
        let Picked::Path(path) = files::pick_shared_certificate_target(&suggested) else {
            return;
        };
        made.shared = Some(match std::fs::write(&path, &made.certificate) {
            Ok(()) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("digital-id-shared bytes={}", made.certificate.len())
                });
                t::shared(&file_name_of(&path))
            }
            Err(e) => {
                trace_refused("share-write");
                t::write_failed(&e.to_string())
            }
        });
    }

    /// The fields, the refusal, and the buttons.
    fn filling_view(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(
            egui::RichText::new(t::disclosure())
                .color(theme.palette.text_muted)
                .small(),
        );
        ui.add_space(6.0);
        let working = matches!(self.stage, Stage::Working(_));
        ui.add_enabled_ui(!working, |ui| {
            self.identity_fields(ui);
            self.key_fields(ui);
            self.password_fields(ui, theme);
        });
        if let Stage::Filling(Some(error)) = &self.stage {
            ui.add_space(4.0);
            let label = ui.label(egui::RichText::new(error.clone()).color(theme.palette.danger));
            crate::diag::ui_rect(REGION_ERROR, label.rect);
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if working {
                ui.add(egui::Spinner::new().color(theme.palette.text));
                ui.label(t::working());
                return;
            }
            let go = ui.button(t::create_go());
            crate::diag::ui_rect(REGION_GO, go.rect);
            self.go_requested |= go.clicked();
            self.close_requested |= ui.button(t::cancel()).clicked();
        });
    }

    /// Name, organisation, e-mail and country.
    fn identity_fields(&mut self, ui: &mut egui::Ui) {
        let rows = [
            (t::name_label(), &mut self.name, Some(REGION_NAME)),
            (t::organisation_label(), &mut self.organisation, None),
            (t::email_label(), &mut self.email, None),
            (t::country_label(), &mut self.country, None),
        ];
        for (label, value, region) in rows {
            ui.label(label);
            // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
            // every field in this window: the first press leaves the box, the second
            // cancels.
            let field = ui.add(egui::TextEdit::singleline(value).desired_width(FIELD_WIDTH));
            if let Some(region) = region {
                crate::diag::ui_rect(region, field.rect);
            }
        }
    }

    /// The key, what it may be used for, and how long it lasts.
    fn key_fields(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(t::key_label());
            egui::ComboBox::from_id_salt("sign-create-key")
                .selected_text(t::key_choice(self.key))
                .show_ui(ui, |ui| {
                    for key in KEYS {
                        ui.selectable_value(&mut self.key, key, t::key_choice(key));
                    }
                });
        });
        let rsa = self.key != IdKeyAlgorithm::EcdsaP256;
        self.encrypt &= rsa;
        ui.add_enabled(
            rsa,
            egui::Checkbox::new(&mut self.encrypt, t::encrypt_label()),
        )
        .on_disabled_hover_text(t::encrypt_needs_rsa());
        ui.horizontal(|ui| {
            ui.label(t::validity_label());
            ui.add(
                egui::DragValue::new(&mut self.years)
                    .range(1..=100)
                    .suffix(t::validity_suffix()),
            );
        });
    }

    /// The password, typed twice, and what is done with it.
    fn password_fields(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.add_space(4.0);
        for (label, value, region) in [
            (t::password_label(), &mut self.password, REGION_PASSWORD),
            (t::confirm_label(), &mut self.confirm, REGION_CONFIRM),
        ] {
            ui.label(label);
            // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
            // every field in this window: the first press leaves the box, the second
            // cancels.
            let field = ui.add(
                egui::TextEdit::singleline(value)
                    .password(true)
                    .desired_width(FIELD_WIDTH),
            );
            crate::diag::ui_rect(region, field.rect);
        }
        ui.label(
            egui::RichText::new(t::password_note())
                .color(theme.palette.text_muted)
                .small(),
        );
    }

    /// What was made, its fingerprint, and the share and close buttons.
    fn done_view(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let Stage::Done(made) = &self.stage else {
            return;
        };
        ui.label(t::created(&made.file));
        ui.label(&made.validity);
        ui.add_space(4.0);
        ui.label(t::fingerprint_label());
        let print = ui.label(egui::RichText::new(&made.fingerprint).monospace());
        crate::diag::ui_rect(REGION_FINGERPRINT, print.rect);
        let fingerprint = made.fingerprint.clone();
        if let Some(shared) = &made.shared {
            ui.label(egui::RichText::new(shared.clone()).color(theme.palette.text_muted));
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button(t::copy_fingerprint()).clicked() {
                ui.ctx().copy_text(fingerprint);
            }
            let share = ui.button(t::share()).on_hover_text(t::share_hover());
            crate::diag::ui_rect(REGION_SHARE, share.rect);
            self.share_requested |= share.clicked();
            let done = ui.button(t::done());
            crate::diag::ui_rect(REGION_DONE, done.rect);
            self.close_requested |= done.clicked();
        });
    }
}

/// A save location beside `source`, named after `name` when it gives one.
fn suggested(source: &Path, name: &str, extension: &str) -> PathBuf {
    let stem: String = name
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .collect();
    let stem = if stem.trim().is_empty() {
        t::default_file_stem()
    } else {
        stem.trim()
    };
    source
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(format!("{stem}.{extension}"))
}

/// `bytes` as upper-case hex pairs joined by `sep`.
fn hex(bytes: &[u8], sep: &str) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(sep)
}

/// The fingerprint as two lines of sixteen pairs, for reading aloud.
fn fingerprint_lines(fingerprint: &[u8; 32]) -> String {
    format!(
        "{}\n{}",
        hex(&fingerprint[..16], " "), // ui-text-exempt: a separator between hex pairs
        hex(&fingerprint[16..], " ")  // ui-text-exempt: a separator between hex pairs
    )
}

/// The key as a trace token.
fn key_token(key: IdKeyAlgorithm) -> &'static str {
    match key {
        IdKeyAlgorithm::Rsa3072 => "rsa3072", // ui-text-exempt: trace token, never displayed
        IdKeyAlgorithm::EcdsaP256 => "p256",  // ui-text-exempt: trace token, never displayed
        _ => "rsa2048",                       // ui-text-exempt: trace token, never displayed
    }
}

/// The refusal as a trace token.
fn refusal_token(error: &IdError) -> &'static str {
    match error {
        IdError::EmptyCommonName => "empty-name", // ui-text-exempt: trace token, never displayed
        IdError::EmptyPassword => "empty-password", // ui-text-exempt: trace token, never displayed
        IdError::PasswordCharacter { .. } => "password-character", // ui-text-exempt: trace token, never displayed
        IdError::BadCountry { .. } => "bad-country", // ui-text-exempt: trace token, never displayed
        IdError::BadEmail { .. } => "bad-email",     // ui-text-exempt: trace token, never displayed
        IdError::FieldTooLong { .. } => "too-long",  // ui-text-exempt: trace token, never displayed
        IdError::ValidityOutOfRange { .. } => "validity", // ui-text-exempt: trace token, never displayed
        IdError::IterationsOutOfRange { .. } => "iterations", // ui-text-exempt: trace token, never displayed
        IdError::EncryptionNeedsRsa => "encryption-needs-rsa", // ui-text-exempt: trace token, never displayed
        IdError::RandomUnavailable(_) => "random", // ui-text-exempt: trace token, never displayed
        IdError::KeyOperation(_) => "key-operation", // ui-text-exempt: trace token, never displayed
        _ => "other",                              // ui-text-exempt: trace token, never displayed
    }
}

/// Trace a refusal by token; the sentence is on screen.
fn trace_refused(kind: &'static str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("digital-id-refused kind={kind}")
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggested_names_the_file_after_the_holder() {
        let p = suggested(Path::new("C:/a/doc.pdf"), " Jane O'Neil ", "pfx");
        assert_eq!(p, Path::new("C:/a").join("Jane ONeil.pfx"));
        let p = suggested(Path::new("doc.pdf"), "  ", "cer");
        assert_eq!(p, Path::new("").join("digital-id.cer"));
    }

    #[test]
    fn fingerprint_reads_as_two_lines_of_sixteen() {
        let mut f = [0u8; 32];
        f[0] = 0xAB;
        f[31] = 0x01;
        let s = fingerprint_lines(&f);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("AB 00"));
        assert!(lines[1].ends_with("00 01"));
        assert_eq!(lines[0].split(' ').count(), 16);
    }

    #[test]
    fn spec_carries_the_form() {
        let form = CreateId {
            name: " Jane ".into(),
            encrypt: true,
            years: 7,
            ..CreateId::default()
        };
        let spec = form.spec(1_790_000_000);
        assert_eq!(spec.common_name, "Jane");
        assert_eq!(spec.usage, IdUsage::SigningAndEncryption);
        assert_eq!(spec.valid_years, 7);
        assert_eq!(spec.not_before_unix, 1_790_000_000);
    }
}
