//! # `text::images` — the words the Insert-image window shows
//!
//! ## What this surface is
//!
//! `edit.insert_image` was registered, drawn on Edit ▸ Insert, listed in
//! `reach`'s `SCAFFOLDED` set with **no recorded reason at all**, and inert.
//! `EditSession::add_image` has shipped the whole time.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/images.md`.

use pdfcer_core::edit::ImageFit;
use pdfcer_core::image_import::{ImageFormat, RecompressReason};

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Insert image"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "The picture is added to the page as content — not as a comment — so it \
     prints and it is part of the drawing. Undo removes it."
}

/// The label on the source-file row.
#[must_use]
pub const fn source_label() -> &'static str {
    "File"
}

/// The label on the picture's own size row.
#[must_use]
pub const fn source_size_label() -> &'static str {
    "Picture"
}

/// The picture's format and pixel dimensions.
#[must_use]
pub fn source_size(format: ImageFormat, width_px: u32, height_px: u32) -> String {
    format!("{} · {width_px} × {height_px} pixels", format_name(format))
}

/// A raster format's name — **the engine's own**, never a local table.
#[must_use]
pub const fn format_name(format: ImageFormat) -> &'static str {
    format.name()
}

/// The picture's natural size on paper, and where that number came from.
#[must_use]
pub fn natural_size(width_mm: f64, height_mm: f64, declared_dpi: Option<(f64, f64)>) -> String {
    // Whole millimetres, half away from zero, like every other length. The dpi
    // values keep `{:.0}`: a resolution is not a length, nothing converted it
    // from points, and `units` holds no opinion about how one reads.
    let width_mm = crate::units::whole(width_mm);
    let height_mm = crate::units::whole(height_mm);
    match declared_dpi {
        Some((x, y)) if (x - y).abs() < 0.5 => {
            format!("{width_mm} × {height_mm} mm at the {x:.0} dpi the file declares")
        }
        Some((x, y)) => {
            format!("{width_mm} × {height_mm} mm at the {x:.0} × {y:.0} dpi the file declares")
        }
        None => format!(
            "{width_mm} × {height_mm} mm — the file declares no resolution, so pdfcer \
             reads one pixel as one point"
        ),
    }
}

/// The heading over the placement controls.
#[must_use]
pub const fn placement_heading() -> &'static str {
    "Where it goes"
}

/// The page-number label.
#[must_use]
pub fn placement_page(page_number: usize) -> String {
    format!("On page {page_number}")
}

/// Why the page is stated and not chosen.
#[must_use]
pub const fn placement_page_hint() -> &'static str {
    "Page the canvas is showing. Close this, go to another page, and open it \
     again to place there."
}

/// The label on the left-edge field.
#[must_use]
pub const fn placement_x() -> &'static str {
    "From the left"
}

/// The label on the bottom-edge field.
#[must_use]
pub const fn placement_y() -> &'static str {
    "From the bottom"
}

/// The label on the width field.
#[must_use]
pub const fn placement_width() -> &'static str {
    "Width"
}

/// The label on the height field.
#[must_use]
pub const fn placement_height() -> &'static str {
    "Height"
}

/// The heading over the fit choice.
#[must_use]
pub const fn fit_heading() -> &'static str {
    "If the shapes differ"
}

/// A fit mode's name.
#[must_use]
pub const fn fit_name(fit: ImageFit) -> &'static str {
    match fit {
        ImageFit::Contain => "Keep the picture's shape",
        ImageFit::Stretch => "Fill the box exactly",
        // `ImageFit` is `#[non_exhaustive]`, so this arm is FORCED rather than
        // chosen — see `D:/dev/rag/rust/`'s finding of the same name. A third
        // mode pdfcer gains renders as the engine's own debug name rather than
        // as a blank radio nobody can pick knowingly.
        // ui-text-exempt: a fallback naming an engine variant this build has no word for
        _ => "Another way",
    }
}

/// What each fit mode costs.
#[must_use]
pub const fn fit_hint(fit: ImageFit) -> &'static str {
    match fit {
        ImageFit::Contain => {
            "The picture is centred in the box and one side may be smaller than \
             you asked for."
        }
        ImageFit::Stretch => {
            "The box is honoured exactly and the picture is distorted if its \
             shape differs. Right when the box came from a measurement."
        }
        _ => "",
    }
}

/// **What resolution this placement will be**, before it is committed.
#[must_use]
pub fn dpi_preview(effective_dpi: (f64, f64), below_screen_resolution: bool) -> String {
    let (dx, dy) = effective_dpi;
    let dpi = if (dx - dy).abs() < 0.5 {
        format!("{dx:.0} dpi")
    } else {
        format!("{dx:.0} × {dy:.0} dpi")
    };
    if below_screen_resolution {
        format!("At this size the picture is {dpi} — it will look soft in print.")
    } else {
        format!("At this size the picture is {dpi}.")
    }
}

/// Where the picture will actually land, previewed from the engine's own
/// arithmetic.
#[must_use]
pub fn placed_note(width_mm: f64, height_mm: f64) -> String {
    let width_mm = crate::units::whole(width_mm);
    let height_mm = crate::units::whole(height_mm);
    format!("It will land {width_mm} × {height_mm} mm, centred in that box.")
}

/// The commit button.
#[must_use]
pub const fn insert_button() -> &'static str {
    "Insert"
}

/// The cancel button.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// A placement that is not on the page.
#[must_use]
pub const fn off_the_page() -> &'static str {
    "That box is not on the sheet. Reduce the size, or move it back inside."
}

/// A box with no area.
#[must_use]
pub const fn no_area() -> &'static str {
    "Give the box a width and a height."
}

/// **Why pdfcer re-encoded the picture instead of storing the file's bytes.**
#[must_use]
pub const fn recompress_reason(reason: RecompressReason) -> &'static str {
    match reason {
        RecompressReason::AlphaSplit => {
            "the picture carries transparency, which PDF stores as a separate \
             mask"
        }
        RecompressReason::NoCompressedSource => {
            "the file was not compressed, so pdfcer compressed it"
        }
        RecompressReason::SourceCodecNotReusable => {
            "the file was compressed in a way PDF cannot carry unchanged. The \
             pixels are exactly the file's"
        }
        RecompressReason::LosslessRequested => {
            "you asked for lossless storage and the file was in a lossy format"
        }
        RecompressReason::JpegRequested => "you asked for JPEG storage",
        // ui-text-exempt: forced by `#[non_exhaustive]`; says what happened without inventing why
        _ => "PDF could not carry the file's own bytes unchanged",
    }
}

/// The file could not be read as an image, in the engine's own words.
#[must_use]
pub fn import_failed(detail: &str) -> String {
    format!("That file was not inserted. {detail}")
}

// ---------------------------------------------------------------------------
// After the fact — what the placement did that the operator cannot see
// ---------------------------------------------------------------------------

/// **The disclosures image placement owes**, assembled into the sentences the
/// status bar shows.
#[must_use]
pub fn placement_disclosures(
    effective_dpi: (f64, f64),
    below_screen_resolution: bool,
    letterboxed: bool,
    aspect_distorted: bool,
    recompressed: Option<RecompressReason>,
    source_bytes: usize,
    stored_bytes: usize,
) -> Vec<String> {
    let mut out = Vec::new();

    let (dx, dy) = effective_dpi;
    let dpi = if (dx - dy).abs() < 0.5 {
        format!("{dx:.0} dpi")
    } else {
        format!("{dx:.0} × {dy:.0} dpi")
    };
    if below_screen_resolution {
        out.push(format!(
            "At this size the picture is {dpi} — below one pixel per point, so \
             it will look soft in print. It looks fine on screen either way."
        ));
    } else {
        out.push(format!("At this size the picture is {dpi}."));
    }

    if aspect_distorted {
        out.push(
            "The picture was stretched to fill the box, so its shape has \
             changed."
                .to_owned(),
        );
    } else if letterboxed {
        out.push(
            "The picture kept its shape, so it does not fill the box you gave \
             it."
            .to_owned(),
        );
    }

    if let Some(reason) = recompressed {
        out.push(format!(
            "pdfcer re-encoded the picture rather than storing the file's own \
             bytes — {}. {}",
            recompress_reason(reason),
            byte_change(source_bytes, stored_bytes)
        ));
    }
    out
}

/// What decoding a GIF or TIFF left out or reinterpreted, one sentence each.
#[must_use]
pub fn source_decoding_notes(d: &pdfcer_core::edit::ImageAuthorDisclosures) -> Vec<String> {
    let mut out = Vec::new();
    if d.gif_frames_ignored > 0 {
        out.push(format!(
            "The GIF is animated; only its first frame was placed, and {} {} left out.",
            d.gif_frames_ignored,
            if d.gif_frames_ignored == 1 {
                "frame was"
            } else {
                "frames were"
            }
        ));
    }
    if d.tiff_pages_ignored > 0 {
        out.push(format!(
            "The TIFF has {} more {}; only the first was placed.",
            d.tiff_pages_ignored,
            if d.tiff_pages_ignored == 1 {
                "page"
            } else {
                "pages"
            }
        ));
    }
    if d.tiff_extra_samples_dropped > 0 {
        out.push(format!(
            "The TIFF carried {} extra {} with no stated meaning; {} left out rather than read as transparency.",
            d.tiff_extra_samples_dropped,
            if d.tiff_extra_samples_dropped == 1 { "channel" } else { "channels" },
            if d.tiff_extra_samples_dropped == 1 { "it was" } else { "they were" }
        ));
    }
    if d.tiff_associated_alpha_unpremultiplied {
        out.push(
            "The TIFF's transparency was converted for PDF; the faintest edges may differ slightly."
                .to_owned(),
        );
    }
    if d.tiff_white_is_zero_inverted {
        out.push(
            "The TIFF stores white as zero, so its tones were flipped to print as it looks."
                .to_owned(),
        );
    }
    if d.tiff_palette_assumed_8bit {
        out.push(
            "The TIFF's colour table was read as 8-bit because every value fits; if its colours look wrong, that guess is why."
                .to_owned(),
        );
    }
    out
}

/// How the stored size compares with the source file's.
#[must_use]
fn byte_change(source: usize, stored: usize) -> String {
    let from = crate::text::panels::byte_size(source);
    let to = crate::text::panels::byte_size(stored);
    if stored > source {
        format!("{from} in the file became {to} in the document.")
    } else {
        format!("{from} in the file became {to} in the document — smaller.")
    }
}

// ---------------------------------------------------------------------------
// Drawings — an SVG or EMF placed as vector artwork
// ---------------------------------------------------------------------------

/// What a drawing is, on the window's Picture row; `kind` is
/// `Picture::kind`.
#[must_use]
pub fn drawing_source(kind: &str) -> String {
    format!(
        "{} drawing · placed as lines and fills, not pixels",
        kind.to_ascii_uppercase()
    )
}

/// A drawing's own size on paper.
#[must_use]
pub fn drawing_natural_size(width_mm: f64, height_mm: f64) -> String {
    let width_mm = crate::units::whole(width_mm);
    let height_mm = crate::units::whole(height_mm);
    format!("{width_mm} × {height_mm} mm at its own size")
}

/// How a drawing meets the box, in place of the raster fit choice.
#[must_use]
pub const fn drawing_fills_the_box() -> &'static str {
    "The drawing fills the box. A box of a different shape stretches it."
}

/// What the import could not carry exactly, before the drawing is placed;
/// `summary` is the engine's operator line.
#[must_use]
pub fn drawing_not_exact(summary: &str) -> String {
    format!("Not everything in the file comes across exactly — {summary}.")
}

/// What a drawing's placement did that the page does not show.
#[must_use]
pub fn drawing_disclosures(distorted: bool, notes: Option<&str>) -> Vec<String> {
    let mut out = vec!["The drawing was placed as vector artwork.".to_owned()];
    if distorted {
        out.push("The drawing was stretched to fill the box, so its shape has changed.".to_owned());
    }
    if let Some(summary) = notes {
        out.push(drawing_not_exact(summary));
    }
    out
}

/// A drawing placed as a stamp, when the operator had set an author name or
/// an opacity the engine's drawing-stamp verb cannot take.
#[must_use]
pub const fn drawing_stamp_unsigned() -> &'static str {
    "A drawing's stamp carries no author name, date or opacity yet, so this one is unsigned and opaque."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A soft placement says so, and a fine one still states the number.
    #[test]
    fn the_resolution_is_always_stated_and_a_soft_one_says_why_it_matters() {
        let soft = placement_disclosures((30.0, 30.0), true, false, false, None, 0, 0);
        assert!(soft[0].contains("30 dpi"), "{soft:?}");
        assert!(soft[0].contains("soft in print"), "{soft:?}");

        let fine = placement_disclosures((300.0, 300.0), false, false, false, None, 0, 0);
        assert!(fine[0].contains("300 dpi"), "{fine:?}");
        assert!(!fine[0].contains("soft"), "{fine:?}");
    }

    /// Distortion and letterboxing are different sentences, and never both.
    #[test]
    fn a_placement_is_letterboxed_or_distorted_and_not_both() {
        let boxed = placement_disclosures((72.0, 72.0), false, true, false, None, 0, 0);
        assert!(
            boxed.iter().any(|s| s.contains("kept its shape")),
            "{boxed:?}"
        );

        let squashed = placement_disclosures((72.0, 72.0), false, false, true, None, 0, 0);
        assert!(
            squashed.iter().any(|s| s.contains("stretched")),
            "{squashed:?}"
        );
        assert!(
            !squashed.iter().any(|s| s.contains("kept its shape")),
            "{squashed:?}"
        );
    }

    /// A clean placement says one thing.
    #[test]
    fn nothing_worth_saying_produces_exactly_one_sentence() {
        let clean = placement_disclosures((150.0, 150.0), false, false, false, None, 0, 0);
        assert_eq!(clean.len(), 1, "{clean:?}");
    }

    /// A re-encode names the reason and both byte counts.
    #[test]
    fn a_recompression_names_its_reason_and_both_sizes() {
        let note = placement_disclosures(
            (150.0, 150.0),
            false,
            false,
            false,
            Some(RecompressReason::AlphaSplit),
            1024,
            4096,
        );
        let last = note.last().expect("a recompression note");
        assert!(last.contains("transparency"), "{last}");
        assert!(last.contains("1.0 KB"), "{last}");
        assert!(last.contains("4.0 KB"), "{last}");
    }

    /// Every known re-encode reason has its own sentence, and none reaches
    /// the forced fallback.
    #[test]
    fn every_known_recompression_reason_has_its_own_words() {
        let all = [
            RecompressReason::AlphaSplit,
            RecompressReason::NoCompressedSource,
            RecompressReason::SourceCodecNotReusable,
            RecompressReason::LosslessRequested,
            RecompressReason::JpegRequested,
        ];
        let fallback = recompress_reason_fallback();
        let mut seen = std::collections::BTreeSet::new();
        for reason in all {
            let text = recompress_reason(reason);
            assert_ne!(text, fallback, "{reason:?} fell through to the fallback");
            assert!(
                seen.insert(text),
                "{reason:?} shares its wording with another"
            );
        }
    }

    /// The fallback, reachable only through a variant this build does not know.
    fn recompress_reason_fallback() -> &'static str {
        // Constructed by exclusion: the fallback is what the match returns for
        // a variant not listed, and there is no way to name one. Compared
        // against the arm's own text instead.
        "PDF could not carry the file's own bytes unchanged"
    }

    /// The preview and the after-the-fact disclosure say the same number the
    /// same way.
    #[test]
    fn the_preview_and_the_outcome_state_the_resolution_alike() {
        let preview = dpi_preview((150.0, 150.0), false);
        let outcome = placement_disclosures((150.0, 150.0), false, false, false, None, 0, 0);
        assert!(preview.contains("150 dpi"), "{preview}");
        assert!(outcome[0].contains("150 dpi"), "{outcome:?}");

        let soft_preview = dpi_preview((30.0, 30.0), true);
        let soft_outcome = placement_disclosures((30.0, 30.0), true, false, false, None, 0, 0);
        assert!(soft_preview.contains("soft in print"), "{soft_preview}");
        assert!(
            soft_outcome[0].contains("soft in print"),
            "{soft_outcome:?}"
        );
    }

    /// An unresolved resolution is stated as an assumption, not as a fact
    /// about the file.
    #[test]
    fn an_assumed_resolution_says_it_was_assumed() {
        let declared = natural_size(100.0, 50.0, Some((300.0, 300.0)));
        assert!(declared.contains("the file declares"), "{declared}");

        let assumed = natural_size(100.0, 50.0, None);
        assert!(assumed.contains("declares no resolution"), "{assumed}");
        assert!(
            assumed.contains("one pixel as one point"),
            "and says what pdfcer did instead: {assumed}"
        );
    }
}
