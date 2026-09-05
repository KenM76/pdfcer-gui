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
//! salvaging rather than re-deriving. See `DEFECTS.md` D2 and the [`tests`]
//! module (split out of this file under rule R2, which caps a `.rs` file at
//! 1500 lines): the original had two contrast tests, both
//! of which compared *palette entries chosen by a human as a pair*, and
//! the shipped defect was in the *assignment* — which palette entry ended
//! up as a foreground and which as a background on the actual
//! `egui::Style`. The gate closes that by reading the style back.
//!
//! ★★ **It enumerated ten pairs until 2026-09-04 and was green through
//! three separately shipped contrast defects** (`REVIEW_TRIAGE.md` A15e),
//! because none of the three was one of the ten. It now enumerates
//! twenty-seven: the original widget matrix plus every foreground `egui`
//! resolves through a `Visuals` *accessor* — body text, weak text, strong
//! text, hyperlink, warn, error — on the grounds each is really drawn on,
//! plus the two roles `visuals.selection` serves. [`contrast`]'s own
//! header states which of the three defects the widening reaches and
//! which two it deliberately still cannot, with the gates that do cover
//! those. Knowing what a gate cannot see is part of the gate.
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
    /// **The OPAQUE plate `egui` paints behind a SELECTED WIDGET**, with
    /// [`Self::accent`] as the ink on it.
    ///
    /// This is the value handed to `egui::Visuals::selection.bg_fill`. Every
    /// `ui.selectable_label(true, …)`, every `Button::selected(true)`, the
    /// highlight behind selected text in a `TextEdit`, a `ProgressBar`'s fill
    /// and a `Slider`'s trail are painted with it — `egui` substitutes it at
    /// paint time (`widget_style.rs:151-154`), so it reaches all of them
    /// together whether or not any call site mentions it.
    ///
    /// # ★★★ WHAT THIS IS NOT: it is not [`Self::selection_fill`]
    ///
    /// Read that sentence twice, because merging these two fields back
    /// together would undo defect **T2**'s entire fix and would do it
    /// silently.
    ///
    /// [`Self::selection_fill`] is the **27 %-alpha CANVAS wash** — the tint
    /// laid over a selected object *on the document*, where seeing the object
    /// through the tint is the whole point. It is translucent on purpose, and
    /// a translucent fill is not a dimmer plate: it is a different colour over
    /// every background it meets. Pointing `egui`'s widget channel at it is
    /// what painted nineteen chrome controls with canvas ink on a wash
    /// (luminance gap 72.5 in Dark, floor 90), and pointing a dialog's
    /// affirmative button at it is what made the default action render *paler
    /// than the Cancel beside it*.
    ///
    /// So: **this role is chrome and opaque; `selection_fill` is content and
    /// translucent.** They will often look like relatives — both are the
    /// accent, diluted — and that resemblance is exactly the trap. They must
    /// be able to move independently, because the thing behind them is
    /// different (a panel, whose colour the theme knows; a page, whose colour
    /// the document decides).
    ///
    /// # ★★ Why a plate at all, rather than [`Self::accent`] itself
    ///
    /// Because one channel has to serve two roles and `egui` gives no way to
    /// separate them. `visuals.selection.stroke` is BOTH the ink on this plate
    /// AND the frame stroke of a **focused, mutable `TextEdit`**
    /// (`widgets/text_edit/builder.rs:699-706`), which is drawn on
    /// `text_edit_bg_color()` — [`Self::panel`] in this theme. `TextEdit` has
    /// no `.frame_stroke()`, so there is no per-widget override to escape with.
    ///
    /// With `bg_fill = accent`, the ink had to be [`Self::on_accent`] (a
    /// near-white plate colour under the light presets) and the focus ring
    /// therefore became near-white on a near-white panel: gaps of
    /// **17.9 / 5.0 / 29.1** across Quiet / Airy / Dark. A focused field
    /// looked unfocused. That is `DEFECTS.md` D2's shape for the fourth time.
    ///
    /// Making the *plate* the diluted value instead of the *ink* dissolves the
    /// conflict, because the ink is then [`Self::accent`] — which is already
    /// far from the panel by construction. See [`Theme::write_style`] for the
    /// full six-number derivation.
    pub selected_plate: Color32,
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

    // -----------------------------------------------------------------
    // ★★★ THE RIBBON BAND'S OWN RHYTHM — `mockups/pdfcer-shell.html`
    //
    // Five numbers that exist because the band is the one surface in this
    // application whose vertical proportions were **specified as a
    // picture** rather than derived from `control_height`. The operator's
    // instruction on 2026-09-04 was *"I want everything to look exactly
    // like that including sizing"*, and the mockup's stylesheet is the
    // readable form of "that":
    //
    // ```css
    // .app  { grid-template-rows: 30px 34px 96px 26px … }   /* the band */
    // .ribbon        { padding: 6px 8px 0 }
    // .grp .cap      { font-size: 11px; padding: 3px 0 5px; line-height: 1.1 }
    // .grp .col      { gap: 1px }
    // .rb            { height: 22px }
    // .rb.big        { height: 56px; gap: 4px; padding: 5px 8px 2px;
    //                  min-width: 52px }
    // .rb.big .lb    { font-size: 11px; max-width: 76px }
    // svg.g          { width: 16px }      /* an ordinary control's icon */
    // svg.g.big      { width: 24px }      /* a Large control's icon     */
    // ```
    //
    // ★ Why they live HERE rather than as constants in `ribbon::band`.
    //
    // Because they are theme-dependent and the two other presets prove
    // it. `Airy` exists to be *roomier* — its `control_height` is 28 pt
    // against `Quiet`'s 24 — and a band whose row area were a bare `68.0`
    // would be **shorter than two of its own rows** under that preset
    // (2 × (28 + 8) = 72), so the second row would draw over the caption.
    // A number that has to move when the preset moves is a metric, not a
    // constant, and `every_preset_reserves_room_for_its_own_rows` is the
    // assertion that keeps the relationship true rather than merely
    // true-today.
    //
    // ★★ What is deliberately NOT here: a hover colour. The mockup's
    // `--chrome-3` is commented `/* hover, derived */` in its own palette
    // block — the mock author's arithmetic, not a role this theme
    // publishes. `write_style` already lifts hover toward `surface` with a
    // stated reason ("hover and active lift toward the accent … so the one
    // accent is what the eye tracks"), and inventing a `hover_fill` role
    // to match a derived swatch would be re-deciding a settled question
    // for a picture's sake.
    /// **The band's control-row area**, in points: the height a group's
    /// rows are padded out to before its caption is drawn.
    ///
    /// The mockup's tallest authored column is three `.rb` rows with 1 px
    /// between them — `3 × 22 + 2 × 1 = 68` — and every group's caption
    /// hangs off the bottom of that area (`\.cap { margin-top: auto }`),
    /// which is what makes the captions in a band share one baseline
    /// whether the group above used one row or three.
    ///
    /// ★ It is **not** `GROUP_ROWS × (control_height + gutter)`, which is
    /// what it used to be and is why the shipped band read as cramped
    /// against the mock: that expression is "exactly as tall as two rows",
    /// so a two-row group filled it edge to edge and the caption sat
    /// immediately under the last control. The mockup's area is a
    /// *budget* the rows are laid into, and the slack under a short group
    /// is the thing the operator described as "the group caption sitting
    /// lower".
    ///
    /// # Invariant
    ///
    /// `ribbon_rows ≥ GROUP_ROWS × (control_height + gutter)`. A preset
    /// that violates it draws its second row over its own caption. Asserted
    /// per preset by `crate::ribbon::height_tests`.
    pub ribbon_rows: f32,
    /// **Clear space above the band's first control row** —
    /// `.ribbon { padding: 6px … }`.
    ///
    /// Folded into [`crate::ribbon`]'s band height rather than emitted as a
    /// stray `add_space`, for the reason R128 gives about every other term
    /// in that sum: a padding that is only drawn when there is something to
    /// pad would be absent on a tab whose groups all went to the overflow
    /// menu, and the canvas underneath would move by six points on a tab
    /// click.
    pub ribbon_pad_top: f32,
    /// **The band's secondary text size**, in points: group captions, and
    /// the label under a `Large` control.
    ///
    /// One metric for both because the mockup gives them one number
    /// (`.cap` and `.rb.big .lb` are both `font-size: 11px`) for one
    /// reason — they are the two pieces of text on the band that are
    /// *subordinate* to a control's own label, and they must not compete
    /// with it. Splitting them into two metrics would invite them to
    /// drift apart, which is a distinction no reader of the band could
    /// name.
    ///
    /// ★ This is a **raise**, not a reduction. The captions were drawn
    /// with `RichText::small()`, i.e. `egui`'s `TextStyle::Small`, which
    /// is 9 pt — the mockup asks for 11.
    pub ribbon_caption_pts: f32,
    /// **The icon size on a `Large` control** — `svg.g.big { width: 24px }`.
    ///
    /// Half again the ordinary [`Self::icon_pts`], and that ratio is the
    /// entire visual argument for a Large control: it is not a taller
    /// button with the same picture in it, it is a **bigger picture**. A
    /// Large control drawn with a 16 pt glyph reads as an ordinary control
    /// with an unusual amount of air around it, which is exactly how the
    /// shipped band looked beside the mock.
    pub ribbon_icon_large_pts: f32,
    /// **The height of a `Large` control** — `.rb.big { height: 56px }`.
    ///
    /// Deliberately **less than** [`Self::ribbon_rows`]: in the mockup a
    /// Large control is 56 px inside a 68 px row area, top-aligned
    /// (`.grp .items { align-items: flex-start }`). It spans *most* of the
    /// band rather than all of it, which is what stops a group made only
    /// of Large controls from reading as a solid block.
    pub ribbon_large_pts: f32,
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
                // ★ DERIVED, not picked: `accent` at 30 % over `panel`,
                // composited here so the value that ships is OPAQUE.
                //
                //   0.30·(23, 92,196) + 0.70·(232,232,234)
                //     = (169.3, 190.0, 222.6) → (169, 190, 223)
                //
                // luma 187.9. Ink (`accent`, luma 84.8) reads on it at a gap
                // of **103.1**; it separates from `panel` (232.1) by 44.2, so
                // the plate is visible as a plate. 30 % is the strongest tint
                // that still clears the floor with headroom in this preset —
                // 35 % falls to 95.8 and 40 % to 88.3, i.e. below 90. Airy
                // shares the ratio and lands easier because its panel is white.
                selected_plate: Color32::from_rgb(0xA9, 0xBE, 0xDF),
                label_backdrop: Color32::from_rgba_unmultiplied(250, 250, 250, 220),
                label_text: Color32::from_rgb(20, 20, 20),
            },
            metrics: Metrics {
                control_height: 24.0,
                gutter: 4.0,
                panel_padding: 6.0,
                corner_radius: 3,
                icon_pts: 16.0,
                // The mockup's own numbers, unscaled — `Quiet` is the preset
                // the mock was drawn against, so these are transcriptions
                // rather than derivations. 68 = 3 × 22 + 2 × 1, and it clears
                // this preset's two natural rows (2 × (24 + 4) = 56) by 12 pt,
                // which is the slack the caption hangs below.
                ribbon_rows: 68.0,
                ribbon_pad_top: 6.0,
                ribbon_caption_pts: 11.0,
                ribbon_icon_large_pts: 24.0,
                ribbon_large_pts: 56.0,
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
                // ★ Same derivation as Quiet — `accent` at 30 % — over THIS
                // preset's panel, which is pure white:
                //
                //   0.30·(23, 92,196) + 0.70·(255,255,255)
                //     = (185.4, 206.1, 237.3) → (185, 206, 237)
                //
                // luma 203.8; ink gap **118.9**, separation from `panel` 51.2.
                // The ratio is inherited, the value is not, because a tint is
                // a relationship to the ground under it and Airy's ground is
                // fifteen levels lighter than Quiet's. A shared literal would
                // have been a coincidence rather than a rule.
                selected_plate: Color32::from_rgb(0xB9, 0xCE, 0xED),
                ..quiet.palette
            },
            metrics: Metrics {
                control_height: 28.0,
                gutter: 8.0,
                panel_padding: 12.0,
                corner_radius: 6,
                icon_pts: 17.0,
                // ★★★ NOT the mockup's literals, and this preset is the proof
                // that the ribbon rhythm had to be a metric.
                //
                // `Airy` exists to be roomier: `control_height` 28 against
                // Quiet's 24, `gutter` 8 against 4. Two of its natural rows
                // cost `2 × (28 + 8) = 72` pt — **more than the mockup's 68**
                // — so transcribing 68 here would lay the second row of every
                // wrapped group over its own caption. 84 = 3 × 26 + 2 × 3,
                // the same three-rows-and-two-gaps shape at this preset's
                // scale, and it clears 72 by the same proportion Quiet's 68
                // clears 56.
                //
                // The rest scale with it: the caption keeps the mock's
                // one-point-below-`Body` relationship, the Large icon keeps
                // the 1.5× ratio to `icon_pts` (17 × 1.5 = 25.5 → 26, kept
                // even so the glyph raster lands on a whole point), and the
                // Large control keeps its "most of the row area, not all of
                // it" proportion — 64/84 against Quiet's 56/68.
                ribbon_rows: 84.0,
                ribbon_pad_top: 8.0,
                ribbon_caption_pts: 12.0,
                ribbon_icon_large_pts: 26.0,
                ribbon_large_pts: 64.0,
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
                // ★★ RAISED FROM `#FF6B6B` ON 2026-09-04, BY THE PAIR-GATE
                // WIDENING (`REVIEW_TRIAGE.md` A15e). It is the one shipped
                // value that widening actually caught.
                //
                // `Theme::write_style` hands this role to
                // `visuals.error_fg_color`, which every dialog reads for the
                // line that says the operator must act — and a dialog is
                // painted on `window_fill`, i.e. on `panel`. The old value
                // measured:
                //
                //   luma(#FF6B6B) = 0.2126·255 + 0.7152·107 + 0.0722·107
                //                 = 138.46
                //   luma(panel)   =  48.72   ⇒ gap 89.74, floor 90.
                //
                // ★ It is a MARGINAL miss and the arithmetic says why: the
                // crude Rec. 709 measure weights red at 0.2126, so a
                // saturated red scores far below how it reads. Under WCAG the
                // same pair is 4.71:1 — a comfortable AA pass. The colour was
                // not invisible; the gate was right at the edge of its own
                // resolution.
                //
                // ★★ It was still fixed at the ROLE rather than exempted, and
                // the reason is not the 0.26 itself: it is that 0.26 of
                // headroom is not headroom. `panel` and `accent` are both
                // live values in this preset — `selected_plate` is derived
                // from `accent`, and Dark's focus ring already clears the
                // floor by six — so the next chrome edit would have spent it
                // silently, and the pair would have crossed the line inside a
                // change about something else. An exemption would also have
                // been the wrong shape: exemptions are for pairs no theme
                // value can satisfy (see `contrast::EXEMPTIONS`), and this one
                // is satisfied by twelve levels of green and blue.
                //
                //   luma(#FF7B7B) = 0.2126·255 + 0.7152·123 + 0.0722·123
                //                 = 151.06
                //   ⇒ on `panel` (48.72)   gap 102.34
                //   ⇒ on `surface` (37.86) gap 113.20
                //
                // The hue is unchanged (still a pure-red-channel salmon); only
                // its lightness moved, by 16 levels on two channels, which is
                // below the threshold at which the colour stops reading as the
                // same warning red. The light presets keep `#C02A2A` from
                // `..quiet.palette`: their grounds are light, so the DARK red
                // is the one that separates, and this preset's problem is the
                // mirror of theirs.
                danger: Color32::from_rgb(0xFF, 0x7B, 0x7B),
                notice: Color32::from_rgb(0xE0, 0xA0, 0x40),
                // ★★★ THE ONE PRESET WHERE THE LIGHT PRESETS' DERIVATION
                // CANNOT BE USED, AND THE ARITHMETIC THAT PROVES IT.
                //
                // Still derived from `accent`, but from `accent` ALONE:
                //
                //   0.15·(76,154,255) = (11.4, 23.1, 38.25) → (11, 23, 38)
                //
                // i.e. the accent at 15 % of its own intensity — the same hue,
                // deepened, rather than the same hue diluted.
                //
                // Why not "30 % over `panel`" like the other two. Because in
                // this preset the accent (luma 144.7) is LIGHTER than the panel
                // (48.7), so mixing toward the panel moves the plate the wrong
                // way: it lands BETWEEN ink and panel, and the ink gap collapses
                // — 81.3 at 15 %, 76.8 at 20 %, 66.9 at 30 %, all under the
                // floor of 90. The ceiling on that whole family is 96.0, reached
                // only at 0 % where the "plate" IS the panel and there is no
                // plate at all. The plate must therefore go past the panel, to
                // the far side.
                //
                // The result: luma 21.5. Ink (`accent`, 144.7) reads on it at a
                // gap of **123.2** — the widest of the three presets — and it
                // separates from `panel` by 27.2 and from `surface` (37.9) by
                // 16.4, so a selected row reads as a recessed well rather than
                // as a lit one. 20 % was measured too (ink gap 115.7) and
                // rejected: it buys nothing and halves the separation from the
                // panel to 19.7.
                //
                // ⚠ This is the preset to re-measure first if the accent ever
                // changes. Its focus ring — `accent` on `panel` — clears the
                // floor by six (96.0), the tightest number in the theme.
                selected_plate: Color32::from_rgb(0x0B, 0x17, 0x26),
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

    /// **The accent fill and the foreground guaranteed to read on it**, as one
    /// value, for anything that paints an "this is the default action" surface.
    ///
    /// Returns `(accent, on_accent)`. Use it for an affirmative dialog button,
    /// an active tab, a selected mode chip — any chrome that must look
    /// *emphatically enabled*.
    ///
    /// # ★★★ Why this exists as a function rather than two field reads
    ///
    /// Because the two fields have been separated twice, and both times the
    /// result was a surface that shipped looking broken while every gate stayed
    /// green.
    ///
    /// 1. **DEFECTS.md D2** — the active ribbon tab took `egui`'s *selection*
    ///    visuals and a plate colour meant for content, and rendered near-white
    ///    on light grey. [`Palette::on_accent`]'s own doc comment records the
    ///    root cause: two roles that must vary independently had been welded
    ///    together.
    /// 2. **The operator's print-dialog report, 2026-09-03** — the affirmative
    ///    button in *every* dialog was filled with
    ///    [`Palette::selection_fill`], a **27 %-opacity** wash whose real job is
    ///    tinting selected objects on a canvas. Over a light panel it
    ///    composites *paler than an ordinary button's opaque fill*, so the
    ///    default action looked **disabled**. He pressed it a dozen times and
    ///    found a dozen queued print jobs.
    ///
    /// Both were correctly sourced from the theme. Neither used a literal, so
    /// `tools/gates/check-theme-colors.sh` — which forbids raw `Color32`
    /// outside this module — had nothing to say about either. **The rule that
    /// gate enforces is "no invented colours"; it cannot enforce "the right
    /// role".** A named pair is the mechanism that can: there is now one
    /// spelling of *"paint something as the emphasised action"*, and a preset
    /// that changes its accent moves every such surface together.
    ///
    /// ★ Not `strong_text_color()` for the foreground. That follows
    /// `override_text_color`, which is the **body text** colour — near-black
    /// under the light presets. On a saturated accent that is poor contrast,
    /// and under a preset whose accent is dark it would be black on black.
    /// [`Palette::on_accent`] is the theme's own answer and inverts per preset.
    ///
    /// ★ Deliberately NOT `selection.bg_fill`, and the reason has changed
    /// shape twice without changing conclusion. It was wrong when that channel
    /// carried [`Palette::selection_fill`] — a 27 % wash, incident 2 above —
    /// and it is still wrong now that it carries [`Palette::selected_plate`],
    /// because a plate is a *diluted* accent chosen to be readable under
    /// `accent` INK. An emphasised action wants the accent at full strength
    /// with [`Palette::on_accent`] on it. Asking for `selection.bg_fill` gets
    /// you whichever of those two stories the widget channel is telling this
    /// month, which is the entire argument for naming the pair.
    #[must_use]
    pub fn accent_pair(ctx: &egui::Context) -> (egui::Color32, egui::Color32) {
        let theme = Self::of(ctx);
        (theme.palette.accent, theme.palette.on_accent)
    }

    /// **The ink `egui` paints on a selected widget's plate** — the colour a
    /// selected control's own drawing must match.
    ///
    /// Returns [`Palette::accent`], which is what
    /// `visuals.selection.stroke.color` carries. Use it only where a call site
    /// hand-draws something *inside* a control `egui` has already styled as
    /// selected, and therefore has to match a colour it did not choose: a
    /// tinted glyph on a toggle, a hand-painted chevron, a custom check mark.
    ///
    /// # ★★ What to use instead, almost always: nothing
    ///
    /// `egui` styles a selected widget correctly on its own — it substitutes
    /// both fills and the text colour out of this channel at paint time
    /// (`widget_style.rs:151-154`). A `ui.selectable_label(true, …)` needs no
    /// colour from anyone. Reach for this **only** when you are drawing an
    /// extra mark on top of a plate `egui` painted, because that mark is the
    /// one thing `egui` cannot colour for you.
    ///
    /// # What this is NOT
    ///
    /// Not [`Theme::canvas_selection_ink`], although today both return
    /// `accent`. That equality is a coincidence of the current palette and not
    /// a contract: one is the ink on a chrome plate, the other is the outline
    /// over a document page, and the whole point of defect T2's fix was that
    /// those two must be able to move apart. Calling the wrong one is how the
    /// canvas silently re-tunes when the chrome is re-tuned.
    ///
    /// Not [`Theme::accent_pair`] either — that is the *emphasised action*
    /// pair (`accent` + `on_accent`), a stronger surface than "selected".
    #[must_use]
    pub fn selected_widget_ink(ctx: &egui::Context) -> egui::Color32 {
        Self::of(ctx).palette.accent
    }

    /// **The plate `egui` paints behind a selected widget and the ink on it**,
    /// as `(plate, ink)`.
    ///
    /// Returns `(selected_plate, accent)` — bit-for-bit what
    /// [`Theme::write_style`] puts into `visuals.selection`. The pair form
    /// exists for the same reason [`Theme::accent_pair`] does: the two values
    /// are only correct *together*, and a call site that paints its own
    /// selected surface should state both in one breath rather than fetch a
    /// fill here and a foreground there. That is precisely how D2 happened,
    /// three times.
    #[must_use]
    pub fn selected_widget_pair(ctx: &egui::Context) -> (egui::Color32, egui::Color32) {
        let theme = Self::of(ctx);
        (theme.palette.selected_plate, theme.palette.accent)
    }

    /// **The ink a selected thing in the CONTENT AREA is outlined with.**
    ///
    /// Outlines, node marks, grips, rubber-band borders, drop carets, ruler
    /// span markers, the selected form field's box — everything drawn *over
    /// the document* to say "this is what you have picked".
    ///
    /// # ★★★ Why this is a named function and not `visuals().selection.stroke`
    ///
    /// Because that is where it used to be read from, and it was the wrong
    /// address. `egui::Visuals::selection` is `egui`'s styling channel for
    /// **selected widgets** — see [`Theme::write_style`], which quotes the
    /// four lines of `egui-0.35.0/src/widget_style.rs` that substitute it into
    /// every `Button::selected(true)`. For as long as the theme pointed that
    /// channel at the canvas, the canvas won and every selected chrome control
    /// in the application was painted with canvas ink: accent text on a 27 %
    /// wash, luminance gap 72.5 in the Dark preset against a floor of 90.
    ///
    /// The standing lesson this project keeps re-learning, in its own words:
    /// *a correctly-sourced value used for the wrong role passes every gate —
    /// expose the PAIR behind a purpose-named function.* [`Theme::accent_pair`]
    /// is that mechanism for chrome; this and [`Theme::canvas_selection_fill`]
    /// are it for content. A call site that asks for
    /// `canvas_selection_ink` cannot accidentally be asking for the chrome
    /// role, because the two questions now have different spellings.
    ///
    /// ★ "Canvas" here means the application's content area — the region
    /// [`Palette::content_backdrop`] sits behind. The shell has no opinion
    /// about what is drawn there.
    ///
    /// # What this is NOT for
    ///
    /// **Never for chrome.** A selected dock tab, a toggled ribbon button, a
    /// pressed mode chip, a highlighted menu row: those are widgets, `egui`
    /// already styles them from `visuals.selection`, and if one needs to state
    /// its own colours the pair is [`Theme::accent_pair`]. Using canvas ink on
    /// chrome is the *inverse* of the defect above and produces the same
    /// class of failure — a colour chosen against a background nobody paired
    /// it with.
    ///
    /// It also may not be written into a document. See this module's header:
    /// if a colour can end up in a saved file it is not a theme colour.
    #[must_use]
    pub fn canvas_selection_ink(ctx: &egui::Context) -> egui::Color32 {
        Self::of(ctx).palette.accent
    }

    /// **The translucent tint a selected object in the CONTENT AREA is
    /// washed with**, and the fill of the rubber-band that is picking one.
    ///
    /// Returns [`Palette::selection_fill`], which is **27 % alpha on
    /// purpose**: seeing the object through the tint is the entire point of a
    /// selection wash over a drawing. That is also precisely why it is unfit
    /// for chrome — a translucent fill is not a dimmer accent, it is a
    /// different colour over every background it meets, which is how a
    /// dialog's default button once rendered paler than the Cancel beside it
    /// (see [`Theme::accent_pair`], incident 2).
    ///
    /// # What this is NOT for
    ///
    /// Not a widget fill, not a button, not a tab, not a plate to draw
    /// [`Palette::on_accent`] on. See [`Theme::canvas_selection_ink`] for the
    /// full argument and for why reaching into `visuals().selection.bg_fill`
    /// is now a gate failure (`tools/gates/check-selection-channel.sh`).
    #[must_use]
    pub fn canvas_selection_fill(ctx: &egui::Context) -> egui::Color32 {
        Self::of(ctx).palette.selection_fill
    }

    /// **Both content-area selection roles at once**, as `(ink, fill)`.
    ///
    /// The pair form exists for the same reason [`Theme::accent_pair`] does:
    /// the two values are only correct *together*, and a call site that draws
    /// a washed rectangle with an outline around it should state them in one
    /// breath rather than fetch two unrelated-looking colours. Use it wherever
    /// both are needed; use the singles where only one is.
    #[must_use]
    pub fn canvas_selection_pair(ctx: &egui::Context) -> (egui::Color32, egui::Color32) {
        let theme = Self::of(ctx);
        (theme.palette.accent, theme.palette.selection_fill)
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
    /// `every_rendered_pair_is_readable_in_every_preset`, and its
    /// doc comment explains why the two tests that already existed could
    /// not have caught this.
    ///
    /// ★ It was called `..._widget_pair_...` until 2026-09-04, when
    /// `REVIEW_TRIAGE.md` A15e widened [`contrast::pairs`] from ten pairs
    /// to twenty-seven. The word was dropped because it had stopped being
    /// true, and the citation was updated here at the same time — a
    /// renamed test cited by an old name is the drift
    /// `every_declared_share_is_still_a_share` exists to shame.
    fn write_style(style: &mut egui::Style, p: &Palette, m: &Metrics, preset: Preset) {
        let v = &mut style.visuals;

        v.dark_mode = matches!(preset, Preset::Dark);
        v.override_text_color = Some(p.text);
        v.panel_fill = p.surface;
        v.window_fill = p.panel;
        v.extreme_bg_color = p.panel;
        v.faint_bg_color = p.panel;
        v.window_stroke = egui::Stroke::new(1.0, p.outline);
        // ★★★ `visuals.selection` IS EGUI'S WIDGET CHANNEL. IT IS NOT THE
        // CANVAS'S. — defect T2, `REVIEW_TRIAGE.md` §2b, fixed 2026-09-04.
        //
        // These two lines used to read:
        //
        // ```text
        // v.selection.bg_fill = p.selection_fill;                  // a 27 % wash
        // v.selection.stroke  = Stroke::new(1.0, p.accent);        // canvas ink
        // ```
        //
        // which handed `egui`'s **selected-widget** styling channel to the
        // *content area*. That is not a stylistic preference; it is a
        // documented `egui` contract, and the consequence is mechanical.
        // `egui-0.35.0/src/widget_style.rs:151-154`, verbatim:
        //
        // ```text
        // if classes.has(SELECTED_CLASS) {
        //     visuals.weak_bg_fill = self.visuals.selection.bg_fill;
        //     visuals.bg_fill      = self.visuals.selection.bg_fill;
        //     visuals.fg_stroke    = self.visuals.selection.stroke;
        //     ws.text.color        = self.visuals.selection.stroke.color;
        // }
        // ```
        //
        // So **every** bare `ui.selectable_label(true, …)` and every
        // `Button::selected(true)` in the application — nineteen of them at
        // the time of writing, across the ribbon, the menus and eight panels —
        // painted `accent`-coloured text on a 27 %-alpha wash. Measured
        // luminance gap in the Dark preset: **72.5**, against this module's
        // own readable floor of 90 ([`contrast::READABLE_LUMA_GAP`]). Not one
        // of those call sites is wrong. They ask `egui` for "selected"; the
        // theme was answering with the wrong pair.
        //
        // ★★ Why this was invisible to every gate we owned ON THE DAY, and
        // read the tense: this paragraph describes a state of the world that
        // has since been changed on purpose, and it must not be cited as a
        // present fact. `check-theme-colors` forbids **invented** colours, and
        // both values were correctly sourced from the palette. `contrast::pairs`
        // enumerated the five widget states × two fills, reading `fg_stroke`
        // against `bg_fill` — and the selected pair was in none of them,
        // because `egui` substitutes it *after* the style is read. The colours
        // were named, the gate was green, and the surface was unreadable. That
        // is the third time this exact shape has shipped (`DEFECTS.md` D2), and
        // it is why `tools/gates/check-selection-channel.sh` exists.
        //
        // ⇒ **AMENDED 2026-09-04 by `REVIEW_TRIAGE.md` A15e: the gate can see
        // it now.** `contrast::pairs` reproduces the substitution above and
        // enumerates both roles this channel serves —
        // `contrast::Origin::SelectedWidget` over each ground, and
        // `Origin::FocusRing` — so re-pointing this channel at a wash fails
        // `every_rendered_pair_is_readable_in_every_preset` rather than
        // shipping. `check-selection-channel.sh` remains the gate over CALL
        // SITES reading `visuals.selection` directly, which is a different
        // question and still not one a `Style` can answer.
        //
        // ★★★ The reason this correction is written in rather than the old
        // sentence being deleted: the old sentence was TRUE and became FALSE,
        // and a paragraph that says "the gate cannot see X" is exactly the kind
        // of premise a future reader cites to justify not adding a check. That
        // is `REVIEW_TRIAGE.md` T1's whole lesson, one file over.
        //
        // ★ The pair below is `egui`'s own design for the channel — its stock
        // light theme pairs a pale blue `bg_fill` with a dark blue `stroke`,
        // i.e. *a plate and the ink that reads on it*, never a translucent
        // tint. This theme now says the same thing in its own palette's words:
        // [`Palette::selected_plate`] and [`Palette::accent`].
        //
        // ★★★ AND THE CANVAS DID NOT LOSE ANYTHING. The ~33 content-area
        // readers that used to reach through this channel now call
        // [`Theme::canvas_selection_ink`] and [`Theme::canvas_selection_fill`],
        // which return `accent` and `selection_fill` — *the identical values
        // this channel used to carry*. The canvas renders pixel-for-pixel as
        // before; what changed is that its colours now arrive by a name that
        // says what they are for, so re-tuning chrome cannot silently re-tune
        // the page overlay again.
        //
        // ═══════════════════════════════════════════════════════════════════
        // ★★★ AND THE SECOND ROLE THIS CHANNEL SERVES: THE FOCUSED-TEXTEDIT
        // RING. THIS IS THE PART THAT MAKES THE PLATE A PLATE.
        // ═══════════════════════════════════════════════════════════════════
        //
        // `egui` reuses `selection.stroke` as the frame stroke of a **focused,
        // mutable `TextEdit`**. Verbatim, `widgets/text_edit/builder.rs:699-706`:
        //
        // ```text
        // let background_color = background_color
        //     .unwrap_or_else(|| ui.visuals().text_edit_bg_color());
        // let (corner_radius, background_color, stroke) = if text_mutable {
        //     if allocated.response.has_focus() {
        //         (visuals.corner_radius, background_color,
        //          ui.visuals().selection.stroke)
        // ```
        //
        // and `Visuals::text_edit_bg_color()` falls back to `extreme_bg_color`,
        // which this function points at [`Palette::panel`] eight lines up.
        // `TextEdit` exposes no `.frame_stroke()`, so there is NO per-widget
        // override: whatever is in this channel is the ring, everywhere.
        //
        // ⚠ THE FIRST ATTEMPT AT T2 PUT `on_accent` HERE AND BROKE THAT RING.
        // `on_accent` is a near-white plate colour under the light presets, so
        // the ring became near-white on a near-white panel — gaps of
        // **17.9 / 5.0 / 29.1** (Quiet / Airy / Dark); Airy is white on white
        // to within five levels of luminance, and a focused field looked
        // unfocused. `DEFECTS.md` D2's shape for the FOURTH time, and this one
        // was introduced by the fix for the third.
        //
        // ★★ THE ANALYSIS THAT SAID THE TWO ROLES WERE IRRECONCILABLE WAS
        // ARITHMETICALLY RIGHT AND STRUCTURALLY WRONG. It ran:
        //
        //   · an ink readable on `accent`  (luma 84.8) needs luma ≥ 174.8
        //   · a ring readable on `panel`   (luma 232.1) needs luma ≤ 142.1
        //   · ⇒ empty intersection, in both light presets.
        //
        // Every step holds — but only under the assumption that
        // `selection.bg_fill` IS `accent`. It does not have to be. Dilute the
        // PLATE instead of the INK and the same two constraints are satisfied
        // by one colour, because the ink is then `accent` itself, which is far
        // from the panel by construction (that is what an accent is for):
        //
        //   preset │ selected pair            │ focus ring
        //          │ accent on selected_plate │ accent on panel
        //   ───────┼──────────────────────────┼─────────────────
        //   Quiet  │ 84.8 vs 187.9 →  103.1   │  84.8 vs 232.1 → 147.3
        //   Airy   │ 84.8 vs 203.8 →  118.9   │  84.8 vs 255.0 → 170.2
        //   Dark   │ 144.7 vs 21.5 →  123.2   │ 144.7 vs  48.7 →  96.0
        //
        // — six numbers, floor 90, tightest 96.0. Both roles, one channel, no
        // call-site change and no hand-drawn ring. The per-preset derivations
        // and why Dark needs its own are on [`Palette::selected_plate`] and on
        // the three preset constructors; the assertion is
        // `both_roles_the_selection_channel_serves_are_readable_in_every_preset`.
        //
        // ★ Three consequences worth stating, since nothing at a call site
        // will announce them:
        //
        //  1. A SELECTED control no longer looks identical to a PRESSED one.
        //     `widgets.active` keeps the full `accent` + `on_accent` pair
        //     twenty lines down, so "you are pressing this" is now louder than
        //     "this one is on". That is the correct hierarchy — the first is
        //     momentary, the second is a persistent state — and it is what
        //     `egui`'s stock themes do.
        //  2. Selected TEXT inside a `TextEdit` improved rather than
        //     regressed. `text_selection/visuals.rs:39-40` takes its highlight
        //     from `bg_fill` and its text from `stroke.color`, i.e. the same
        //     pair, so it clears the floor by the same 103 / 119 / 123.
        //  3. A `ProgressBar` improved most of all. It fills with `bg_fill`
        //     but labels with `override_text_color` when set — which this
        //     theme sets to `text` — so its label used to be `text` on
        //     `accent`: a gap of 56.7 in Quiet. On the plate it is 159.8.
        //
        // ★ The blinking caret (`visuals.text_cursor`, a separate 2 pt stroke
        // this function does not touch) is unchanged and remains the other
        // focus cue; the ring is now a real second one rather than a decoration
        // that happened to be invisible.
        v.selection.bg_fill = p.selected_plate;
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
mod tests;
