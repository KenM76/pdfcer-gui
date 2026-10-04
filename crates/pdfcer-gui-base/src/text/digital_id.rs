//! # `text::digital_id` — every operator-facing string on the Sign window's
//! *Create a digital ID* form
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/digital_id.md`.

use pdfcer_core::sign::digital_id::{IdError, IdKeyAlgorithm};

/// The button beside *Choose certificate…*.
#[must_use]
pub const fn create_button() -> &'static str {
    "Create a digital ID…"
}

/// Hover on [`create_button`].
#[must_use]
pub const fn create_hover() -> &'static str {
    "Make a new certificate file (.pfx) to sign with, if you do not have one."
}

/// The form's heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Create a digital ID"
}

/// What a self-signed ID proves and what it does not, above the fields.
#[must_use]
pub const fn disclosure() -> &'static str {
    "A digital ID you create here is self-signed. A signature made with it \
     proves that the document has not changed since it was signed, and that \
     whoever signed held this file and its password. It does not prove who \
     you are: readers see your name as unverified until they choose to trust \
     your certificate. Send them the certificate, and read them its \
     fingerprint so they can check it is yours."
}

/// The name field.
#[must_use]
pub const fn name_label() -> &'static str {
    "Your name"
}

/// The organisation field.
#[must_use]
pub const fn organisation_label() -> &'static str {
    "Organisation (optional)"
}

/// The e-mail field.
#[must_use]
pub const fn email_label() -> &'static str {
    "E-mail (optional)"
}

/// The country field.
#[must_use]
pub const fn country_label() -> &'static str {
    "Country code (optional, two letters, such as CA)"
}

/// The key chooser.
#[must_use]
pub const fn key_label() -> &'static str {
    "Key"
}

/// One key choice.
#[must_use]
pub const fn key_choice(key: IdKeyAlgorithm) -> &'static str {
    match key {
        IdKeyAlgorithm::Rsa3072 => "RSA 3072-bit",
        IdKeyAlgorithm::EcdsaP256 => "ECDSA P-256 (signing only)",
        _ => "RSA 2048-bit (what Acrobat creates)",
    }
}

/// The encryption-usage tick box.
#[must_use]
pub const fn encrypt_label() -> &'static str {
    "Also usable for encrypting documents sent to me"
}

/// Why [`encrypt_label`] is greyed for an ECDSA key.
#[must_use]
pub const fn encrypt_needs_rsa() -> &'static str {
    "An ECDSA key can only sign. Choose an RSA key to use the ID for encryption too."
}

/// The validity field's label.
#[must_use]
pub const fn validity_label() -> &'static str {
    "Valid for"
}

/// The validity field's unit.
#[must_use]
pub const fn validity_suffix() -> &'static str {
    " years"
}

/// The password field.
#[must_use]
pub const fn password_label() -> &'static str {
    "Password for the new file"
}

/// The repeated password field.
#[must_use]
pub const fn confirm_label() -> &'static str {
    "Type the password again"
}

/// What the password does, said where it is typed.
#[must_use]
pub const fn password_note() -> &'static str {
    "The password protects the file and cannot be recovered: without it the \
     file cannot sign. It is not saved or written to any log."
}

/// The button that creates and saves the file.
#[must_use]
pub const fn create_go() -> &'static str {
    "Create and save…"
}

/// Closes the form without creating anything.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// Shown while the key is generated.
#[must_use]
pub const fn working() -> &'static str {
    "Creating the key. This takes a few seconds."
}

/// The two password fields differ.
#[must_use]
pub const fn mismatch() -> &'static str {
    "The two passwords are different. Type the same password in both boxes."
}

/// The save dialog's title.
#[must_use]
pub const fn save_title() -> &'static str {
    "Save the new digital ID"
}

/// The save dialog's filter.
#[must_use]
pub const fn pfx_filter() -> &'static str {
    "Digital ID (*.pfx)"
}

/// The file name offered when the name field gives nothing usable.
#[must_use]
pub const fn default_file_stem() -> &'static str {
    "digital-id"
}

/// The file could not be written.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("The file could not be written: {detail}")
}

/// The clock is before 1970, so no validity can be dated.
#[must_use]
pub const fn clock_unusable() -> &'static str {
    "This computer's clock is set before 1970, so the certificate cannot be \
     given a start date. Correct the clock and try again."
}

/// The engine's refusal, in words the operator can act on.
#[must_use]
pub fn refusal(error: &IdError) -> String {
    match error {
        IdError::EmptyCommonName => {
            "Type your name. It is the name your signatures show.".to_owned()
        }
        IdError::EmptyPassword => {
            "Type a password. The file cannot be created without one.".to_owned()
        }
        IdError::PasswordCharacter { character } => format!(
            "The password contains “{character}”, which a .pfx password cannot hold. Use \
             letters, digits and ordinary symbols."
        ),
        IdError::BadCountry { value } => format!(
            "“{value}” is not a country code. Use two letters, such as CA, or leave it empty."
        ),
        IdError::BadEmail { value } => format!(
            "“{value}” cannot go in a certificate as an e-mail address: it must be plain \
             letters, digits and symbols with one @, at most 255 characters."
        ),
        IdError::FieldTooLong { field, len, max } => format!(
            "The {} is too long: {len} bytes, and the limit is {max}.",
            field_name(field)
        ),
        IdError::ValidityOutOfRange { years } => {
            format!("{years} years is not allowed. Choose between 1 and 100 years.")
        }
        IdError::EncryptionNeedsRsa => encrypt_needs_rsa().to_owned(),
        IdError::RandomUnavailable(detail) => {
            format!("This computer gave no random numbers to make a key from: {detail}")
        }
        IdError::KeyOperation(detail) => format!("Making the key failed: {detail}"),
        other => format!("The digital ID was not created: {other}"),
    }
}

/// The engine's field name, as this form labels it.
fn field_name(field: &str) -> &str {
    match field {
        "common name" => "name",
        other => other,
    }
}

/// The success sentence.
#[must_use]
pub fn created(file: &str) -> String {
    format!("Created {file}. It is now the certificate chosen above, and is open.")
}

/// The key and the end of validity.
#[must_use]
pub fn validity(key: &str, until: &str) -> String {
    format!("Key: {key}. Valid until {until}.")
}

/// The fingerprint's label.
#[must_use]
pub const fn fingerprint_label() -> &'static str {
    "SHA-256 fingerprint"
}

/// Copies the fingerprint.
#[must_use]
pub const fn copy_fingerprint() -> &'static str {
    "Copy fingerprint"
}

/// Saves the certificate alone, for others to trust.
#[must_use]
pub const fn share() -> &'static str {
    "Save the certificate to share…"
}

/// Hover on [`share`].
#[must_use]
pub const fn share_hover() -> &'static str {
    "Save the certificate without your key, as a .cer file, so the people who \
     check your signatures can choose to trust it."
}

/// The share save dialog's title.
#[must_use]
pub const fn share_title() -> &'static str {
    "Save the certificate to share"
}

/// The share save dialog's filter.
#[must_use]
pub const fn cer_filter() -> &'static str {
    "Certificate (*.cer)"
}

/// The certificate was saved.
#[must_use]
pub fn shared(file: &str) -> String {
    format!("Saved {file}. It holds no key and no password; it is safe to send.")
}

/// Closes the finished form.
#[must_use]
pub const fn done() -> &'static str {
    "Done"
}
