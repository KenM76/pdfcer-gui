//! # `app::actions::export` — writing part of the document out as something
//! else
//!
//! ## Why this is its own file
//!
//! The sixth sibling of [`super::apply`], drawn along the same seam the other
//! five are: *what class of thing does this verb act on?* — pages there,
//! annotations in `annots`, the dimensioning model in `dimensions`, page
//! content in `apply`, redaction marks in `redact`. This is **what leaves the
//! document**.
//!
//! It is a real subject rather than a size-driven cut, and the evidence is the
//! property every verb here shares and no verb elsewhere does: **none of them
//! changes the document at all.** No `vector_edit`, no undo entry, no epoch
//! bump, no cache invalidation. They read the open file and write a different
//! one, which makes every rule the mutation funnel enforces irrelevant to them
//! and every rule about *file* handling — a save picker, an overwrite, a
//! partial write — apply instead.
//!
//! `super::pages::extract` is the same shape and stayed in `pages` because its
//! subject is a page set. If a third export lands, that is the moment to move
//! it here.
//!
//! ## ★ Why an export is an `Action` at all
//!
//! `super::apply`'s header answers it for `SaveCopy` and the answer is the same
//! here: **a native file dialog must not open inside a layout pass.** It is a
//! modal OS window that blocks the thread, and opening one from a widget's
//! `clicked()` branch means egui is part-way through building a frame that will
//! not finish until the operator has answered.
//!
//! Nothing about the document is being ordered — there is nothing to order —
//! so the funnel's *invariant* does not apply. Its **reason** does.

use crate::app::state::OpenDoc;

/// Write one page's vector geometry as an ASCII DXF.
///
/// ## What this owes the operator, and why it is not optional
///
/// `DxfOutcome` is the disclosure half, and two of its counts are the reason
/// this feature is worth having over any generic converter:
///
/// - **`skipped_images`** — DXF has no raster entity, so a picture on the page
///   is simply not in the file. The engine's own words for why that must be
///   said: *"an operator whose drawing was half annotation gets a DXF that
///   looks like the geometry went missing, and 'the labels are not in this
///   file' is a sentence they need **before** they open it in SOLIDWORKS, not
///   after."*
/// - **`unreadable_text`** — text pdfcer could not decode, kept apart from
///   `skipped_text` (which the operator asked for) because one is a choice and
///   the other is a fact about the source PDF. Rolling them together would let
///   the second hide inside the first.
///
/// ## ★ Why the geometry is fetched here and not carried in the action
///
/// `PageObjects` is a decomposition of a whole page — every path, every text
/// run, every image placement — and the shell already holds one, cached, keyed
/// on `(page, epoch)`. Carrying it through the action queue would clone it for
/// a value the apply phase can borrow, and a **stale** clone at that: the queue
/// drains after the frame, and an edit raised earlier in the same frame would
/// leave the export describing the page as it was.
///
/// Fetching it here means the export sees the document as it stands when the
/// export runs, which is the only reading that can be defended.
pub(super) fn dxf(doc: &mut OpenDoc, page: usize, options: &pdfcer_core::export::dxf::DxfOptions) {
    // The decomposition, from the cache the canvas and the Objects panel share.
    // `None` is reachable — a page still being read, or one whose content
    // streams could not be resolved — and is a decline rather than a failure:
    // nothing was written and the sentence says which.
    let Some(dxf) = doc
        .page_objects()
        .map(|provider| pdfcer_core::export::dxf::write_dxf(provider.page_objects(), options))
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("export-dxf-declined page={page} reason=no-decomposition")
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::export_dxf::no_geometry().to_owned(),
        );
        return;
    };
    let (text, outcome) = dxf;

    // ★ The picker AFTER the write, not before.
    //
    // The write is pure and cannot fail — `write_dxf` returns no `Result`, and
    // its doc says why: *"the writer cannot fail on well-formed input, and
    // malformed input is skipped and counted rather than refused."* So doing it
    // first costs nothing and buys the property that matters: the operator is
    // never asked where to put a file that turns out to be empty. If a future
    // slice gives the writer a refusal, this ordering is what lets the refusal
    // be reported before a save dialog has been opened.
    let suggested = suggested_path(doc);
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested, crate::text::export_dxf::save_dialog_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("export-dxf-cancelled page={page}")
        });
        return;
    };

    match std::fs::write(&target, text.as_bytes()) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-dxf page={page} bytes={} polylines={} circles={} arcs={} \
                     splines={} skipped_text={} skipped_images={} unreadable_text={}",
                    text.len(),
                    outcome.polylines,
                    outcome.circles,
                    outcome.arcs,
                    outcome.splines,
                    outcome.skipped_text,
                    outcome.skipped_images,
                    outcome.unreadable_text
                )
            });
            // ★ Recorded through `record_note` rather than returned from a
            // `vector_edit` closure, because there is no edit to ride in on —
            // the same case `canvas::interact` records for a caret that cannot
            // be placed. Stamped with the CURRENT epoch, so the sentences stand
            // until the next real edit moves past them.
            //
            // The list is joined rather than recorded one at a time: the slot
            // holds one disclosure, and the last writer would win.
            let notes = crate::text::export_dxf::exported(&target.display().to_string(), &outcome);
            super::record_edit_disclosure(Some(super::EditDisclosure {
                epoch: doc.edit_epoch,
                notes,
            }));
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-dxf-failed page={page} detail={error}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_dxf::export_failed(&error.to_string()),
            );
        }
    }
}

/// **Write the form's values out as FDF, XFDF or CSV.**
///
/// `file.export_form_data`, wired 2026-08-27 — a command that had been
/// registered, drawn on File ▸ Export, and **inert for the whole life of the
/// project**.
///
/// # ★★★ The recorded reason was false, and it is the sixth of these
///
/// Its `SCAFFOLDED` entry read:
///
/// > ~~Blocked on a writer that does not exist. `FEATURES.md`'s Forms row:
/// > "fill ✅ …; create field, flatten and FDF/XFDF/CSV still ⬜" — and this
/// > command IS the FDF/XFDF/CSV half.~~
///
/// Three writers exist and two of them have since `Pass 7.1`:
/// `fdf::FormData::to_fdf`, `to_xfdf`, and `formcsv::to_csv`, reached through
/// `EditSession::export_form_data`. The `FEATURES.md` row it cites was itself
/// stale, so — like `edit.form_flatten` two hours earlier — the entry was a
/// **citation of a citation** and nothing had re-read either.
///
/// ⇒ The rule now written on the allow-list's own assertion: *when you touch
/// that list for any purpose, re-derive the reason of the entry beside the one
/// you came for.* This one was found by doing exactly that.
///
/// # ★★ The format is chosen by the EXTENSION, not by a third dialog
///
/// One picker, three formats, decided by what the operator types or picks in
/// the *Save as type* box — which is how every application on this desktop
/// does it, and which `crate::text::tool`'s rule about conventions makes the
/// default answer rather than a shortcut.
///
/// The alternative — a format dialog, then a picker — is two modal windows for
/// one act, and it puts the choice **before** the operator has thought about
/// where the file goes, which is the order they think in reversed.
///
/// An unrecognised extension is **FDF**, and that is a decision rather than a
/// fallback: FDF is the format the standard defines for this data (§12.7.8),
/// it is what Acrobat writes, and it is the only one of the three that a
/// reader can import without being told what it is.
///
/// # ★★★ The CSV disclosure is not optional, and it is about a spreadsheet
/// rather than about a PDF
///
/// `formcsv::to_csv` **neutralises** values that would otherwise be executed as
/// formulas when the file is opened in a spreadsheet — a leading `=`, `+`, `-`
/// or `@`. That is a real and well-known injection route, and pdfcer doing the
/// right thing silently would leave an operator believing their exported data
/// is byte-identical to what the form holds.
///
/// It is not. `neutralised` counts how many, `neutralised_fields` names them,
/// and both are reported. Rule 4's *"the half that survives is the point"*: an
/// inference the operator cannot see still owes them an off-canvas sentence.
///
/// # Nothing about the document changes
///
/// No `vector_edit`, no epoch bump, no cache invalidation — this module's
/// header explains why that is what makes these verbs a family. The
/// disclosures ride `record_edit_disclosure` at the current epoch, so they
/// stand until the next real edit moves past them.
pub(super) fn form_data(doc: &mut OpenDoc) {
    // ★ The data first, the picker second — `dxf`'s ordering and its reason:
    // the operator is never asked where to put a file that turns out to be
    // empty. A document with no `/AcroForm` has nothing to export, and that is
    // a decline with a sentence rather than a dialog followed by a zero-byte
    // file.
    let Some(data) = doc.session.export_form_data() else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-form-data-declined reason=no-acroform".to_owned()
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::export_form::no_form().to_owned(),
        );
        return;
    };
    if data.fields.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-form-data-declined reason=no-fields".to_owned()
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::export_form::no_fields().to_owned(),
        );
        return;
    }

    let suggested = suggested_form_path(doc);
    let crate::app::files::Picked::Path(target) = crate::app::files::pick_save_path(
        &suggested,
        crate::text::export_form::save_dialog_title(),
    ) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-form-data-cancelled".to_owned()
        });
        return;
    };

    // ★ The document's own path as the FDF `/F` source, so a reader importing
    // the data knows which file it came from. `to_fdf`'s parameter is exactly
    // that, and passing `None` would produce a valid file that has forgotten
    // its subject.
    let source = doc.path.to_string_lossy().into_owned();
    let extension = target
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let (bytes, mut notes) = match extension.as_str() {
        // ui-text-exempt: file extensions, matched not displayed.
        "xfdf" => (
            data.to_xfdf(Some(&source)),
            vec![crate::text::export_form::wrote_xfdf(data.fields.len())],
        ),
        "csv" => {
            let export = pdfcer_core::formcsv::to_csv(&data);
            let mut notes = vec![crate::text::export_form::wrote_csv(data.fields.len())];
            // ★★ The neutralisation disclosure — see the header. Reported only
            // when it fired, because a form with no formula-shaped values owes
            // no sentence, and a bar that narrates non-events stops being read.
            if export.neutralised > 0 {
                notes.push(crate::text::export_form::neutralised(
                    export.neutralised,
                    &export.neutralised_fields,
                ));
            }
            (export.csv, notes)
        }
        _ => (
            data.to_fdf(Some(&source)),
            vec![crate::text::export_form::wrote_fdf(data.fields.len())],
        ),
    };

    match std::fs::write(&target, &bytes) {
        Ok(()) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-form-data format={extension} fields={} bytes={}",
                    data.fields.len(),
                    bytes.len()
                )
            });
            notes.push(crate::text::export_form::written_to(
                &target.display().to_string(),
            ));
            super::record_edit_disclosure(Some(super::EditDisclosure {
                epoch: doc.edit_epoch,
                notes,
            }));
        }
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-form-data-failed detail={error}")
            });
            super::record_note(
                doc.edit_epoch,
                crate::text::export_form::export_failed(&error.to_string()),
            );
        }
    }
}

/// Where the form-data save dialog opens, and what it calls the file.
///
/// Beside the document and named after it, with `.fdf` — [`suggested_path`]'s
/// rule and its reason. The extension is the **default format** as well as a
/// suggestion, which is why it is FDF: see [`form_data`]'s header.
fn suggested_form_path(doc: &OpenDoc) -> std::path::PathBuf {
    let mut path = doc.path.clone();
    let stem = path
        .file_stem()
        .map_or_else(|| "form".to_owned(), |s| s.to_string_lossy().into_owned());
    path.set_file_name(stem);
    path.set_extension("fdf"); // ui-text-exempt: a file extension, never displayed as prose
    path
}

/// Where the save dialog opens, and what it calls the file.
///
/// Beside the document, named after it, with a `.dxf` extension. The same rule
/// `super::pages::suggested_path` follows for an extract, and for its reason: a
/// picker that opens in the last-used directory of some other application is a
/// picker that makes the operator navigate back to their own project every
/// time.
fn suggested_path(doc: &OpenDoc) -> std::path::PathBuf {
    let mut path = doc.path.clone();
    let stem = path
        .file_stem()
        .map_or_else(|| "export".to_owned(), |s| s.to_string_lossy().into_owned());
    path.set_file_name(stem);
    // `set_extension` rather than pushing a string: a document called
    // `plan.rev2.pdf` has a stem of `plan.rev2`, and appending would produce
    // `plan.rev2.dxf` either way — but a document with no extension at all
    // would gain one only through this call.
    path.set_extension("dxf"); // ui-text-exempt: a file extension, never displayed as prose
    path
}
