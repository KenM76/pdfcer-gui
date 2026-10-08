//! Security ▸ Add validation evidence…: embed operator-chosen
//! certificates, CRLs and OCSP responses in the signed document's `/DSS`
//! (PAdES B-LT), through `EditSession::add_validation_material`.
//!
//! One undo entry (`CommandKind::AddValidationMaterial`), applied to the open document; the shell's Save appends an
//! incremental update, which is what keeps the existing signatures valid.
//! The engine fetches nothing and this shell fetches nothing: the operator
//! supplies the files. The signers' own certificates are always included.
//!
//! Refusals are pre-checked from `signature_census` so each gets its own
//! sentence; the engine's own checks still run and remain authoritative.

use std::path::PathBuf;

use pdfcer_core::edit::EditError;
use pdfcer_core::sign::ltv::{MaterialKind, ValidationMaterial};
use pdfcer_gui_base::sign::evidence::{self, Unreadable};

use crate::app::state::OpenDoc;
use crate::text::evidence as t;

/// Pick files, read them, and embed what they hold.
pub(super) fn add(doc: &mut OpenDoc) {
    let census = doc.session.signature_census();
    if census.signatures == 0 {
        return refuse(doc, "unsigned", t::unsigned().to_owned());
    }
    if census.certifications > 0 && census.certification_permission == Some(1) {
        return refuse(
            doc,
            "no-changes-certified",
            t::no_changes_certified().to_owned(),
        );
    }
    let paths = crate::app::files::pick_validation_evidence();
    if paths.is_empty() {
        crate::diag::trace(|| "evidence-cancelled".to_owned()); // ui-text-exempt: diagnostic trace, never displayed
        return;
    }
    let (material, sources) = match gather(&paths) {
        Ok(read) => read,
        Err(note) => return refuse(doc, "unreadable", note),
    };
    let epoch = doc.edit_epoch;
    super::funnel::vector_edit(doc, "evidence-added", 0, paths.len(), |session| {
        match session.add_validation_material(&material) {
            Ok(report) if report.is_empty() => {
                crate::app::actions::record_note(epoch, t::nothing_new().to_owned());
                Err("nothing-new".to_owned())
            }
            Ok(report) => {
                let mut notes = vec![t::added(
                    report.certs_added,
                    report.crls_added,
                    report.ocsps_added,
                    report.signature_certificates_added,
                )];
                if report.duplicates_skipped > 0 {
                    notes.push(t::duplicates(report.duplicates_skipped));
                }
                if report.ocsps_wrapped > 0 {
                    notes.push(t::wrapped(report.ocsps_wrapped));
                }
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "evidence-applied certs={} crls={} ocsps={} own={} dup={} wrapped={} \
                         carried={}",
                        report.certs_added,
                        report.crls_added,
                        report.ocsps_added,
                        report.signature_certificates_added,
                        report.duplicates_skipped,
                        report.ocsps_wrapped,
                        report.carried_forward
                    )
                });
                Ok(notes)
            }
            Err(EditError::ValidationMaterialUnreadable {
                kind,
                index,
                reason,
            }) => {
                let file = sources.name(kind, index);
                crate::app::actions::record_note(epoch, t::blob_refused(&file, &reason));
                Err(format!("unreadable kind={} index={index}", kind.as_str())) // ui-text-exempt: funnel refusal detail, traced only
            }
            Err(other) => {
                crate::app::actions::record_note(epoch, t::refused(&other.to_string()));
                Err(other.to_string())
            }
        }
    });
}

/// Which file each supplied blob came from, per kind, in supply order — the
/// engine names a refused blob by its kind and its index within that kind.
#[derive(Default)]
struct Sources {
    certs: Vec<String>,
    crls: Vec<String>,
    ocsps: Vec<String>,
}

impl Sources {
    fn list(&mut self, kind: MaterialKind) -> &mut Vec<String> {
        match kind {
            MaterialKind::Crl => &mut self.crls,
            MaterialKind::Ocsp => &mut self.ocsps,
            _ => &mut self.certs,
        }
    }

    fn name(&self, kind: MaterialKind, index: usize) -> String {
        let list = match kind {
            MaterialKind::Crl => &self.crls,
            MaterialKind::Ocsp => &self.ocsps,
            _ => &self.certs,
        };
        list.get(index).cloned().unwrap_or_default()
    }
}

fn gather(paths: &[PathBuf]) -> Result<(ValidationMaterial, Sources), String> {
    let mut material = ValidationMaterial::new();
    let mut sources = Sources::default();
    for path in paths {
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let bytes = std::fs::read(path).map_err(|e| t::file_unreadable(&name, &e.to_string()))?;
        let blobs = evidence::read(&name, &bytes).map_err(|why| {
            let why = match why {
                Unreadable::BadBase64 => t::why_bad_base64().to_owned(),
                Unreadable::UnknownLabel(label) => t::why_unknown_label(&label),
                Unreadable::Unterminated => t::why_unterminated().to_owned(),
                Unreadable::Empty => t::why_empty().to_owned(),
            };
            t::file_unreadable(&name, &why)
        })?;
        for (kind, der) in blobs {
            sources.list(kind).push(name.clone());
            material = match kind {
                MaterialKind::Crl => material.with_crl(der),
                MaterialKind::Ocsp => material.with_ocsp(der),
                _ => material.with_cert(der),
            };
        }
    }
    Ok((material, sources))
}

fn refuse(doc: &OpenDoc, reason: &str, note: String) {
    crate::diag::trace(|| format!("evidence-refused reason={reason}")); // ui-text-exempt: diagnostic trace, never displayed
    crate::app::actions::record_note(doc.edit_epoch, note);
}
