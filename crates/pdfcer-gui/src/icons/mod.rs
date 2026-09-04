//! # icons — the SVG-path → tiny-skia → egui-texture icon pipeline
//!
//! Turns a set of hand-authored outline SVGs into tinted, DPI-correct egui
//! images for the ribbon, the menus and any hand-drawn control. Nothing in
//! this module knows what any icon *means* — the meaning lives in
//! [`Icon`]'s variant names and their doc comments; this module knows how to
//! turn a path `d` attribute into pixels, how not to do it twice, and how to
//! report the one thing it cannot draw.
//!
//! Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\icons.rs` (Class A,
//! `SALVAGE.md`: *"SVG path data rasterized at physical pixel size rather
//! than pre-baked PNGs. Mostly data."*). The source was 1,747 code lines and
//! 383 test lines in one file, over this project's 1,500-line limit, so it
//! is split along the seams the original already had:
//!
//! | module | what it owns |
//! |---|---|
//! | [`assets`] | the art itself, verbatim, and its **provenance record** |
//! | [`catalog`] | [`Icon`] — which glyphs exist, what each one means, and the key vocabulary |
//! | [`svg`] | the SVG element scanner and the tiny-skia rasterizer (the path `d` grammar is `svg::path`) |
//! | [`cache`] | one raster per (icon, physical size, weight), memoized |
//! | [`paint`] | the `egui-shell` painter seam, and the missing-icon mark |
//!
//! ## ★ Why a hand-rolled SVG subset parser instead of a crate
//!
//! egui renders no vector art natively, so an SVG has to become a raster
//! somewhere. The three candidate pipelines, and why this one won:
//!
//! 1. **A runtime SVG crate (`resvg`/`usvg`).** Correct and general, but a
//!    NEW Cargo dependency — and `resvg` is MPL-2.0 (weak copyleft). The
//!    standing rule is that an agent does not add a dependency solo,
//!    copyleft or not, and this was an explicit operator go/no-go.
//!    **Rejected by the operator, 2026-08-02.**
//! 2. **Pre-rasterize to PNG at build time.** Zero dependencies, but the
//!    resolution is baked: a raster sized for a 16 pt slot at 100% display
//!    scale is visibly soft at 150%/200% Windows scaling, and would be wrong
//!    again for any future "larger toolbar icons" accessibility option. It
//!    was also *not executable on the operator's machine* — no SVG
//!    rasterizer is installed (no Inkscape, no ImageMagick, and
//!    `cairosvg`'s libcairo fails to load), so the conversion step had no
//!    tool to run.
//! 3. **Parse the path data ourselves and stroke it with `tiny-skia`** —
//!    what this module does. `tiny-skia` is ALREADY reachable as
//!    `pdfcer_render::tiny_skia`, so this adds **zero** new crates. It
//!    rasterizes at whatever physical pixel size the current display scale
//!    implies, so icons are crisp at any DPI — strictly better than (2) and
//!    free of (1)'s licensing question.
//!
//! **This is the load-bearing decision of the whole module**, and it is the
//! one `SALVAGE.md` singles out. Everything else here — the cache key
//! including the physical size, the mask-plus-tint theming, the refusal to
//! guess at malformed path data — follows from choosing (3), and any future
//! "simplification" that pre-bakes rasters gives up crispness at every
//! display scale the operator's machine is not currently set to.
//!
//! The cost of (3) is that a subset SVG parser now exists in this repo, and
//! a parser that silently mis-reads its input is worse than no parser at
//! all. That risk is contained two ways: the parser **refuses** rather than
//! guesses (see [`svg`]), and [`tests::every_icon_parses`] parses every
//! shipped asset, so a malformed or out-of-subset icon fails the test gate
//! instead of shipping as a wrong glyph.
//!
//! ## ★ Theming: one raster per icon, tinted at draw time
//!
//! Every asset is `stroke="currentColor"` — a single-colour outline with no
//! palette — so each icon is rasterized ONCE as a white-on-transparent
//! coverage mask and takes its colour at draw time from the **theme's
//! foreground**, never from a baked constant. In the ribbon that colour
//! arrives as `IconRequest::tint`, which `egui-shell` reads from
//! `ui.style().interact(&response).fg_stroke.color`; in a hand-drawn control
//! it is [`image`]'s `ui.visuals().text_color()`.
//!
//! Consequences, all of them deliberate:
//!
//! * Light theme, dark theme, hovered, active and disabled all share ONE
//!   raster. There are no light/dark asset pairs to keep in sync, and
//!   structurally no way for an icon to end up hardcoded-black on a dark
//!   background — the failure `tools/gates/check-theme-colors.sh` exists to
//!   catch is removed here by construction rather than by policing.
//! * The tint is therefore **not** part of the cache key. Re-tinting is
//!   free; re-rastering is not.
//! * Disabled controls need no fade logic at all. egui's own
//!   `Ui::disable()` multiplies the painter's opacity, which applies to
//!   textured meshes exactly as it applies to text — so an icon fades
//!   precisely the way the text beside it does.
//!
//! ## ★ Weight, and why selected state is not colour alone
//!
//! [`IconWeight::Bold`] rasterizes the same art with the stroke width
//! multiplied. It exists because of the standing rule that **selected state
//! is never colour alone**. Text toggles satisfy that rule by going bold; an
//! icon has no text to embolden, so the *glyph* goes bold instead.
//!
//! Note the seam limitation recorded in [`paint`]: `IconRequest` does not
//! carry the control's selected state, so the ribbon path cannot apply the
//! weight cue today. It is fully implemented and reachable through
//! [`toggle_image`] for anything the application draws itself.
//!
//! ## ★ DPI
//!
//! Icons are laid out in **logical points** ([`ICON_PTS`], or whatever
//! square the ribbon reserved) but rasterized at
//! `points * pixels_per_point()` **physical pixels**, then drawn back at the
//! logical size. Rasterizing at the logical size instead would make every
//! icon visibly soft on any HiDPI display. Because the physical size is part
//! of the cache key, a display-scale change re-rasterizes automatically
//! rather than reusing a stale, wrongly-sized texture — asking for the right
//! size *is* the cache invalidation.
//!
//! ## Wiring it into the ribbon
//!
//! ```ignore
//! let mut icons = pdfcer_gui::icons::paint_ribbon_icon;
//! let report = Ribbon::new(&registry, &conditions, &manifest)
//!     .with_icon_painter(&mut icons)
//!     .render(ui, &mut ribbon_state);
//! ```
//!
//! Until that painter is supplied, `egui-shell` draws text labels
//! everywhere: `ribbon::qat::shows_label` refuses to go icon-only unless the
//! application can actually paint. See [`paint`]'s header — supplying a
//! painter is what turns the ribbon from a row of text buttons into a
//! ribbon.

pub mod assets;
pub mod cache;
pub mod catalog;
pub mod paint;
pub mod svg;

/// Font-glyph coverage: *can the stack actually draw this character?*
///
/// Test-only, and the sibling of [`paint_missing_mark`] on the text side —
/// same question ("what happens when a mark cannot be drawn?"), different
/// pipeline. It carries the widened glyph gate over `crate::text`, and the
/// finding that `egui`'s own `has_glyph` answers the question incorrectly.
/// See `DEFECTS.md` D12 and the module header.
#[cfg(test)]
pub mod glyphs;

pub use cache::IconCache;
pub use catalog::Icon;
pub use paint::{paint_icon, paint_missing_mark, paint_ribbon_icon};
pub use svg::{IconArt, IconError};

use cache::with_cache;

/// Icon edge length in **logical points** for a control this crate draws
/// itself.
///
/// The ribbon does not use this — it reserves a square from
/// `egui_shell::theme::Metrics::icon_pts` and hands the rect to the painter,
/// which is the right layering: the shell owns its own metrics. This is the
/// size for menus, the status bar and anything else drawn with [`image`].
///
/// # Recorded deviation: 16 pt, not the 18–20 px the ui-spec suggested
///
/// The salvage source's toolbar button was 28×24 pt and egui's default
/// `button_padding` is (4,1), leaving a 20×22 pt content box. The ui-spec
/// §4.1 asked for "roughly 18–20px … leaving a few px of padding on every
/// side"; those two halves of the sentence conflict — 18 pt in a 20 pt box
/// leaves 1 pt, not "a few".
///
/// 16 pt honours the paragraph's actual intent (the click target stays
/// meaningfully larger than the visible glyph — the Fitts's-law win the spec
/// is really asking for), leaves 2 pt of padding horizontally and 3 pt
/// vertically, and pairs optically with egui's 12.5 pt body text on
/// icon+text controls, which an 18 pt glyph does not. It also happens to
/// match `egui_shell`'s own `Metrics::icon_pts` for the Quiet and Dark
/// presets, so the two families of control agree by default.
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
///
/// The texture is rasterized at `ICON_PTS * pixels_per_point()` PHYSICAL
/// pixels and then declared to be `ICON_PTS` logical points wide, which is
/// what makes it crisp on a HiDPI display instead of a stretched blur. See
/// this module's header, "DPI".
///
/// Prefer [`paint_icon`] where a `Painter` and a rect are already in hand
/// (anything inside a laid-out widget); this returns an [`egui::Image`]
/// widget for the case where the caller is composing a layout and wants the
/// icon to take part in it.
pub fn image_tinted(
    ui: &egui::Ui,
    icon: Icon,
    weight: IconWeight,
    tint: egui::Color32,
) -> egui::Image<'static> {
    let ctx = ui.ctx();
    let px = (ICON_PTS * ctx.pixels_per_point()).round().max(1.0) as u32;
    let handle = with_cache(|cache| cache.texture(ctx, icon, px, weight));
    let sized = egui::load::SizedTexture::new(handle.id(), egui::vec2(ICON_PTS, ICON_PTS));
    egui::Image::from_texture(sized)
        .fit_to_exact_size(egui::vec2(ICON_PTS, ICON_PTS))
        .tint(tint)
}

/// An icon in the ordinary (non-selected) state.
///
/// The tint is `ui.visuals().text_color()` read from the CALLER's `Ui`,
/// which is what makes an icon inside `add_enabled_ui(false, …)` fade in
/// lockstep with the text beside it, with no disabled-state logic of its
/// own. It is also why no colour is chosen here: the theme already set that
/// visual.
pub fn image(ui: &egui::Ui, icon: Icon) -> egui::Image<'static> {
    image_tinted(ui, icon, IconWeight::Regular, ui.visuals().text_color())
}

/// An icon in the selected/active state of a toggle.
///
/// Two cues at once, neither of which is the background fill egui already
/// paints: the accent tint AND [`IconWeight::Bold`]. That layering is the
/// standing "selected state is never colour alone" rule surviving the loss
/// of a text label to embolden.
///
/// # ★★★ WHICH BACKGROUND THIS GLYPH IS DRAWN ON, since it is not this
/// function that paints it
///
/// The plate underneath is **`egui`'s selected-widget fill** — the theme's
/// [`egui_shell::theme::Palette::selected_plate`]. `egui` substitutes it into
/// both `bg_fill` and `weak_bg_fill` for anything carrying `SELECTED_CLASS`
/// (`egui-0.35.0/src/widget_style.rs:151-154`), so a toggle drawn with
/// `Button::image(...).selected(true)` gets that plate whether or not the call
/// site mentions a colour. This function's only job is to put the ink that
/// reads on it into the glyph.
///
/// [`Theme::selected_widget_ink`] is *defined* as that ink, and
/// `egui_shell::theme::tests::the_selected_widget_accessors_agree_with_the_style_egui_will_paint`
/// asserts it equals `visuals.selection.stroke.color` in every preset — so the
/// pairing is held by an assertion, not by this paragraph.
///
/// # ★★ Why not `ui.visuals().selection.stroke.color`, which is the same value
///
/// It **was** that read, and it was this file that held
/// `check-selection-channel.sh`'s one file-level exemption. The exemption is
/// gone and so is the read.
///
/// Same value, different promise. `visuals.selection` is a raw `egui` channel
/// whose meaning the theme decides and has now re-decided twice in two days —
/// it carried the canvas's 27 % wash (defect T2), then `accent` + `on_accent`
/// (which broke the focused-`TextEdit` ring `egui` drives from the *same*
/// field), and now `selected_plate` + `accent`. Each of those re-pointings
/// silently changed what this glyph would be tinted with, and nothing here
/// would have failed. A named accessor cannot drift that way: it is checked
/// against the shipped style, and a future re-pointing has to walk past a red
/// test that names this call site.
///
/// ★ Note the ink is deliberately NOT [`egui_shell::theme::Theme::accent_pair`]'s
/// `on_accent`. That pair is the *emphasised action* surface — the full accent
/// at full strength — and a selected toggle is a quieter thing: a diluted plate
/// with accent ink. Tinting this glyph `on_accent` would put a near-white mark
/// on a pale plate, which is `DEFECTS.md` D2 exactly.
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

    /// ★ Every shipped asset must parse.
    ///
    /// This is the gate that makes the hand-rolled parser safe to rely on: a
    /// malformed or out-of-subset icon fails `cargo test` rather than
    /// shipping as a blank button. It walks [`Icon::ALL`], which is why
    /// `catalog::tests::all_is_exhaustive_and_free_of_duplicates` exists —
    /// a variant missing from `ALL` is not merely untested, it is *silently*
    /// untested.
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

    /// ★ The set's one style exception, asserted from both sides.
    ///
    /// A future "style cleanup" must not quietly turn redaction's honest
    /// solid bar into an outline, and no other icon may drift into being
    /// filled. The fill is semantic, not decorative: every other tool in
    /// this application draws or measures, and this one obliterates.
    ///
    /// It is also the pipeline's only coverage of the fill path, so an
    /// "audit" that outlined it would silently delete a test as well as a
    /// meaning.
    ///
    /// ★★ **Widened on 2026-08-19, and the widening is the point rather than a
    /// concession.** The rule this test enforces is *fill is semantic, never
    /// decorative*, and the black-arrow / white-arrow pair is the purest
    /// available instance of it: `cursor` and `cursor-node` have
    /// **byte-identical outlines** and differ only in fill, and that difference
    /// has meant "the whole object" versus "the points inside it" in every
    /// vector editor since Illustrator 88.
    ///
    /// So the assertion is no longer `icon == Icon::Redact` but membership of a
    /// named set with a reason per member. A future audit that outlines one of
    /// these deletes a meaning as well as a test — which is what the original
    /// comment warned about, applied to a larger set.
    #[test]
    fn fill_is_semantic_and_the_set_that_uses_it_is_closed() {
        /// Every icon entitled to a fill, and why.
        ///
        /// - [`Icon::Redact`] — every other tool draws or measures; this one
        ///   obliterates.
        /// - [`Icon::Cursor`] — the filled half of the arrow pair. Its hollow
        ///   twin is the ONLY thing distinguishing the two tools.
        /// - [`Icon::CursorNode`] — the hollow arrow, plus one filled anchor
        ///   square among three outlined ones, which says *this is the point
        ///   you picked* in the same language `canvas::overlay` draws on the
        ///   page itself.
        /// - [`Icon::RedactSelection`] and [`Icon::ApplyRedactions`] —
        ///   **2026-09-04, and they inherit the reason rather than extending
        ///   it.** Both are members of the redaction family, both draw the
        ///   same solid bar [`Icon::Redact`] draws, and both act on the same
        ///   irreversible thing. An outline-only redaction glyph understates a
        ///   feature that removes content permanently, and that argument does
        ///   not weaken because the command is scoped to a selection or is the
        ///   apply step. ★ Adding them was a decision, not a formality: the
        ///   honest alternative was to outline these two and leave the fill to
        ///   the parent tool, and it was rejected because it would make the
        ///   family's most destructive member — Apply, the one that cannot be
        ///   undone — the palest picture of the three.
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

    /// ★★★ **Look at them.** Writes a contact sheet of every icon in the set to
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
    /// Because on 2026-09-04 thirty-six glyphs were adopted at once, and the
    /// tests that guard the set answer *"does it parse"*, *"does it draw more
    /// than twenty pixels"* and *"is the fill semantic"*. None of them can see
    /// that two icons look **the same**, which is the exact defect the batch
    /// was adopted to fix — four form tools and four measure tools were each
    /// rendering as one picture, and every test was green throughout.
    ///
    /// This project's standing rule, learned twice: **a layout or rendering
    /// defect has exactly one oracle, and it is a rendered image.** The same
    /// review that supplied this art was only correctly assessed once somebody
    /// rendered it instead of reading its source.
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
        // ★ Absolute, from the manifest dir. `cargo test` runs with the CRATE
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

    /// ★★★ **No two icons may render as the same picture.**
    ///
    /// # Why this test had to be written, and why it is a raster comparison
    ///
    /// Every other test in this module asks whether an icon DREW something:
    /// does it parse, does it produce more than twenty lit pixels, is its fill
    /// semantic, does CRLF change it. None of them can see two icons that draw
    /// the same thing — and on 2026-09-04 that was not hypothetical. **Four
    /// form-field tools shared one asset and four measure tools shared
    /// another**: eight controls rendering as two pictures, in a ribbon whose
    /// own module header says those controls are *"distinguishable only by icon
    /// and tooltip"*. The whole suite was green for weeks.
    ///
    /// That particular shape was visible in [`Icon::source`], which is why
    /// [`super::catalog::tests`]' `only_the_documented_assets_are_shared`
    /// catches it. This catches the shape that one **cannot**: two DIFFERENT
    /// assets that happen to draw nearly the same marks. That is what a future
    /// "consistency pass" or a careless re-draw produces, and it is what the
    /// enum's doc comments spend paragraphs warning about, pair by pair —
    /// [`Icon::Back`] vs [`Icon::ChevronLeft`], [`Icon::ShowPoints`] vs
    /// [`Icon::EditObjects`], [`Icon::Layers`] vs [`Icon::Combine`]. Those
    /// warnings had no enforcement until now.
    ///
    /// # ★★ Same-asset pairs are excluded, deliberately and by construction
    ///
    /// Two roles pointing at one asset render identically **on purpose** and
    /// are already governed by their own test, which names them and fails if a
    /// third appears. Comparing them here would report a difference of exactly
    /// zero for a state another test has already blessed — a gate that fires on
    /// a correct state is one people learn to ignore. So the comparison is over
    /// distinct SOURCES, not distinct variants.
    ///
    /// # ★★★ The threshold and the exemptions are MEASURED, not chosen
    ///
    /// `closest_pairs` ranks every pair at 16 px. Over the set as it stood on
    /// 2026-09-04 it produced, in order:
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
    /// would have made the test assert almost nothing.
    ///
    /// ★ `new-document ~ new-from-template` at 0.211 is also the dash support
    /// added earlier the same day, working: the ONLY difference between those
    /// two glyphs is a `stroke-dasharray` placeholder box. Before that landed
    /// they would have measured far closer, and this test — had it existed —
    /// would have caught the visual duplicate the whole icon batch was blocked
    /// on. It is kept as the tightest genuine pair for exactly that reason.
    ///
    /// ★★ 16 px and not 32: the raster the operator sees is the one that must
    /// discriminate. Two glyphs that separate cleanly at 32 and collapse at 16
    /// are a defect, and measuring at 32 would hide precisely that case.
    #[test]
    fn no_two_icons_render_as_the_same_picture() {
        /// Below this symmetric-difference ratio at 16 px, two icons are the
        /// same picture as far as an operator glancing at a ribbon is
        /// concerned. See the doc comment for the measurement behind it.
        const TOO_ALIKE: f32 = 0.15;

        /// Pairs that are SUPPOSED to look alike, each with the reason.
        ///
        /// - The **magnifier family**. `zoom-in`, `zoom-out` and `zoom-region`
        ///   share a lens because they are three aims of one act, and every
        ///   application that has ever had a zoom control draws them that way.
        ///   Their whole distinction is the mark inside the lens, which is a
        ///   few pixels at 16 px by construction. ⚠ `zoom-out ~ zoom-in`
        ///   measures **0.103**, which is genuinely tight — it is recorded here
        ///   rather than smoothed away, and if the operator ever reports that
        ///   the two zoom buttons are hard to tell apart, this line is the
        ///   evidence that it was known and where to look.
        /// - The **arrow-leaves-container family**. `upload` (worn by
        ///   `insert-pages` and `import-form-data`) and `export` both show
        ///   content crossing a boundary, because both commands move data
        ///   across the document's edge. They differ in direction, which is the
        ///   distinction that matters and is the one an operator reads.
        const DELIBERATELY_ALIKE: &[(&str, &str)] = &[
            ("zoom-in", "zoom-out"),
            ("zoom-in", "zoom-region"),
            ("zoom-out", "zoom-region"),
            // ★ `("insert-pages", "export")` was here until 2026-09-04, when
            // `insert-pages` stopped wearing `upload` and took art of its own.
            // The pair now measures well clear of the floor, so the exemption
            // has nothing to exempt — and an exemption with nothing behind it
            // is a hole waiting for a future pair to fall into silently.
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
                // ★ Two roles on ONE asset are governed by their own test, and
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
                // condition. (Found by falsification — a planted duplicate
                // slipped through a `ptr::eq` guard because the two literals
                // WERE interned.)
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

    /// ★ CRLF line endings must not change a single pixel.
    ///
    /// The assets are ordinary text files, so a repository with
    /// `* text=auto` converts them to CRLF on checkout under
    /// `core.autocrlf=true` — which means the `&'static str` every
    /// `include_str!` in [`assets`] produces gains a `\r` before every `\n`
    /// on a fresh clone, on a machine that has never built this tree before.
    ///
    /// SVG is text and CRLF is harmless *provided* the scanner treats `\r`
    /// as a separator everywhere `\n` is one. This pins that: the same asset
    /// with every line ending doubled must produce identical geometry AND an
    /// identical raster, so a fresh clone on a machine with autocrlf on
    /// cannot ship blank icons.
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

    /// ★ Every icon key a shell command names has an icon behind it.
    ///
    /// This is the mismatch that puts blank boxes back in the ribbon, and it
    /// is invisible to the compiler: `Command::with_icon` takes a `String`,
    /// so a key with no glyph is a perfectly well-typed program that draws a
    /// slashed box where an operator expects a control.
    ///
    /// The check is done against the real command registry rather than
    /// against a hand-maintained list, because a hand-maintained list is one
    /// more thing to forget to update — and forgetting it would make this
    /// test pass while the ribbon was wrong, which is worse than not having
    /// it.
    #[test]
    fn every_command_icon_key_exists_in_the_catalogue() {
        let mut registry = egui_shell::CommandRegistry::new();
        crate::shell::commands::register(&mut registry);
        assert!(
            !registry.is_empty(),
            "the command registry is empty — this test would pass vacuously"
        );

        let mut missing: Vec<(String, String)> = Vec::new();
        for command in registry.iter() {
            if let Some(key) = command.icon.as_deref()
                && Icon::from_key(key).is_none()
            {
                missing.push((command.id.clone(), key.to_owned()));
            }
        }
        assert!(
            missing.is_empty(),
            "commands name icon keys the set has no glyph for: {missing:?}"
        );
    }
}
