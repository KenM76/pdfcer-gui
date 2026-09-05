//! # `text::export_image` — the words the Export-image window shows, and the
//! sentences an image export owes afterwards
//!
//! ## Why this is a NEW catalog module rather than lines added to an old one
//!
//! Two reasons, and the second is the load-bearing one.
//!
//! The small one is R2: [`crate::text::commands`] sits at the ceiling and
//! [`crate::text::status`] is close to it, so a feature's worth of copy had
//! nowhere to go in either.
//!
//! The real one is that this catalog has a **subject**, and it is not "another
//! export". `crate::text::export_dxf` is about a *scale somebody can defend*;
//! `crate::text::export_form` is about a *spreadsheet that must not execute the
//! values it was handed*. This one is about **what a picture format can and
//! cannot hold**, and every sentence in it is a consequence of that: JPEG has
//! no alpha channel, PNG has one and a `pHYs` chunk that decides how large the
//! page lands in Word, and SVG has geometry but no text.
//!
//! ## ★★★ The sentence this whole catalog exists for
//!
//! The operator, 2026-09-03, verbatim:
//!
//! > *"note that there had better be full support (including transparency
//! > where supported!)"*
//!
//! **The parenthesis is the instruction.** *Where supported* is an admission
//! that one of the four formats cannot do it, and what it asks for is that
//! pdfcer say which — not that pdfcer quietly pick a background and hand back
//! a file that looks right on screen and prints with a white box round the
//! drawing.
//!
//! The engine's note says the same thing in the imperative: *"refuse a
//! 'transparent' JPEG **by name** in your UI, never flatten silently."* So
//! [`jpeg_has_no_alpha`] is drawn beside the control that would offer the
//! impossible combination, and [`transparent_jpeg_refused`] is what an export
//! that somehow reached the writer with both set says instead of writing
//! anything. Two sentences for one rule is deliberate: the first is a
//! **prevention** and lives in the window, the second is a **refusal** and
//! lives in the receipt, and a build that lost the window would still not
//! flatten anybody's page without saying so.
//!
//! ## ★★ Rule 4's shape: the disclosure is off-canvas, after the fact
//!
//! Nothing here is drawn on the page or on the preview. Everything in the
//! second half of this file is a line for the disclosure slot —
//! `app::actions::record_edit_disclosure` — which is where an export's
//! consequences are already reported (`export_dxf::exported`,
//! `export_form::neutralised`). An operator sees the picture they asked for,
//! and then reads what could not be expressed exactly.
//!
//! ★ **The one that is easiest to leave out is [`svg_text_is_outlines`], and it
//! is the one most owed.** `pdfcer-render`'s SVG writer says it in its own
//! header — *"Text is glyph outlines"* — and the consequence is invisible in
//! every way an operator would check: the SVG opens in Inkscape, the words are
//! there, they are the right shape, and they cannot be selected, searched,
//! re-typed or re-flowed, and they do not carry the font. That is exactly the
//! inference Rule 4 exists for: *a change pdfcer made that the operator cannot
//! see and would not guess.*
//!
//! ## Rule 15
//!
//! No sentence here says "dimension". Nothing in an image export reads the
//! **ce dimensions** the operator has drawn or the **pdf dimensions** a CAD
//! exporter left on the page — a raster is what the renderer paints and an SVG
//! is what the recorder recorded, and neither consults the dimensioning model
//! at all. That is a real difference from `export_dxf`, whose whole window is
//! arranged round the ce dimensions, and it is why this catalog offers no
//! scale control: **a picture has no scale to get wrong.** It has a
//! resolution, which is a different claim, and [`dpi_hint`] says which.

use pdfcer_render::display_list::ExportTally;

use crate::app::actions::imageexport::{EmfCounts, ImageFormat, Impossible};

// ===========================================================================
// THE WINDOW
// ===========================================================================

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Export image"
}

/// The paragraph under the title.
///
/// Says what leaves and what does not, in the order an operator meets it: a
/// picture of the page, one file per page, and — the part they cannot check
/// afterwards — that pdfcer will say what it could not carry exactly.
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
        // ★ The acronym AND what it stands for, unlike the other three, and
        // that asymmetry is deliberate. PNG, JPEG and SVG are names an
        // operator has met; "EMF" is one almost nobody has, and a radio
        // reading only "EMF" is a radio nobody presses. The long form is what
        // the Windows *Paste Special* list itself says, which is where this
        // operator will have seen it if they have seen it anywhere.
        ImageFormat::Emf => "EMF (Windows metafile)",
    }
}

/// ★ What each format is FOR, in one line under its radio.
///
/// Not decoration. The operator's own examples were *"copy and paste vector
/// graphics into word or inkscape"*, and the choice between these three is
/// exactly the choice that decides whether that works — so the line says what
/// the receiving program will be able to do with the file, which is the
/// question being asked, rather than restating the acronym.
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
        // ★★ Both vector hints name PROGRAMS rather than properties, because
        // the choice between these two is not a choice about fidelity — it is
        // a choice about which program is going to open the file, and an
        // operator who has just been told "SVG keeps lines as lines" has no
        // way to know that their copy of LibreOffice will refuse it.
        ImageFormat::Svg => {
            "Lines stay lines, so it can be scaled up without going blocky and \
             edited in Inkscape or Illustrator. Text arrives as outlines \
             rather than as words."
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

/// ★ The typed range does not name any page in this document.
///
/// Drawn where the range is typed, not saved for the receipt: this is a
/// mistake the operator can fix in the box in front of them, and a refusal
/// that arrives after a save dialog has been answered is a refusal that wasted
/// their time.
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

/// ★★ What the number means, and it means two different things.
///
/// For PNG and JPEG it decides the size of the picture and is written **into**
/// the file, which is the whole of why Word places the result correctly. For
/// SVG the geometry is exact at any value and the number only governs anything
/// that had to be embedded as a picture — a shading with no gradient form, a
/// soft mask, an image the PDF already carried.
///
/// Saying the same thing for both would be wrong in one of the two cases, and
/// the case it would be wrong in is the one where an operator wonders why
/// raising the number did nothing to the sharpness of their lines.
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
        // ★ Worded separately from SVG rather than sharing its arm, because
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

/// ★ The pixel size this resolution will produce, before anything is written.
///
/// A resolution is an abstraction; a number of pixels is the thing that
/// actually lands on disk and the thing that can be refused. Shown live so the
/// operator who types 1200 finds out here rather than from a failure.
#[must_use]
pub fn dpi_pixels(width: u32, height: u32) -> String {
    format!("The largest page comes out {width} by {height} pixels.")
}

/// ★★ The requested resolution cannot be rendered at all.
///
/// `pdfcer_render::MAX_PIXMAP_EDGE` is 16,384 pixels and a render past it is
/// refused by the engine. Said here, in the window, with the number that would
/// have to come down — because the alternative is a save dialog followed by a
/// failure the operator cannot act on.
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

/// ★★★ **The refusal, drawn beside the control that would offer the
/// impossible combination.**
///
/// The operator asked for *"full support (including transparency where
/// supported!)"*, and the engine's note is imperative about the other half:
/// *"refuse a 'transparent' JPEG by name in your UI, never flatten silently."*
///
/// So the checkbox is drawn for JPEG and drawn **disabled**, with this under
/// it. Three separable claims, and each is there for a reason:
///
/// 1. **It is the format, not pdfcer.** *"has no way to store one"* — no
///    future version of pdfcer changes that, and a sentence that read like a
///    gap somebody might fix would be a promise.
/// 2. **What would happen instead**, said before it happens: a solid
///    background. This is the fact an operator otherwise discovers in print.
/// 3. **What to do about it** — the two formats that can, by name.
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

/// ★ What quality costs, in the terms this operator's files are in.
///
/// A CAD drawing is line art, and line art is the content JPEG treats worst.
/// The sentence names that rather than talking about compression ratios,
/// because the decision being made is *"can I send this to somebody"* rather
/// than *"how does the codec work"*.
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

/// ★ How several pages are named, said BEFORE the save dialog opens.
///
/// One file per page, and the operator names one of them. Every reference
/// application does this — Acrobat's *Export ▸ Image* asks for a base name and
/// writes `Base_Page_1.png` — but "the name you type is a stem" is not
/// something a save dialog can say, so the window says it here, while the
/// operator can still change their mind about the range.
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
///
/// Names the file, the page, the pixels and the resolution **that is written
/// into the file**. The last of those is the one the engine's note singles
/// out: *"without `pHYs` Word places a 300 DPI page four times too large"*, so
/// the receipt states that the number went in rather than leaving the operator
/// to discover on paste whether it did.
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
///
/// ★ Its own line rather than [`wrote_svg`]'s, and the difference is the
/// second number. An SVG's `ops` count is the whole file; a metafile's is
/// only the part that stayed geometry, and the balance became bitmaps. A
/// receipt that reported `ops` alone would describe a file that is half
/// pictures in exactly the same words as one that is all lines.
///
/// ⇒ So both are named, side by side, and the reader can see the ratio
/// without doing arithmetic. `emf_fidelity` then says *what* the pictures
/// were.
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

/// ★★★ **A transparent JPEG was requested and NOTHING was written.**
///
/// The second half of the rule [`jpeg_has_no_alpha`] states. That one prevents
/// the combination in the window; this one is what the writer says if it is
/// ever handed the combination anyway — a keyboard route, a restored plan, a
/// later build with a different window.
///
/// ★ It refuses rather than flattening, and that is the whole point. Flattening
/// would produce a file the operator can open, that looks almost right, and
/// whose white background they meet when it is already in somebody else's
/// document. A refusal costs them one press and tells them which press to make
/// instead.
#[must_use]
pub const fn transparent_jpeg_refused() -> &'static str {
    "Nothing was written. A transparent JPEG does not exist — the format has \
     no way to store transparency — and pdfcer will not quietly put your page \
     on a white background instead. Export as PNG or SVG to keep it clear, or \
     clear Keep transparency to have the white on purpose."
}

/// Turn an impossible combination into the sentence that refuses it.
///
/// One arm today. It is a `match` rather than a bare call so that the day a
/// second impossibility is named in
/// [`crate::app::actions::imageexport::Impossible`], the compiler asks for its
/// sentence rather than letting it inherit this one.
#[must_use]
pub const fn refused(why: Impossible) -> &'static str {
    match why {
        Impossible::TransparentJpeg => transparent_jpeg_refused(),
    }
}

/// ★★★ **What the SVG could not express exactly** — Rule 4's whole content.
///
/// `ExportTally` is the engine's own count of what a recording had to
/// approximate, and every field below is a fact the operator cannot get any
/// other way: the file opens, the picture looks right, and the thing that was
/// approximated is invisible until somebody prints it on a press or edits it
/// in Inkscape.
///
/// # ★★ Four decisions in this function, each of which could have been made
/// wrongly and quietly
///
/// 1. **`shadings_as_gradients` is not a shortfall and is not reported as
///    one.** The engine's own doc calls it *"a count of fidelity, not
///    shortfall"*, and `ExportTally::is_exact` deliberately zeroes it before
///    comparing. A gradient that went out as a real `<linearGradient>` is the
///    good outcome; listing it beside the losses would teach the operator to
///    skim the list, which is how a disclosure stops being read.
/// 2. **`soft_masks_kept` is likewise fidelity** — the mask survived as a
///    `<mask>` element — but it is still mentioned, because `is_exact()`
///    counts it as inexact and because Word's SVG importer is where it will
///    fail. The sentence says *kept*, and then says *where it will not be
///    honoured*, which is a different and more useful claim than "lost".
/// 3. **The always-true one is stated unconditionally.** Text is glyph
///    outlines in every SVG this writer produces, so there is no counter for
///    it and there never will be; a disclosure that only listed *counted*
///    things would silently omit the largest single surprise in the format.
/// 4. **An exact page says so, in one line.** The alternative — saying nothing
///    — is indistinguishable from a build where the disclosure broke.
///
/// `dashed` and `blends` are passed separately because they live on
/// `SvgOutcome` rather than on the tally; the engine's note put them in the
/// tally's paragraph and the source does not, and the source wins.
#[must_use]
pub fn svg_fidelity(tally: &ExportTally, dashed: usize, blends: usize) -> Vec<String> {
    // Always, and first: the one nothing counts. See clause 3 above.
    let mut out = vec![svg_text_is_outlines().to_owned()];

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

/// ★★★ The one nobody counts, and the one most owed.
///
/// `pdfcer-render`'s SVG writer states it in its own header — *"Text is glyph
/// outlines. That is what 'renders identically everywhere' costs"* — and the
/// consequence is invisible to every check an operator would make. The file
/// opens. The words are there. They are the right shape and in the right
/// place. And they cannot be selected, searched, corrected or re-flowed, and
/// the font did not travel.
///
/// So the sentence names all four consequences rather than the mechanism: an
/// operator who reads "text is converted to paths" learns nothing they can
/// act on, and an operator who reads "you will not be able to edit the words"
/// knows immediately whether it matters to them.
#[must_use]
pub const fn svg_text_is_outlines() -> &'static str {
    "Text in an SVG is written as outlines, not as words. It will look right \
     in any program, and it cannot be selected, searched or re-typed there, \
     and no font travels with it."
}

/// ★★★ **What the metafile could not express exactly** — Rule 4's content for
/// EMF, and it is a longer confession than the SVG's.
///
/// The engine's note names the obligation in one clause: *"`EmfOutcome`
/// carries what became a bitmap (translucent solids, blend modes, gradients,
/// images, groups) — **disclose those**"*. This is that.
///
/// # ★★ Why this is not `svg_fidelity` with a different noun
///
/// The two formats fail at different places, and folding them into one
/// function would mean each page's disclosure was worded for the other one.
///
/// | | SVG | EMF |
/// |---|---|---|
/// | a translucent solid | `fill-opacity`, exact | **a bitmap** |
/// | a gradient | `<linearGradient>`, exact | **a bitmap** |
/// | an image the PDF carried | embedded, exact | a bitmap (same pixels, but counted) |
/// | a blend mode | `mix-blend-mode`, Inkscape honours it | **dropped, and the element rasterised** |
/// | a nonzero fill with several subpaths | `fill-rule`, exact | exact — but **LibreOffice 24.x ignores it** |
///
/// The right-hand column is why an operator would choose EMF and what it
/// costs them, and none of it is visible in the file: a metafile that is half
/// `EMR_ALPHABLEND` opens, plays, and looks correct at 100%.
///
/// # ★ The five reasons are given as a breakdown of one total, not five
/// sentences
///
/// `rasters_embedded` is the engine's own sum and the number that matters —
/// *how much of my drawing stopped being lines*. The five causes are the
/// diagnosis, and a reader who does not need it should be able to stop after
/// the first clause. Five separate sentences would bury the total in the
/// middle of them.
///
/// # ★★★ The LibreOffice clause is unconditional on its counter and is NOT
/// folded in with the rest
///
/// `nonzero_fills_multi_subpath` is the only entry here that is not a loss at
/// all — the metafile records the fill rule correctly. It is a warning about
/// **one named reader**, which is the very reader this format exists to
/// serve, and the failure it predicts (a solid shape drawn with holes in it)
/// is the kind an operator blames on pdfcer. So it gets its own sentence, it
/// names the program and the version, and it says what will look wrong.
#[must_use]
pub fn emf_fidelity(counts: &EmfCounts) -> Vec<String> {
    // Always, and first: the one nothing counts. `svg_fidelity`'s clause 3,
    // and the same reasoning — a disclosure that listed only *counted* things
    // would go silent on the largest surprise the format holds.
    let mut out = vec![emf_text_is_outlines().to_owned()];

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
///
/// ★ Worded separately rather than sharing the SVG's sentence, because the
/// SVG's names the format — *"Text in an SVG"* — and a receipt that told an
/// operator about their SVG after they exported an EMF is a receipt about
/// somebody else's file. The consequences are identical and are listed in the
/// same order, deliberately, so the two read as the same fact about two
/// formats rather than as two different problems.
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

    /// ★★★ **Asking for a transparent JPEG is refused BY NAME.**
    ///
    /// The assertion this catalog exists for, and the operator's own
    /// parenthesis — *"(including transparency where supported!)"* — is what
    /// makes it a requirement rather than a nicety. The engine's note is
    /// imperative: *"refuse a 'transparent' JPEG by name in your UI, never
    /// flatten silently."*
    ///
    /// Both halves are asserted, because they are two different mechanisms:
    /// the window's prevention and the writer's refusal. A build that lost one
    /// must still not flatten anybody's page in silence.
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

    /// ★★ **A PNG's receipt states the resolution that went INTO the file.**
    ///
    /// The engine's note names the defect: *"without `pHYs` Word places a 300
    /// DPI page four times too large."* An operator who is told only "300 DPI"
    /// learns what was asked for; one who is told it is recorded in the file
    /// learns that the paste will be the right size.
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

    /// ★★★ **An exact SVG reports nothing lost — but still says text became
    /// outlines.**
    ///
    /// The trap this asserts against is a disclosure that only lists counted
    /// things. Text-as-outlines has no counter and never will, so a tally-only
    /// implementation would go completely silent on the single largest
    /// surprise the format has.
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

    /// ★★ **An inexact SVG names each thing it lost.**
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

    /// ★ **A native gradient is fidelity and is not confessed as a loss.**
    ///
    /// `ExportTally::is_exact` zeroes `shadings_as_gradients` before comparing,
    /// on the engine's own reasoning — *"a native gradient is exact; it is
    /// counted, not confessed"* — and this catalog must agree with it. A
    /// disclosure that listed the good outcome beside the bad ones would train
    /// the operator to skim, which is how a disclosure stops working.
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

    /// ★ The resolution hint says a DIFFERENT thing for SVG, because the
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
        // ★★ EMF gets its OWN vector sentence rather than sharing SVG's. The
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

    /// ★★★ **Every format has a name, a hint and a resolution hint, and no
    /// two formats share a name.**
    ///
    /// The sweep that makes adding a fifth format safe. A `match` arm that
    /// fell through to another format's wording would compile, would draw a
    /// radio, and would describe somebody else's file — and nothing but this
    /// test would notice.
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

    /// ★★ **The EMF radio spells out what the acronym means.**
    ///
    /// Unlike PNG, JPEG and SVG, "EMF" is a name almost no operator has met,
    /// and a radio reading only "EMF" is a radio nobody presses. The long form
    /// is the one the Windows *Paste Special* list itself uses.
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

    /// ★★★ **An exact metafile reports nothing lost — and still says text
    /// became outlines.**
    ///
    /// `svg_fidelity`'s clause 3, restated for EMF because the always-true
    /// loss is the same and the sentence is not: the SVG's names the SVG, and
    /// a receipt about somebody else's format is a receipt about somebody
    /// else's file.
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

    /// ★★★ **Every one of the five reasons a part became a bitmap is named,
    /// with its own number, in one sentence.**
    ///
    /// The engine's note is imperative about this: *"`EmfOutcome` carries what
    /// became a bitmap (translucent solids, blend modes, gradients, images,
    /// groups) — **disclose those**"*. A disclosure that gave only the total
    /// would tell an operator that thirteen things went wrong and nothing
    /// about which knob to turn.
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

    /// ★★★ **A metafile whose RECORDING was exact but whose ALPHA was not is
    /// reported as inexact.**
    ///
    /// The trap `EmfCounts::is_exact` exists for. `ExportTally` describes the
    /// recording, which is shared with the SVG writer and knows nothing about
    /// EMF's missing per-primitive alpha — so a page that recorded perfectly
    /// and then had forty translucent rectangles turned into bitmaps has an
    /// **exact tally** and an **inexact metafile**.
    ///
    /// ⇒ Asking the tally alone is how a disclosure comes to say *"nothing had
    /// to be approximated"* over a file that is half pictures.
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

    /// ★★★ **The LibreOffice hole warning names the program AND the version,
    /// and says what will look wrong.**
    ///
    /// `nonzero_fills_multi_subpath` is the only entry in the EMF disclosure
    /// that is not a loss — the metafile records the fill rule correctly. It
    /// is a warning about one named reader, and that reader is the entire
    /// reason this format is offered. The failure it predicts (a solid shape
    /// drawn with holes) is the kind an operator blames on pdfcer.
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

    /// ★★ **The recording's own losses are worded exactly as the SVG's are.**
    ///
    /// They are the same losses arriving by the same route — the shared export
    /// recording — and an operator who has read one receipt should recognise
    /// the other. Divergent wording for an identical fact reads as two
    /// different problems.
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

    /// ★★ **The EMF receipt gives the geometry AND the picture count.**
    ///
    /// An SVG's `ops` is the whole file; a metafile's is only the part that
    /// stayed lines. A receipt reporting `ops` alone would describe a file
    /// that is half `EMR_ALPHABLEND` in exactly the same words as one that is
    /// all geometry.
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

    /// ★ **The two always-true outline sentences name their own formats.**
    ///
    /// They describe the same loss and must not be the same string: a receipt
    /// that told an operator about "an SVG" after they exported a metafile is
    /// a receipt about a file they do not have.
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
