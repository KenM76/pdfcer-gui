//! # `app::actions::imageexport` — what an image export IS, decided before
//! anything is written
//!
//! ## Why this is a module and not four fields on a `WriteAction` variant
//!
//! [`super::write::WriteAction::Dxf`] carries `pdfcer_core::export::dxf::
//! DxfOptions` — **the engine's own struct** — and its doc says why: *"it **is**
//! the value the writer takes, and rebuilding it in the apply arm would put a
//! second constructor in the path."*
//!
//! There is no engine struct to carry here, and that is a fact about the
//! feature rather than a gap in the engine. The engine offers three unrelated
//! writers — `export::encode_png`, `export::encode_jpeg`, `svg::export_svg_view`
//! — each with its own options type and its own error type, and *"which of the
//! three, over which pages, at what resolution, keeping transparency or not"*
//! is a question none of them asks. It is a **shell** question, invented by the
//! window, and the shell owns the type that answers it.
//!
//! ⇒ So this module holds the vocabulary and, more importantly, **every part of
//! the decision that can be got wrong without a window open**: which
//! combinations are impossible, which pages a scope names, what each file is
//! called, and what raster scale a resolution is. All of it is pure, all of it
//! is tested, and none of it needs an `egui::Context` to prove.
//!
//! ## ★★★ [`Impossible`] is the point of the module
//!
//! The operator asked for *"full support (including transparency where
//! supported!)"*. The parenthesis concedes that one of the three formats cannot
//! do it and asks pdfcer to be the one that says so. The engine's note is
//! imperative about the same thing:
//!
//! > **refuse a "transparent" JPEG by name in your UI, never flatten silently**
//!
//! A `bool` returned from a validity check would satisfy the letter of that and
//! lose the name. [`Impossible`] is an enum with one variant for exactly that
//! reason: `crate::text::export_image::refused` matches on it, so the day a
//! second impossibility is named the compiler asks for its sentence rather than
//! letting it inherit the first one's. **A refusal without a name is a refusal
//! that will one day be worded for the wrong reason.**
//!
//! ## ★★ Why the plan carries a RESOLVED page list
//!
//! `pages: Vec<usize>`, not a scope and a string. The window already has to
//! parse the typed range to decide whether its Export button is usable, so
//! parsing again in the apply phase would be a second call to the same parser
//! against a document that may have changed pages in between — the exact
//! staleness `super::export::dxf`'s header refuses for `PageObjects`, arriving
//! by a different door.
//!
//! `ExportDxfDialog` freezes its page index at open for the same reason and
//! states it: *"an operator who opens this on page 7 and pages away must not
//! export page 9."*
//!
//! ## Rule 15
//!
//! Nothing here reads the dimensioning model, so neither **ce dimensions** nor
//! **pdf dimensions** appear in this module. [`ImagePlan::dpi`] is a
//! *resolution*, which is a claim about pixels; it is not a *scale*, which is a
//! claim about the real world and is what `export::dxf` needs a whole window to
//! establish. **A picture has no scale to get wrong**, which is why this
//! feature is safe to offer without the calibration machinery the DXF export
//! cannot ship without.

use std::path::{Path, PathBuf};

/// Which of the three writers an export goes through.
///
/// Deliberately a shell enum rather than a string or an extension. The
/// extension is *derived* from it ([`Self::extension`]) rather than being it —
/// the reverse would mean a plan could hold `"jpg"` and `"jpeg"` as two
/// different formats and the match arms would have to keep agreeing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Every pixel as rendered, with an alpha channel and a `pHYs` resolution.
    Png,
    /// Every pixel as rendered, composited onto an opaque colour first because
    /// the format has nowhere else to put them.
    Jpeg,
    /// The renderer's own recording, replayed as vector geometry.
    Svg,
}

impl ImageFormat {
    /// Every format, in the order the window offers them.
    ///
    /// PNG first because it is the answer that is right for a drawing and
    /// wrong for nothing; JPEG second because it is the one an operator will
    /// look for by name; SVG last because it is the one whose consequences
    /// (text becomes outlines) need reading about first.
    pub const ALL: [Self; 3] = [Self::Png, Self::Jpeg, Self::Svg];

    /// The file extension, lower case, without a dot.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            // ui-text-exempt: file extensions, written to disk and matched, never
            // displayed as prose. The format's DISPLAY name is
            // `crate::text::export_image::format_name`.
            Self::Png => "png",
            // `jpg` rather than `jpeg`: it is what every camera, every Windows
            // dialog and every operator on this desktop writes, and the
            // decoder does not care. The catalog still calls the FORMAT
            // "JPEG", which is its name; this is only what the file is called.
            Self::Jpeg => "jpg",
            Self::Svg => "svg",
        }
    }

    /// Whether this format can carry transparency at all.
    ///
    /// ★ The one function in the module that is a statement about the file
    /// formats rather than about pdfcer. PNG has an alpha channel (ISO
    /// 15948 §6.1); SVG is a document with a background nobody has to paint;
    /// JPEG (ITU-T T.81) has neither and no version of it ever will.
    #[must_use]
    pub const fn can_be_transparent(self) -> bool {
        match self {
            Self::Png | Self::Svg => true,
            Self::Jpeg => false,
        }
    }

    /// Whether the output is geometry rather than pixels.
    ///
    /// Read by the window to decide whether to offer a JPEG quality control,
    /// and by the resolution hint, which means a different thing for a vector
    /// format and has to say so.
    #[must_use]
    pub const fn is_vector(self) -> bool {
        matches!(self, Self::Svg)
    }
}

/// Which pages the window is currently offering.
///
/// Lives here rather than in the dialog because [`resolve_pages`] is the
/// function worth testing and it takes one, and because a test of "what does
/// *All* mean on a three-page document" should not need a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageScope {
    /// The page on screen when the window opened, and only that one.
    CurrentPage,
    /// Every page in the document, in document order.
    AllPages,
    /// Whatever the operator typed, parsed by the print dialog's own parser.
    Typed,
}

/// A combination the operator can ask for and pdfcer will not perform.
///
/// See the module header. One variant today; the enum exists so that the
/// sentence and the condition cannot drift apart, and so that a second one
/// cannot silently borrow this one's wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Impossible {
    /// ★★★ **A transparent JPEG.** The format has no alpha channel, and the
    /// engine's note names the wrong answer explicitly: *"never flatten
    /// silently."*
    ///
    /// Flattening would produce a file that opens, looks nearly right, and
    /// carries a white rectangle the operator meets when the drawing is
    /// already inside somebody else's document.
    TransparentJpeg,
}

/// **Everything the writer needs, frozen at the moment Export was pressed.**
///
/// `Clone` and `PartialEq` because it rides `super::write::WriteAction`, which
/// is both.
#[derive(Debug, Clone, PartialEq)]
pub struct ImagePlan {
    /// Which writer.
    pub format: ImageFormat,
    /// The 0-based pages, in the order they will be written. Resolved by the
    /// window — see the module header for why it is not a scope and a string.
    pub pages: Vec<usize>,
    /// Dots per inch. The raster scale is `dpi / 72` ([`scale_for`]); for SVG
    /// this is `SvgOptions::raster_dpi` and governs only what has to be
    /// embedded as a picture.
    pub dpi: f32,
    /// Whether the page's own transparency survives.
    ///
    /// ★ Kept as asked even when the format cannot honour it, rather than being
    /// cleared on the operator's behalf. Clearing it here is exactly the silent
    /// flatten the engine's note forbids: the plan would then describe an
    /// export nobody requested, and [`Self::impossible`] would have nothing
    /// left to refuse.
    pub transparent: bool,
    /// JPEG encoder quality, `1..=100`. Meaningless for the other two and
    /// carried anyway, so that switching format and switching back does not
    /// lose the number the operator chose.
    pub quality: u8,
}

impl ImagePlan {
    /// ★★★ **The combination this plan asks for and pdfcer will not perform**,
    /// or `None`.
    ///
    /// The whole of the module header's argument lands here. It is checked in
    /// **two** places and that is deliberate rather than redundant:
    ///
    /// 1. The window, which disables the control and draws
    ///    `crate::text::export_image::jpeg_has_no_alpha` beside it, so the
    ///    combination cannot ordinarily be requested at all.
    /// 2. The writer, which refuses and writes nothing.
    ///
    /// Two mechanisms for one rule, because they fail differently. A window
    /// can be bypassed — a keymap, a restored plan, a later build with a
    /// different window — and the property that must survive all of those is
    /// *pdfcer never flattens a page onto white without saying so*. A guard
    /// only in the window would make that property a property of the window.
    #[must_use]
    pub const fn impossible(&self) -> Option<Impossible> {
        if self.transparent && !self.format.can_be_transparent() {
            return Some(Impossible::TransparentJpeg);
        }
        None
    }

    /// Whether this export writes more than one file, which changes how each
    /// one is named. See [`output_path`].
    #[must_use]
    pub fn is_multi_file(&self) -> bool {
        self.pages.len() > 1
    }
}

/// The raster scale a resolution asks for.
///
/// PDF user space is 72 units to the inch (ISO 32000-1 §8.3.2.3), so this is
/// the definition rather than a convention — the engine's own SVG writer
/// computes `raster_dpi / 72.0` in the same words.
///
/// ★ It is a function rather than an inline division at three call sites
/// because the three call sites are the raster render, the SVG options and the
/// pixel-size preview the window draws, and a preview that disagreed with the
/// render by a stray rounding would be a preview that lies about the file.
#[must_use]
pub fn scale_for(dpi: f32) -> f32 {
    if dpi.is_finite() && dpi > 0.0 {
        dpi / 72.0
    } else {
        // The same fallback the engine applies to a nonsense `raster_dpi`
        // (`svg.rs`), so a plan built from a corrupted preference cannot
        // produce a zero-pixel render that reports as an engine failure.
        300.0 / 72.0
    }
}

/// The pixels a page of `width_pt` × `height_pt` occupies at `dpi`.
///
/// Rounded the way the renderer rounds — `ceil`, so a page never loses its
/// last column — and returned as the pair the window shows and the guard
/// checks.
#[must_use]
pub fn pixel_size(width_pt: f32, height_pt: f32, dpi: f32) -> (u32, u32) {
    let scale = scale_for(dpi);
    let px = |pt: f32| -> u32 {
        let v = (pt * scale).ceil();
        if v.is_finite() && v > 0.0 {
            // `as` after a finite, positive check: the saturating cast is the
            // one behaviour wanted here, and the guard above is what makes it
            // unreachable for any page a PDF can describe.
            v.min(f64::from(u32::MAX) as f32) as u32
        } else {
            0
        }
    };
    (px(width_pt), px(height_pt))
}

/// Which pages a scope names, or `None` when a typed range names none.
///
/// ★ The typed case delegates to `crate::dialogs::print::tabs::parse_page_range`
/// — **the print dialog's parser, called rather than copied**. Three surfaces
/// already do this (`dialogs::ocr`, `dialogs::insert_pages`, the print dialog
/// itself), and that module's own doc gives the reason: a second range parser
/// is a second set of answers to *"is `1,1` two exports of page one?"* and
/// *"does `5-3` mean anything?"*, and an operator who learns the syntax in one
/// window is entitled to it in the next.
///
/// `None` is *"you typed something that names no page"*, which is the window's
/// signal to refuse the Export button and say so. An empty `Some` is not
/// produced — the parser does not return one — but is treated as `None` by the
/// caller anyway, on `dialogs::ocr`'s precedent.
#[must_use]
pub fn resolve_pages(
    scope: PageScope,
    typed: &str,
    page_count: usize,
    current_page: usize,
) -> Option<Vec<usize>> {
    match scope {
        PageScope::CurrentPage => {
            if current_page < page_count {
                Some(vec![current_page])
            } else {
                None
            }
        }
        PageScope::AllPages => {
            if page_count == 0 {
                None
            } else {
                Some((0..page_count).collect())
            }
        }
        PageScope::Typed => crate::dialogs::print::tabs::parse_page_range(typed, page_count)
            .filter(|p| !p.is_empty()),
    }
}

/// **What one page's file is called**, given the name the operator chose.
///
/// # ★★ The multi-file convention, and why it is not invented here
///
/// One page: the file is exactly what they typed. Several: the chosen name
/// becomes a **stem**, and each page gets `-p<N>` before the extension, 1-based
/// so it matches the page number in the window and on the status bar.
///
/// That is Acrobat's own behaviour for *Export ▸ Image* (it writes
/// `Base_Page_1.png` from one save dialog), and `crate::text::tool`'s standing
/// rule about conventions makes a reference application's answer the default
/// rather than a shortcut. The alternative — a folder picker plus a separate
/// name box — is two questions for one act, and it asks the second one before
/// the operator has thought about the first.
///
/// ★ The window states the pattern **before** the save dialog opens
/// (`crate::text::export_image::multi_page_naming`), because a save dialog
/// cannot say "the name you type is a stem" and an operator who did not expect
/// it would go looking for a file that is not there.
///
/// # The extension comes from the FORMAT, not from what was typed
///
/// An operator who types `drawing.pdf` into a PNG export gets `drawing.png`.
/// `export_form`'s opposite rule — the extension picks the format — is right
/// there because three formats share one picker and the operator has no other
/// way to choose. Here the format is already chosen, in a radio group, above
/// the button they pressed; letting a stray extension override it would mean a
/// window that shows one format and writes another.
///
/// # ★★★ `set_file_name` with the extension already on it, NOT `set_extension`
///
/// This looks like the long way round and it is the only correct one, and the
/// finding is worth keeping because a neighbouring module gets it wrong in a
/// comment.
///
/// `super::export::suggested_path` builds a DXF name with
/// `set_file_name(stem)` followed by `set_extension("dxf")`, and its comment
/// claims *"a document called `plan.rev2.pdf` has a stem of `plan.rev2`, and
/// appending would produce `plan.rev2.dxf` either way"*. **It does not.**
/// `Path::set_extension` replaces everything after the LAST dot, so
/// `plan.rev2` + `set_extension("dxf")` is `plan.dxf` — the revision is
/// silently deleted.
///
/// ⇒ On a CAD desktop that is not a cosmetic difference. `plan.rev2.pdf` and
/// `plan.rev3.pdf` both export to `plan.png`, and the second one **overwrites
/// the first** in a save dialog that offers to do exactly that. So the name is
/// assembled as one string and set once, and
/// [`tests::a_dotted_document_name_keeps_its_revision`] is what holds it there.
#[must_use]
pub fn output_path(chosen: &Path, format: ImageFormat, page_index: usize, multi: bool) -> PathBuf {
    let mut path = chosen.to_path_buf();
    let stem = path
        .file_stem()
        .map_or_else(|| "page".to_owned(), |s| s.to_string_lossy().into_owned());
    let name = if multi {
        // 1-based: the number in the filename is the number the operator sees
        // on the page. `saturating_add` for the same reason every page-number
        // display in this crate uses it — an index of `usize::MAX` is not
        // reachable and a panic in a filename builder would be absurd.
        format!(
            "{stem}-p{}.{}",
            page_index.saturating_add(1),
            format.extension()
        )
    } else {
        format!("{stem}.{}", format.extension())
    };
    path.set_file_name(name);
    path
}

/// Where the save dialog opens, and what it calls the file.
///
/// Beside the document and named after it, with the chosen format's extension
/// — `super::export::suggested_path`'s rule and its reason: *"a picker that
/// opens in the last-used directory of some other application is a picker that
/// makes the operator navigate back to their own project every time."*
///
/// ★ Assembled the same way [`output_path`] is, and for that function's stated
/// reason: `set_extension` on a `plan.rev2` stem eats the revision.
#[must_use]
pub fn suggested_path(document: &Path, format: ImageFormat) -> PathBuf {
    let mut path = document.to_path_buf();
    let stem = path
        .file_stem()
        .map_or_else(|| "page".to_owned(), |s| s.to_string_lossy().into_owned());
    path.set_file_name(format!("{stem}.{}", format.extension()));
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plan over `pages`, PNG, transparent, at 300.
    fn plan(format: ImageFormat, transparent: bool) -> ImagePlan {
        ImagePlan {
            format,
            pages: vec![0],
            dpi: 300.0,
            transparent,
            quality: 90,
        }
    }

    /// ★★★ **A transparent JPEG is refused, and it is refused BY NAME.**
    ///
    /// The assertion this module exists for. The engine's note: *"refuse a
    /// 'transparent' JPEG by name in your UI, never flatten silently."* The
    /// operator's own parenthesis — *"(including transparency where
    /// supported!)"* — is what makes it a requirement rather than a nicety.
    ///
    /// Note what is asserted about the plan itself: `transparent` is still
    /// `true` afterwards. A plan that quietly cleared the flag would pass a
    /// naive check and would BE the silent flatten.
    #[test]
    fn a_transparent_jpeg_is_impossible_and_says_which_combination_it_is() {
        let jpeg = plan(ImageFormat::Jpeg, true);
        assert_eq!(jpeg.impossible(), Some(Impossible::TransparentJpeg));
        assert!(
            jpeg.transparent,
            "the plan must still describe what was ASKED for; clearing the flag \
             on the operator's behalf is the silent flatten the refusal exists \
             to prevent"
        );
    }

    /// ★★ Every other combination of format and transparency is possible.
    ///
    /// Asserted as a sweep rather than three cases, so a fourth format cannot
    /// be added without this test having an opinion about it.
    #[test]
    fn every_combination_except_a_transparent_jpeg_is_allowed() {
        for format in ImageFormat::ALL {
            for transparent in [false, true] {
                let expected = transparent && !format.can_be_transparent();
                assert_eq!(
                    plan(format, transparent).impossible().is_some(),
                    expected,
                    "{format:?} transparent={transparent}"
                );
            }
        }
        assert!(!plan(ImageFormat::Jpeg, false).impossible().is_some());
        assert!(plan(ImageFormat::Png, true).impossible().is_none());
        assert!(plan(ImageFormat::Svg, true).impossible().is_none());
    }

    /// ★ JPEG is the only format that cannot hold transparency, and that is a
    /// fact about the formats rather than about pdfcer.
    #[test]
    fn jpeg_is_the_only_format_with_no_alpha() {
        assert!(ImageFormat::Png.can_be_transparent());
        assert!(ImageFormat::Svg.can_be_transparent());
        assert!(!ImageFormat::Jpeg.can_be_transparent());
    }

    /// **One page keeps the name the operator typed.**
    #[test]
    fn a_single_page_export_is_named_exactly_what_was_chosen() {
        let out = output_path(Path::new("C:/d/drawing.png"), ImageFormat::Png, 0, false);
        assert_eq!(out, PathBuf::from("C:/d/drawing.png"));
    }

    /// ★★ **Several pages become a stem plus a 1-based page number.**
    ///
    /// The number is what the operator sees on screen, not the index — an
    /// off-by-one here produces a set of files whose names disagree with the
    /// document by one, which is exactly the kind of wrongness nobody notices
    /// until they are matching drawings to a schedule.
    #[test]
    fn several_pages_become_a_stem_and_a_page_number_starting_at_one() {
        let chosen = Path::new("C:/d/drawing.png");
        assert_eq!(
            output_path(chosen, ImageFormat::Png, 0, true),
            PathBuf::from("C:/d/drawing-p1.png")
        );
        assert_eq!(
            output_path(chosen, ImageFormat::Png, 6, true),
            PathBuf::from("C:/d/drawing-p7.png")
        );
    }

    /// ★★ **The extension comes from the format, never from what was typed.**
    ///
    /// The window shows a radio group; a stray extension in the save dialog
    /// must not silently override it, or the window shows one format and the
    /// file is another.
    #[test]
    fn the_extension_is_the_formats_and_overrides_whatever_was_typed() {
        let chosen = Path::new("C:/d/drawing.pdf");
        assert_eq!(
            output_path(chosen, ImageFormat::Png, 0, false),
            PathBuf::from("C:/d/drawing.png")
        );
        assert_eq!(
            output_path(chosen, ImageFormat::Jpeg, 0, false),
            PathBuf::from("C:/d/drawing.jpg")
        );
        assert_eq!(
            output_path(chosen, ImageFormat::Svg, 0, false),
            PathBuf::from("C:/d/drawing.svg")
        );
    }

    /// ★★★ **A revision in the document's name survives the export.**
    ///
    /// `plan.rev2.pdf` is a real CAD filename shape, and this test caught a
    /// real bug when it was first run: `Path::set_extension` replaces
    /// everything after the LAST dot, so `plan.rev2` became `plan.png` and the
    /// revision was gone.
    ///
    /// ⇒ Why that matters more than tidiness: `plan.rev2.pdf` and
    /// `plan.rev3.pdf` would both suggest `plan.png`, and the second export
    /// **overwrites the first**, in a save dialog whose only warning is the
    /// generic one about a file already existing. See [`output_path`]'s header,
    /// which also records that the DXF export's own comment claims the opposite
    /// behaviour and is wrong about it.
    #[test]
    fn a_dotted_document_name_keeps_its_revision() {
        assert_eq!(
            suggested_path(Path::new("C:/d/plan.rev2.pdf"), ImageFormat::Png),
            PathBuf::from("C:/d/plan.rev2.png")
        );
        // And through the per-page namer too, which is the one that actually
        // writes files: a five-page rev2 must not collapse onto a rev3.
        assert_eq!(
            output_path(Path::new("C:/d/plan.rev2.png"), ImageFormat::Png, 0, true),
            PathBuf::from("C:/d/plan.rev2-p1.png")
        );
    }

    /// The three scopes name the pages they say they do.
    #[test]
    fn the_scopes_name_the_pages_they_claim_to() {
        assert_eq!(
            resolve_pages(PageScope::CurrentPage, "", 5, 2),
            Some(vec![2])
        );
        assert_eq!(
            resolve_pages(PageScope::AllPages, "", 3, 0),
            Some(vec![0, 1, 2])
        );
        assert_eq!(
            resolve_pages(PageScope::Typed, "1-2,4", 5, 0),
            Some(vec![0, 1, 3])
        );
    }

    /// ★ **A range that names no page is `None`, not an empty export.**
    ///
    /// The window turns this into a sentence beside the box and a disabled
    /// Export button. Collapsing it to `Some(vec![])` would let the operator
    /// answer a save dialog and receive nothing.
    #[test]
    fn a_range_naming_no_page_is_refused_rather_than_exported_empty() {
        assert_eq!(resolve_pages(PageScope::Typed, "9", 3, 0), None);
        assert_eq!(resolve_pages(PageScope::Typed, "", 3, 0), None);
        assert_eq!(resolve_pages(PageScope::Typed, "abc", 3, 0), None);
        assert_eq!(resolve_pages(PageScope::CurrentPage, "", 0, 0), None);
        assert_eq!(resolve_pages(PageScope::AllPages, "", 0, 0), None);
    }

    /// ★★ **The raster scale is dots-per-inch over 72**, which is PDF user
    /// space's definition rather than a convention.
    ///
    /// The 72 is what makes a 300 DPI export of an A4 page 2480 pixels wide
    /// instead of some other number, and it is the same arithmetic the engine
    /// does internally for SVG. A shell that used 96 here would produce files
    /// a third too large with a `pHYs` chunk that says otherwise — which is
    /// precisely the Word-places-it-wrong defect, arriving from our side.
    #[test]
    fn the_raster_scale_is_the_resolution_over_seventy_two() {
        assert!((scale_for(72.0) - 1.0).abs() < f32::EPSILON);
        assert!((scale_for(300.0) - 300.0 / 72.0).abs() < f32::EPSILON);
        // A4 at 300 DPI: 595.276 pt × 841.89 pt.
        let (w, h) = pixel_size(595.276, 841.89, 300.0);
        assert_eq!((w, h), (2481, 3508));
    }

    /// A nonsense resolution falls back rather than producing nothing.
    ///
    /// The engine does the same for `SvgOptions::raster_dpi`, and the reason
    /// is shared: a zero scale produces a zero-pixel render, which surfaces as
    /// an engine failure about a raster size the operator never typed.
    #[test]
    fn a_nonsense_resolution_falls_back_to_print_grade() {
        for bad in [0.0, -5.0, f32::NAN, f32::INFINITY] {
            assert!(
                (scale_for(bad) - 300.0 / 72.0).abs() < f32::EPSILON,
                "{bad}"
            );
        }
    }

    /// One page is one file; two are several.
    #[test]
    fn a_plan_knows_whether_it_writes_more_than_one_file() {
        let mut p = plan(ImageFormat::Png, false);
        assert!(!p.is_multi_file());
        p.pages = vec![0, 1];
        assert!(p.is_multi_file());
    }
}
