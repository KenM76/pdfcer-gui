//! # `app::opennotes` — what the operator is told the moment a file opens
//!
//! Two questions, asked once, at the moment the operator has the file and has
//! not yet acted on it, answered on the status bar's disclosure row:
//!
//! 1. **Is the visible page a cover sheet?** A §7.6.7 wrapper shows an
//!    unencrypted cover in front of a payload pdfcer cannot decrypt; rendered
//!    silently, it reads as the drawing. `wrapper::detect` is one catalog
//!    lookup.
//! 2. **Does the file reach outside itself?** `app::reachout`.
//! 3. **Is the page a dynamic form's stand-in?** A dynamic XFA form's PDF
//!    pages are written for programs without a form engine; its own pages are
//!    in `view.form_preview`. `pdfcer_core::xfa::form_kind` is one catalog
//!    lookup.
//!
//! All are sentences, never dialogs: pdfcer executes no action, so nothing
//! is about to happen. Silent on an ordinary file. Each sentence is traced in
//! full (`wrapper-disclosed`, `reach-out-disclosed`), because the status row
//! traces nothing and the wording is the feature.

use crate::app::state::OpenDoc;

/// Record the open-time sentences for `doc`, the wrapper's first.
pub fn disclose(doc: &OpenDoc) {
    let mut notes = Vec::new();
    let wrapper = pdfcer_core::wrapper::detect(&doc.session.view());
    if let Some(engine) = wrapper.message() {
        let sentence = crate::text::securitynotes::wrapper_status(&engine);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "wrapper-disclosed payloads={} named={}",
                wrapper.payload_count,
                u8::from(wrapper.payload_name.is_some())
            )
        });
        notes.push(sentence);
    }
    let reach = crate::app::reachout::scan(&doc.session);
    if reach.worth_saying() {
        let sentence = crate::text::reachout::disclosure(reach);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed. It carries
            // the operator-facing sentence so a check can read it.
            format!("reach-out-disclosed text={sentence:?}")
        });
        notes.push(sentence);
    }
    if pdfcer_core::xfa::form_kind(&*doc.session).is_dynamic() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "xfa-dynamic-disclosed".to_owned()
        });
        notes.push(crate::text::xfapreview::opened_dynamic().to_owned());
    }
    notes.extend(crate::app::rc4::on_open(doc));
    if !notes.is_empty() {
        crate::app::actions::record_notes(doc.edit_epoch, notes);
    }
}
