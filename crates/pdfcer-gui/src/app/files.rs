//! # `app::files` — how a path gets from an operator (or a harness) to
//! [`crate::app::actions::Action::Open`]
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/files.md`.

use std::ffi::OsString;
use std::path::PathBuf;

/// The environment variable that answers the dialog instead of opening it.
const DIAG_INSERT_PATH: &str = "PDFCER_DIAG_INSERT_PATH"; // ui-text-exempt: an environment variable name, never displayed
/// The seam that answers the **image** picker.
const DIAG_IMAGE_PATH: &str = "PDFCER_DIAG_IMAGE_PATH"; // ui-text-exempt: an environment variable name, never displayed

pub const DIAG_OPEN_PATH: &str = "PDFCER_DIAG_OPEN_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The seam that answers the **attach a file** picker.
/// Answers [`pick_font_file`] without a dialog, for `ui-verify`.
pub const DIAG_FONT_FILE_PATH: &str = "PDFCER_DIAG_FONT_FILE_PATH"; // ui-text-exempt: an environment variable name, never displayed
pub const DIAG_ATTACH_PATH: &str = "PDFCER_DIAG_ATTACH_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The seam that answers the **3D model to insert** picker.
pub const DIAG_MODEL_PATH: &str = "PDFCER_DIAG_MODEL_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The seam that answers the **save a 3D model as a mesh** dialog.
pub const DIAG_MESH_SAVE_PATH: &str = "PDFCER_DIAG_MESH_SAVE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The seam that answers the **save an attachment out** dialog.
pub const DIAG_ATTACHMENT_SAVE_PATH: &str = "PDFCER_DIAG_ATTACHMENT_SAVE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_form_data_source`].
pub const DIAG_FORM_DATA_PATH: &str = "PDFCER_DIAG_FORM_DATA_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The text file `file.import_text` reads, for a driven check.
pub const DIAG_TEXT_IMPORT_PATH: &str = "PDFCER_DIAG_TEXT_IMPORT_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_font_folder`].
pub const DIAG_FONT_FOLDER_PATH: &str = "PDFCER_DIAG_FONT_FOLDER"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_acrobat`] — `OPERATOR_REQUESTS.md` O122.
pub const DIAG_ACROBAT_PATH: &str = "PDFCER_DIAG_ACROBAT_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_trust_store`] — the signature-trust work,
/// 2026-09-05.
pub const DIAG_TRUST_STORE_PATH: &str = "PDFCER_DIAG_TRUST_STORE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The harness seam for [`pick_certificate`] — the `.pfx`/`.p12` a driven
/// check signs with.
#[cfg(feature = "signing")]
pub const DIAG_CERTIFICATE_PATH: &str = "PDFCER_DIAG_CERTIFICATE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// The environment variable that answers the **save** dialog instead of
/// opening it.
pub const DIAG_SAVE_PATH: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name, never displayed

/// **The seam for the MULTI-file picker** — `OPERATOR_REQUESTS.md` O68.
pub const DIAG_MERGE_SOURCES: &str = "PDFCER_DIAG_MERGE_SOURCES"; // ui-text-exempt: an environment variable name, never displayed

/// What asking for a document produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Picked {
    /// The operator (or the diagnostic seam) named this file.
    Path(PathBuf),
    /// The operator dismissed the dialog. Nothing happens, and nothing is
    /// traced beyond the fact — a cancelled Open is a complete, correct,
    /// uninteresting outcome.
    Cancelled,
    /// This build has no way to ask.
    ///
    /// **[`native_pick`] can no longer produce this** — `rfd::FileDialog`
    /// answers `Some` or `None` and has no third case. It survives because
    /// [`from_env`] can still answer it, and because the distinction is worth
    /// more than the variant costs: the moment a build appears that cannot
    /// open a picker (a headless target, a feature-stripped build under the
    /// capability-modularity rule), the alternative is silence that looks
    /// exactly like a cancelled dialog.
    ///
    /// Deleting it would be the sort of tidying that removes the only
    /// difference between "the button does nothing" and "the operator changed
    /// their mind" — which is the distinction this whole enum exists for.
    Unavailable,
}

/// **Ask for a document to open.**
#[must_use]
pub fn pick_document() -> Picked {
    if let Some(raw) = std::env::var_os(DIAG_OPEN_PATH) {
        let answer = match queued(&raw) {
            Some(answer) => answer,
            // Not a queue, so the single-valued seam answers exactly as it
            // always has. `from_env` cannot return `None` for a value that is
            // `Some`; the fallback is spelled rather than unwrapped because a
            // seam is not worth a panic.
            None => from_env(Some(raw)).unwrap_or(Picked::Cancelled),
        };
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "open-picked source=env answer={answer:?}"
            )
        });
        return answer;
    }
    let answer = native_pick();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "open-picked source=native answer={answer:?}"
        )
    });
    answer
}

thread_local! {
    /// How many answers this process has already taken from the open queue.
    static OPEN_QUEUE_POS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Take this call's answer from a `;`-separated queue, one entry per call.
fn queued(raw: &OsString) -> Option<Picked> {
    let text = raw.to_string_lossy();
    if !text.contains(';') {
        return None;
    }
    let entries: Vec<&str> = text.split(';').collect();
    let taken = OPEN_QUEUE_POS.with(|pos| {
        let index = pos.get();
        pos.set(index + 1);
        entries.get(index).copied()
    });
    Some(match taken {
        Some(entry) if !entry.is_empty() => Picked::Path(PathBuf::from(entry)),
        _ => Picked::Cancelled,
    })
}

/// Read the diagnostic seam, if it is set. Pure, so it can be tested.
#[must_use]
pub fn from_env(value: Option<OsString>) -> Option<Picked> {
    let value = value?;
    if value.is_empty() {
        // A deliberate, reachable answer rather than an oversight: it is how
        // a harness drives the branch in which the operator says no, without
        // which "Open changed nothing" cannot be distinguished from "Open was
        // never reached".
        return Some(Picked::Cancelled);
    }
    Some(Picked::Path(PathBuf::from(value)))
}

/// **Turn what the picker said into what the application does about it.**
pub fn raise(picked: Picked, actions: &mut Vec<crate::app::actions::Action>) {
    match picked {
        Picked::Path(path) => actions.push(crate::app::actions::Action::Open(path)),
        Picked::Cancelled => {}
        Picked::Unavailable => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "open-unavailable reason=no-picker-in-this-build".to_owned()
        }),
    }
}

/// Ask the platform for its own file picker.
fn native_pick() -> Picked {
    rfd::FileDialog::new()
        .set_title(crate::text::files::open_dialog_title())
        .add_filter(crate::text::files::filter_pdf(), &["pdf"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path)
}

/// **Ask which PDF to take pages from** — `pages.insert_from_file`.
#[must_use]
pub fn pick_image_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_IMAGE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("image-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::images::window_title())
        .add_filter(
            crate::text::files::filter_image(),
            &["png", "jpg", "jpeg", "bmp", "gif", "tif", "tiff"],
        )
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("image-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which form-data file to read.**
#[must_use]
pub fn pick_form_data_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_FORM_DATA_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("form-data-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::export_form::import_dialog_title())
        .add_filter(
            crate::text::files::filter_form_data(),
            &["fdf", "xfdf", "csv"],
        )
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("form-data-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which text file to import as pages** — `file.import_text`,
/// `pdfcer-core` `Pass 252.0`.
pub fn pick_text_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_TEXT_IMPORT_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-import-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::import_text::window_title())
        .add_filter(crate::text::files::filter_text(), &["txt"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("text-import-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which file to embed in the document** (ISO 32000-1 §7.11.4.1).
#[must_use]
pub fn pick_attachment_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_ATTACH_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("attach-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::panels::attachments::attach_dialog_title())
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("attach-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which 3D model file to place on the page.**
#[must_use]
pub fn pick_model_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_MODEL_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("model-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::panels::models::insert_dialog_title())
        .add_filter(
            crate::text::panels::models::insert_filter(),
            &["u3d", "prc"],
        )
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("model-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask where to write one attachment out.**
#[must_use]
pub fn pick_attachment_target(suggested: &std::path::Path) -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_ATTACHMENT_SAVE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("attachment-save-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let mut dialog = rfd::FileDialog::new()
        .set_title(crate::text::panels::attachments::save_dialog_title())
        .add_filter(crate::text::files::filter_all(), &["*"]);
    if let Some(dir) = suggested.parent().filter(|d| !d.as_os_str().is_empty()) {
        dialog = dialog.set_directory(dir);
    }
    if let Some(name) = suggested.file_name() {
        dialog = dialog.set_file_name(name.to_string_lossy());
    }
    let answer = dialog.save_file().map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("attachment-save-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask where to write a 3D model's mesh.** An `.obj` ending chooses OBJ;
/// anything else is STL.
#[must_use]
pub fn pick_mesh_target(suggested: &std::path::Path) -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_MESH_SAVE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("mesh-save-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let mut dialog = rfd::FileDialog::new()
        .set_title(crate::text::panels::models::mesh_dialog_title())
        .add_filter(crate::text::panels::models::mesh_filter_stl(), &["stl"])
        .add_filter(crate::text::panels::models::mesh_filter_obj(), &["obj"]);
    if let Some(dir) = suggested.parent().filter(|d| !d.as_os_str().is_empty()) {
        dialog = dialog.set_directory(dir);
    }
    if let Some(name) = suggested.file_name() {
        dialog = dialog.set_file_name(name.to_string_lossy());
    }
    let answer = dialog.save_file().map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("mesh-save-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which font file to restyle text into.**
#[must_use]
pub fn pick_font_file() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_FONT_FILE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("font-file-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::panels::face::font_file_dialog_title())
        .add_filter(
            crate::text::panels::face::font_file_filter(),
            &["ttf", "otf"],
        )
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("font-file-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which folder pdfcer may take fonts from.**
#[must_use]
pub fn pick_font_folder() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_FONT_FOLDER_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("font-folder-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::settings::font_folder_dialog_title())
        .pick_folder()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("font-folder-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask which program is Acrobat** — `OPERATOR_REQUESTS.md` O122's Browse
/// button.
#[must_use]
pub fn pick_acrobat() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_ACROBAT_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("acrobat-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::acrobat::path_dialog_title())
        .add_filter(crate::text::acrobat::path_filter_name(), &["exe"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("acrobat-picked source=native answer={answer:?}")
    });
    answer
}

/// **Ask where Acrobat's downloaded trust list is** — the Settings ▸ Digital
/// signatures Browse button.
#[must_use]
pub fn pick_trust_store() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_TRUST_STORE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("trust-store-picked source=env answer={answer:?}")
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::trust::store_path_browse())
        .add_filter(crate::text::trust::store_path_filter(), &["acrodata"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("trust-store-picked source=native answer={answer:?}")
    });
    answer
}

/// **The certificate file to sign with** — a PKCS#12 `.pfx` or `.p12`.
#[cfg(feature = "signing")]
#[must_use]
pub fn pick_certificate() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_CERTIFICATE_PATH)) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // `answer` is NOT interpolated, unlike every sibling in this
            // file, and the asymmetry is deliberate: this path names the file
            // holding the operator's private key, and a trace is kept as
            // evidence. Whether a path was supplied is the whole diagnostic
            // question; which path it was is not ours to publish.
            format!(
                "certificate-picked source=env chosen={}",
                u8::from(matches!(answer, Picked::Path(_)))
            )
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::sign::certificate_picker_title())
        .add_filter(crate::text::sign::certificate_filter(), &["pfx", "p12"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed. See above for why
        // the path itself is absent.
        format!(
            "certificate-picked source=native chosen={}",
            u8::from(matches!(answer, Picked::Path(_)))
        )
    });
    answer
}

#[must_use]
pub fn pick_insert_source() -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_INSERT_PATH)) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "insert-picked source=env answer={answer:?}"
            )
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::pages::insert_dialog_title())
        .add_filter(crate::text::files::filter_pdf(), &["pdf"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_file()
        .map_or(Picked::Cancelled, Picked::Path);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "insert-picked source=native answer={answer:?}"
        )
    });
    answer
}

/// The seam for [`pick_validation_evidence`]: `;`-separated paths.
#[cfg(feature = "signing")]
pub const DIAG_EVIDENCE_FILES: &str = "PDFCER_DIAG_EVIDENCE_FILES"; // ui-text-exempt: an environment variable name, never displayed

/// **Ask for the certificate, CRL and OCSP files Add validation evidence…
/// embeds.** Empty means cancelled.
#[cfg(feature = "signing")]
#[must_use]
pub fn pick_validation_evidence() -> Vec<PathBuf> {
    let (source, answer) = if let Some(raw) = std::env::var_os(DIAG_EVIDENCE_FILES) {
        let text = raw.to_string_lossy().into_owned();
        let paths = text
            .split(';')
            .filter(|part| !part.is_empty())
            .map(PathBuf::from)
            .collect();
        ("env", paths)
    } else {
        let t = crate::text::evidence::picker_title();
        let paths = rfd::FileDialog::new()
            .set_title(t)
            .add_filter(
                crate::text::evidence::picker_filter(),
                &["cer", "crt", "der", "pem", "crl", "ocsp", "ors", "resp"],
            )
            .add_filter(crate::text::files::filter_all(), &["*"])
            .pick_files()
            .unwrap_or_default();
        ("native", paths)
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("evidence-picked source={source} n={}", answer.len())
    });
    answer
}

/// **Ask for several PDFs to combine into a new one** —
/// `OPERATOR_REQUESTS.md` O68.
#[must_use]
pub fn pick_merge_sources() -> Vec<PathBuf> {
    if let Some(raw) = std::env::var_os(DIAG_MERGE_SOURCES) {
        let text = raw.to_string_lossy().into_owned();
        let answer: Vec<PathBuf> = text
            .split(';')
            .filter(|part| !part.is_empty())
            .map(PathBuf::from)
            .collect();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "merge-picked source=env n={}",
                answer.len()
            )
        });
        return answer;
    }
    let answer = rfd::FileDialog::new()
        .set_title(crate::text::files::merge_dialog_title())
        .add_filter(crate::text::files::filter_pdf(), &["pdf"])
        .add_filter(crate::text::files::filter_all(), &["*"])
        .pick_files()
        .unwrap_or_default();
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "merge-picked source=native n={}",
            answer.len()
        )
    });
    answer
}

/// **Ask where to write a new document.**
#[must_use]
pub fn pick_save_path(suggested: &std::path::Path, title: &str) -> Picked {
    if let Some(answer) = from_env(std::env::var_os(DIAG_SAVE_PATH)) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "save-picked source=env answer={answer:?}"
            )
        });
        return answer;
    }
    let answer = native_save(suggested, title);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "save-picked source=native answer={answer:?}"
        )
    });
    answer
}

/// The platform save dialog, pre-filled from `suggested` and headed `title`.
fn native_save(suggested: &std::path::Path, title: &str) -> Picked {
    let mut dialog = rfd::FileDialog::new()
        .set_title(title)
        .add_filter(crate::text::files::filter_pdf(), &["pdf"]);
    if let Some(dir) = suggested.parent().filter(|d| !d.as_os_str().is_empty()) {
        dialog = dialog.set_directory(dir);
    }
    if let Some(name) = suggested.file_name() {
        dialog = dialog.set_file_name(name.to_string_lossy());
    }
    dialog.save_file().map_or(Picked::Cancelled, Picked::Path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::PdfcerApp;
    use crate::app::actions::Action;
    use crate::app::state::Status;
    use crate::panels::objects::test_support::engine_fixture;

    /// A four-page fixture that really opens.
    fn fixture() -> PathBuf {
        engine_fixture("pageops/four-pages.pdf")
    }

    /// A value with no separator is not a queue, so the old seam is untouched.
    #[test]
    fn a_single_path_is_not_a_queue_and_never_runs_out() {
        let raw = OsString::from("C:/drawings/one.pdf");
        assert_eq!(queued(&raw), None);
        assert_eq!(queued(&raw), None);
    }

    /// Entries come back in order, one per call.
    #[test]
    fn a_joined_list_answers_one_document_per_call_in_order() {
        let raw = OsString::from("a.pdf;b.pdf;c.pdf");
        assert_eq!(queued(&raw), Some(Picked::Path(PathBuf::from("a.pdf"))));
        assert_eq!(queued(&raw), Some(Picked::Path(PathBuf::from("b.pdf"))));
        assert_eq!(queued(&raw), Some(Picked::Path(PathBuf::from("c.pdf"))));
    }

    /// Past the end the answer is a declined dialog, not a native picker.
    #[test]
    fn an_exhausted_queue_declines_rather_than_opening_a_dialog() {
        let raw = OsString::from("only.pdf;");
        assert_eq!(queued(&raw), Some(Picked::Path(PathBuf::from("only.pdf"))));
        assert_eq!(queued(&raw), Some(Picked::Cancelled));
        assert_eq!(queued(&raw), Some(Picked::Cancelled));
    }

    /// The handler token the ribbon would raise for `id`.
    fn token_for(app: &PdfcerApp, id: &str) -> egui_shell::commands::HandlerToken {
        app.commands
            .get(id)
            .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic, never displayed
            .handler
    }

    /// **The picker's answer becomes an action, and only a path does.**
    #[test]
    fn only_a_picked_path_becomes_an_action() {
        let mut actions = Vec::new();
        raise(Picked::Path(PathBuf::from("D:\\sheet.pdf")), &mut actions);
        assert_eq!(actions, vec![Action::Open(PathBuf::from("D:\\sheet.pdf"))]);

        let mut actions = Vec::new();
        raise(Picked::Cancelled, &mut actions);
        assert!(actions.is_empty(), "a dismissed dialog opens nothing");

        let mut actions = Vec::new();
        raise(Picked::Unavailable, &mut actions);
        assert!(actions.is_empty(), "a build with no picker opens nothing");
    }

    /// **`file.close` raises the Close action, and applying it empties the
    /// shell.**
    #[test]
    fn the_close_command_empties_the_shell() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = PdfcerApp::new();
        app.open_path(fixture());
        assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
        app.panels.set_focus(3);

        let mut actions = Vec::new();
        app.dispatch_token(&ctx, token_for(&app, "file.close"), &mut actions);
        assert_eq!(actions, vec![Action::Close]);

        app.apply_actions(actions, 1.0);
        assert!(matches!(app.status, Status::Empty));
        assert_eq!(
            app.panels.focus(),
            None,
            "closing must forget the paint-order indices the panels held, exactly as \
             opening does — they name positions in a document that is no longer open"
        );
    }

    /// **The Open action opens, from every starting state.**
    #[test]
    fn the_open_action_opens_whether_or_not_something_is_already_open() {
        let mut app = PdfcerApp::new();
        assert!(matches!(app.status, Status::Empty));

        app.apply_actions(vec![Action::Open(fixture())], 1.0);
        assert!(
            matches!(app.status, Status::Open(_)),
            "an Open with nothing open is the ordinary case, not a refused one"
        );

        // …and again over a document that is already open, which is the
        // second-file case the whole command exists for.
        app.apply_actions(vec![Action::Open(fixture())], 1.0);
        assert!(matches!(app.status, Status::Open(_)));

        // A Close with nothing open is a no-op rather than a panic: a
        // customized keymap can reach any command from any state.
        app.apply_actions(vec![Action::Close, Action::Close], 1.0);
        assert!(matches!(app.status, Status::Empty));
    }

    /// **`file.save_copy` raises the SaveCopy action, through the real token
    /// lookup.**
    #[test]
    fn the_save_copy_command_raises_the_save_action() {
        let ctx = egui::Context::default();
        let mut app = PdfcerApp::new();
        app.open_path(fixture());
        assert!(matches!(app.status, Status::Open(_)), "the fixture opens");

        let mut actions = Vec::new();
        app.dispatch_token(&ctx, token_for(&app, "file.save_copy"), &mut actions);
        assert_eq!(
            actions,
            vec![Action::SaveCopy],
            "`file.save_copy` must raise an action rather than falling through to \
             `command-unimplemented`, and it must raise it rather than opening the picker here — \
             see `crate::app::save` section 4 on the frame-timing requirement"
        );
    }

    /// **Nothing is pending, so nothing is blocked — and the gate is real.**
    #[test]
    fn the_dirty_document_gate_blocks_nothing_in_a_build_with_no_save() {
        let mut app = PdfcerApp::new();
        app.open_path(fixture());
        assert!(!app.save_pending(), "there is no save path in this build");

        app.apply_actions(vec![Action::Close], 1.0);
        assert!(matches!(app.status, Status::Empty));
    }

    /// **The diagnostic seam answers the dialog, in all three shapes.**
    #[test]
    fn the_diagnostic_seam_answers_the_dialog() {
        assert_eq!(from_env(None), None, "unset must not answer at all");
        assert_eq!(
            from_env(Some(OsString::from("D:\\drawings\\sheet.pdf"))),
            Some(Picked::Path(PathBuf::from("D:\\drawings\\sheet.pdf")))
        );
        assert_eq!(
            from_env(Some(OsString::new())),
            Some(Picked::Cancelled),
            "an empty value is how a harness drives the cancel path without a dialog"
        );
    }

    /// A path with a space, and one that is not ASCII, both survive the seam.
    #[test]
    fn the_seam_does_not_mangle_a_real_path() {
        for raw in [
            "C:\\Program Files\\a drawing.pdf",
            "D:\\Zeichnungen\\Übersicht.pdf",
        ] {
            assert_eq!(
                from_env(Some(OsString::from(raw))),
                Some(Picked::Path(PathBuf::from(raw)))
            );
        }
    }
}
