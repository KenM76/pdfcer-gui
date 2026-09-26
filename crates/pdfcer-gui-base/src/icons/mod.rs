//! # icons — the SVG-path → tiny-skia → egui-texture icon pipeline
//!
//! Turns a set of hand-authored outline SVGs into tinted, DPI-correct egui
//! images for the ribbon, the menus and any hand-drawn control. Nothing in
//! this module knows what any icon *means* — the meaning lives in
//! [`Icon`]'s variant names and their doc comments; this module knows how to
//! turn a path `d` attribute into pixels, how not to do it twice, and how to
//! report the one thing it cannot draw.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/icons/mod.md`.

/// The optional two-colour icon set: which part of which glyph takes which
/// accent, and the switch.
pub mod accent;
pub mod assets;
pub mod cache;
pub mod catalog;
pub mod paint;
pub mod svg;

pub use cache::IconCache;
pub use catalog::Icon;
pub use paint::{paint_icon, paint_missing_mark, paint_ribbon_icon};
pub use svg::{IconArt, IconError};

use cache::with_cache;

/// Icon edge length in **logical points** for a control this crate draws
/// itself.
pub const ICON_PTS: f32 = 16.0;

/// How heavily an icon's outline is stroked.
///
/// See this module's header, "Weight": this is the non-colour selected-state
/// cue that replaces bolding a text label on controls that have no text.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum IconWeight {
    /// The asset's authored stroke width — every ordinary control.
    #[default]
    Regular,
    /// Stroke width scaled up — selected/active toggles only.
    Bold,
}

/// Build the drawable image for `icon`, tinted `tint`, at [`ICON_PTS`]
/// logical points.
pub fn image_tinted(
    ui: &egui::Ui,
    icon: Icon,
    weight: IconWeight,
    tint: egui::Color32,
) -> egui::Image<'static> {
    let ctx = ui.ctx();
    let px = (ICON_PTS * ctx.pixels_per_point()).round().max(1.0) as u32;
    let baked = accent::baked(ctx, icon, tint, ui.is_enabled());
    let handle = with_cache(|cache| cache.texture_baked(ctx, icon, px, weight, baked));
    // A baked glyph carries its colours; the tint only applies the alpha.
    // NOT A THEME COLOUR: an alpha multiplier, not a colour.
    let tint = if baked.is_some() {
        egui::Color32::from_white_alpha(tint.a())
    } else {
        tint
    };
    let sized = egui::load::SizedTexture::new(handle.id(), egui::vec2(ICON_PTS, ICON_PTS));
    egui::Image::from_texture(sized)
        .fit_to_exact_size(egui::vec2(ICON_PTS, ICON_PTS))
        .tint(tint)
}

/// An icon in the ordinary (non-selected) state.
pub fn image(ui: &egui::Ui, icon: Icon) -> egui::Image<'static> {
    image_tinted(ui, icon, IconWeight::Regular, ui.visuals().text_color())
}

/// An icon in the selected/active state of a toggle.
pub fn selected_image(ui: &egui::Ui, icon: Icon) -> egui::Image<'static> {
    let tint = egui_shell::theme::Theme::selected_widget_ink(ui.ctx());
    image_tinted(ui, icon, IconWeight::Bold, tint)
}

/// The right image for a toggle in either state — the two-line helper that
/// keeps every call site from re-deriving the same `if selected`.
pub fn toggle_image(ui: &egui::Ui, icon: Icon, selected: bool) -> egui::Image<'static> {
    if selected {
        selected_image(ui, icon)
    } else {
        image(ui, icon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shipped asset must parse.
    #[test]
    fn every_icon_parses() {
        for &icon in Icon::ALL {
            let art = IconArt::parse(icon.source())
                .unwrap_or_else(|e| panic!("icon '{}' failed to parse: {e}", icon.name()));
            assert!(
                art.shape_count() > 0,
                "icon '{}' parsed to zero shapes",
                icon.name()
            );
        }
    }

    /// Every asset must also rasterize to something visible. A glyph that
    /// parses but draws nothing (e.g. every shape `stroke="none"`) would
    /// otherwise sail through [`every_icon_parses`].
    #[test]
    fn every_icon_rasterizes_to_visible_pixels() {
        for &icon in Icon::ALL {
            let art = IconArt::parse(icon.source()).expect("parses");
            let img = art.rasterize(32, IconWeight::Regular);
            assert_eq!(img.size, [32, 32]);
            let lit = img.pixels.iter().filter(|p| p.a() > 0).count();
            assert!(lit > 20, "icon '{}' rasterized nearly blank", icon.name());
        }
    }

    /// The set's one style exception, asserted from both sides.
    #[test]
    fn fill_is_semantic_and_the_set_that_uses_it_is_closed() {
        /// Every icon entitled to a fill, and why.
        const FILLED: &[Icon] = &[
            Icon::Redact,
            Icon::Cursor,
            Icon::CursorNode,
            Icon::RedactSelection,
            Icon::ApplyRedactions,
        ];

        for &icon in Icon::ALL {
            let art = IconArt::parse(icon.source()).expect("parses");
            assert_eq!(
                art.has_fill(),
                FILLED.contains(&icon),
                "fill expectation violated for '{}'",
                icon.name()
            );
        }
    }

    /// **Look at them.** Writes a contact sheet of every icon in the set to
    /// `target/icon-contact-sheet.png`, at the 16 px they actually ship at and
    /// again at 32 px.
    ///
    /// `#[ignore]` because it is an INSTRUMENT, not an assertion — it cannot
    /// fail, and a test that cannot fail must not sit in the suite pretending
    /// to be evidence. Run it deliberately:
    ///
    /// ```text
    /// cargo test -p pdfcer-gui contact_sheet -- --ignored --nocapture
    /// ```
    ///
    /// # Why this exists
    ///
    /// The tests that guard the set answer *"does it parse"*, *"does it draw
    /// more than twenty pixels"* and *"is the fill semantic"*. None of them can
    /// see that two icons look **the same** — the state in which four form
    /// tools and four measure tools each render as one picture with every test
    /// green.
    ///
    /// This project's standing rule: **a layout or rendering defect has exactly
    /// one oracle, and it is a rendered image.** Art is judged by rendering it,
    /// never by reading its source.
    #[test]
    #[ignore = "an instrument, not an assertion — writes a PNG for a human to look at"]
    fn contact_sheet() {
        const COLS: usize = 12;
        const CELL: usize = 40;
        const PX: u32 = 32;
        let rows = Icon::ALL.len().div_ceil(COLS);
        let (w, h) = (COLS * CELL, rows * CELL);
        let mut buf = vec![0u8; w * h * 4];
        for (i, &icon) in Icon::ALL.iter().enumerate() {
            let art = IconArt::parse(icon.source()).expect("parses");
            let img = art.rasterize(PX, IconWeight::Regular);
            let (ox, oy) = ((i % COLS) * CELL + 4, (i / COLS) * CELL + 4);
            for y in 0..PX as usize {
                for x in 0..PX as usize {
                    let p = img.pixels[y * PX as usize + x];
                    let o = ((oy + y) * w + ox + x) * 4;
                    // Ink is `currentColor`; paint it black on white so the
                    // sheet reads like the light presets rather than like a
                    // transparency checkerboard.
                    let a = f32::from(p.a()) / 255.0;
                    let v = (255.0 * (1.0 - a)) as u8;
                    buf[o] = v;
                    buf[o + 1] = v;
                    buf[o + 2] = v;
                    buf[o + 3] = 255;
                }
            }
        }
        // Absolute, from the manifest dir. `cargo test` runs with the CRATE
        // as cwd, not the workspace root, so a relative "target/…" resolves to
        // a directory that does not exist and the write fails with a bare
        // "cannot find the path specified" — which reads like a permissions
        // problem and is not one.
        let path = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../target/icon-contact-sheet.png"
        ));
        let map = pdfcer_render::tiny_skia::Pixmap::from_vec(
            buf,
            pdfcer_render::tiny_skia::IntSize::from_wh(w as u32, h as u32).expect("size"),
        )
        .expect("pixmap");
        map.save_png(path).expect("write the sheet");
        println!(
            "contact sheet: {} icons -> {}",
            Icon::ALL.len(),
            path.display()
        );
    }

    /// A measurement, not an assertion: how alike is the most-alike pair?
    ///
    /// Prints every pair of icons whose 16 px rasters differ by less than 25 %
    /// of their lit pixels, worst first. Run it before choosing a threshold for
    /// [`no_two_icons_render_as_the_same_picture`], and after adding art.
    ///
    /// ```text
    /// cargo test -p pdfcer-gui closest_pairs -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "a measurement, not an assertion — run it and read the numbers"]
    fn closest_pairs() {
        let sheets: Vec<_> = Icon::ALL.iter().map(|&i| (i, lit_mask(i, 16))).collect();
        let mut pairs = Vec::new();
        for (a, (ia, ma)) in sheets.iter().enumerate() {
            for (ib, mb) in &sheets[a + 1..] {
                let d = difference(ma, mb);
                if d < 0.25 {
                    pairs.push((d, ia.name(), ib.name()));
                }
            }
        }
        pairs.sort_by(|x, y| x.0.total_cmp(&y.0));
        for (d, a, b) in &pairs {
            println!("{:6.3}  {a}  ~  {b}", d);
        }
        println!("{} pair(s) under 25 %", pairs.len());
    }

    /// The set of pixels an icon lights at `px`, as a bitmask.
    fn lit_mask(icon: Icon, px: u32) -> Vec<bool> {
        let art = IconArt::parse(icon.source()).expect("parses");
        art.rasterize(px, IconWeight::Regular)
            .pixels
            .iter()
            .map(|p| p.a() > 96)
            .collect()
    }

    /// Symmetric difference over union — 0.0 identical, 1.0 disjoint.
    fn difference(a: &[bool], b: &[bool]) -> f32 {
        let (mut diff, mut union) = (0usize, 0usize);
        for (x, y) in a.iter().zip(b) {
            if *x || *y {
                union += 1;
            }
            if x != y {
                diff += 1;
            }
        }
        if union == 0 {
            return 0.0;
        }
        diff as f32 / union as f32
    }

    /// **No two icons may render as the same picture.**
    ///
    /// # Why it is a raster comparison
    ///
    /// Every other test in this module asks whether an icon DREW something:
    /// does it parse, does it produce more than twenty lit pixels, is its fill
    /// semantic, does CRLF change it. None of them can see two icons that draw
    /// the same thing — and four form-field tools on one asset plus four
    /// measure tools on another is eight controls rendering as two pictures,
    /// in a ribbon where a control is distinguishable only by its icon and its
    /// tooltip, with the whole suite green.
    ///
    /// Shared ART is visible in [`Icon::source`], which is why
    /// [`super::catalog::tests`]' `only_the_documented_assets_are_shared`
    /// catches it. This catches the shape that one **cannot**: two DIFFERENT
    /// assets that happen to draw nearly the same marks. That is what a
    /// "consistency pass" or a careless re-draw produces, and it is what the
    /// enum's doc comments warn about pair by pair — [`Icon::Back`] vs
    /// [`Icon::ChevronLeft`], [`Icon::ShowPoints`] vs [`Icon::EditObjects`],
    /// [`Icon::Layers`] vs [`Icon::Combine`]. This is the enforcement behind
    /// those warnings.
    ///
    /// # Same-asset pairs are excluded, deliberately and by construction
    ///
    /// Two roles pointing at one asset render identically **on purpose** and
    /// are already governed by their own test, which names them and fails if a
    /// third appears. Comparing them here would report a difference of exactly
    /// zero for a state another test has already blessed — a gate that fires on
    /// a correct state is one people learn to ignore. So the comparison is over
    /// distinct SOURCES, not distinct variants.
    ///
    /// # The threshold and the exemptions are MEASURED, not chosen
    ///
    /// `closest_pairs` ranks every pair at 16 px. Over the current set it
    /// produces, in order:
    ///
    /// ```text
    /// 0.000  open ~ font-folders                   (one asset — excluded here)
    /// 0.000  import-form-data ~ insert-pages       (one asset — excluded here)
    /// 0.103  zoom-out ~ zoom-in                    (the magnifier family)
    /// 0.125  zoom-in  ~ zoom-region                (the magnifier family)
    /// 0.185  import-form-data ~ export             (the arrow-leaves-container family)
    /// 0.185  insert-pages     ~ export             (the arrow-leaves-container family)
    /// 0.211  new-document ~ new-from-template      ← the real minimum
    /// 0.217  recognise-text ~ render-diagnostics
    /// 0.225  zoom-out ~ zoom-region                (the magnifier family)
    /// ```
    ///
    /// So `0.15` sits below the real minimum with about 40 % of headroom, and
    /// the two families above it are exempted BY NAME with a reason each —
    /// rather than the threshold being lowered to 0.09 to swallow them, which
    /// would make the test assert almost nothing.
    ///
    /// `new-document ~ new-from-template` at 0.211 is the tightest genuine
    /// pair, and it is also [`svg`]'s dash support working: the ONLY difference
    /// between those two glyphs is a `stroke-dasharray` placeholder box.
    /// Without dashes they measure far closer than the threshold, which is why
    /// the pair is kept as the floor rather than exempted.
    ///
    /// 16 px and not 32: the raster the operator sees is the one that must
    /// discriminate. Two glyphs that separate cleanly at 32 and collapse at 16
    /// are a defect, and measuring at 32 would hide precisely that case.
    #[test]
    fn no_two_icons_render_as_the_same_picture() {
        /// Below this symmetric-difference ratio at 16 px, two icons are the
        /// same picture as far as an operator glancing at a ribbon is
        /// concerned. See the doc comment for the measurement behind it.
        const TOO_ALIKE: f32 = 0.15;

        /// Pairs that are SUPPOSED to look alike, each with the reason.
        const DELIBERATELY_ALIKE: &[(&str, &str)] = &[
            ("zoom-in", "zoom-out"),
            ("zoom-in", "zoom-region"),
            ("zoom-out", "zoom-region"),
            // No `("insert-pages", "export")` entry: `insert-pages` has art
            // of its own rather than wearing `upload`, so that pair measures
            // well clear of the floor. An exemption with nothing behind it is a
            // hole waiting for a future pair to fall into silently.
            ("import-form-data", "export"),
        ];

        let exempt = |a: &str, b: &str| {
            DELIBERATELY_ALIKE
                .iter()
                .any(|&(x, y)| (x == a && y == b) || (x == b && y == a))
        };

        let sheets: Vec<_> = Icon::ALL.iter().map(|&i| (i, lit_mask(i, 16))).collect();
        let mut worst: Option<(f32, &str, &str)> = None;
        for (index, (ia, ma)) in sheets.iter().enumerate() {
            for (ib, mb) in &sheets[index + 1..] {
                // Two roles on ONE asset are governed by their own test, and
                // the division of labour is exact: `only_the_documented_assets_are_shared`
                // buckets `Icon::ALL` by `source()` CONTENT and fails on any
                // undocumented bucket of more than one. So byte-identical art —
                // whether deliberately shared or accidentally duplicated into a
                // second file — is already caught there, loudly and by name.
                //
                // ⇒ This test owns the case that one structurally cannot see:
                // art that is DIFFERENT text and yet the SAME picture. Compared
                // by content rather than by pointer, deliberately: two
                // `include_str!`s of byte-identical files may or may not be
                // interned to one pointer depending on the compiler, and a skip
                // condition that changes with the optimiser is not a skip
                // condition — a planted duplicate walks straight past a
                // `ptr::eq` guard whenever the two literals are interned.
                if ia.source() == ib.source() {
                    continue;
                }
                if exempt(ia.name(), ib.name()) {
                    continue;
                }
                let d = difference(ma, mb);
                if worst.is_none_or(|(w, _, _)| d < w) {
                    worst = Some((d, ia.name(), ib.name()));
                }
            }
        }
        let (d, a, b) = worst.expect("the set has at least two distinct assets");
        assert!(
            d >= TOO_ALIKE,
            "'{a}' and '{b}' render as the same picture at 16 px (difference {d:.3} < \
             {TOO_ALIKE}). Two controls that look identical are two controls the operator \
             cannot tell apart, which is the defect eight tools shipped with until 2026-09-04. \
             Run `cargo test -p pdfcer-gui closest_pairs -- --ignored --nocapture` for the whole \
             ranking before deciding which of the two to redraw — and if the resemblance is \
             DELIBERATE, add the pair to DELIBERATELY_ALIKE with its reason rather than lowering \
             the threshold, which would silently retire the test for every other pair too."
        );
    }

    /// CRLF line endings must not change a single pixel.
    #[test]
    fn crlf_line_endings_parse_identically() {
        for &icon in Icon::ALL {
            let lf = icon.source().replace("\r\n", "\n");
            let crlf = lf.replace('\n', "\r\n");
            let a = IconArt::parse(&lf).expect("LF parses");
            let b = IconArt::parse(&crlf)
                .unwrap_or_else(|e| panic!("CRLF form of '{}' failed: {e}", icon.name()));
            assert_eq!(
                a.shape_count(),
                b.shape_count(),
                "CRLF changed shape count for '{}'",
                icon.name()
            );
            assert_eq!(
                a.rasterize(24, IconWeight::Regular).pixels,
                b.rasterize(24, IconWeight::Regular).pixels,
                "CRLF changed the raster for '{}'",
                icon.name()
            );
        }
    }
}
