//! # `app::actions::imageexport` — what an image export IS, decided before
//! anything is written
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/imageexport.md`.

use std::path::{Path, PathBuf};

/// Which of the four writers an export goes through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Every pixel as rendered, with an alpha channel and a `pHYs` resolution.
    Png,
    /// Every pixel as rendered, composited onto an opaque colour first because
    /// the format has nowhere else to put them.
    Jpeg,
    /// The renderer's own recording, replayed as vector geometry.
    Svg,
    /// **The same recording, written as a Windows Enhanced Metafile**
    /// ([MS-EMF]) — `pdfcer_render::emf::export_emf`.
    ///
    /// # Why a fourth format, when SVG already carries vectors
    ///
    /// Because one large family of programs on this desktop cannot read the
    /// SVG. The engine's note is specific about which and why:
    /// **LibreOffice 24.x has no route to a foreign SVG clipboard entry
    /// before 25.2**, so EMF is its *only* vector import on Windows. The same
    /// is true of Office's *Paste Special ▸ Picture (Enhanced Metafile)*, of
    /// Visio, CorelDRAW and most CAD importers, and of anything that predates
    /// the SVG-on-the-clipboard convention entirely.
    ///
    /// ⇒ So this is not "SVG for Windows". It is the format that reaches a
    /// second, disjoint set of programs, and an operator who has been handed
    /// an SVG their copy of LibreOffice will not open has been handed
    /// nothing.
    ///
    /// # What it costs, which is why [`crate::text::export_image`] has a
    /// whole disclosure for it
    ///
    /// EMF has **no per-primitive alpha**. Opaque geometry goes out as real
    /// GDI path records and is exact; everything else — a translucent solid,
    /// a blend mode, a gradient, an image the PDF already carried, a
    /// transparency group — is replayed as an `EMR_ALPHABLEND` bitmap at the
    /// chosen resolution. `EmfOutcome` counts each of those five separately,
    /// and the receipt names them, because a metafile that is half geometry
    /// and half pictures looks identical to one that is all geometry until
    /// somebody scales it up.
    Emf,
}

impl ImageFormat {
    /// Every format, in the order the window offers them.
    pub const ALL: [Self; 4] = [Self::Png, Self::Jpeg, Self::Svg, Self::Emf];

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
            // ui-text-exempt: file extension. [MS-EMF] names no other one, and
            // Windows associates `.emf` with the metafile picture handler.
            Self::Emf => "emf",
        }
    }

    /// Whether this format can carry transparency at all.
    #[must_use]
    pub const fn can_be_transparent(self) -> bool {
        match self {
            Self::Png | Self::Svg | Self::Emf => true,
            Self::Jpeg => false,
        }
    }

    /// Whether the output is geometry rather than pixels.
    #[must_use]
    pub const fn is_vector(self) -> bool {
        matches!(self, Self::Svg | Self::Emf)
    }
}

/// Which pages the window is currently offering.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Impossible {
    /// **A transparent JPEG.** The format has no alpha channel, and the
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
    /// Kept as asked even when the format cannot honour it, rather than being
    /// cleared on the operator's behalf. Clearing it here is exactly the silent
    /// flatten the engine's note forbids: the plan would then describe an
    /// export nobody requested, and [`Self::impossible`] would have nothing
    /// left to refuse.
    pub transparent: bool,
    /// JPEG encoder quality, `1..=100`. Meaningless for the other two and
    /// carried anyway, so that switching format and switching back does not
    /// lose the number the operator chose.
    pub quality: u8,
    /// SVG and EMF: write text as text (`SvgText::KeepText` /
    /// `EmfText::KeepText`) rather than outlines. Ignored by the raster formats.
    pub keep_text: bool,
}

impl ImagePlan {
    /// **The combination this plan asks for and pdfcer will not perform**,
    /// or `None`.
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

/// **What an EMF export had to give up, in a shape a test can build.**
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EmfCounts {
    /// Vector ops written as real GDI path records — the part that is exact.
    pub ops: usize,
    /// `EMR_ALPHABLEND` records written, of any origin. The **total** of the
    /// five reasons below, as the engine counts it; not re-derived here,
    /// because a sum computed on this side would drift the day the engine
    /// gained a sixth reason and this struct did not.
    pub rasters_embedded: usize,
    /// Solid fills and strokes with a constant alpha below 1. EMF has no
    /// per-primitive alpha, so each became a bitmap of itself.
    pub ops_rasterised_for_alpha: usize,
    /// Ops whose blend mode EMF cannot express, drawn `Normal` inside a
    /// bitmap.
    pub blend_modes_dropped: usize,
    /// Gradients — native `<linearGradient>`/`<radialGradient>` in the SVG,
    /// bitmaps here.
    pub gradients_rasterised: usize,
    /// The file's own images.
    pub images_embedded: usize,
    /// Transparency groups (opacity, soft mask, blend) rasterised whole.
    pub layers_rasterised: usize,
    /// Strokes whose dash pattern was pre-applied to the geometry.
    pub dashed_strokes_pre_applied: usize,
    /// Nonzero-rule fills with more than one subpath. **LibreOffice 24.x
    /// ignores the fill rule**, so these are the fills it may draw with holes
    /// — and LibreOffice 24.x is the single reason this format is offered, so
    /// this counter is the one most worth saying out loud.
    pub nonzero_fills_multi_subpath: usize,
    /// The export recording's own tally — what was rasterised or approximated
    /// **before** the metafile writer saw it. Derives `Default`, so it needs
    /// no shadow of its own.
    pub tally: pdfcer_render::display_list::ExportTally,
}

impl From<&pdfcer_render::emf::EmfOutcome> for EmfCounts {
    fn from(outcome: &pdfcer_render::emf::EmfOutcome) -> Self {
        Self {
            ops: outcome.ops,
            rasters_embedded: outcome.rasters_embedded,
            ops_rasterised_for_alpha: outcome.ops_rasterised_for_alpha,
            blend_modes_dropped: outcome.blend_modes_dropped,
            gradients_rasterised: outcome.gradients_rasterised,
            images_embedded: outcome.images_embedded,
            layers_rasterised: outcome.layers_rasterised,
            dashed_strokes_pre_applied: outcome.dashed_strokes_pre_applied,
            nonzero_fills_multi_subpath: outcome.nonzero_fills_multi_subpath,
            tally: outcome.tally,
        }
    }
}

impl EmfCounts {
    /// Whether the whole page went out as real geometry.
    #[must_use]
    pub fn is_exact(&self) -> bool {
        self.rasters_embedded == 0
            && self.dashed_strokes_pre_applied == 0
            && self.nonzero_fills_multi_subpath == 0
            && self.tally.is_exact()
    }
}

/// The raster scale a resolution asks for.
#[must_use]
pub fn scale_for(dpi: f32) -> f32 {
    if dpi.is_finite() && dpi > 0.0 {
        crate::units::scale_from_dpi(f64::from(dpi)) as f32
    } else {
        // The same fallback the engine applies to a nonsense `raster_dpi`
        // (`svg.rs`), so a plan built from a corrupted preference cannot
        // produce a zero-pixel render that reports as an engine failure.
        crate::units::scale_from_dpi(300.0) as f32
    }
}

/// The pixels a page of `width_pt` × `height_pt` occupies at `dpi`.
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
            keep_text: false,
        }
    }

    /// **A transparent JPEG is refused, and it is refused BY NAME.**
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

    /// Every other combination of format and transparency is possible.
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
        assert!(plan(ImageFormat::Jpeg, false).impossible().is_none());
        assert!(plan(ImageFormat::Png, true).impossible().is_none());
        assert!(plan(ImageFormat::Svg, true).impossible().is_none());
        assert!(plan(ImageFormat::Emf, true).impossible().is_none());
    }

    /// JPEG is the only format that cannot hold transparency, and that is a
    /// fact about the formats rather than about pdfcer.
    #[test]
    fn jpeg_is_the_only_format_with_no_alpha() {
        assert!(ImageFormat::Png.can_be_transparent());
        assert!(ImageFormat::Svg.can_be_transparent());
        assert!(ImageFormat::Emf.can_be_transparent());
        assert!(!ImageFormat::Jpeg.can_be_transparent());
        // Said as a sweep too, so a fifth format cannot arrive without this
        // test having an opinion about which side of the line it is on.
        assert_eq!(
            ImageFormat::ALL
                .iter()
                .filter(|f| !f.can_be_transparent())
                .count(),
            1,
            "exactly one format has no way to store transparency; if a second \
             arrives, `Impossible` needs a second variant and `refused` a \
             second sentence"
        );
    }

    /// **The two vector formats are both vector, and neither is routed by
    /// that predicate.**
    #[test]
    fn both_vector_formats_report_as_vector_and_the_rasters_do_not() {
        assert!(ImageFormat::Svg.is_vector());
        assert!(ImageFormat::Emf.is_vector());
        assert!(!ImageFormat::Png.is_vector());
        assert!(!ImageFormat::Jpeg.is_vector());
    }

    /// **Every format has its own extension**, which is what keeps a
    /// four-way export from writing two of them to the same suggested name.
    #[test]
    fn no_two_formats_share_an_extension() {
        let mut seen: Vec<&str> = ImageFormat::ALL.iter().map(|f| f.extension()).collect();
        seen.sort_unstable();
        let count = seen.len();
        seen.dedup();
        assert_eq!(
            seen.len(),
            count,
            "two formats share an extension: {seen:?}"
        );
        assert_eq!(ImageFormat::Emf.extension(), "emf");
    }

    /// **One page keeps the name the operator typed.**
    #[test]
    fn a_single_page_export_is_named_exactly_what_was_chosen() {
        let out = output_path(Path::new("C:/d/drawing.png"), ImageFormat::Png, 0, false);
        assert_eq!(out, PathBuf::from("C:/d/drawing.png"));
    }

    /// **Several pages become a stem plus a 1-based page number.**
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

    /// **The extension comes from the format, never from what was typed.**
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
        assert_eq!(
            output_path(chosen, ImageFormat::Emf, 0, false),
            PathBuf::from("C:/d/drawing.emf")
        );
    }

    /// **A revision in the document's name survives the export.**
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

    /// **A range that names no page is `None`, not an empty export.**
    #[test]
    fn a_range_naming_no_page_is_refused_rather_than_exported_empty() {
        assert_eq!(resolve_pages(PageScope::Typed, "9", 3, 0), None);
        assert_eq!(resolve_pages(PageScope::Typed, "", 3, 0), None);
        assert_eq!(resolve_pages(PageScope::Typed, "abc", 3, 0), None);
        assert_eq!(resolve_pages(PageScope::CurrentPage, "", 0, 0), None);
        assert_eq!(resolve_pages(PageScope::AllPages, "", 0, 0), None);
    }

    /// **The raster scale is dots-per-inch over 72**, which is PDF user
    /// space's definition rather than a convention.
    #[test]
    fn the_raster_scale_is_the_resolution_over_seventy_two() {
        assert!((scale_for(72.0) - 1.0).abs() < f32::EPSILON);
        assert!((scale_for(300.0) - 300.0 / 72.0).abs() < f32::EPSILON);
        // A4 at 300 DPI: 595.276 pt × 841.89 pt.
        let (w, h) = pixel_size(595.276, 841.89, 300.0);
        assert_eq!((w, h), (2481, 3508));
    }

    /// A nonsense resolution falls back rather than producing nothing.
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
