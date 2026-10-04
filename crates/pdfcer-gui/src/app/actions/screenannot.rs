//! # `app::actions::screenannot` — place a media clip in a page region
//!
//! `/Screen` (§12.5.6.18) through `EditSession::add_screen_annotation`, one
//! undo entry. The clip is picked by [`pick`] in `BeginTextAnnot`'s apply
//! arm, before the dialog opens, and read by [`place`] when the dialog's Add
//! drains. The MIME type, trigger and temporary-file permission are named in
//! the status note, because what plays is the reader's decision.

use std::path::{Path, PathBuf};

use pdfcer_core::annot_author::{Color, MediaTempAccess, ScreenSpec, ScreenTrigger};
use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Rect;

use crate::app::state::OpenDoc;
use crate::text::screenannot as t;

/// Ask which clip to place. `None` when the operator cancelled.
pub(super) fn pick() -> Option<PathBuf> {
    match crate::app::files::pick_media_source() {
        crate::app::files::Picked::Path(path) => Some(path),
        _ => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                "screen-annot-cancelled".to_owned()
            });
            None
        }
    }
}

/// What the dialog accepted.
pub(super) struct Placed<'a> {
    /// The 0-based page.
    pub page: usize,
    /// The play region, in PDF user space.
    pub rect: Rect,
    /// The clip.
    pub file: &'a Path,
    /// Its MIME type.
    pub content_type: &'a str,
    /// What starts playback.
    pub trigger: ScreenTrigger,
    /// The `/TF` permission.
    pub temp_access: MediaTempAccess,
    /// The trimmed description, or `None`.
    pub description: Option<&'a str>,
}

/// Read the clip, then author the region.
pub(super) fn place(
    doc: &mut OpenDoc,
    placed: &Placed<'_>,
    ink: (f64, f64, f64),
    opacity: Option<f64>,
) {
    let bytes = match std::fs::read(placed.file) {
        Ok(bytes) => bytes,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("screen-annot-unreadable detail={error}")
            });
            return super::record_note(doc.edit_epoch, t::unreadable(&error.to_string()));
        }
    };
    let name = placed
        .file
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let page = placed.page;
    let disclosure = t::placed(
        &name,
        page,
        placed.content_type,
        placed.trigger,
        placed.temp_access,
    );
    let (r, g, b) = ink;
    let mut spec = ScreenSpec::new(placed.rect, &name, placed.content_type, bytes);
    spec.color = Color::Rgb(r, g, b);
    spec.trigger = placed.trigger;
    spec.temp_access = placed.temp_access;
    // The engine writes no author for a screen: its `/T` is the title.
    let options = MarkupOptions {
        note: placed
            .description
            .map(|d| super::annots::signed_note(d, None)),
        opacity,
        ..Default::default()
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "screen-annot-read page={page} bytes={} type={} trigger={} temp={} described={}",
            spec.bytes.len(),
            spec.content_type,
            crate::dialogs::textannot::trigger_token(spec.trigger),
            String::from_utf8_lossy(spec.temp_access.as_bytes()),
            options.note.is_some(),
        )
    });
    super::apply::vector_edit(doc, "add-screen-annot", page, 1, |session| {
        let id: ObjId = session.add_screen_annotation(page, &spec, &options)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("screen-annot-placed page={page} id={}", id.num)
        });
        Ok::<_, pdfcer_core::edit::EditError>(vec![disclosure])
    });
}
