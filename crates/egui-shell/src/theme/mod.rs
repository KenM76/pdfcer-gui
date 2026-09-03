//! What the application looks like — the single place that decides.
//!
//! # Why this module exists
//!
//! *Salvaged from `D:\Dev\pdfce\crates\pdfce-gui\src\theme.rs` (601
//! lines, 2026-08-12). The reasoning below is the original's and is the
//! most valuable thing being transferred; what changed is recorded under
//! "What changed in the salvage" at the end of this header.*
//!
//! Until the source module existed, nothing set a style at all. The whole
//! application ran on `egui`'s stock appearance, and every colour it drew
//! beyond that was a `Color32::from_rgb(…)` literal at its use site — 26
//! of them, four named, the rest inline in a 27,000-line file. There was
//! no answer to "what colour is this application's accent?" other than
//! reading the source.
//!
//! That is not a cosmetic problem, it is a *change-cost* problem. A
//! restyle under those conditions is a sweep through every call site
//! where the failure mode is not a crash but INCONSISTENCY — the sites
//! you miss leave two-thirds of a theme, which looks worse than none and
//! cannot be caught by a test that only knows about compilation.
//!
//! So the look is data, in one place, and a CI gate
//! (`check-theme-colors.sh`) forbids raw colours outside it. That is the
//! same shape as a gated string catalogue (every operator-visible string
//! in one module) and a gated icon set (every glyph, checked by parse and
//! raster tests). Both of those already make their kind of change safe;
//! this is the third.
//!
//! # ★ CHROME IS THEMED. CONTENT COLOUR IS NOT. THEY ARE NOT THE SAME KIND.
//!
//! This is the distinction that makes a colour sweep dangerous, and it is
//! the reason the gate has an escape hatch rather than being absolute.
//!
//! Some colours in an application are written **into the document the
//! operator is editing**. In the application this module was salvaged
//! from, two were: the colour of an annotation the operator authors,
//! which reaches the annotation's `/C` entry and its appearance stream,
//! and the same colour as offered by the properties panel.
//!
//! Those are the *operator's* choice about *document content*. They are
//! not chrome, they are not the application's, and a theme must never
//! touch them: restyling the application would silently change the colour
//! of markup a user is about to commit to a file, and the change would
//! only be visible after saving. A dark theme that quietly authored
//! pale-grey annotations onto a white page would be a data defect wearing
//! a cosmetic disguise.
//!
//! Everything else — panel backgrounds, selection highlights, snap
//! guides, node marks, measurement previews, the canvas backdrop — is
//! chrome, belongs here, and changes with the theme.
//!
//! The rule for anyone adding a colour: **if it can end up in a saved
//! file, it is not a theme colour.** Mark such a site with the literal
//! comment `// DOCUMENT COLOUR:` and the gate will allow it, because the
//! gate's job is to catch the colour someone forgot to name, not to
//! forbid the two that must stay where they are.
//!
//! # Overlay colours are semantics, not decoration
//!
//! An application's overlay palette is not free choice. Its entries carry
//! meaning the operator is expected to learn. In the salvage source:
//!
//! - the node mark and the subpath outline were different colours because
//!   they answered different questions ("a point is here" vs "this run is
//!   one subpath");
//! - the measurement preview and the committed dimension differed because
//!   one is a proposal and one is document state — the application's own
//!   inferences must be visibly distinct from what the operator
//!   committed;
//! - form-field chrome had a hue of its own, distinct from the
//!   object-selection accent, because it means "a control lives here"
//!   rather than "this is selected".
//!
//! A theme may re-tune those hues. It may **not** collapse two of them
//! into one, and the original enforced that for every preset: any theme
//! in which two semantically distinct roles resolve to the same colour
//! failed the build. Colour was never the only cue for any of these —
//! each also carried a shape, a dash pattern or a label — but a theme
//! that merges two roles removes a cue that was doing work, and it would
//! do so silently.
//!
//! **Those role names are the application's vocabulary, not the shell's**,
//! so they do not live in [`Palette`]. They live in [`Overlays`], which
//! is a named, ordered set of application colour roles that the shell
//! stores, hands back, and can check for collisions — carrying the
//! *enforcement* across without carrying the *domain*. See the
//! [`overlays`] module.
//!
//! # Why presets, and why the operator can switch at runtime
//!
//! Three presets ship. That is not indecision — the right look for an
//! application is a question about the operator's environment, not about
//! the code, and it cannot be settled by reading the source. A CAD user
//! on a dark toolchain, a document reviewer on a bright monitor and a
//! long-session editor genuinely want different things.
//!
//! Switching is live, from a settings surface, because the alternative is
//! choosing a look from screenshots — and a screenshot cannot show what
//! an hour in the application feels like.
//!
//! # Metrics travel with the palette
//!
//! [`Metrics`] carries the spacing and sizing decisions that make a look
//! coherent — control height, gutter, panel padding, corner radius. They
//! belong with the palette rather than in a second module because they
//! are not independent: a generous-padding theme with a dense theme's
//! control height reads as a mistake, and keeping them in one struct
//! makes that combination unrepresentable rather than merely discouraged.
//!
//! # The rendered-pair contrast gate
//!
//! [`contrast`] is new, and it is the reason this module was worth
//! salvaging rather than re-deriving. See `DEFECTS.md` D2 and the tests
//! at the bottom of this file: the original had two contrast tests, both
//! of which compared *palette entries chosen by a human as a pair*, and
//! the shipped defect was in the *assignment* — which palette entry ended
//! up as a foreground and which as a background on the actual
//! `egui::Style`. The gate closes that by reading the style back.
//!
//! # What changed in the salvage
//!
//! 1. **`eframe::egui` → `egui`.** The shell does not depend on `eframe`.
//! 2. **`DEFECTS.md` D2 is fixed** — see `Theme::write_style`.
//! 3. **A new [`Palette::on_accent`] role**, because the D2 defect's root
//!    cause was reaching for a *plate* colour to use as a *text* colour.
//! 4. **Application-specific overlay roles moved out** of [`Palette`]
//!    into [`Overlays`]. `node_mark`, `subpath_outline`,
//!    `dimension_selected`, `preview`, `guide` and `field_chrome` were
//!    a vector-PDF-editor's vocabulary and cannot be made domain-neutral
//!    without renaming them into meaninglessness.
//! 5. **A cross-crate test was dropped.** The original asserted that the
//!    engine crate's default settings token resolved to this module's
//!    default preset. That seam does not exist here — the shell has no
//!    engine — and the equivalent test belongs in the application. See
//!    [`Preset::from_key`]'s doc comment, which states the obligation.
//! 6. **`egui::Id::new("pdfcer-theme")` → `"egui-shell-theme"`.**

pub mod contrast;
pub mod overlays;

pub use contrast::ContrastFailure;
pub use overlays::{Overlays, RoleCollision};

use egui::Color32;

/// The named colour roles the shell draws chrome with.
///
/// Every field is a **role**, never a colour name: `accent`, not `blue`.
/// A theme that wants a green accent should not have to be read as "blue,
/// except it's green now" — which is exactly what a field called `blue`
/// forces on the next reader.
///
/// This struct is deliberately small and deliberately generic. Anything
/// that names a concept belonging to one application's domain goes in
/// [`Overlays`] instead; see this module's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// The window and panel background.
    pub surface: Color32,
    /// A panel or dock sitting on top of [`Self::surface`], one step
    /// separated from it.
    pub panel: Color32,
    /// The area behind the application's main content — deliberately its
    /// own role rather than reusing [`Self::surface`], because the
    /// content must read as an object ON something, and a backdrop equal
    /// to the panel makes its edge disappear.
    pub content_backdrop: Color32,
    /// Ordinary text.
    pub text: Color32,
    /// Secondary text: captions, hints, counts.
    pub text_muted: Color32,
    /// The single accent — selection, focus, the active tab.
    pub accent: Color32,
    /// Text and icons drawn **on** [`Self::accent`].
    ///
    /// # Why this is a role of its own, and not `label_backdrop`
    ///
    /// This field did not exist in the salvage source, and its absence is
    /// the root cause of `DEFECTS.md` D2. The original needed a light
    /// foreground for the accent-filled active state and reached for
    /// [`Self::label_backdrop`] — a *near-opaque plate colour meant to sit
    /// over content* — because it happened to be light.
    ///
    /// That is a category error with a delayed cost. Two roles that must
    /// vary independently were welded together, so the moment the accent
    /// fill went missing (which is what D2 actually was) the foreground
    /// had no background to justify it, and near-white text landed on
    /// light grey. Naming the role makes the pairing explicit and makes
    /// the contrast gate in [`contrast`] able to check it.
    ///
    /// It is also what lets the dark preset do the right thing: its
    /// accent is a *light* blue, so its `on_accent` is near-black, which
    /// is the opposite of what a shared "light plate" colour could ever
    /// have expressed.
    pub on_accent: Color32,
    /// A separator or control border.
    pub outline: Color32,
    /// Something is wrong and the operator must act.
    pub danger: Color32,
    /// Something is worth knowing and nothing is broken.
    ///
    /// Distinct from [`Self::danger`] because a well-built application
    /// reports a great deal that is *disclosure* rather than fault — a
    /// hidden layer, a substituted font, a skipped customization item —
    /// and colouring those as errors is how an operator learns to ignore
    /// the colour that means error.
    pub notice: Color32,
    /// A selected object's fill in the content area (translucent).
    pub selection_fill: Color32,
    /// A label's backdrop, so text stays readable over arbitrary content
    /// (translucent).
    ///
    /// Distinct from [`Self::panel`] because it sits over content whose
    /// colour the application, not the theme, decides.
    pub label_backdrop: Color32,
    /// Text drawn on [`Self::label_backdrop`].
    ///
    /// Not [`Self::text`]: the backdrop is near-opaque and light in every
    /// preset, **including the dark one**, because it sits over CONTENT
    /// rather than over chrome — and the content is whatever colour the
    /// document says.
    pub label_text: Color32,
}

/// Spacing and sizing, travelling with the palette (see the module docs).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Height of an ordinary control.
    pub control_height: f32,
    /// Gap between adjacent controls in a row.
    pub gutter: f32,
    /// Padding inside a panel.
    pub panel_padding: f32,
    /// Corner radius on buttons and panels.
    pub corner_radius: u8,
    /// Icon size in points, for whatever draws the application's icons.
    pub icon_pts: f32,
}

/// A complete look: a palette and its metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// Which preset this is, for persistence and for the picker.
    pub preset: Preset,
    /// The colours.
    pub palette: Palette,
    /// The spacing and sizing.
    pub metrics: Metrics,
}

/// The shipped looks.
///
/// `#[non_exhaustive]` is deliberate: a preset added later must not be a
/// breaking change to anything matching on this, and every consumer
/// should be routing through [`Theme`] rather than branching on the name
/// anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[non_exhaustive]
pub enum Preset {
    /// Muted greys, one restrained accent, tight spacing. The content
    /// dominates and the chrome recedes — the convention a document tool
    /// is measured against.
    #[default]
    Quiet,
    /// Lighter, more generous padding, softer edges, clearer grouping.
    /// Easier to scan; costs screen area.
    Airy,
    /// Dark chrome against light content, as CAD tools do it. High
    /// contrast at the content edge and easier on a long session.
    Dark,
}

impl Preset {
    /// Every preset, for the picker and for the tests that check all of
    /// them. A preset missing here ships unverified.
    pub const ALL: &'static [Preset] = &[Preset::Quiet, Preset::Airy, Preset::Dark];

    /// The settings-file token for this preset.
    ///
    /// Stable identifiers, never the display name: a display string is
    /// operator-visible text and belongs in the application's string
    /// catalogue, where it can be reworded without invalidating everyone's
    /// saved settings.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Preset::Quiet => "quiet",
            Preset::Airy => "airy",
            Preset::Dark => "dark",
        }
    }

    /// Parse a settings-file token, `None` if it names no preset.
    ///
    /// `None` rather than a default so the caller can say *"the settings
    /// file asked for a theme this build does not have"* instead of
    /// silently resetting the operator's choice — the difference between a
    /// note and a mystery.
    ///
    /// # An obligation on the consuming application
    ///
    /// If the application persists a theme choice as a string somewhere
    /// the shell cannot see — a settings file owned by an engine crate,
    /// for instance — then nothing in the type system connects that
    /// literal to this enum, and a rename on either side drifts them
    /// apart. The symptom is a fresh install showing "this settings file
    /// asks for a theme this version does not have" on its very first run:
    /// a message about the operator's file that is really about ours.
    ///
    /// The salvage source carried a test for exactly that seam. It could
    /// not come across, because the seam is between the application and
    /// its own storage, not between the application and the shell. **The
    /// application must carry it**, asserting
    /// `Preset::from_key(&its_default) == Some(Preset::default())`.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|p| p.key() == key)
    }
}

impl Theme {
    /// The theme for a preset.
    #[must_use]
    pub fn new(preset: Preset) -> Self {
        match preset {
            Preset::Quiet => Self::quiet(),
            Preset::Airy => Self::airy(),
            Preset::Dark => Self::dark(),
        }
    }

    /// The default look: muted, tight, and content-first.
    fn quiet() -> Self {
        Self {
            preset: Preset::Quiet,
            palette: Palette {
                surface: Color32::from_rgb(0xF2, 0xF2, 0xF3),
                panel: Color32::from_rgb(0xE8, 0xE8, 0xEA),
                content_backdrop: Color32::from_rgb(0x6E, 0x70, 0x74),
                text: Color32::from_rgb(0x1C, 0x1C, 0x1E),
                text_muted: Color32::from_rgb(0x5E, 0x60, 0x66),
                // Deliberately NOT (30, 110, 220). In the salvage source
                // that was an overlay role's value — "a point is here" —
                // and the first run of the overlay-distinctness test
                // caught the collision: an accent chosen for chrome
                // happened to land exactly on an overlay role that means
                // something else. Deeper, so selection and the overlay
                // stay tellable apart on the same surface. The overlay
                // roles now live in `Overlays`, but the constraint is
                // unchanged and `Overlays::assert_distinct` is how an
                // application re-checks it.
                accent: Color32::from_rgb(0x17, 0x5C, 0xC4),
                on_accent: Color32::from_rgb(0xFA, 0xFA, 0xFA),
                outline: Color32::from_rgb(0xC4, 0xC6, 0xCA),
                danger: Color32::from_rgb(0xC0, 0x2A, 0x2A),
                notice: Color32::from_rgb(0xB0, 0x6A, 0x1A),
                selection_fill: Color32::from_rgba_unmultiplied(90, 140, 220, 70),
                label_backdrop: Color32::from_rgba_unmultiplied(250, 250, 250, 220),
                label_text: Color32::from_rgb(20, 20, 20),
            },
            metrics: Metrics {
                control_height: 24.0,
                gutter: 4.0,
                panel_padding: 6.0,
                corner_radius: 3,
                icon_pts: 16.0,
            },
        }
    }

    /// Lighter and roomier. Same hues, more air.
    fn airy() -> Self {
        let quiet = Self::quiet();
        Self {
            preset: Preset::Airy,
            palette: Palette {
                surface: Color32::from_rgb(0xFA, 0xFA, 0xFB),
                panel: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                content_backdrop: Color32::from_rgb(0x8A, 0x8D, 0x93),
                text: Color32::from_rgb(0x24, 0x26, 0x2B),
                text_muted: Color32::from_rgb(0x6C, 0x70, 0x78),
                outline: Color32::from_rgb(0xDC, 0xDE, 0xE3),
                ..quiet.palette
            },
            metrics: Metrics {
                control_height: 28.0,
                gutter: 8.0,
                panel_padding: 12.0,
                corner_radius: 6,
                icon_pts: 17.0,
            },
        }
    }

    /// Dark chrome, light content.
    ///
    /// `label_backdrop` and `label_text` deliberately stay
    /// dark-text-on-light-plate, because they sit over CONTENT, whose
    /// colour the document decides and the theme does not.
    ///
    /// `on_accent` inverts rather than being inherited, and that is the
    /// clearest demonstration of why it is a role of its own: this
    /// preset's accent is a *light* blue, so the readable foreground on
    /// it is near-black. A single shared "light plate" colour — which is
    /// what the salvage source used here — could not express that, and
    /// the contrast gate in [`contrast`] would refuse the preset if it
    /// tried.
    fn dark() -> Self {
        let quiet = Self::quiet();
        Self {
            preset: Preset::Dark,
            palette: Palette {
                surface: Color32::from_rgb(0x24, 0x26, 0x2A),
                panel: Color32::from_rgb(0x2E, 0x31, 0x36),
                content_backdrop: Color32::from_rgb(0x16, 0x17, 0x1A),
                text: Color32::from_rgb(0xE6, 0xE8, 0xEC),
                text_muted: Color32::from_rgb(0x9A, 0x9E, 0xA6),
                accent: Color32::from_rgb(0x4C, 0x9A, 0xFF),
                on_accent: Color32::from_rgb(0x10, 0x14, 0x1A),
                outline: Color32::from_rgb(0x44, 0x48, 0x4F),
                danger: Color32::from_rgb(0xFF, 0x6B, 0x6B),
                notice: Color32::from_rgb(0xE0, 0xA0, 0x40),
                ..quiet.palette
            },
            metrics: quiet.metrics,
        }
    }

    /// Push this theme into `egui`'s own style.
    ///
    /// # This is the hook that did not exist
    ///
    /// Before the salvage source existed, the only `style_mut` call in the
    /// entire application was a local text-wrap fix inside the ribbon.
    /// Nothing set a background, a text colour, a rounding or a spacing —
    /// so the appearance was `egui`'s defaults plus whatever each call
    /// site drew on top. Calling this once per frame is what makes the
    /// palette actually govern the widgets rather than only the overlays.
    ///
    /// Applied every frame rather than once at startup so a theme change
    /// takes effect immediately, with no restart and no cache to
    /// invalidate. It is a handful of field writes against a struct
    /// `egui` already owns.
    ///
    /// # `all_styles_mut`, not `set_style`
    ///
    /// `egui` keeps a SEPARATE `Style` per light/dark theme and picks
    /// between them from the system setting. Writing only one of them
    /// would make the application's appearance depend on the operator's
    /// OS theme — so a machine set to dark would show a half-styled
    /// application while the developer's light machine looked correct,
    /// which is the worst kind of appearance bug because it is invisible
    /// where it is being written. Both styles get the same palette, and
    /// the preset alone decides whether the application is dark.
    pub fn apply(&self, ctx: &egui::Context) {
        let p = self.palette;
        let m = self.metrics;
        let preset = self.preset;
        ctx.all_styles_mut(move |style| Self::write_style(style, &p, &m, preset));
        // Stash the whole theme where any drawing code can reach it.
        //
        // egui's `Style` has nowhere to put roles that are not egui's
        // vocabulary — a content backdrop, a label plate — so without
        // this the painters would have to be handed a palette through
        // every signature between here and them, and the ones that were
        // missed would silently keep the default: a dark theme with
        // light-theme overlays, which is the two-thirds-of-a-theme failure
        // this module exists to prevent, and no test would see it.
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(Self::CTX_ID), *self));
    }

    /// The `egui::Id` under which [`Theme::apply`] stashes the theme.
    ///
    /// A `&'static str` rather than an `egui::Id` constant because
    /// `egui::Id::new` is not `const`.
    const CTX_ID: &'static str = "egui-shell-theme";

    /// The theme in force for this frame, read back from the context.
    ///
    /// Falls back to the default if nothing has been stashed — which
    /// happens only before the first [`Theme::apply`], i.e. never during a
    /// painted frame.
    #[must_use]
    pub fn of(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp(egui::Id::new(Self::CTX_ID)))
            .unwrap_or_default()
    }

    /// The `egui::Style` this theme produces, standalone.
    ///
    /// # Why this is public
    ///
    /// It is the oracle the contrast gate needs. A test that reads the
    /// *palette* can only check the colours somebody chose; a test that
    /// reads the *style* checks the colours `egui` will actually paint,
    /// including every field the theme forgot to assign — which is the
    /// class of defect `DEFECTS.md` D2 belongs to. Anything that wants to
    /// ask "what will this look like" should ask here, not of
    /// [`Self::palette`].
    ///
    /// Built from `Style::default()`, so it deliberately does not carry
    /// an application's font definitions or text styles. It answers a
    /// question about colour and spacing, which is all this module sets.
    #[must_use]
    pub fn rendered_style(&self) -> egui::Style {
        let mut style = egui::Style::default();
        Self::write_style(&mut style, &self.palette, &self.metrics, self.preset);
        style
    }

    /// Run the rendered-pair contrast gate over this theme.
    ///
    /// A convenience over [`contrast::check`] and [`Self::rendered_style`]
    /// so an application — or a verification harness — can assert the same
    /// property this module's own tests assert, against a theme it built
    /// itself. See [`contrast`] for what is measured and why.
    ///
    /// # Errors
    ///
    /// Returns every failing pair, not just the first, so one run names
    /// the whole problem rather than making the caller iterate.
    pub fn check_contrast(&self, threshold: f32) -> Result<(), Vec<ContrastFailure>> {
        contrast::check(&self.rendered_style(), threshold)
    }

    /// The style write itself, shared by both of `egui`'s per-theme styles.
    ///
    /// # ★ `DEFECTS.md` D2 is fixed here, and this is what was wrong
    ///
    /// The salvage source looped over all five widget states setting
    /// `corner_radius`, `bg_stroke` and `fg_stroke`, then wrote:
    ///
    /// ```text
    /// v.widgets.inactive.weak_bg_fill = p.panel;
    /// v.widgets.hovered.weak_bg_fill  = p.surface;
    /// v.widgets.active.weak_bg_fill   = p.accent;
    /// v.widgets.active.fg_stroke = Stroke::new(1.0, p.label_backdrop);
    /// ```
    ///
    /// `label_backdrop` is `rgba(250,250,250,220)`. Pairing a near-white
    /// foreground with an accent fill is correct. But **only
    /// `weak_bg_fill` was assigned the accent — `widgets.active.bg_fill`
    /// was never set at all.** Widgets that paint their background with
    /// `bg_fill` rather than `weak_bg_fill` — `egui_tiles` tab buttons,
    /// `CollapsingHeader` headers — therefore got a near-white foreground
    /// on `egui`'s default light background. Every collapsible section
    /// heading in the settings dialog and both dock tab labels were
    /// unreadable at 1×.
    ///
    /// The fix has three parts, and only the first is the defect:
    ///
    /// 1. **`bg_fill` is assigned for every state**, not just
    ///    `weak_bg_fill`. `DEFECTS.md` offers this or "stop overriding
    ///    `active.fg_stroke`" as alternatives; assigning both fills is
    ///    strictly better, because it also stops `egui`'s stock greys
    ///    leaking through under the dark preset.
    /// 2. **Every one of the ten fills is assigned**, from the palette. A
    ///    field this function does not write keeps `Style::default()`'s
    ///    value, which is a *light-theme* grey — so under the dark preset
    ///    each unassigned fill was an invisible-text site waiting for the
    ///    right widget to be used. The contrast gate would now refuse the
    ///    theme, but only because there is nothing left unassigned for it
    ///    to miss.
    /// 3. **The foreground on the accent is [`Palette::on_accent`]**, not
    ///    `label_backdrop`. See that field's doc comment: reaching for a
    ///    content-facing plate colour to serve as chrome text is the
    ///    category error that made the pairing invisible in the first
    ///    place.
    ///
    /// The regression test is
    /// `every_rendered_widget_pair_is_readable_in_every_preset`, and its
    /// doc comment explains why the two tests that already existed could
    /// not have caught this.
    fn write_style(style: &mut egui::Style, p: &Palette, m: &Metrics, preset: Preset) {
        let v = &mut style.visuals;

        v.dark_mode = matches!(preset, Preset::Dark);
        v.override_text_color = Some(p.text);
        v.panel_fill = p.surface;
        v.window_fill = p.panel;
        v.extreme_bg_color = p.panel;
        v.faint_bg_color = p.panel;
        v.window_stroke = egui::Stroke::new(1.0, p.outline);
        v.selection.bg_fill = p.selection_fill;
        v.selection.stroke = egui::Stroke::new(1.0, p.accent);
        v.hyperlink_color = p.accent;
        v.error_fg_color = p.danger;
        v.warn_fg_color = p.notice;

        let radius = egui::CornerRadius::same(m.corner_radius);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = radius;
            w.bg_stroke = egui::Stroke::new(1.0, p.outline);
            w.fg_stroke = egui::Stroke::new(1.0, p.text);
        }

        // ★ D2. Both fills, every state, from the palette — see this
        // function's doc comment. `bg_fill` and `weak_bg_fill` are two
        // different backgrounds that different widgets choose between,
        // and a theme that sets one of them has themed an arbitrary
        // subset of its own widgets.
        //
        // Hover and active lift toward the accent rather than toward an
        // arbitrary grey, so the one accent is what the eye tracks.
        v.widgets.noninteractive.bg_fill = p.surface;
        v.widgets.noninteractive.weak_bg_fill = p.surface;
        v.widgets.inactive.bg_fill = p.panel;
        v.widgets.inactive.weak_bg_fill = p.panel;
        v.widgets.hovered.bg_fill = p.surface;
        v.widgets.hovered.weak_bg_fill = p.surface;
        v.widgets.active.bg_fill = p.accent;
        v.widgets.active.weak_bg_fill = p.accent;
        v.widgets.open.bg_fill = p.panel;
        v.widgets.open.weak_bg_fill = p.panel;
        // The one state whose foreground is not `text`: it is the one
        // state whose background is the accent.
        v.widgets.active.fg_stroke = egui::Stroke::new(1.0, p.on_accent);

        style.spacing.item_spacing = egui::vec2(m.gutter, m.gutter);
        style.spacing.button_padding = egui::vec2(m.gutter, m.gutter * 0.5);
        style.spacing.interact_size.y = m.control_height;
        style.spacing.window_margin = egui::Margin::same(clamp_to_i8(m.panel_padding));
    }
}

/// Clamp a points measurement into `egui::Margin`'s `i8` field.
///
/// `egui` stores margins as `i8`. A cast alone would wrap a 200 pt
/// padding to a negative margin, which paints content outside its own
/// panel — a silent geometry defect from a plausible-looking number. This
/// saturates instead, so an absurd metric produces an absurd-but-sane
/// margin that is visible on screen and traceable to its cause.
fn clamp_to_i8(pts: f32) -> i8 {
    let clamped = pts.clamp(f32::from(i8::MIN), f32::from(i8::MAX));
    // `as` is safe here: the value is provably inside i8's range, and a
    // NaN clamps to NaN which `as` maps to 0 — an acceptable answer for a
    // metric that was never a number.
    clamped as i8
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(Preset::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Crude relative luminance, matching [`contrast::luma`].
    ///
    /// Duplicated as a one-line local so the palette-level tests below
    /// read on their own; the rendered-pair gate uses the real one.
    fn luma(c: Color32) -> f32 {
        contrast::luma(c)
    }

    /// **Text is legible on the surface it is drawn on, in every
    /// preset.**
    ///
    /// Salvaged verbatim in intent. A crude relative-luminance gap rather
    /// than a full WCAG contrast ratio: the point is to catch a preset
    /// where someone set a light text colour against a light panel — which
    /// is what a `..quiet` spread does the moment a surface is darkened
    /// and the text is not — and a coarse check that always fires beats a
    /// precise one nobody runs.
    ///
    /// **This test is kept even though the rendered-pair gate subsumes
    /// most of it**, because it fails with a better message: it names the
    /// palette role that is wrong, where the gate names the widget state
    /// that renders wrong. Both are worth having when a preset is being
    /// edited.
    #[test]
    fn text_contrasts_with_its_background_in_every_preset() {
        for preset in Preset::ALL {
            let p = Theme::new(*preset).palette;
            for (name, bg) in [("surface", p.surface), ("panel", p.panel)] {
                let gap = (luma(p.text) - luma(bg)).abs();
                assert!(
                    gap > 90.0,
                    "{preset:?}: `text` on `{name}` has a luminance gap of {gap:.0}, \
                     which is not readable"
                );
            }
            let muted = (luma(p.text_muted) - luma(p.surface)).abs();
            assert!(
                muted > 45.0,
                "{preset:?}: `text_muted` on `surface` is too faint (gap {muted:.0})"
            );
        }
    }

    /// **The label backdrop stays light in every preset, including the
    /// dark one.**
    ///
    /// Salvaged. Labels sit over CONTENT, not over chrome, and the content
    /// is whatever colour the document says — overwhelmingly white. A dark
    /// theme that darkened the label backdrop would put dark text on a
    /// dark plate on a white page, which is unreadable in the one place it
    /// matters most.
    ///
    /// Worth a test because it is precisely the field a careless "make
    /// everything dark" edit would flip.
    ///
    /// **Note what this test does NOT do**, because it is half of why D2
    /// shipped: it asserts `label_backdrop` is light *and stops there*. It
    /// says nothing about what is behind `label_backdrop` when something
    /// draws with it, and in the salvage source something did — the active
    /// widget state's foreground. A test that pins a colour without
    /// pinning its pairing is a test that will agree with the bug.
    #[test]
    fn label_plates_stay_content_facing_not_chrome_facing() {
        for preset in Preset::ALL {
            let p = Theme::new(*preset).palette;
            assert!(
                p.label_backdrop.r() > 200 && p.label_backdrop.b() > 200,
                "{preset:?}: the label backdrop follows the content, not the chrome"
            );
            assert!(
                p.label_text.r() < 80,
                "{preset:?}: label text must be dark, to sit on that backdrop"
            );
        }
    }

    /// **★ `DEFECTS.md` D2's regression test: every foreground `egui`
    /// will actually paint is readable on the background it will actually
    /// paint it on — for all five widget states, both fills, all three
    /// presets.**
    ///
    /// # Why the two tests above could not have caught D2, and this can
    ///
    /// This is the important part of this test, and it generalises past
    /// theming.
    ///
    /// D2 was: `widgets.active.fg_stroke` was set to a near-white plate
    /// colour while `widgets.active.bg_fill` was never assigned the
    /// accent, so `CollapsingHeader` headers and dock tab labels rendered
    /// near-white on light grey. Two tests sat directly adjacent to it:
    ///
    /// - `text_contrasts_with_its_background_in_every_preset` checked
    ///   `text` against `surface` and `panel`. It never touched
    ///   `label_backdrop`, and `label_backdrop` was the foreground that
    ///   failed.
    /// - `label_plates_stay_content_facing_not_chrome_facing` **asserted
    ///   `label_backdrop` stays light** — correct for its stated purpose,
    ///   and it therefore *agreed with the defect*.
    ///
    /// Both are palette-vs-palette tests: they compare two colours a human
    /// deliberately wrote down next to each other. The defect was not in
    /// the palette. It was in the **assignment** — which palette entry
    /// ends up as a foreground and which as a background on the
    /// `egui::Style` that gets painted. No amount of checking the palette
    /// against itself can see that, because the pair that renders was
    /// never a pair anyone wrote down.
    ///
    /// A structural gate could not see it either. The project's
    /// `check-theme-colors.sh` bans raw `Color32` literals outside the
    /// theme module — a real and useful rule that says nothing about
    /// whether the named colours are legible together. As `DEFECTS.md`
    /// puts it: *the gate is structural, not perceptual.*
    ///
    /// So this test reads the **rendered style** back and enumerates the
    /// pairs as `egui` resolves them. Its coverage is defined by the
    /// widget-state matrix rather than by anyone's list, which means a
    /// fill someone forgets to assign in a future preset is caught by the
    /// same assertion that catches a fill someone assigns wrongly. That
    /// property — *the test enumerates the render surface, not the
    /// author's intentions* — is the transferable lesson.
    ///
    /// # On the threshold
    ///
    /// 90 on a 0–255 crude luminance scale, the same figure the salvaged
    /// text test uses, and for the same reason: a coarse check that always
    /// fires beats a precise one nobody runs. It is not a WCAG ratio and
    /// does not claim to be. The values it passes are comfortable — the
    /// tightest real pair in the shipped presets is `on_accent` on
    /// `accent` in the dark preset, at roughly 125.
    #[test]
    fn every_rendered_widget_pair_is_readable_in_every_preset() {
        for preset in Preset::ALL {
            let theme = Theme::new(*preset);
            if let Err(failures) = theme.check_contrast(contrast::READABLE_LUMA_GAP) {
                let detail = failures
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n  ");
                panic!(
                    "{preset:?}: {} rendered widget pair(s) are not readable. \
                     A pair here is a foreground egui WILL paint on a background egui \
                     WILL paint it on — not two palette entries someone chose together, \
                     which is the distinction that let DEFECTS.md D2 ship past two \
                     adjacent tests:\n  {detail}",
                    failures.len()
                );
            }
        }
    }

    /// **The gate is not vacuous: it fails on the defect it was written
    /// for.**
    ///
    /// Without this, `every_rendered_widget_pair_is_readable_in_every_preset`
    /// would pass identically if [`contrast::check`] returned `Ok` for
    /// everything, and would be asserting nothing at all. So this
    /// reconstructs D2 exactly — a light foreground on the active state
    /// with its `bg_fill` left at `egui`'s default — and asserts the gate
    /// catches it and *names the state*.
    ///
    /// This is the same discipline the salvage source applied to its
    /// script-parser tests: a test that proves a typo is rejected is worth
    /// nothing beside a test that proves the correct spelling is accepted.
    #[test]
    fn the_contrast_gate_catches_the_exact_defect_it_was_written_for() {
        let mut style = egui::Style::default();
        // egui's default light `bg_fill` for the active state, i.e. what
        // D2 left in place because nothing assigned it.
        style.visuals.widgets.active.bg_fill = Color32::from_gray(0xC8);
        style.visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(250, 250, 250, 220));

        let failures = contrast::check(&style, contrast::READABLE_LUMA_GAP)
            .expect_err("near-white on light grey must fail the gate");
        assert!(
            failures
                .iter()
                .any(|f| f.state == contrast::WidgetState::Active
                    && f.fill == contrast::FillKind::BgFill),
            "the gate must name the widget state and the fill that failed, \
             so the message points at the line to change; got: {failures:?}"
        );
    }

    /// **`on_accent` is a role, and the presets prove it has to be.**
    ///
    /// If every preset's `on_accent` were the same light colour, the field
    /// would be a constant wearing a role's clothes and the next editor
    /// would be right to inline it — reintroducing exactly the coupling
    /// that made D2 invisible. The dark preset inverts it, and that is the
    /// standing evidence the separation is load-bearing.
    #[test]
    fn on_accent_inverts_where_the_accent_is_light() {
        let quiet = Theme::new(Preset::Quiet).palette;
        let dark = Theme::new(Preset::Dark).palette;
        assert!(
            luma(quiet.on_accent) > luma(quiet.accent),
            "a dark accent takes a light foreground"
        );
        assert!(
            luma(dark.on_accent) < luma(dark.accent),
            "a light accent takes a dark foreground — which a single shared \
             plate colour could never have expressed, and is why `on_accent` \
             is a role rather than a constant"
        );
    }

    /// Settings keys round-trip, and an unknown key is `None` rather than
    /// a silent default. Salvaged.
    #[test]
    fn preset_keys_round_trip_and_unknown_keys_are_refused() {
        for preset in Preset::ALL {
            assert_eq!(Preset::from_key(preset.key()), Some(*preset));
        }
        assert_eq!(Preset::from_key("solarized"), None);
        assert_eq!(Preset::from_key(""), None);
    }

    /// **An absurd metric produces a sane margin.**
    ///
    /// `egui::Margin` is `i8`. A plain cast turns 200 pt of padding into
    /// −56, which paints content outside its own panel: a silent geometry
    /// defect produced by a number that looked fine where it was typed.
    #[test]
    fn an_out_of_range_panel_padding_saturates_rather_than_wrapping() {
        assert_eq!(clamp_to_i8(6.0), 6);
        assert_eq!(clamp_to_i8(200.0), 127);
        assert_eq!(clamp_to_i8(-200.0), -128);
        assert_eq!(clamp_to_i8(f32::NAN), 0);
    }
}
