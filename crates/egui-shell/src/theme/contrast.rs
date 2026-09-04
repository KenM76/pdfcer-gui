//! The rendered-pair contrast gate.
//!
//! # Why this module exists, and what it is a reaction to
//!
//! `DEFECTS.md` D2: every collapsible section heading in the settings
//! dialog and both dock tab labels rendered near-white on light grey. At
//! 1× they were simply not readable. The cause was one unassigned field —
//! `widgets.active.bg_fill` was never given the accent, while
//! `widgets.active.fg_stroke` was given a near-white plate colour — so
//! any widget that paints with `bg_fill` rather than `weak_bg_fill` got a
//! near-white foreground on `egui`'s stock light background.
//!
//! **Two theme tests sat directly adjacent to it and neither could have
//! caught it**, which is the finding worth carrying forward:
//!
//! - One compared `text` against `surface` and `panel`. The foreground
//!   that failed was neither of those.
//! - One asserted the plate colour stays light. That is correct for its
//!   stated purpose, and it therefore **agreed with the defect**.
//!
//! Both are *palette-vs-palette* tests: they compare two colours a human
//! deliberately wrote down beside each other. The defect was not in the
//! palette. It was in the **assignment** — which palette entry ends up as
//! a foreground and which as a background on the `egui::Style` that
//! actually gets painted. The pair that rendered was never a pair anyone
//! wrote down, so no palette test could contain it.
//!
//! A structural gate could not see it either. The project's
//! `check-theme-colors.sh` bans raw `Color32` literals outside the theme
//! module — a real and useful rule that says nothing about whether the
//! named colours are legible together. *The gate was structural, not
//! perceptual.*
//!
//! # What this module does instead
//!
//! It enumerates the **render surface**, not the author's intentions.
//! [`pairs`] reads a real `egui::Style` back and reports every
//! foreground/background pair `egui` will paint from it. [`check`]
//! measures each and returns every failure.
//!
//! The consequence that matters: a fill somebody *forgets* to assign in a
//! future preset is caught by the same assertion that catches a fill
//! somebody assigns *wrongly*, because an unassigned field still has a
//! value and that value still gets painted. A test written as a list of
//! pairs to check would have needed somebody to think of the missing one
//! — which is precisely what did not happen.
//!
//! # ★★★ THE WIDENING, 2026-09-04 — and why ten pairs was not enough
//!
//! Until this date [`pairs`] enumerated **ten**: `egui`'s five widget
//! states × two background fills, foreground always `fg_stroke.color`.
//! That is a good matrix — it is `egui`'s own, rather than a list somebody
//! maintained — and **it was green through three separately shipped
//! contrast defects**, because none of the three lived inside it.
//! `REVIEW_TRIAGE.md` row **A15e** is the finding; these are its subjects:
//!
//! 1. **A plate colour supplied by the CALLER**, through
//!    `RichText::color(...)`. It is not in the `Style` at all, so no
//!    amount of reading the `Style` back can reach it. **Still out of
//!    scope, deliberately** — see "What this module deliberately is not".
//!    `tools/gates/check-plate-colour.sh` is the gate that covers it, and
//!    it is a *structural* gate over call sites because that is the only
//!    place the information exists.
//! 2. **A background chosen by GEOMETRY** — a close button drawn
//!    `.frame(false)` into a sub-rect carved out of another widget's rect,
//!    so the thing behind it is whatever that other widget painted.
//!    **Also still out of scope**, and for a stronger reason: the
//!    background is not a colour anywhere in the process, it is a
//!    consequence of a layout. `REVIEW_TRIAGE.md` A15b (`tabstrip/mod.rs`,
//!    the document tab's close ✕) was fixed by making the enclosing widget
//!    paint its plate across the *whole* rect before the split, which
//!    converts the geometric case into an ordinary stated-plate case that
//!    gate 1 can see. That is the general move: **give the geometry a
//!    stated plate, and the structural gate gets its subject back.**
//! 3. **`visuals.selection`** — which `egui` substitutes for a selected
//!    widget's fill AND its text at paint time
//!    (`egui-0.35.0/src/widget_style.rs:151-154`), and which this theme
//!    had pointed at a 27 %-alpha canvas tint. **That one IS in the
//!    `Style`**, it was simply not in the matrix, and it is now:
//!    [`Origin::SelectedWidget`] and [`Origin::FocusRing`].
//!
//! So the widening adds every foreground a `Style` genuinely renders that
//! the widget matrix cannot reach — the ones `egui` resolves through a
//! `Visuals` accessor rather than storing in a `WidgetVisuals`. Each one
//! carries, on its [`Origin::why`], the `egui` source line that renders it.
//! Twenty-seven pairs now, ten of them the original matrix.
//!
//! ★★ **The one thing that must not happen to this list.** A widened gate
//! that fails on a *correct* state is worse than a narrow one, because
//! people learn to ignore it and then ignore it on the day it is right.
//! Every pair below was measured across all three shipped presets before
//! it was added. Exactly one foreground — `strong_text_color()`, on both
//! chrome grounds — cannot be made to pass by ANY theme value, and it
//! carries a named, reasoned [`Exemption`] with a test that expires it.
//! Nothing here is excluded silently, and the threshold was not moved.
//!
//! # What this module deliberately is not
//!
//! It is **not a WCAG conformance check**. It computes a crude
//! relative-luminance gap on a 0–255 scale, not a contrast ratio against
//! the sRGB transfer function, and it makes no accessibility claim. That
//! is the salvage source's own reasoning about its coarser sibling and it
//! applies with equal force here:
//!
//! > a coarse check that always fires beats a precise one nobody runs.
//!
//! The failure mode being guarded against is not "3.9:1 where 4.5:1 was
//! wanted". It is "white on white", and a crude measure catches that
//! every time, with no colour-science dependency and no argument about
//! which standard applies to a 1 px stroke.
//!
//! ★ The crude measure has one known bias worth stating, because it cost a
//! judgement call during the widening: Rec. 709 weights red at 0.2126, so
//! a saturated red foreground scores far lower here than a human reads it
//! as. The Dark preset's `danger` on `window_fill` measured **89.7**
//! against this floor while measuring **4.71:1** under WCAG AA — a pass
//! there and a fail here. The resolution was to move the *role*, not the
//! threshold and not the pair: 0.26 of headroom is not headroom, and the
//! next accent edit would have spent it. See `Theme::dark`.
//!
//! It is also **not a gate over call sites**. It sees what a `Style`
//! renders; it cannot see a colour a caller passes to `RichText::color`,
//! and it cannot see a background produced by layout rather than by paint.
//! Those two are real and they are covered by
//! `tools/gates/check-plate-colour.sh` and
//! `tools/gates/check-strong-text.sh`, which are structural gates over the
//! source because that is where the information lives. Knowing which
//! defects a gate *cannot* see is what stops the next one being filed as
//! "impossible, the gate is green".
//!
//! An application that wants a real WCAG gate should build one on top of
//! [`pairs`], which is the part that is hard to get right.

use egui::{Color32, Style};

/// The luminance gap below which a rendered pair is considered
/// unreadable.
///
/// 90 on a 0–255 crude luminance scale. The figure is inherited from the
/// salvaged palette-level text test so that both gates agree about what
/// "readable" means; a theme that satisfies one and not the other would
/// produce two contradictory failures for one edit.
///
/// The shipped presets clear it comfortably. The tightest real pair is
/// the Dark preset's focus ring — `accent` on `panel` — at 96.0.
pub const READABLE_LUMA_GAP: f32 = 90.0;

/// Which of `egui`'s five widget states a pair came from.
///
/// Mirrors `egui::style::Widgets`' fields. A local enum rather than a
/// re-export because the point of it is to be *named in a failure
/// message* — "the Active state's bg_fill" is the sentence that points at
/// the line to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    /// Not interactive at all: labels, separators, panel frames.
    NonInteractive,
    /// Interactive, at rest.
    Inactive,
    /// Under the pointer.
    Hovered,
    /// Being pressed, or currently the selected/active one.
    Active,
    /// An open menu, combo box or collapsing header.
    Open,
}

impl WidgetState {
    /// Every state, in `egui`'s own order.
    pub const ALL: &'static [WidgetState] = &[
        WidgetState::NonInteractive,
        WidgetState::Inactive,
        WidgetState::Hovered,
        WidgetState::Active,
        WidgetState::Open,
    ];

    /// The `egui` style entry for this state.
    fn visuals(self, style: &Style) -> &egui::style::WidgetVisuals {
        let w = &style.visuals.widgets;
        match self {
            WidgetState::NonInteractive => &w.noninteractive,
            WidgetState::Inactive => &w.inactive,
            WidgetState::Hovered => &w.hovered,
            WidgetState::Active => &w.active,
            WidgetState::Open => &w.open,
        }
    }
}

/// Which of the two backgrounds a widget may paint with.
///
/// # Why both, and why this distinction is the whole defect
///
/// `egui` gives each widget state two background colours and lets each
/// widget choose. `Button` and `SelectableLabel` paint `weak_bg_fill`;
/// `CollapsingHeader` headers, `egui_tiles` tab buttons and several
/// others paint `bg_fill`. A theme that assigns one and not the other has
/// themed an arbitrary subset of its own widgets, and which subset is
/// decided by `egui`'s internals rather than by the theme's author.
///
/// D2 is exactly that: `weak_bg_fill` was assigned the accent, `bg_fill`
/// was not, and the widgets that lost were the ones nobody happened to
/// look at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillKind {
    /// `WidgetVisuals::bg_fill` — used by `CollapsingHeader` headers,
    /// tab buttons, and anything drawing a solid widget body.
    BgFill,
    /// `WidgetVisuals::weak_bg_fill` — used by `Button` and friends.
    WeakBgFill,
}

impl FillKind {
    /// Both kinds.
    pub const ALL: &'static [FillKind] = &[FillKind::BgFill, FillKind::WeakBgFill];

    /// The colour for this kind, from a state's visuals.
    fn of(self, v: &egui::style::WidgetVisuals) -> Color32 {
        match self {
            FillKind::BgFill => v.bg_fill,
            FillKind::WeakBgFill => v.weak_bg_fill,
        }
    }

    /// The field name, for a failure message.
    fn field(self) -> &'static str {
        match self {
            FillKind::BgFill => "bg_fill",
            FillKind::WeakBgFill => "weak_bg_fill",
        }
    }
}

/// A background that is **not** a widget's own fill.
///
/// # Why this exists at all
///
/// [`FillKind`] answers "which of a widget's two fills". This answers a
/// different question — "what is behind a piece of text that is not
/// inside a widget" — and the widget matrix has no way to express it,
/// which is half of why ten pairs missed three defects.
///
/// The three grounds below are `egui`'s own, and each is reachable from a
/// `Visuals` alone. They are not palette roles: a theme may point all
/// three at one colour (this one points two of them at `Palette::panel`)
/// and the gate must still name them separately, because a *later* theme
/// may not, and because the failure message has to say which surface the
/// reader should go and look at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ground {
    /// `visuals.panel_fill` — every `SidePanel`, `TopBottomPanel` and
    /// `CentralPanel`, via `Frame::side_top_panel` / `Frame::central_panel`
    /// (`egui-0.35.0/src/containers/frame.rs:185-193`). In this
    /// application that is the ribbon, the status bar, the rails and every
    /// docked panel — i.e. most of the pixels most of the time.
    PanelFill,
    /// `visuals.window_fill` — every `Window`, every menu and every popup,
    /// via `Frame::window` / `Frame::menu` / `Frame::popup`
    /// (`frame.rs:195-220`). In this application that is all twenty-odd
    /// dialogs and the whole menu system.
    WindowFill,
    /// `visuals.text_edit_bg_color()` — the inside of a `TextEdit`, which
    /// falls back to `extreme_bg_color` when unset
    /// (`egui-0.35.0/src/style.rs:1146-1148`).
    ///
    /// Its own ground rather than a synonym for [`Self::WindowFill`]
    /// because `egui` genuinely resolves it separately, and because three
    /// distinct foregrounds land on it: the text being typed, the hint
    /// text when the field is empty, and the focus ring.
    TextEditBg,
}

impl Ground {
    /// Every ground.
    pub const ALL: &'static [Ground] = &[Ground::PanelFill, Ground::WindowFill, Ground::TextEditBg];

    /// The colour for this ground.
    fn of(self, v: &egui::Visuals) -> Color32 {
        match self {
            Ground::PanelFill => v.panel_fill,
            Ground::WindowFill => v.window_fill,
            Ground::TextEditBg => v.text_edit_bg_color(),
        }
    }

    /// The accessor name, for a failure message.
    fn field(self) -> &'static str {
        match self {
            Ground::PanelFill => "panel_fill",
            Ground::WindowFill => "window_fill",
            Ground::TextEditBg => "text_edit_bg_color()",
        }
    }
}

/// A text colour `egui` resolves through a [`egui::Visuals`] accessor
/// rather than storing in a `WidgetVisuals`.
///
/// # ★ Why these are invisible to the widget matrix
///
/// Every one of them is computed at paint time from something other than
/// the state's own `fg_stroke`. Reading the five `WidgetVisuals` back
/// therefore cannot reach any of them — the same structural reason the
/// selected pair was missed. Each variant's doc names the `egui` source
/// line that renders it, because a role nobody can point at a renderer
/// for should not be in this list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    /// `visuals.text_color()` — `override_text_color` if set, else
    /// `widgets.noninteractive.fg_stroke.color`
    /// (`egui-0.35.0/src/style.rs:1130-1133`).
    ///
    /// Ordinary body text: every `ui.label`, every heading, the text
    /// inside a `TextEdit` (`widgets/text_edit/builder.rs:463-466`). The
    /// widget matrix covers this colour only *inside* a widget; this
    /// covers it where most of it actually is, which is loose on a panel
    /// or in a dialog.
    Body,
    /// `visuals.weak_text_color()` — `text_color()` gamma-multiplied by
    /// `weak_text_alpha` (0.6 by default) unless `weak_text_color` is set
    /// (`style.rs:1135-1138`).
    ///
    /// `RichText::weak()` (`widget_text.rs:487`) and a `TextEdit`'s hint
    /// text (`text_edit/builder.rs:591`). **This project uses it heavily**
    /// — every explanatory note under a control in Settings and in the
    /// Print dialog, every count and hint in the status bar, the ribbon's
    /// group captions — which makes it the single most likely place for
    /// the next defect of this family, and it was outside the gate.
    ///
    /// ★ The value is *translucent* (a premultiplied 60 % of the body
    /// colour), so it must be composited before it is measured. Its gap is
    /// therefore exactly 0.6 × the body gap on the same ground, which is
    /// worth knowing when reading a failure: if `Weak` fails and `Body`
    /// passes, the theme is fine and `weak_text_alpha` is the dial.
    Weak,
    /// `visuals.strong_text_color()` — `widgets.active.fg_stroke.color`
    /// (`style.rs:1141-1143`).
    ///
    /// `RichText::strong()` (`widget_text.rs:485`) and `egui::Spinner`
    /// (`widgets/spinner.rs:44`).
    ///
    /// ⚠ **This role carries the module's only [`Exemption`]**, and the
    /// reason is structural rather than a tuning miss. See
    /// [`EXEMPTIONS`].
    Strong,
    /// `visuals.hyperlink_color` — `Hyperlink` and `ui.hyperlink`.
    ///
    /// In this theme it is [`crate::theme::Palette::accent`], so it is
    /// also a standing check that the accent itself reads on both chrome
    /// grounds — which the widget matrix only checks *behind* the accent,
    /// never *on* the panel.
    Hyperlink,
    /// `visuals.warn_fg_color` — the colour a caller asks for when
    /// something is worth knowing and nothing is broken.
    ///
    /// Not rendered by any `egui` widget on its own; it is read by call
    /// sites (`ui.colored_label(ui.visuals().warn_fg_color, …)`), of which
    /// this application has some twenty, in the Forms panel and in four
    /// dialogs. It is in the `Style`, the theme assigns it, and it lands
    /// on exactly these two grounds — so it belongs to a gate that
    /// measures what the `Style` renders.
    Warn,
    /// `visuals.error_fg_color` — the same shape as [`Self::Warn`], for
    /// the case where the operator must act.
    ///
    /// ★ This is the pair that the widening actually caught in a shipped
    /// preset: Dark measured **89.7** on [`Ground::WindowFill`] against a
    /// floor of 90. See the module header on the red bias, and
    /// `Theme::dark` for the fix.
    Error,
}

impl TextRole {
    /// Every role.
    pub const ALL: &'static [TextRole] = &[
        TextRole::Body,
        TextRole::Weak,
        TextRole::Strong,
        TextRole::Hyperlink,
        TextRole::Warn,
        TextRole::Error,
    ];

    /// The colour `egui` will resolve for this role.
    fn of(self, v: &egui::Visuals) -> Color32 {
        match self {
            TextRole::Body => v.text_color(),
            TextRole::Weak => v.weak_text_color(),
            TextRole::Strong => v.strong_text_color(),
            TextRole::Hyperlink => v.hyperlink_color,
            TextRole::Warn => v.warn_fg_color,
            TextRole::Error => v.error_fg_color,
        }
    }

    /// The grounds this role is **actually drawn on**.
    ///
    /// ★★ Deliberately not the full cross product. A gate is only worth
    /// what its reader believes, and a pair nobody renders is a line in a
    /// failure list that sends somebody to look for a surface that does
    /// not exist. So each role names the grounds it has a renderer for:
    ///
    /// - [`Self::Body`] and [`Self::Weak`] reach [`Ground::TextEditBg`]
    ///   because a `TextEdit` draws its content and its hint there
    ///   (`text_edit/builder.rs:463-466` and `:591`).
    /// - Nothing draws `strong`, a hyperlink, a warning or an error
    ///   *inside* a text field, so those four stop at the two chrome
    ///   grounds.
    fn grounds(self) -> &'static [Ground] {
        match self {
            TextRole::Body | TextRole::Weak => {
                &[Ground::PanelFill, Ground::WindowFill, Ground::TextEditBg]
            }
            TextRole::Strong | TextRole::Hyperlink | TextRole::Warn | TextRole::Error => {
                &[Ground::PanelFill, Ground::WindowFill]
            }
        }
    }

    /// The accessor name, for a failure message.
    fn field(self) -> &'static str {
        match self {
            TextRole::Body => "text_color()",
            TextRole::Weak => "weak_text_color()",
            TextRole::Strong => "strong_text_color()",
            TextRole::Hyperlink => "hyperlink_color",
            TextRole::Warn => "warn_fg_color",
            TextRole::Error => "error_fg_color",
        }
    }
}

/// Where a [`Pair`] comes from — the thing a failure message has to name.
///
/// # ★★ Why this replaced two plain fields
///
/// [`Pair`] used to carry `state: WidgetState` and `fill: FillKind`, which
/// is a complete description of a pair **if and only if** every pair comes
/// from the widget matrix. The moment a non-widget pair joins the list,
/// those two fields are a lie with no `None` to tell it with: a body-text
/// pair has no widget state and no fill kind.
///
/// The alternative — a parallel list of "other" pairs with their own
/// failure type — was rejected because it splits the one thing this module
/// is for. [`check`] returning *every* failure in one run is what makes a
/// theme edit one rebuild rather than five, and two lists reintroduce the
/// sequencing this module's `check` doc argues against.
///
/// So the origin became a sum type, and each variant answers the three
/// questions a 2 a.m. reader has: **what colour**, **on what**, and
/// **what renders it**. The widget variant's [`Self::fg_path`] and
/// [`Self::bg_path`] reproduce the old message's wording exactly, so no
/// existing failure text lost a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// One of the original ten: a widget state's `fg_stroke` on one of
    /// that same state's two fills.
    Widget {
        /// Which of `egui`'s five states.
        state: WidgetState,
        /// Which of the state's two fills.
        fill: FillKind,
    },
    /// A `Visuals`-level text colour on a non-widget background.
    Text {
        /// Which accessor `egui` resolves the colour through.
        role: TextRole,
        /// What is behind it.
        ground: Ground,
    },
    /// `visuals.selection.stroke.color` on `visuals.selection.bg_fill`,
    /// composited over a ground.
    ///
    /// # Why the ground is part of the identity
    ///
    /// Because `selection.bg_fill` may be translucent, and it *was* — the
    /// value that shipped through defect T2 was a 27 %-alpha canvas wash.
    /// A translucent plate is not a dimmer plate; it is a different colour
    /// over every background it meets, so "is the ink readable on the
    /// plate" has no answer until you say what is under the plate. With
    /// the opaque `selected_plate` that ships now, both grounds resolve to
    /// the same number — and that redundancy is the point: it is what
    /// makes a regression back to a wash show up as two failures rather
    /// than as an argument about which ground to have measured.
    SelectedWidget {
        /// What is behind the plate.
        ground: Ground,
    },
    /// `visuals.selection.stroke.color` on `visuals.text_edit_bg_color()`
    /// — the frame of a **focused, mutable `TextEdit`**
    /// (`egui-0.35.0/src/widgets/text_edit/builder.rs:699-706`).
    ///
    /// The second, easily forgotten role of the selection channel.
    /// `TextEdit` has no `.frame_stroke()`, so whatever is in that channel
    /// is the ring, everywhere, with no per-widget escape. An earlier fix
    /// satisfied the selected-widget half and left this one at gaps of
    /// 17.9 / 5.0 / 29.1 — white on white in Airy to within five levels of
    /// luminance. It has no `ground` field because there is only one
    /// ground it is ever drawn on.
    FocusRing,
}

impl Origin {
    /// How to name the foreground, in `egui`'s own field terms.
    ///
    /// A path a reader can grep for, not a description.
    #[must_use]
    pub fn fg_path(self) -> String {
        match self {
            Origin::Widget { state, .. } => format!("widgets.{state:?}.fg_stroke"),
            Origin::Text { role, .. } => role.field().to_owned(),
            Origin::SelectedWidget { .. } | Origin::FocusRing => {
                "selection.stroke.color".to_owned()
            }
        }
    }

    /// How to name the background, in `egui`'s own field terms.
    #[must_use]
    pub fn bg_path(self) -> String {
        match self {
            Origin::Widget { state, fill } => format!("widgets.{state:?}.{}", fill.field()),
            Origin::Text { ground, .. } => ground.field().to_owned(),
            Origin::SelectedWidget { ground } => {
                format!("selection.bg_fill over {}", ground.field())
            }
            Origin::FocusRing => "text_edit_bg_color()".to_owned(),
        }
    }

    /// **What actually renders this pair**, in one clause.
    ///
    /// # Why a failure carries this and not just the two field paths
    ///
    /// Because the field path says which line to edit and says nothing
    /// about whether editing it is the right move. A reader who is told
    /// `weak_text_color() on panel_fill` still has to go and find out
    /// what draws with it before they can judge whether the theme is
    /// wrong or the call site is. This is that sentence, written once,
    /// beside the measurement.
    #[must_use]
    pub fn why(self) -> &'static str {
        match self {
            Origin::Widget {
                fill: FillKind::BgFill,
                ..
            } => "CollapsingHeader headers, tab buttons, and anything drawing a solid widget body",
            Origin::Widget {
                fill: FillKind::WeakBgFill,
                ..
            } => "Button, SelectableLabel and friends",
            Origin::Text {
                role: TextRole::Body,
                ground: Ground::TextEditBg,
            } => "the text you type into a TextEdit (text_edit/builder.rs:463-466)",
            Origin::Text {
                role: TextRole::Body,
                ..
            } => "every ui.label and heading loose on this surface",
            Origin::Text {
                role: TextRole::Weak,
                ground: Ground::TextEditBg,
            } => "a TextEdit's hint text when the field is empty (text_edit/builder.rs:591)",
            Origin::Text {
                role: TextRole::Weak,
                ..
            } => "RichText::weak() — captions, hints and counts (widget_text.rs:487)",
            Origin::Text {
                role: TextRole::Strong,
                ..
            } => "RichText::strong() and egui::Spinner (widget_text.rs:485, spinner.rs:44)",
            Origin::Text {
                role: TextRole::Hyperlink,
                ..
            } => "ui.hyperlink and Hyperlink",
            Origin::Text {
                role: TextRole::Warn,
                ..
            } => "every call site reading visuals().warn_fg_color",
            Origin::Text {
                role: TextRole::Error,
                ..
            } => "every call site reading visuals().error_fg_color",
            Origin::SelectedWidget { .. } => {
                "every selectable_label(true, …) and Button::selected(true) \
                 (widget_style.rs:151-154)"
            }
            Origin::FocusRing => {
                "the frame of a focused, mutable TextEdit — no .frame_stroke() exists \
                 (text_edit/builder.rs:699-706)"
            }
        }
    }
}

/// One foreground/background pair as `egui` will paint it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pair {
    /// Where this pair comes from, and what renders it.
    pub origin: Origin,
    /// The foreground, as `egui` resolves it.
    pub fg: Color32,
    /// The background, already composited if it was itself translucent.
    pub bg: Color32,
    /// The crude relative-luminance gap between the two, after
    /// compositing a translucent foreground over the background.
    ///
    /// Stored rather than recomputed so a caller reporting a failure and
    /// a caller ranking pairs cannot disagree about the number.
    pub gap: f32,
}

/// A pair that failed the gate.
///
/// Carries the whole [`Pair`] plus the threshold it was measured against,
/// because a failure message that says "gap 41" without saying "needed
/// 90" makes the reader go and find the threshold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastFailure {
    /// Where the failing pair came from.
    pub origin: Origin,
    /// The foreground that was measured.
    pub fg: Color32,
    /// The background it was measured against.
    pub bg: Color32,
    /// The gap that was measured.
    pub gap: f32,
    /// The gap that was required.
    pub threshold: f32,
}

impl std::fmt::Display for ContrastFailure {
    /// A one-line diagnostic naming both field paths, both colours, the
    /// measurement, the threshold, and what renders the pair.
    ///
    /// **This is diagnostic text, not operator-visible copy.** It is
    /// written for a failing test, a CI log or a verification harness. An
    /// application that wants to surface a theme problem to a user should
    /// render the structured fields itself, in its own string catalogue —
    /// the shell has no business deciding how another project words a
    /// message to its operator.
    ///
    /// ★ The line keeps the ten-pair version's wording word for word for a
    /// widget pair, because that message was good and the widening had no
    /// licence to spend it. Two things were added, both because the
    /// widening produced a failure the old wording served badly:
    ///
    /// - the trailing ` — {why}` clause, for every origin — a reader who
    ///   knows `widgets.Active.bg_fill` failed is better off still for
    ///   being told, on the same line, that it is what tab buttons paint;
    /// - **one decimal place on the gap.** The first real failure the
    ///   widening found measured 89.74 against a floor of 90, and at
    ///   `{:.0}` it printed as *"luminance gap 90, needs 90"* — a
    ///   diagnostic that reads as a bug in the gate. The threshold keeps
    ///   `{:.0}` because it is a round number by construction.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:?} on {} {:?}: luminance gap {:.1}, needs {:.0} — {}",
            self.origin.fg_path(),
            self.fg,
            self.origin.bg_path(),
            self.bg,
            self.gap,
            self.threshold,
            self.origin.why(),
        )
    }
}

/// A pair that is measured, is allowed to fail, and says why in writing.
///
/// # ★★★ Why an exemption rather than a narrower list or a lower floor
///
/// Both of the easy answers are worse:
///
/// - **Dropping the pair** makes the gate lie by omission. The next
///   reader has no way to tell "we looked and decided" from "nobody
///   thought of it", which is precisely the state that let three defects
///   through the ten-pair matrix.
/// - **Lowering [`READABLE_LUMA_GAP`]** spends one pair's problem out of
///   every other pair's budget. The floor is shared with the palette-level
///   text test on purpose, so moving it here moves what "readable" means
///   for the whole theme.
///
/// So the pair stays in [`pairs`], is measured on every run, and is
/// skipped by [`check`] with a reason attached.
///
/// # ★★ And the exemption expires
///
/// An allow-list entry outliving its subject is a defect this project has
/// paid for twice in one week: a gate exemption whose premise expired
/// within a day, and `check-strong-text.sh` blessing a site on a sentence
/// that had stopped being true (`REVIEW_TRIAGE.md` T1). A blessing left
/// behind reads as a decision somebody made, and becomes a precedent for a
/// state nobody argued for.
///
/// `theme::tests::every_contrast_exemption_still_has_a_subject` is what
/// stops that: it asserts every entry below still names an origin
/// [`pairs`] produces **and** still fails in at least one shipped preset.
/// Fix the underlying pair and the exemption goes red until it is deleted.
#[derive(Debug, Clone, Copy)]
pub struct Exemption {
    /// The pair this covers.
    pub origin: Origin,
    /// The date the exemption was granted, so a reader can date the
    /// argument without going to `git blame`.
    pub granted: &'static str,
    /// Why this pair cannot be fixed by any theme value, and what covers
    /// it instead.
    pub reason: &'static str,
}

/// Every pair allowed to fail, with its argument.
///
/// ★ Two entries, one role, one argument. Keep it that way: an exemption
/// list is a budget, not a mechanism, and the moment it grows a third
/// unrelated entry the honest move is to ask whether the pair being
/// exempted should be in [`pairs`] at all.
pub const EXEMPTIONS: &[Exemption] = &[
    Exemption {
        origin: Origin::Text {
            role: TextRole::Strong,
            ground: Ground::PanelFill,
        },
        granted: "2026-09-04",
        reason: STRONG_TEXT_REASON,
    },
    Exemption {
        origin: Origin::Text {
            role: TextRole::Strong,
            ground: Ground::WindowFill,
        },
        granted: "2026-09-04",
        reason: STRONG_TEXT_REASON,
    },
];

/// The argument for the two [`EXEMPTIONS`], written once because it is
/// one argument.
///
/// # The mechanism, because the exemption looks arbitrary without it
///
/// `egui` has no role for emphasised text. `strong_text_color()` **is**
/// `widgets.active.fg_stroke.color` (`egui-0.35.0/src/style.rs:1141-1143`)
/// — the ink chosen for the ACCENT FILL, because the active state is the
/// accent-filled one. This theme therefore sets it to
/// `Palette::on_accent`, which is correct and necessary and is the exact
/// separation `DEFECTS.md` D2 was fixed by introducing.
///
/// So the pair `strong_text_color()` on `panel_fill` asks for a colour
/// that reads on *two* grounds a theme deliberately keeps far apart. In
/// the shipped presets it measures 7.9 / 0.1 / 18.3 on `panel_fill` and
/// 17.9 / 5.0 / 29.1 on `window_fill`. **There is no value that fixes
/// this**: raising `on_accent` toward the panel breaks it on the accent,
/// which is the ground it exists for. Both halves cannot be satisfied by
/// one field, and `egui` offers no second field.
///
/// # What covers it instead, since the gate cannot
///
/// `DEFECTS.md` D11 ruled `RichText::strong()` out of this application for
/// exactly this reason, and `tools/gates/check-strong-text.sh` enforces
/// the rule structurally — `.strong()` is permitted only when the same
/// statement takes the colour back with `.color(...)`. That is a call-site
/// gate because the information is at the call site; see this module's
/// header on what a `Style`-level gate cannot see.
///
/// ⚠ **One widget escapes both**: `egui::Spinner` resolves
/// `strong_text_color()` itself (`widgets/spinner.rs:44`) unless the call
/// site passes `.color(...)`, and a bare `ui.spinner()` names no colour
/// for a source gate to check. A spinner is therefore invisible on a
/// window in every preset. That is a call-site defect, not a theme one,
/// and it is recorded here rather than fixed here so the next reader of
/// this exemption finds the consequence attached to the argument.
const STRONG_TEXT_REASON: &str = "\
`strong_text_color()` IS `widgets.active.fg_stroke.color`, the ink chosen for the ACCENT \
FILL, and egui offers no separate emphasis role — so no theme value can read on both the \
accent and a panel. DEFECTS.md D11 rules `RichText::strong()` out of this application and \
tools/gates/check-strong-text.sh enforces it at the call site, which is where the \
information is. The one widget that reaches the role without naming a colour is \
egui::Spinner (widgets/spinner.rs:44); a bare `ui.spinner()` is invisible on a window in \
every preset and must pass `.color(...)`.";

/// The exemption covering `origin`, if any.
#[must_use]
pub fn exemption_for(origin: Origin) -> Option<&'static Exemption> {
    EXEMPTIONS.iter().find(|e| e.origin == origin)
}

/// Crude relative luminance of a colour, on a 0–255 scale.
///
/// The Rec. 709 coefficients applied directly to sRGB bytes, with no
/// linearization. This is not photometrically correct and is not trying
/// to be — see the module header on why a coarse measure is the right
/// tool for the failure being guarded against, and on the one bias
/// (saturated reds score low) that has actually changed a decision.
///
/// The alpha channel is ignored. Composite first with [`over`] if it
/// matters; [`pairs`] does.
#[must_use]
pub fn luma(c: Color32) -> f32 {
    0.2126 * f32::from(c.r()) + 0.7152 * f32::from(c.g()) + 0.0722 * f32::from(c.b())
}

/// Composite `fg` over `bg` using `fg`'s alpha, returning the opaque
/// result.
///
/// # Why this is necessary rather than fussy
///
/// The colour at the heart of D2 was `rgba(250,250,250,220)` — a
/// *translucent* near-white. Measuring its luminance as if it were opaque
/// overstates its contrast against a dark background and understates it
/// against a light one, and a plate colour used as a foreground is
/// exactly the case where that error is largest. A gate that got this
/// wrong would be wrong specifically about the defect it exists to catch.
///
/// ★ The widening added two more translucent things to measure and it did
/// not have to add any arithmetic for either: `weak_text_color()` is a
/// premultiplied 60 % of the body colour, and `selection.bg_fill` may be a
/// wash. Both go through this function.
///
/// `Color32` in `egui` is premultiplied, so the source channels are
/// already scaled by alpha and the composite is `src + dst·(1−a)`.
#[must_use]
pub fn over(fg: Color32, bg: Color32) -> Color32 {
    let a = f32::from(fg.a()) / 255.0;
    let mix = |s: u8, d: u8| -> u8 {
        let v = f32::from(s) + f32::from(d) * (1.0 - a);
        // Saturating rather than wrapping: a premultiplied source whose
        // channel exceeds its own alpha (which malformed input can
        // produce) must clip to white, never wrap to black.
        v.clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(
        mix(fg.r(), bg.r()),
        mix(fg.g(), bg.g()),
        mix(fg.b(), bg.b()),
    )
}

/// The luminance gap between a foreground and a background, compositing
/// the foreground's alpha over the background first.
#[must_use]
pub fn gap(fg: Color32, bg: Color32) -> f32 {
    (luma(over(fg, bg)) - luma(bg)).abs()
}

/// Every foreground/background pair a style will render.
///
/// Twenty-seven of them, in three groups:
///
/// | group | count | what it is |
/// |---|---:|---|
/// | [`Origin::Widget`] | 10 | `egui`'s five states × two fills — the original matrix |
/// | [`Origin::Text`] | 14 | six `Visuals` text accessors over the grounds each is drawn on |
/// | selection | 3 | the selected-widget plate over two grounds, plus the focus ring |
///
/// The first group's enumeration is over `egui`'s own matrix rather than
/// over a list somebody maintained, which is what makes an *unassigned*
/// field as visible to the gate as a *wrongly assigned* one. The other two
/// groups **are** lists, unavoidably — `egui` resolves them through
/// accessors rather than storing them in a matrix — so each entry carries
/// the source line that renders it ([`Origin::why`]) and
/// `the_pair_matrix_covers_every_origin_it_claims_to` holds the count.
///
/// Exempt pairs are included here. [`pairs`] reports what is *rendered*;
/// [`check`] decides what is *allowed to fail*. Keeping those two
/// separate is what lets the exemption-expiry test measure a pair it is
/// simultaneously excusing.
#[must_use]
pub fn pairs(style: &Style) -> Vec<Pair> {
    let v = &style.visuals;
    let mut out = Vec::with_capacity(27);

    let mut push = |origin: Origin, fg: Color32, bg: Color32| {
        out.push(Pair {
            origin,
            fg,
            bg,
            gap: gap(fg, bg),
        });
    };

    // Group 1 — the original ten.
    for &state in WidgetState::ALL {
        let w = state.visuals(style);
        let fg = w.fg_stroke.color;
        for &fill in FillKind::ALL {
            push(Origin::Widget { state, fill }, fg, fill.of(w));
        }
    }

    // Group 2 — the `Visuals` text accessors, each on the grounds it is
    // actually drawn on.
    for &role in TextRole::ALL {
        let fg = role.of(v);
        for &ground in role.grounds() {
            push(Origin::Text { role, ground }, fg, ground.of(v));
        }
    }

    // Group 3 — the two roles `egui` drives from `visuals.selection`.
    //
    // ★ The selected-widget plate is composited over the ground BEFORE it
    // is used as a background, because a wash in that channel is exactly
    // the defect this covers (T2) and a wash has no luminance of its own.
    // The focus ring needs no such compositing: `text_edit_bg_color()` is
    // already a ground.
    let ink = v.selection.stroke.color;
    for &ground in &[Ground::PanelFill, Ground::WindowFill] {
        let plate = over(v.selection.bg_fill, ground.of(v));
        push(Origin::SelectedWidget { ground }, ink, plate);
    }
    push(Origin::FocusRing, ink, Ground::TextEditBg.of(v));

    out
}

/// Measure every rendered pair in `style` against `threshold`.
///
/// Pairs covered by an [`Exemption`] are measured and then skipped; see
/// [`EXEMPTIONS`] for why they are not simply absent from [`pairs`].
///
/// # Errors
///
/// Returns **every** failing pair rather than the first, so one run names
/// the whole problem. A gate that reports one failure at a time turns a
/// theme edit into a sequence of rebuilds, and the second failure is
/// often the one that explains the first.
pub fn check(style: &Style, threshold: f32) -> Result<(), Vec<ContrastFailure>> {
    let failures: Vec<ContrastFailure> = pairs(style)
        .into_iter()
        .filter(|p| p.gap < threshold && exemption_for(p.origin).is_none())
        .map(|p| ContrastFailure {
            origin: p.origin,
            fg: p.fg,
            bg: p.bg,
            gap: p.gap,
            threshold,
        })
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The matrix is complete: every origin the module claims to cover is
    /// really produced, and nothing else is.
    ///
    /// Worth its own test because the widget half of this module's value
    /// is that its coverage is defined by `egui`'s matrix rather than by a
    /// list — and the *other* half is a list, which is precisely the part
    /// that can silently shrink. If a state were dropped from
    /// [`WidgetState::ALL`], or a role from [`TextRole::ALL`], the gate
    /// would still pass everything it looked at and nothing else would
    /// notice.
    #[test]
    fn the_pair_matrix_covers_every_origin_it_claims_to() {
        let pairs = pairs(&Style::default());

        for &state in WidgetState::ALL {
            for &fill in FillKind::ALL {
                assert!(
                    pairs
                        .iter()
                        .any(|p| p.origin == Origin::Widget { state, fill }),
                    "{state:?}/{fill:?} is not measured, so a defect there is invisible"
                );
            }
        }
        for &role in TextRole::ALL {
            for &ground in role.grounds() {
                assert!(
                    pairs
                        .iter()
                        .any(|p| p.origin == Origin::Text { role, ground }),
                    "{role:?} on {ground:?} is not measured, so a defect there is invisible"
                );
            }
        }
        for &ground in &[Ground::PanelFill, Ground::WindowFill] {
            assert!(
                pairs
                    .iter()
                    .any(|p| p.origin == Origin::SelectedWidget { ground }),
                "the selected-widget plate over {ground:?} is not measured — this is \
                 REVIEW_TRIAGE.md T2, the defect the widening was written for"
            );
        }
        assert!(
            pairs.iter().any(|p| p.origin == Origin::FocusRing),
            "the focused-TextEdit ring is not measured — this is the half of the \
             selection channel that was lost the first time it was re-pointed"
        );

        let widgets = pairs
            .iter()
            .filter(|p| matches!(p.origin, Origin::Widget { .. }))
            .count();
        assert_eq!(widgets, 10, "five widget states × two fills");
        assert_eq!(
            pairs.len(),
            27,
            "10 widget + 14 text + 3 selection. If this fails, the fix is not to edit \
             the number: it is to check the new origin is in one of the ALL lists and \
             carries a `why`, and only then to update this count."
        );

        let mut seen: Vec<Origin> = Vec::new();
        for p in &pairs {
            assert!(
                !seen.contains(&p.origin),
                "{:?} is measured twice; a duplicate origin double-reports one failure",
                p.origin
            );
            seen.push(p.origin);
        }
    }

    /// Every exemption names an origin that really exists.
    ///
    /// The other half — "and still fails" — needs the shipped presets and
    /// therefore lives in `theme::tests`, next to the loop over
    /// `Preset::ALL`. This half needs nothing but the module and belongs
    /// with it.
    #[test]
    fn every_exemption_names_an_origin_the_gate_produces() {
        let pairs = pairs(&Style::default());
        for e in EXEMPTIONS {
            assert!(
                pairs.iter().any(|p| p.origin == e.origin),
                "an exemption granted {} covers {:?}, which `pairs` no longer produces. \
                 Delete it: an allow-list entry that outlives its subject reads as a \
                 decision somebody made and is a precedent for a state nobody argued for.",
                e.granted,
                e.origin
            );
            assert!(
                !e.reason.is_empty(),
                "{:?} is exempted without an argument",
                e.origin
            );
        }
    }

    /// A translucent foreground is composited before being measured.
    ///
    /// The specific number matters: `rgba(250,250,250,220)` is the plate
    /// colour from D2, and treating it as opaque is the mistake that
    /// would make this gate wrong about its own defect.
    #[test]
    fn a_translucent_foreground_is_composited_not_treated_as_opaque() {
        let plate = Color32::from_rgba_unmultiplied(250, 250, 250, 220);
        let on_black = over(plate, Color32::BLACK);
        let on_white = over(plate, Color32::WHITE);
        assert!(
            luma(on_black) < luma(on_white),
            "the same translucent colour must resolve differently over \
             different backgrounds, or the gate is measuring a colour that \
             is never painted"
        );
        // Over black, 86% of a near-white: clearly lighter than mid-grey,
        // clearly not the 250 an opaque reading would give.
        assert!(
            luma(on_black) > 180.0 && luma(on_black) < 230.0,
            "unexpected composite luminance {}",
            luma(on_black)
        );
    }

    /// **A translucent SELECTION PLATE is composited too, and its ground
    /// changes the answer.**
    ///
    /// This is defect T2's arithmetic in miniature and the reason
    /// [`Origin::SelectedWidget`] carries a ground at all. A 27 %-alpha
    /// wash is not a dimmer plate; it is a different colour over every
    /// background it meets, so the same theme values must produce two
    /// different gaps over two different grounds. If they did not, the
    /// ground field would be decoration and a regression to a wash could
    /// hide behind whichever ground happened to be measured.
    #[test]
    fn a_translucent_selection_plate_measures_differently_on_each_ground() {
        let mut style = Style::default();
        style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(90, 140, 220, 70);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, Color32::from_rgb(23, 92, 196));
        style.visuals.panel_fill = Color32::WHITE;
        style.visuals.window_fill = Color32::BLACK;

        let pairs = pairs(&style);
        let on_panel = pairs
            .iter()
            .find(|p| {
                p.origin
                    == Origin::SelectedWidget {
                        ground: Ground::PanelFill,
                    }
            })
            .expect("the selected pair over panel_fill must be measured");
        let on_window = pairs
            .iter()
            .find(|p| {
                p.origin
                    == Origin::SelectedWidget {
                        ground: Ground::WindowFill,
                    }
            })
            .expect("the selected pair over window_fill must be measured");
        assert!(
            (on_panel.gap - on_window.gap).abs() > 50.0,
            "a 27 % wash over white and over black must not measure the same; got \
             {:.1} and {:.1}",
            on_panel.gap,
            on_window.gap
        );
    }

    /// White on white fails; black on white passes. The floor test.
    #[test]
    fn the_gate_separates_the_obvious_cases() {
        assert!(gap(Color32::WHITE, Color32::WHITE) < READABLE_LUMA_GAP);
        assert!(gap(Color32::BLACK, Color32::WHITE) > READABLE_LUMA_GAP);
        assert!(gap(Color32::WHITE, Color32::BLACK) > READABLE_LUMA_GAP);
    }

    /// Every failure is reported, not just the first.
    ///
    /// A gate that stops at the first failure turns a theme edit into a
    /// sequence of rebuilds. `check`'s contract says all of them; this is
    /// what holds it to that.
    #[test]
    fn check_reports_every_failing_pair_not_only_the_first() {
        let mut style = Style::default();
        // Make the whole widget matrix white-on-white: all ten must fail
        // and all ten must be named.
        for w in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            w.fg_stroke = egui::Stroke::new(1.0, Color32::WHITE);
            w.bg_fill = Color32::WHITE;
            w.weak_bg_fill = Color32::WHITE;
        }
        let failures =
            check(&style, READABLE_LUMA_GAP).expect_err("white on white cannot be readable");
        for &state in WidgetState::ALL {
            for &fill in FillKind::ALL {
                assert!(
                    failures
                        .iter()
                        .any(|f| f.origin == Origin::Widget { state, fill }),
                    "{state:?}/{fill:?} failed but was not reported"
                );
            }
        }
    }

    /// **An exempt pair is measured and then excused — not hidden.**
    ///
    /// Two assertions, and the second is the one that matters: [`pairs`]
    /// must still contain the failing pair, so the expiry test can see the
    /// subject it is excusing. A `check` that filtered at enumeration time
    /// would make the exemption unfalsifiable.
    #[test]
    fn an_exempt_pair_is_excused_by_check_but_still_measured_by_pairs() {
        let mut style = Style::default();
        // `strong_text_color()` is `widgets.active.fg_stroke.color`.
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, Color32::WHITE);
        style.visuals.panel_fill = Color32::WHITE;
        style.visuals.window_fill = Color32::WHITE;

        let strong_on_panel = Origin::Text {
            role: TextRole::Strong,
            ground: Ground::PanelFill,
        };
        let measured = pairs(&style)
            .into_iter()
            .find(|p| p.origin == strong_on_panel)
            .expect("an exempt pair must still be enumerated");
        assert!(
            measured.gap < READABLE_LUMA_GAP,
            "white on white must measure as a failure even when it is excused"
        );

        let reported = check(&style, READABLE_LUMA_GAP).err().unwrap_or_default();
        assert!(
            !reported.iter().any(|f| f.origin == strong_on_panel),
            "an exempted origin must not be reported by `check`"
        );
        assert!(
            exemption_for(strong_on_panel).is_some(),
            "and the exemption must be findable by origin"
        );
    }

    /// The failure message names both field paths, both colours and what
    /// renders the pair.
    ///
    /// The message is the deliverable. A gate that says "contrast failed"
    /// has told the reader to go and re-derive what this function already
    /// knew.
    ///
    /// ★ The widget case's wording is asserted verbatim, because the
    /// widening had no licence to spend a message that was already good.
    /// The only deliberate change is the gap's decimal place; see
    /// [`ContrastFailure`]'s `Display`.
    #[test]
    fn a_failure_names_the_line_to_change() {
        let widget = ContrastFailure {
            origin: Origin::Widget {
                state: WidgetState::Active,
                fill: FillKind::BgFill,
            },
            fg: Color32::WHITE,
            bg: Color32::WHITE,
            gap: 0.0,
            threshold: READABLE_LUMA_GAP,
        };
        let text = widget.to_string();
        assert!(
            text.starts_with(
                "widgets.Active.fg_stroke #FF_FF_FF_FF on \
                 widgets.Active.bg_fill #FF_FF_FF_FF: \
                 luminance gap 0.0, needs 90"
            ),
            "the widget message must be unchanged from the ten-pair version; got: {text}"
        );
        assert!(text.contains("tab buttons"), "{text}");

        // A non-widget pair describes its own origin rather than
        // borrowing a widget state it does not have.
        let weak = ContrastFailure {
            origin: Origin::Text {
                role: TextRole::Weak,
                ground: Ground::PanelFill,
            },
            fg: Color32::WHITE,
            bg: Color32::WHITE,
            gap: 0.0,
            threshold: READABLE_LUMA_GAP,
        };
        let text = weak.to_string();
        assert!(text.contains("weak_text_color()"), "{text}");
        assert!(text.contains("panel_fill"), "{text}");
        assert!(text.contains("captions"), "{text}");
        assert!(!text.contains("widgets."), "{text}");

        let ring = ContrastFailure {
            origin: Origin::FocusRing,
            fg: Color32::WHITE,
            bg: Color32::WHITE,
            gap: 0.0,
            threshold: READABLE_LUMA_GAP,
        };
        let text = ring.to_string();
        assert!(text.contains("selection.stroke.color"), "{text}");
        assert!(text.contains("text_edit_bg_color()"), "{text}");
        assert!(text.contains("frame_stroke"), "{text}");
    }
}
