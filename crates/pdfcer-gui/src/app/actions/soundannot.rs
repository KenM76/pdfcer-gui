//! # `app::actions::soundannot` — attach a recording to a page
//!
//! `/Sound` (§12.5.6.16) through `EditSession::add_sound_annotation`, one
//! undo entry. The WAV is picked by [`pick`] in `BeginTextAnnot`'s apply arm,
//! before the dialog opens, and read and converted by [`place`] when the
//! dialog's Add drains. Every conversion `SoundData::from_wav` reports is
//! named in the status note; a recording it refuses places nothing.

use std::path::{Path, PathBuf};

use pdfcer_core::annot_author::{Color, SoundIcon, SoundSpec};
use pdfcer_core::edit::MarkupOptions;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::sound::{SoundData, WavImportOptions};

use crate::app::prefs::Prefs;
use crate::app::state::OpenDoc;
use crate::text::soundannot as t;

/// Ask which recording to attach. `None` when the operator cancelled.
pub(super) fn pick() -> Option<PathBuf> {
    match crate::app::files::pick_sound_source() {
        crate::app::files::Picked::Path(path) => Some(path),
        _ => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                "sound-annot-cancelled".to_owned()
            });
            None
        }
    }
}

/// What the dialog accepted.
pub(super) struct Placed<'a> {
    /// The 0-based page.
    pub page: usize,
    /// The icon's rectangle, in PDF user space.
    pub rect: Rect,
    /// The WAV file.
    pub file: &'a Path,
    /// The icon.
    pub icon: SoundIcon,
    /// The trimmed description, or `None`.
    pub description: Option<&'a str>,
    /// How the WAV is converted.
    pub import: WavImportOptions,
}

/// Trace `line` and record `note` as the refusal; nothing is placed.
fn refuse(epoch: u64, line: String, note: String) {
    crate::diag::trace(|| line);
    super::record_note(epoch, note);
}

/// Read and convert the recording, then author the icon.
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
            // ui-text-exempt: diagnostic trace, never displayed
            let line = format!("sound-annot-unreadable detail={error}");
            return refuse(doc.edit_epoch, line, t::unreadable(&error.to_string()));
        }
    };
    let import = match SoundData::from_wav(&bytes, &placed.import) {
        Ok(import) => import,
        Err(error) => {
            // ui-text-exempt: diagnostic trace, never displayed
            let line = format!("sound-annot-refused detail={error}");
            return refuse(doc.edit_epoch, line, t::refused(&error.to_string()));
        }
    };
    let name = placed
        .file
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let disclosure = t::placed(&name, placed.page, &import.sound, &import.conversions);
    let (r, g, b) = ink;
    let mut spec = SoundSpec::new(placed.rect, import.sound);
    spec.icon = placed.icon.clone();
    spec.color = Color::Rgb(r, g, b);
    let options = MarkupOptions {
        note: placed
            .description
            .map(|d| super::annots::signed_note(d, Some(&prefs.author_name))),
        opacity,
        ..Default::default()
    };
    let page = placed.page;
    let conversions = import.conversions.len();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "sound-annot-read page={page} rate={} channels={} bits={} samples={} conversions={conversions} icon={} described={}",
            spec.sound.rate,
            spec.sound.channels,
            spec.sound.bits,
            spec.sound.samples.len(),
            String::from_utf8_lossy(spec.icon.name()),
            options.note.is_some(),
        )
    });
    let Ok((options, receipt)) = super::drawlayer::onto(doc, "add-sound-annot", options) else {
        return;
    };
    super::apply::vector_edit(doc, "add-sound-annot", page, 1, |session| {
        let id: ObjId = session.add_sound_annotation(page, &spec, &options)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("sound-annot-placed page={page} id={}", id.num)
        });
        Ok::<Vec<String>, pdfcer_core::edit::EditError>(
            std::iter::once(disclosure).chain(receipt).collect(),
        )
    });
}
