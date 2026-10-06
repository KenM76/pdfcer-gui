//! # `app::actions::attachannot` — attach a file to a page as a marker
//!
//! `/FileAttachment` (§12.5.6.15) through
//! `EditSession::add_file_attachment_annotation`, one undo entry. The file is
//! picked by [`pick`] in `BeginTextAnnot`'s apply arm — before the dialog
//! opens, as the reference readers do, and outside any layout pass — and
//! read by [`place`] when the dialog's Add drains.

use std::path::{Path, PathBuf};

use pdfcer_core::annot_author::{AttachmentIcon, Color, FileAttachmentSpec};
use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Rect;

use crate::app::prefs::Prefs;
use crate::app::state::OpenDoc;
use crate::text::attachannot as t;

/// Ask which file to attach, through the same picker (and harness seam,
/// `PDFCER_DIAG_ATTACH_PATH`) as a document-level attachment. `None` when
/// the operator cancelled, which places nothing.
pub(super) fn pick() -> Option<PathBuf> {
    match crate::app::files::pick_attachment_source() {
        crate::app::files::Picked::Path(path) => Some(path),
        _ => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                "attach-annot-cancelled".to_owned()
            });
            None
        }
    }
}

/// What the dialog accepted.
pub(super) struct Placed<'a> {
    /// The 0-based page.
    pub page: usize,
    /// The marker's rectangle, in PDF user space.
    pub rect: Rect,
    /// The file to embed.
    pub file: &'a Path,
    /// The marker's icon.
    pub icon: AttachmentIcon,
    /// The trimmed description, or `None`.
    pub description: Option<&'a str>,
}

/// Read the file and author the marker.
///
/// With no description no `MarkupOptions::note` is passed: the engine writes
/// the note's text to both `/Contents` and the filespec's `/Desc`, and an
/// empty key is one a later reader must interpret, where an absent one says
/// there is no description.
pub(super) fn place(
    doc: &mut OpenDoc,
    prefs: &Prefs,
    placed: &Placed<'_>,
    ink: (f64, f64, f64),
    opacity: Option<f64>,
) {
    let bytes = match std::fs::read(placed.file) {
        Ok(bytes) => bytes,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("attach-annot-unreadable detail={error}")
            });
            super::record_note(doc.edit_epoch, t::unreadable(&error.to_string()));
            return;
        }
    };
    let name = placed.file.file_name().map_or_else(
        || pdfcer_core::attachments::FALLBACK_SAFE_NAME.to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    let (r, g, b) = ink;
    let mut spec: FileAttachmentSpec = FileAttachmentSpec::new(placed.rect, &name, bytes);
    spec.icon = placed.icon.clone();
    spec.color = Color::Rgb(r, g, b);
    let options = MarkupOptions {
        note: placed
            .description
            .map(|d| super::annots::signed_note(d, Some(&prefs.author_name))),
        opacity,
        ..Default::default()
    };
    let Ok((options, receipt)) = super::drawlayer::onto(doc, "add-attachment-annot", options)
    else {
        return;
    };
    let page = placed.page;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "attach-annot-read page={page} name={:?} bytes={} icon={} described={} w={:.1} h={:.1}",
            spec.file_name,
            spec.bytes.len(),
            String::from_utf8_lossy(spec.icon.name()),
            options.note.is_some(),
            spec.rect.urx - spec.rect.llx,
            spec.rect.ury - spec.rect.lly,
        )
    });
    super::apply::vector_edit(doc, "add-attachment-annot", page, 1, |session| {
        let id: ObjId = session.add_file_attachment_annotation(page, &spec, &options)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("attach-annot-placed page={page} id={}", id.num)
        });
        Ok::<Vec<String>, pdfcer_core::edit::EditError>(
            std::iter::once(t::placed(&name, page, size))
                .chain(receipt)
                .collect(),
        )
    });
}
