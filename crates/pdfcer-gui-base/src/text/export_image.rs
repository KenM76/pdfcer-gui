//! # `text::export_image` — the words the Export-image window shows, and the
//! sentences an image export owes afterwards
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/export_image.md`.

use pdfcer_render::display_list::ExportTally;

use crate::imageexport::{EmfCounts, ImageFormat, Impossible};

// ===========================================================================
// THE WINDOW
// ===========================================================================

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Export image"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "Write pages out as pictures another program can open. One file per page. \
     Afterwards pdfcer says what it could not carry across exactly."
}

/// The heading over the format radios.
#[must_use]
pub const fn format_heading() -> &'static str {
    "Format"
}

/// The name of one format, as the radio beside it reads.
#[must_use]
pub const fn format_name(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::Svg => "SVG",
        // The acronym AND what it stands for, unlike the other three, and
        // that asymmetry is deliberate. PNG, JPEG and SVG are names an
        // operator has met; "EMF" is one almost nobody has, and a radio
        // reading only "EMF" is a radio nobody presses. The long form is what
        // the Windows *Paste Special* list itself says, which is where this
        // operator will have seen it if they have seen it anywhere.
        ImageFormat::Emf => "EMF (Windows metafile)",
    }
}

/// What each format is FOR, in one line under its radio.
#[must_use]
pub const fn format_hint(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => {
            "Every pixel exactly as pdfcer drew it, and it can keep a clear \
             background. The safe answer for a drawing."
        }
        ImageFormat::Jpeg => {
            "Smaller files, and photographs survive it well. Line art and text \
             pick up smudges around the edges, and it cannot hold a clear \
             background at all."
        }
        // Both vector hints name PROGRAMS rather than properties, because
        // the choice between these two is not a choice about fidelity — it is
        // a choice about which program is going to open the file, and an
        // operator who has just been told "SVG keeps lines as lines" has no
        // way to know that their copy of LibreOffice will refuse it.
        ImageFormat::Svg => {
            "Lines stay lines, so it can be scaled up without going blocky and \
             edited in Inkscape or Illustrator. Text arrives as outlines \
             rather than as words, unless you keep it as text below."
        }
        ImageFormat::Emf => {
            "Also lines rather than pixels, for the programs that will not \
             open an SVG — LibreOffice 24, Visio, CorelDRAW, and Word's Paste \
             Special. Anything see-through in the page becomes a picture."
        }
    }
}

/// The heading over the page controls.
#[must_use]
pub const fn pages_heading() -> &'static str {
    "Pages"
}

/// The *this page only* radio, naming the page so the choice is checkable.
#[must_use]
pub fn pages_current(page_number: usize) -> String {
    format!("This page only (page {page_number})")
}

/// The *every page* radio, naming the count for the same reason.
#[must_use]
pub fn pages_all(count: usize) -> String {
    match count {
        1 => "Every page (1 page)".to_owned(),
        n => format!("Every page ({n} pages)"),
    }
}

/// The *typed range* radio.
#[must_use]
pub const fn pages_range() -> &'static str {
    "Pages:"
}

/// What the range box accepts, beside it.
#[must_use]
pub const fn pages_range_hint() -> &'static str {
    "For example 1-3, 7, 10-12"
}

/// The typed range does not name any page in this document.
#[must_use]
pub fn pages_range_invalid(count: usize) -> String {
    format!(
        "That does not name any page in this document, which has {count}. \
         Nothing will be exported until it does."
    )
}

/// The heading over the resolution control.
#[must_use]
pub const fn dpi_heading() -> &'static str {
    "Resolution"
}

/// The label beside the resolution field.
#[must_use]
pub const fn dpi_label() -> &'static str {
    "Dots per inch"
}

/// What the number means, and it means two different things.
#[must_use]
pub const fn dpi_hint(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png | ImageFormat::Jpeg => {
            "How large the picture is, and it is written into the file so Word \
             and its like place it at the page's real size. 300 is print \
             grade; 96 is screen grade."
        }
        ImageFormat::Svg => {
            "Lines and curves are exact at any value. This only governs the \
             parts that have to be embedded as a picture — shadings with no \
             gradient form, soft masks, and pictures the PDF already carried."
        }
        // Worded separately from SVG rather than sharing its arm, because
        // the list of things that "have to be embedded as a picture" is much
        // longer here — every gradient and everything see-through, not just
        // the awkward cases — so the same sentence would understate it by a
        // long way on exactly the pages where the number matters.
        ImageFormat::Emf => {
            "Lines and curves are exact at any value. This governs everything \
             that has to become a picture instead, which in a metafile is \
             more: every gradient, and anything see-through."
        }
    }
}

/// The pixel size this resolution will produce, before anything is written.
#[must_use]
pub fn dpi_pixels(width: u32, height: u32) -> String {
    format!("The largest page comes out {width} by {height} pixels.")
}

/// The requested resolution cannot be rendered at all.
#[must_use]
pub fn dpi_too_large(width: u32, height: u32, limit: u32) -> String {
    format!(
        "At that resolution the largest page would be {width} by {height} \
         pixels, and pdfcer cannot draw a picture with a side longer than \
         {limit}. Lower the resolution."
    )
}

/// The heading over the background controls.
#[must_use]
pub const fn background_heading() -> &'static str {
    "Background"
}

/// The keep-transparency checkbox.
#[must_use]
pub const fn keep_transparency() -> &'static str {
    "Keep transparency"
}

/// What keeping it does, for the format that can.
#[must_use]
pub const fn keep_transparency_hint() -> &'static str {
    "Anywhere the page is blank stays clear instead of becoming white, so the \
     drawing sits on whatever it is placed over."
}

/// What clearing it does.
#[must_use]
pub const fn flatten_hint() -> &'static str {
    "The page is written onto solid white, the way it looks on screen."
}

/// **The refusal, drawn beside the control that would offer the
/// impossible combination.**
#[must_use]
pub const fn jpeg_has_no_alpha() -> &'static str {
    "A JPEG cannot be transparent — the format has no way to store one, so \
     anything clear on this page would come out on a solid background. Choose \
     PNG or SVG if you need it to stay clear."
}

/// The heading over the JPEG quality control.
#[must_use]
pub const fn quality_heading() -> &'static str {
    "Quality"
}

/// The label beside the quality field.
#[must_use]
pub const fn quality_label() -> &'static str {
    "JPEG quality"
}

/// What quality costs, in the terms this operator's files are in.
#[must_use]
pub const fn quality_hint() -> &'static str {
    "Lower makes a smaller file and smudges the edges of lines and letters. \
     Drawings need it high; 90 is a good place to stay."
}

/// The Export button.
#[must_use]
pub const fn export_button() -> &'static str {
    "Export…"
}

/// The Cancel button.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The title on the save dialog the Export button opens.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Export image"
}

/// How several pages are named, said BEFORE the save dialog opens.
#[must_use]
pub fn multi_page_naming(count: usize, example: &str) -> String {
    format!(
        "{count} files will be written, one per page, beside the name you \
         choose — {example}."
    )
}

// ===========================================================================
// THE RECEIPT — everything below is off-canvas, after the export.
// ===========================================================================

/// A raster export succeeded.
#[must_use]
pub fn wrote_raster(path: &str, page_number: usize, width: u32, height: u32, dpi: f32) -> String {
    format!(
        "Page {page_number} written to {path} — {width} by {height} pixels, \
         and {dpi:.0} dots per inch is recorded in the file so it is placed \
         at the page's real size."
    )
}

/// An SVG export succeeded.
#[must_use]
pub fn wrote_svg(path: &str, page_number: usize, ops: usize) -> String {
    format!("Page {page_number} written to {path} — {ops} drawing operations.")
}

/// An EMF export succeeded.
#[must_use]
pub fn wrote_emf(path: &str, page_number: usize, ops: usize, rasters: usize) -> String {
    if rasters == 0 {
        format!(
            "Page {page_number} written to {path} — {ops} drawing operations, \
             all of them real lines."
        )
    } else {
        format!(
            "Page {page_number} written to {path} — {ops} drawing operations, \
             plus {rasters} part(s) that had to go in as pictures."
        )
    }
}

/// Several files were written; this replaces the per-file line.
#[must_use]
pub fn wrote_many(count: usize, first: &str, last: &str) -> String {
    format!("{count} files written, from {first} to {last}.")
}

/// The transparency the operator asked for was kept.
#[must_use]
pub const fn transparency_kept() -> &'static str {
    "The blank parts of the page are clear rather than white, so it can be \
     placed over something."
}

/// The page was written onto white because that is what was asked for.
#[must_use]
pub const fn flattened_to_white() -> &'static str {
    "The page is on solid white, as you asked."
}

/// The page was written onto the colour the operator typed, `hex` as `#rrggbb`.
#[must_use]
pub fn flattened_to(hex: &str) -> String {
    format!("The page is on solid {hex}, as you asked.")
}

/// The background-colour field's label.
#[must_use]
pub const fn background_colour_label() -> &'static str {
    "Colour:"
}

/// Under the field when what is typed is not a colour; Export waits for one.
#[must_use]
pub const fn background_colour_refused() -> &'static str {
    "That is not a colour. Type six hex digits, like #ffffff for white, or \
     pick one with the swatch."
}

/// The rendering-standard row's heading.
#[must_use]
pub const fn standard_heading() -> &'static str {
    "Draw it as"
}

/// The rendering-standard choice that leaves the operator's settings alone.
#[must_use]
pub const fn standard_own_settings() -> &'static str {
    "Your settings"
}

/// Under the rendering-standard choice: what choosing one does and does not do.
#[must_use]
pub const fn standard_hint() -> &'static str {
    "A standard changes how colours and pictures are drawn for this export \
     only. Your settings are not changed."
}

/// The receipt's sentence for an export drawn under `title`, which set
/// `changed` of the operator's settings for this export only.
#[must_use]
pub fn drawn_as(title: &str, changed: usize) -> String {
    match changed {
        0 => format!("Drawn as {title}, which matched your settings already."),
        1 => format!("Drawn as {title}, which changed 1 of your settings for this export only."),
        n => format!("Drawn as {title}, which changed {n} of your settings for this export only."),
    }
}

/// **A transparent JPEG was requested and NOTHING was written.**
#[must_use]
pub const fn transparent_jpeg_refused() -> &'static str {
    "Nothing was written. A transparent JPEG does not exist — the format has \
     no way to store transparency — and pdfcer will not quietly put your page \
     on a white background instead. Export as PNG or SVG to keep it clear, or \
     clear Keep transparency to have the white on purpose."
}

/// Turn an impossible combination into the sentence that refuses it.
#[must_use]
pub const fn refused(why: Impossible) -> &'static str {
    match why {
        Impossible::TransparentJpeg => transparent_jpeg_refused(),
    }
}

/// **What the SVG could not express exactly** — Rule 4's whole content.
#[must_use]
pub fn svg_fidelity(tally: &ExportTally, dashed: usize, blends: usize) -> Vec<String> {
    // Always, and first: the one nothing counts. See clause 3 above.
    svg_fidelity_with(
        vec![svg_text_is_outlines().to_owned()],
        tally,
        dashed,
        blends,
    )
}

/// [`svg_fidelity`] with the text sentences supplied by the caller: the
/// outlines sentence, or `crate::text::export_keeptext::svg_kept` when the
/// export kept text. They lead; the recording's losses follow.
#[must_use]
pub fn svg_fidelity_with(
    text: Vec<String>,
    tally: &ExportTally,
    dashed: usize,
    blends: usize,
) -> Vec<String> {
    let mut out = text;

    if tally.shadings_rasterised > 0 {
        out.push(match tally.shadings_rasterised {
            1 => "1 shaded area had no equivalent in SVG and was embedded as a \
                  picture at the resolution above, so it will go blocky if the \
                  file is scaled up a long way."
                .to_owned(),
            n => format!(
                "{n} shaded areas had no equivalent in SVG and were embedded \
                 as pictures at the resolution above, so they will go blocky \
                 if the file is scaled up a long way."
            ),
        });
    }
    if tally.soft_masks_kept > 0 {
        out.push(match tally.soft_masks_kept {
            1 => "1 soft mask was kept as a mask, which Inkscape honours and \
                  Word's importer does not."
                .to_owned(),
            n => format!(
                "{n} soft masks were kept as masks, which Inkscape honours and \
                 Word's importer does not."
            ),
        });
    }
    if tally.overprint_approximated > 0 {
        out.push(format!(
            "{} paint(s) set to overprint were drawn as ordinary paint. On a \
             printing press those inks would mix; here the top one covers what \
             is under it.",
            tally.overprint_approximated
        ));
    }
    if tally.nonseparable_approximated > 0 {
        out.push(format!(
            "{} paint(s) using a hue, saturation, colour or luminosity blend \
             were drawn normally, so their colour where they overlap is an \
             approximation.",
            tally.nonseparable_approximated
        ));
    }
    if tally.non_isolated_groups_isolated > 0 {
        out.push(format!(
            "{} group(s) that were meant to blend with what is behind them \
             were drawn as though they stood alone.",
            tally.non_isolated_groups_isolated
        ));
    }
    if tally.colorant_buffer_on_screen > 0 {
        out.push(
            "This page asks to be blended in printing inks and was blended in \
             screen colours instead, so overlaps are close rather than exact."
                .to_owned(),
        );
    }
    if tally.tiling_patterns > 0 {
        out.push(format!(
            "{} tiling pattern(s) could not be written as a repeating fill.",
            tally.tiling_patterns
        ));
    }
    if dashed > 0 {
        out.push(format!(
            "{dashed} dashed line(s) were written as the individual dashes. \
             The picture is right; what is lost is the ability to change the \
             dash pattern later."
        ));
    }
    if blends > 0 {
        out.push(format!(
            "{blends} element(s) use a blend mode. Inkscape honours it; Word's \
             importer draws them normally."
        ));
    }

    // See clause 4. `is_exact()` is the engine's own reading of its own tally
    // and is asked rather than re-derived here, so a ninth counter arriving
    // makes this line stop appearing rather than making it lie.
    if tally.is_exact() && dashed == 0 && blends == 0 {
        out.push(
            "Everything else on this page went out as real geometry — nothing \
             had to be approximated."
                .to_owned(),
        );
    }
    out
}

/// The one nobody counts, and the one most owed.
#[must_use]
pub const fn svg_text_is_outlines() -> &'static str {
    "Text in an SVG is written as outlines, not as words. It will look right \
     in any program, and it cannot be selected, searched or re-typed there, \
     and no font travels with it."
}

/// **What the metafile could not express exactly** — Rule 4's content for
/// EMF, and it is a longer confession than the SVG's.
#[must_use]
pub fn emf_fidelity(counts: &EmfCounts) -> Vec<String> {
    // Always, and first: the one nothing counts. `svg_fidelity`'s clause 3,
    // and the same reasoning — a disclosure that listed only *counted* things
    // would go silent on the largest surprise the format holds.
    emf_fidelity_with(vec![emf_text_is_outlines().to_owned()], counts)
}

/// [`emf_fidelity`] with the text sentences supplied by the caller: the
/// outlines sentence, or `crate::text::export_keeptext::emf_kept` when the
/// export kept text.
#[must_use]
pub fn emf_fidelity_with(text: Vec<String>, counts: &EmfCounts) -> Vec<String> {
    let mut out = text;

    if counts.rasters_embedded > 0 {
        out.push(format!(
            "{} part(s) of the page could not be lines in a metafile and were \
             put in as pictures at the resolution above — {} see-through, {} \
             using a blend mode, {} gradients, {} pictures the PDF already \
             had, and {} see-through groups. They will go blocky if the file \
             is scaled up a long way.",
            counts.rasters_embedded,
            counts.ops_rasterised_for_alpha,
            counts.blend_modes_dropped,
            counts.gradients_rasterised,
            counts.images_embedded,
            counts.layers_rasterised,
        ));
        // Its own sentence because it names a program, and because it is the
        // one caveat that does not apply to the format's own readers. The
        // engine's note: *"Inkscape's EMF importer draws nothing for the alpha
        // bitmaps"* — which is survivable on the clipboard, where Inkscape
        // takes the SVG instead, and is not survivable for a `.emf` file.
        out.push(
            "Inkscape's metafile import draws none of those pictures. If \
             Inkscape is where this is going, export SVG instead."
                .to_owned(),
        );
    }
    if counts.nonzero_fills_multi_subpath > 0 {
        out.push(format!(
            "{} filled shape(s) are made of several loops. LibreOffice 24 \
             ignores which loops are holes and which are solid, so those \
             shapes may open there with holes in them. LibreOffice 25.2 and \
             Word do not have this problem.",
            counts.nonzero_fills_multi_subpath
        ));
    }
    if counts.dashed_strokes_pre_applied > 0 {
        out.push(format!(
            "{} dashed line(s) were written as the individual dashes. The \
             picture is right; what is lost is the ability to change the dash \
             pattern later.",
            counts.dashed_strokes_pre_applied
        ));
    }
    // The recording's own losses — everything that had already been
    // approximated before the metafile writer saw the page. Worded as in
    // `svg_fidelity`, because they are the same losses arriving by the same
    // route and an operator who has read one should recognise the other.
    if counts.tally.overprint_approximated > 0 {
        out.push(format!(
            "{} paint(s) set to overprint were drawn as ordinary paint. On a \
             printing press those inks would mix; here the top one covers what \
             is under it.",
            counts.tally.overprint_approximated
        ));
    }
    if counts.tally.nonseparable_approximated > 0 {
        out.push(format!(
            "{} paint(s) using a hue, saturation, colour or luminosity blend \
             were drawn normally, so their colour where they overlap is an \
             approximation.",
            counts.tally.nonseparable_approximated
        ));
    }
    if counts.tally.colorant_buffer_on_screen > 0 {
        out.push(
            "This page asks to be blended in printing inks and was blended in \
             screen colours instead, so overlaps are close rather than exact."
                .to_owned(),
        );
    }
    if counts.tally.tiling_patterns > 0 {
        out.push(format!(
            "{} tiling pattern(s) could not be written as a repeating fill.",
            counts.tally.tiling_patterns
        ));
    }

    // See `svg_fidelity`'s clause 4: an exact page must SAY so, or a broken
    // disclosure is indistinguishable from a clean one. `EmfCounts::is_exact`
    // is asked rather than re-derived, and it deliberately means something
    // stricter than `tally.is_exact()` — see its own doc.
    if counts.is_exact() {
        out.push(
            "Everything else on this page went out as real lines — nothing had \
             to become a picture."
                .to_owned(),
        );
    }
    out
}

/// The metafile's own always-true loss, [`svg_text_is_outlines`]'s twin.
#[must_use]
pub const fn emf_text_is_outlines() -> &'static str {
    "Text in a metafile is written as outlines, not as words. It will look \
     right in any program, and it cannot be selected, searched or re-typed \
     there, and no font travels with it."
}

/// The page could not be drawn at all.
#[must_use]
pub fn render_failed(page_number: usize, detail: &str) -> String {
    format!("Page {page_number} could not be drawn, so nothing was written. {detail}")
}

/// The picture was drawn and the encoder refused it.
#[must_use]
pub fn encode_failed(page_number: usize, detail: &str) -> String {
    format!("Page {page_number} could not be written as this format. {detail}")
}

/// The file could not be written.
#[must_use]
pub fn write_failed(detail: &str) -> String {
    format!("Nothing was written. {detail}")
}

/// There are no pages to export.
#[must_use]
pub const fn no_pages() -> &'static str {
    "Nothing was exported — no page was selected."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Asking for a transparent JPEG is refused BY NAME.**
    #[test]
    fn a_transparent_jpeg_is_refused_by_name() {
        let window = jpeg_has_no_alpha();
        assert!(window.contains("JPEG"), "it names the format: {window}");
        assert!(
            window.contains("cannot be transparent"),
            "it names the impossibility: {window}"
        );
        assert!(
            window.contains("PNG") && window.contains("SVG"),
            "it names what to do instead: {window}"
        );

        let receipt = refused(Impossible::TransparentJpeg);
        assert!(
            receipt.starts_with("Nothing was written."),
            "a refusal must say nothing happened, first: {receipt}"
        );
        assert!(
            receipt.contains("will not quietly"),
            "the refusal must say it declined to flatten, not merely that it \
             could not: {receipt}"
        );
    }

    /// **A PNG's receipt states the resolution that went INTO the file.**
    #[test]
    fn a_raster_receipt_says_the_resolution_is_in_the_file() {
        let note = wrote_raster("C:\\d\\a.png", 2, 2550, 3300, 300.0);
        assert!(note.contains("2550 by 3300"), "{note}");
        assert!(note.contains("300 dots per inch"), "{note}");
        assert!(
            note.contains("recorded in the file"),
            "the point is that it TRAVELS, not that it was chosen: {note}"
        );
        assert!(note.contains("real size"), "{note}");
    }

    /// **An exact SVG reports nothing lost — but still says text became
    /// outlines.**
    #[test]
    fn an_exact_svg_still_discloses_that_text_became_outlines() {
        let tally = ExportTally::default();
        assert!(tally.is_exact(), "the fixture must be the exact case");
        let notes = svg_fidelity(&tally, 0, 0);
        let joined = notes.join(" | ");
        assert!(
            joined.contains("written as outlines"),
            "the always-true disclosure is missing: {joined}"
        );
        assert!(
            joined.contains("nothing had to be approximated"),
            "an exact page must SAY it was exact, or a broken disclosure looks \
             the same as a clean one: {joined}"
        );
        assert!(
            !joined.contains("embedded as a picture"),
            "nothing was rasterised; it must not claim otherwise: {joined}"
        );
    }

    /// **An inexact SVG names each thing it lost.**
    #[test]
    fn an_inexact_svg_reports_what_it_could_not_express() {
        let mut tally = ExportTally::default();
        tally.shadings_rasterised = 3;
        tally.soft_masks_kept = 1;
        tally.overprint_approximated = 2;
        assert!(!tally.is_exact());

        let joined = svg_fidelity(&tally, 4, 1).join(" | ");
        assert!(joined.contains("3 shaded areas"), "{joined}");
        assert!(joined.contains("1 soft mask was kept"), "{joined}");
        assert!(joined.contains("2 paint(s) set to overprint"), "{joined}");
        assert!(joined.contains("4 dashed line(s)"), "{joined}");
        assert!(joined.contains("1 element(s) use a blend mode"), "{joined}");
        assert!(
            !joined.contains("nothing had to be approximated"),
            "an inexact page must NOT claim it was exact: {joined}"
        );
    }

    /// **A native gradient is fidelity and is not confessed as a loss.**
    #[test]
    fn a_gradient_that_went_out_natively_is_not_reported_as_a_loss() {
        let mut tally = ExportTally::default();
        tally.shadings_as_gradients = 9;
        assert!(tally.is_exact(), "the engine's own reading");
        let joined = svg_fidelity(&tally, 0, 0).join(" | ");
        assert!(
            joined.contains("nothing had to be approximated"),
            "a page whose only shading went out as a real gradient is exact: {joined}"
        );
        assert!(!joined.contains("blocky"), "{joined}");
    }

    /// The resolution hint says a DIFFERENT thing for SVG, because the
    /// number means a different thing.
    #[test]
    fn the_resolution_hint_is_not_the_same_sentence_for_a_vector_format() {
        let raster = dpi_hint(ImageFormat::Png);
        let vector = dpi_hint(ImageFormat::Svg);
        assert_ne!(raster, vector);
        assert!(
            raster.contains("written into the file"),
            "the raster case's whole point: {raster}"
        );
        assert!(
            vector.contains("exact at any value"),
            "the vector case must say raising it will not sharpen the lines: {vector}"
        );
        assert_eq!(dpi_hint(ImageFormat::Jpeg), raster);
        // EMF gets its OWN vector sentence rather than sharing SVG's. The
        // list of things that must become a picture is much longer in a
        // metafile — every gradient, everything see-through — so SVG's wording
        // would understate it on exactly the pages where the number matters.
        let metafile = dpi_hint(ImageFormat::Emf);
        assert_ne!(metafile, raster);
        assert_ne!(
            metafile, vector,
            "sharing SVG's arm would tell an EMF operator that only the \
             awkward cases are rasterised, which is false"
        );
        assert!(metafile.contains("exact at any value"), "{metafile}");
    }

    /// **Every format has a name, a hint and a resolution hint, and no
    /// two formats share a name.**
    #[test]
    fn every_format_is_named_and_described_in_its_own_words() {
        let mut names: Vec<&str> = Vec::new();
        for format in ImageFormat::ALL {
            let name = format_name(format);
            assert!(!name.is_empty(), "{format:?} has no name");
            assert!(
                !format_hint(format).is_empty(),
                "{format:?} has no hint under its radio"
            );
            assert!(
                !dpi_hint(format).is_empty(),
                "{format:?} has no resolution hint"
            );
            names.push(name);
        }
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            count,
            "two formats share a display name; the radio group would show the \
             same label twice"
        );
        assert_eq!(count, 4, "PNG, JPEG, SVG, EMF");
    }

    /// **The EMF radio spells out what the acronym means.**
    #[test]
    fn the_metafile_radio_does_not_read_as_a_bare_acronym() {
        let name = format_name(ImageFormat::Emf);
        assert!(name.contains("EMF"), "{name}");
        assert!(
            name.contains("metafile"),
            "the acronym alone teaches nothing: {name}"
        );
        // The hint names the PROGRAMS, which is the actual question being
        // answered by the choice between SVG and EMF.
        let hint = format_hint(ImageFormat::Emf);
        assert!(hint.contains("LibreOffice"), "{hint}");
        assert!(hint.contains("Paste Special"), "{hint}");
    }

    /// **An exact metafile reports nothing lost — and still says text
    /// became outlines.**
    #[test]
    fn an_exact_metafile_still_discloses_that_text_became_outlines() {
        let counts = EmfCounts {
            ops: 412,
            ..EmfCounts::default()
        };
        assert!(counts.is_exact(), "the fixture must be the exact case");
        let joined = emf_fidelity(&counts).join(" | ");
        assert!(
            joined.contains("Text in a metafile"),
            "the always-true disclosure is missing, or is the SVG's: {joined}"
        );
        assert!(
            joined.contains("nothing had to become a picture"),
            "an exact metafile must SAY it was exact, or a broken disclosure \
             looks the same as a clean one: {joined}"
        );
        assert!(
            !joined.contains("Inkscape's metafile import"),
            "nothing was rasterised, so the Inkscape caveat does not apply: \
             {joined}"
        );
    }

    /// **Every one of the five reasons a part became a bitmap is named,
    /// with its own number, in one sentence.**
    #[test]
    fn a_rasterised_metafile_names_all_five_reasons_with_their_counts() {
        let counts = EmfCounts {
            ops: 90,
            rasters_embedded: 13,
            ops_rasterised_for_alpha: 4,
            blend_modes_dropped: 2,
            gradients_rasterised: 3,
            images_embedded: 1,
            layers_rasterised: 3,
            ..EmfCounts::default()
        };
        assert!(!counts.is_exact());
        let joined = emf_fidelity(&counts).join(" | ");
        assert!(joined.contains("13 part(s)"), "the total: {joined}");
        assert!(joined.contains("4 see-through"), "{joined}");
        assert!(joined.contains("2 using a blend mode"), "{joined}");
        assert!(joined.contains("3 gradients"), "{joined}");
        assert!(
            joined.contains("1 pictures the PDF already had"),
            "{joined}"
        );
        assert!(joined.contains("3 see-through groups"), "{joined}");
        assert!(
            joined.contains("Inkscape's metafile import draws none"),
            "the engine's note: Inkscape's EMF importer draws nothing for the \
             alpha bitmaps, and for a FILE there is no SVG to fall back on: \
             {joined}"
        );
        assert!(
            !joined.contains("nothing had to become a picture"),
            "thirteen parts became pictures; it must not claim otherwise: \
             {joined}"
        );
    }

    /// **A metafile whose RECORDING was exact but whose ALPHA was not is
    /// reported as inexact.**
    #[test]
    fn an_exact_tally_does_not_make_a_rasterised_metafile_exact() {
        let counts = EmfCounts {
            rasters_embedded: 40,
            ops_rasterised_for_alpha: 40,
            ..EmfCounts::default()
        };
        assert!(
            counts.tally.is_exact(),
            "the fixture's point: the RECORDING was exact"
        );
        assert!(
            !counts.is_exact(),
            "forty parts became bitmaps; the metafile is not exact, whatever \
             the recording's tally says"
        );
        let joined = emf_fidelity(&counts).join(" | ");
        assert!(
            !joined.contains("nothing had to become a picture"),
            "this is the sentence that would have been the lie: {joined}"
        );
    }

    /// **The LibreOffice hole warning names the program AND the version,
    /// and says what will look wrong.**
    #[test]
    fn the_libreoffice_hole_warning_names_the_program_and_the_versions() {
        let counts = EmfCounts {
            nonzero_fills_multi_subpath: 5,
            ..EmfCounts::default()
        };
        assert!(!counts.is_exact());
        let joined = emf_fidelity(&counts).join(" | ");
        assert!(joined.contains("5 filled shape(s)"), "{joined}");
        assert!(joined.contains("LibreOffice 24"), "{joined}");
        assert!(
            joined.contains("holes"),
            "it must say what will LOOK wrong, not that a fill rule was \
             ignored: {joined}"
        );
        assert!(
            joined.contains("25.2") && joined.contains("Word"),
            "it must say who does NOT have the problem, or an operator cannot \
             act on it: {joined}"
        );
    }

    /// **The recording's own losses are worded exactly as the SVG's are.**
    #[test]
    fn the_shared_recording_losses_read_the_same_in_both_formats() {
        let mut tally = ExportTally::default();
        tally.overprint_approximated = 2;
        tally.nonseparable_approximated = 1;
        tally.tiling_patterns = 3;

        let svg = svg_fidelity(&tally, 0, 0).join(" | ");
        let emf = emf_fidelity(&EmfCounts {
            tally,
            ..EmfCounts::default()
        })
        .join(" | ");

        for shared in [
            "2 paint(s) set to overprint were drawn as ordinary paint.",
            "1 paint(s) using a hue, saturation, colour or luminosity blend",
            "3 tiling pattern(s) could not be written as a repeating fill.",
        ] {
            assert!(svg.contains(shared), "missing from the SVG: {shared}");
            assert!(emf.contains(shared), "missing from the EMF: {shared}");
        }
    }

    /// **The EMF receipt gives the geometry AND the picture count.**
    #[test]
    fn the_metafile_receipt_reports_what_stayed_lines_and_what_did_not() {
        let mixed = wrote_emf("C:\\d\\a.emf", 3, 400, 12);
        assert!(mixed.contains("400 drawing operations"), "{mixed}");
        assert!(
            mixed.contains("12 part(s) that had to go in as pictures"),
            "{mixed}"
        );

        let clean = wrote_emf("C:\\d\\a.emf", 3, 400, 0);
        assert!(
            clean.contains("all of them real lines"),
            "a metafile with nothing rasterised should say so rather than \
             report a zero: {clean}"
        );
        assert!(!clean.contains("pictures"), "{clean}");
    }

    /// **The two always-true outline sentences name their own formats.**
    #[test]
    fn each_vector_format_confesses_its_outlines_in_its_own_name() {
        let svg = svg_text_is_outlines();
        let emf = emf_text_is_outlines();
        assert_ne!(svg, emf);
        assert!(svg.contains("in an SVG"), "{svg}");
        assert!(emf.contains("in a metafile"), "{emf}");
        // The consequences are identical and are listed in the same order, so
        // the two read as one fact about two formats.
        for consequence in ["selected, searched", "no font travels with it"] {
            assert!(svg.contains(consequence), "{svg}");
            assert!(emf.contains(consequence), "{emf}");
        }
    }
}
