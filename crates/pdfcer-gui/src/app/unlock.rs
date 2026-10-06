//! # `app::unlock` — edits an encrypted document refuses, and reopening it
//!
//! An engine verb refuses an edit to an encrypted document for one of two
//! causes (`EncryptedRefusal`): the password that opened it does not grant the
//! permission, or it uses RC4 and the session has not allowed edits under it.
//! The funnel asks [`Refusal::encrypted`] of the verb's typed error and, when
//! it is that refusal, puts the cause's own decline in the slot with its
//! remedy: Allow edits under RC4, or [`COMMAND`].
//!
//! # Contract
//!
//! - The refusal is recognised by variant, never by message: the engine's
//!   reply to G118 calls the message wording, not contract.
//! - [`COMMAND`] is shown while the document was opened without its owner
//!   password and withholds an edit permission ([`SHOWN`]); it is enabled when
//!   that holds and the document has no unsaved changes ([`READY`]). Pressed,
//!   the slot becomes `Status::NeedsPassword` with its own reading, so the
//!   ordinary password prompt asks again.
//! - Traces `edit-encrypted-refused cause=password|rc4` and
//!   `unlock-reopen path=`.

use pdfcer_core::crypto::{AuthKind, PermissionBit};
use pdfcer_core::document::EncryptedRefusal;

use crate::app::state::OpenDoc;

/// The command that reopens the document for its owner password.
pub const COMMAND: &str = "file.unlock"; // ui-text-exempt: command id, never displayed

/// Published while [`COMMAND`] is shown.
pub const SHOWN: &str = "doc.locked"; // ui-text-exempt: condition name, never displayed

/// Published while [`COMMAND`] may be pressed.
pub const READY: &str = "doc.unlockable"; // ui-text-exempt: condition name, never displayed

/// The permissions some editing verb needs (ISO 32000-2 Table 22).
const EDIT_BITS: [PermissionBit; 4] = [
    PermissionBit::ModifyContents,
    PermissionBit::Annotate,
    PermissionBit::FillForms,
    PermissionBit::Assemble,
];

/// An engine error that may be the encryption refusal.
pub trait Refusal {
    /// Whether this is the verb's encryption refusal variant.
    fn encrypted(&self) -> bool;
}

impl Refusal for pdfcer_core::edit::EditError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::DocumentEncrypted)
    }
}

impl Refusal for pdfcer_core::text_edit::EditError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Encrypted)
    }
}

impl Refusal for pdfcer_core::text_edit::AddTextError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Encrypted)
    }
}

impl Refusal for pdfcer_core::text_edit::FormatError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Encrypted)
    }
}

impl Refusal for pdfcer_core::text_edit::ReflowApplyError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Encrypted)
    }
}

impl Refusal for pdfcer_core::text_edit::BlockEditError {
    fn encrypted(&self) -> bool {
        match self {
            Self::Block(e) => e.encrypted(),
            Self::Text(e) => e.encrypted(),
            _ => false,
        }
    }
}

impl Refusal for pdfcer_core::text_edit::PlaceTextError {
    fn encrypted(&self) -> bool {
        match self {
            Self::Insert(e) => e.encrypted(),
            Self::Add(e) => e.encrypted(),
            _ => false,
        }
    }
}

impl Refusal for pdfcer_core::ocr::layer::OcrLayerError {
    fn encrypted(&self) -> bool {
        matches!(self, Self::Encrypted)
    }
}

/// A shell-worded error: the verb that produced it recorded its own decline.
impl Refusal for String {
    fn encrypted(&self) -> bool {
        false
    }
}

/// Put the cause's decline in the slot when `error` is the encryption
/// refusal. Overwrites what the verb said: its sentence could not know the
/// cause.
pub fn record(doc: &OpenDoc, error: &impl Refusal) {
    if !error.encrypted() {
        return;
    }
    let Some(cause) = doc.session.encryption_refusal_cause() else {
        return;
    };
    record_cause(cause);
}

/// [`record`] for a caller that already holds the session and knows the
/// refusal was encryption's.
pub fn record_cause(cause: EncryptedRefusal) {
    let rc4 = cause == EncryptedRefusal::Rc4NotAllowed;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "edit-encrypted-refused cause={}",
            if rc4 { "rc4" } else { "password" }
        )
    });
    if rc4 {
        crate::app::status::decline::record_rc4_refused();
    } else {
        crate::app::status::decline::record_password_refused();
    }
}

/// Whether `doc` was opened without its owner password and withholds an edit.
#[must_use]
pub fn locked(doc: &OpenDoc) -> bool {
    doc.session
        .document()
        .encryption()
        .is_some_and(|e| e.auth != AuthKind::Owner && EDIT_BITS.iter().any(|&b| !e.grants(b)))
}

/// Publish [`SHOWN`] and [`READY`].
pub fn publish(doc: &OpenDoc, set: &mut egui_shell::commands::ConditionSet) {
    if locked(doc) {
        set.set(SHOWN);
        if !crate::app::save::has_unsaved_edits(doc) {
            set.set(READY);
        }
    }
}
