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
pub fn selected_image(ui: &egui::Ui, icon: Icon) -> egui::Image<'static> {
    let tint = ui.visuals().selection.stroke.color;
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
        const FILLED: &[Icon] = &[Icon::Redact, Icon::Cursor, Icon::CursorNode];

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
