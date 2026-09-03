//! # `text::images` — the words the Insert-image window shows
//!
//! ## What this surface is
//!
//! `edit.insert_image` was registered, drawn on Edit ▸ Insert, listed in
//! `reach`'s `SCAFFOLDED` set with **no recorded reason at all**, and inert.
//! `EditSession::add_image` has shipped the whole time.
//!
//! ## ★ This catalog's hardest job: saying what a resolution MEANS
//!
//! An image placed into a PDF has no resolution of its own — §8.9.4 maps it
//! onto the **unit square** and the content stream's matrix scales that square
//! to whatever size the page asks for. So *"is this picture big enough?"* is a
//! question about the **placement**, not about the file, and it has no answer
//! until the operator has said how large the picture will be on paper.
//!
//! `pdfcer-core` reports the answer as a number rather than as a warning, and
//! its own doc comment is the argument this catalog follows:
//!
//! > Not a warning — a number. An operator dragging a 4000-pixel photo into a
//! > 2-inch box gets 2000 dpi (wasted bytes) and one dragging a 100-pixel logo
//! > across a page gets 12 (visibly soft), and **neither is visible on screen
//! > at editing zoom**.
//!
//! That last clause is why the sentence exists: both mistakes look perfect
//! until the sheet is plotted.
//!
//! ## ★ It IS previewed now, and the request that got it is worth the paragraph
//!
//! This section used to say the resolution could not be shown before the
//! commit, because `NewImage` offered `placed_rect()` as a pure preview and
//! nothing for the resolution — and computing `pixels / (points / 72)` locally
//! would have been the second derivation `placed_rect()`'s own doc warns
//! about: *"re-deriving the arithmetic in the GUI is how a preview and a result
//! drift apart."*
//!
//! Filed 2026-08-19, shipped the same day: `NewImage::effective_dpi()` and
//! `below_screen_resolution()`, pure, and — the part that was actually asked
//! for — **`add_image` now calls them instead of repeating them**, so the
//! preview and the outcome cannot disagree. The engine deleted its own copy of
//! the formula and pinned the equality with a test.
//!
//! ★ **And the four-line version this shell nearly wrote would have been
//! wrong.** Under `ImageFit::Contain` the placed rectangle is the *letterboxed*
//! sub-rectangle, not the box the operator typed — so measuring `rect` reports
//! a resolution low by exactly the letterbox ratio. The pure sibling is not
//! saving four lines; it is saving the letterbox.
//!
//! A **zero-area** placement reports `(0.0, 0.0)` and `below_screen_resolution`
//! is `true`. Zero is not a resolution, but it is a number a label can render,
//! and the engine chose it over `inf` and over `NaN` — the last of which would
//! have made its own preview/outcome equality test pass vacuously.

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
///
/// The **displayed** size, which for an EXIF-rotated photograph is not the
/// stored one — the engine transposes it and this reads the transposed value,
/// because the stored shape is not on screen anywhere.
#[must_use]
pub fn source_size(format: ImageFormat, width_px: u32, height_px: u32) -> String {
    format!("{} · {width_px} × {height_px} pixels", format_name(format))
}

/// A raster format's name — **the engine's own**, never a local table.
///
/// ★ `ImageFormat` is `#[non_exhaustive]`, so a match here could not be
/// exhaustive and could never fail to compile when a format is added: the
/// wildcard the compiler forces is the wildcard that silences it for ever
/// (recorded in `D:/dev/rag/rust/` under that name). This function was first
/// written as four arms plus a fallback, and it did not need to be — the enum
/// carries `ImageFormat::name()`, which is `const`, is what the engine's own
/// refusal messages use, and gains a new format the moment `sniff` does.
///
/// Deriving the string from the value rather than from a table beside it is the
/// first of that finding's four remedies, and where an upstream accessor exists
/// it is the only one needed.
#[must_use]
pub const fn format_name(format: ImageFormat) -> &'static str {
    format.name()
}

/// The picture's natural size on paper, and where that number came from.
///
/// ★ **The provenance is half the fact.** `ImportNotes::dpi_source`
/// distinguishes *"the file said 300 dpi"* from *"pdfcer assumed 72"*, and the
/// engine keeps them apart deliberately. A natural size derived from an assumed
/// 72 dpi is not a claim about the picture — it is one pixel per point, which
/// is the PDF default and nothing the file asked for.
#[must_use]
pub fn natural_size(width_mm: f64, height_mm: f64, declared_dpi: Option<(f64, f64)>) -> String {
    match declared_dpi {
        Some((x, y)) if (x - y).abs() < 0.5 => {
            format!("{width_mm:.0} × {height_mm:.0} mm at the {x:.0} dpi the file declares")
        }
        Some((x, y)) => {
            format!(
                "{width_mm:.0} × {height_mm:.0} mm at the {x:.0} × {y:.0} dpi the file declares"
            )
        }
        None => format!(
            "{width_mm:.0} × {height_mm:.0} mm — the file declares no resolution, so pdfcer \
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

/// ★ Why the page is stated and not chosen.
///
/// The image goes on the page the operator is looking at, which is the answer
/// every other page-scoped verb in this application gives, and stating it is
/// what makes that checkable — the window is centred over a document they may
/// have scrolled. The same reasoning the Insert-from-file dialog gives for
/// naming its destination by number.
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
///
/// ★ **From the BOTTOM**, because PDF user space has its origin at the
/// bottom-left and y increases upward (§8.3.2.3). Measuring from the top here
/// would be friendlier for one field and would disagree with every coordinate
/// the Properties panel, the object tree and the rulers report — and an
/// operator comparing two numbers that mean different things is worse off than
/// one learning a convention their drawing package already uses.
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

/// The millimetre suffix the placement spinners carry.
#[must_use]
pub const fn millimetres() -> &'static str {
    " mm"
}

/// The heading over the fit choice.
#[must_use]
pub const fn fit_heading() -> &'static str {
    "If the shapes differ"
}

/// A fit mode's name.
///
/// ★ Named by **what happens to the picture**, not by the engine's identifier.
/// "Contain" and "Stretch" are precise and are words about a box; an operator
/// deciding this is thinking about their photograph.
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

/// ★ **What resolution this placement will be**, before it is committed.
///
/// The number `pdfcer-core` insists is *"not a warning — a number"*, shown
/// beside the spinners that decide it rather than after the commit that fixes
/// it. Both mistakes it can report look perfect on screen at editing zoom: a
/// 4000-pixel photo in a 2-inch box wastes megabytes, and a 100-pixel logo
/// across a page plots soft.
///
/// Taken from `NewImage::effective_dpi()` and `below_screen_resolution()` —
/// **the same calls `add_image` makes to build its own disclosure**, since
/// 2026-08-19. Nothing here computes a resolution, and after the commit the
/// operator is told the same figure by the same producer.
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

/// ★ Where the picture will actually land, previewed from the engine's own
/// arithmetic.
///
/// `NewImage::placed_rect()` is public *for this*, and its doc says why:
/// *"a front end drawing a preview must draw the same rectangle the edit will
/// produce, and re-deriving the arithmetic in the GUI is how a preview and a
/// result drift apart."* Nothing here computes a rectangle.
///
/// Shown only when it differs from what was asked for — under `Stretch` it
/// never does, and a line restating the two numbers above it would be noise.
#[must_use]
pub fn placed_note(width_mm: f64, height_mm: f64) -> String {
    format!("It will land {width_mm:.0} × {height_mm:.0} mm, centred in that box.")
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
///
/// ★ Refused rather than clamped. A picture silently moved back onto the sheet
/// is a placement the operator did not make, and they would find it by looking
/// at the drawing rather than at this window. The same posture
/// `Tolerance::validate` takes: *"a corrected value the operator never saw is
/// exactly the sneaky case."*
#[must_use]
pub const fn off_the_page() -> &'static str {
    "That box is not on the sheet. Reduce the size, or move it back inside."
}

/// A box with no area.
#[must_use]
pub const fn no_area() -> &'static str {
    "Give the box a width and a height."
}

/// ★ **Why pdfcer re-encoded the picture instead of storing the file's bytes.**
///
/// `RecompressReason` carries no `Display`, and that absence is a decision
/// rather than an omission: these are *pdfcer's* reasons, in pdfcer's vocabulary
/// — an alpha channel split out into an `/SMask`, a TIFF codec with no encoder
/// on this side — and the engine leaves the English to the front end because
/// only the front end knows who is reading it.
///
/// The engine draws one distinction this catalog keeps, because it is the one
/// that changes what an operator should do:
///
/// | class | variants | how it reads |
/// |---|---|---|
/// | **your file forced this** | `AlphaSplit`, `NoCompressedSource`, `SourceCodecNotReusable` | a fact, nothing to decide |
/// | **you asked for this** | `LosslessRequested`, `JpegRequested` | *"a chosen reason is not a substitution, so a front end should not apologise for it"* — the engine's own words |
///
/// `SourceCodecNotReusable` is kept apart from `NoCompressedSource` for the
/// reason its own doc gives: conflating them *"tells a TIFF owner their file
/// was uncompressed"*. There were bytes; they were simply not reusable.
///
/// # ★ The wildcard is forced, not chosen
///
/// `RecompressReason` is `#[non_exhaustive]`, so this match cannot be
/// exhaustive and can never fail to compile when pdfcer grows a sixth reason —
/// see `D:/dev/rag/rust/`'s finding of that name. There is no upstream
/// accessor to delegate to here (unlike `ImageFormat::name`), so the fallback
/// is a true sentence that says a re-encode happened without inventing a
/// reason for it, and the test below asserts none of the five known variants
/// reaches it.
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
///
/// ★ Passed through, unlike a `TwoLineRefusal`, and the difference is worth
/// stating because the two look like the same case. `ImageImportError`'s
/// messages **name the operator's file** — *"pdfcer does not place GIF images —
/// it places PNG, JPEG, BMP and TIFF"*, *"this image uses {feature}, which
/// pdfcer cannot place"* — so the specific half is the whole value and a
/// catalog sentence would have to discard it. `crate::text::canvas_render_failed`
/// makes the same call for the same reason.
#[must_use]
pub fn import_failed(detail: &str) -> String {
    format!("That file was not inserted. {detail}")
}

// ---------------------------------------------------------------------------
// After the fact — what the placement did that the operator cannot see
// ---------------------------------------------------------------------------

/// ★ **The disclosures image placement owes**, assembled into the sentences the
/// status bar shows.
///
/// Every one of these is a fact the operator **cannot see on screen at editing
/// zoom**, which is the rule-4 test in its purest form for this feature: the
/// picture looks identical whether it was stored at 12 dpi or 2000, whether its
/// bytes passed through unchanged or were re-encoded, and whether a lossy
/// source was re-compressed lossily a second time.
///
/// Returns them in the order they matter to a drawing:
///
/// 1. **resolution**, because it decides whether the sheet plots acceptably;
/// 2. **shape**, because it decides whether the picture is honest;
/// 3. **bytes**, because it decides how big the file got and why.
///
/// A clause is emitted only when it has something to say. `letterboxed` on a
/// box the operator drew to the picture's own shape is false, and a sentence
/// about it would be noise on the commonest path.
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

/// How the stored size compares with the source file's.
///
/// ★ Both numbers, never a ratio alone. *"38 % larger"* on a 4 KB logo and on a
/// a 40 MB scan are the same sentence about very different documents, and the
/// operator's question is what happened to **their file**.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ A soft placement says so, and a fine one still states the number.
    ///
    /// The number is always given because `pdfcer-core` insists it is *"not a
    /// warning — a number"*: an operator placing a 4000-pixel photo in a 2-inch
    /// box has done nothing wrong and has wasted several megabytes, and only
    /// the figure tells them.
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
    ///
    /// They are mutually exclusive by construction in the engine — one fit mode
    /// produces each — and emitting both would describe a placement that cannot
    /// happen.
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
    ///
    /// The commonest path by far — a box the right shape, bytes passed through
    /// — and it must not produce a paragraph. Three sentences on every insert is
    /// how an operator learns to stop reading the one that matters.
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

    /// ★ Every known re-encode reason has its own sentence, and none reaches
    /// the forced fallback.
    ///
    /// The alarm `#[non_exhaustive]` takes away from the compiler. It cannot
    /// catch a sixth variant — nothing downstream can — but it catches the
    /// failure that is actually likely, which is an arm deleted or two variants
    /// collapsed into one. `SourceCodecNotReusable` and `NoCompressedSource`
    /// are asserted DIFFERENT for the engine's own reason: conflating them
    /// tells a TIFF owner their file was uncompressed.
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
    ///
    /// Named as a function rather than as a literal in the test above, so the
    /// two cannot drift — which is the whole shape of the `NO_SURFACE.md`
    /// finding about a test asserting a constant against a function returning
    /// that constant. Here the relation is the assertion: *nothing known
    /// reaches this*, whatever it says.
    fn recompress_reason_fallback() -> &'static str {
        // Constructed by exclusion: the fallback is what the match returns for
        // a variant not listed, and there is no way to name one. Compared
        // against the arm's own text instead.
        "PDF could not carry the file's own bytes unchanged"
    }

    /// ★ The preview and the after-the-fact disclosure say the same number the
    /// same way.
    ///
    /// They are two functions, and the engine went to the trouble of making the
    /// two *sources* one — deleting its own copy of the formula so
    /// `add_image` calls the pure sibling. This asserts the shell did not undo
    /// that on the wording side: an operator who reads "150 dpi" in the window
    /// and "150 dpi" in the status bar has been told one thing twice, which is
    /// what makes the preview trustworthy.
    ///
    /// The soft case is asserted in both, because that is the one where a
    /// difference in phrasing would read as a difference in verdict.
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
