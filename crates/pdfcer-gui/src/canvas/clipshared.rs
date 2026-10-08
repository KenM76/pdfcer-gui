//! # `canvas::clipshared` — a selection another pdfcer-gui window copied
//!
//! Contract: [`adopt`] reads the clip another window published under
//! `clipimage::OBJECT_CLIP_FORMAT` when this window holds no clip of its own or
//! the OS clipboard has changed since this window last copied. A clip it reads
//! is stored as this window's own, so the paste that follows is the same
//! lossless `paste_objects` a copy in this window gives, and a second paste
//! needs no second read. Bytes that are absent or are not an `ObjectClip` leave
//! the ordinary OS paste to run.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/clipshared.md`.

use pdfcer_core::vector::ObjectClip;

use crate::canvas::clipboard::Clipped;

/// The page a clip from another window records as its source. No page of
/// this document is it, so a paste without a pointer lands in place rather
/// than offset.
pub const FOREIGN_PAGE: usize = usize::MAX;

/// The other window's clip, stored as this window's own, or `None`.
pub fn adopt(ctx: &egui::Context, has_clip: bool) -> Option<Clipped> {
    if has_clip && !crate::canvas::clipseq::changed(ctx) {
        return None;
    }
    let bytes = read()?;
    let clip = ObjectClip::from_bytes(&bytes).ok()?;
    let anchor = (!clip.bbox().is_empty()).then(|| {
        let b = clip.bbox();
        ((b.min.x + b.max.x) / 2.0, (b.min.y + b.max.y) / 2.0)
    });
    let (count, annotations) = (
        clip.len(),
        crate::canvas::annotclip::Plan::of(&clip).carried(),
    );
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "clip-adopted objects={count} annots={annotations} bytes={}",
            bytes.len()
        )
    });
    let clipped = Clipped::Selection {
        count,
        annotations,
        annot_ids: Vec::new(),
        left_behind: Vec::new(),
        thin: 0,
        in_form_left: 0,
        bytes,
        page: FOREIGN_PAGE,
        anchor,
    };
    crate::canvas::clipboard::store(ctx, clipped.clone());
    Some(clipped)
}

/// The other window's bytes; under test, none, so no unit test reads the
/// machine's clipboard.
fn read() -> Option<Vec<u8>> {
    #[cfg(test)]
    return None;
    #[cfg(not(test))]
    pdfcer_gui_base::clippaste::own_clip()
}
