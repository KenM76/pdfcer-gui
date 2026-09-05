//! The band — the row of captioned groups beneath the active tab.
//!
//! # ★ The one closure every group goes through
//!
//! This module's central design decision is that `captioned_group` is
//! the **only** function in this crate that draws a ribbon group, and
//! that it emits the caption itself, after the body, with no branch that
//! can skip it.
//!
//! That is not defensive style. It is a fix for a defect that actually
//! shipped, recorded in the salvage source's own doc comment
//! (`D:\Dev\pdfce\crates\pdfce-gui\src\ribbon_ui.rs`):
//!
//! > Two sites previously bypassed the predicate and therefore drew no
//! > caption at all: `LayoutReset` used a bare `tab.shows(..)`, and
//! > `Show` and `Panels` shared one `shows(A) || shows(B)` block. Both
//! > were visible in the 2026-08-08 capture as unlabelled floating
//! > controls.
//!
//! Two caption-less groups shipped. They were found by a **screenshot**,
//! not by a test, and the reason is instructive: nothing was wrong. Each
//! site compiled, each drew its controls, each passed every test the
//! project had. The rule "a group has a caption" lived in a convention
//! that two call sites happened not to follow.
//!
//! The predecessor's fix was to make the caption a *consequence of
//! drawing the group* rather than a separate statement — the body is
//! handed in as a closure, so there is no code path that shows a group
//! without captioning it. That shape is carried across here, and
//! strengthened in three ways:
//!
//! 1. **The caption is never empty.** The manifest's caption is
//!    `Option<String>` because a *layer* may omit it (see
//!    [`crate::manifest`]); `caption_text` falls back to the group's
//!    **id**, which is never empty in a well-formed manifest. So even an
//!    unvalidated manifest cannot produce a bare band — it produces an
//!    ugly caption that names the group that needs fixing.
//! 2. **Overflowed groups go through the same closure.** A group that
//!    moved into the "⏷ N more" menu is still a group and still gets its
//!    caption. Routing the menu through a second, simpler drawing path is
//!    exactly how the two shipped defects happened.
//! 3. **The counts are returned and asserted.** `BandOutcome` carries
//!    `groups_rendered` and `captions_emitted`; they are `debug_assert`ed
//!    equal at the end of every band, and
//!    `every_rendered_group_emits_a_caption` asserts it in release too,
//!    against a manifest that deliberately includes a caption-less group.
//!
//! # ★ Two rows, and a height that does not depend on the tab
//!
//! A band is **[`plan::GROUP_ROWS`] control rows tall on every tab**, and a
//! group whose controls are wider than [`plan::GROUP_WRAP_WIDTH`] wraps onto
//! the second row rather than running on. Both halves are
//! `mockups/ribbon.html`'s:
//!
//! ```css
//! .band  { display:flex; align-items:stretch; padding:8px 10px 4px; min-height:86px }
//! .group { display:flex; flex-direction:column; justify-content:space-between;
//!          padding:0 13px }
//! .gcmds { display:flex; flex-wrap:wrap; gap:5px; align-items:flex-start; max-width:440px }
//! ```
//!
//! Three properties, and this module now has all three. `.gcmds` **wraps**
//! ([`plan::wrap_group`] decides where). The band has a **fixed height**
//! ([`band_height`]) rather than being as tall as its content. The group is a
//! **column with the caption pinned to the bottom** — `justify-content:
//! space-between` — which is what [`captioned_group`]'s `rows_height`
//! argument buys: every caption in the band sits on one baseline, whether
//! the group above it used one row or two.
//!
//! # ★ The two paddings, and the one that was free
//!
//! Both are in the CSS above and neither was drawn until 2026-08-14. They are
//! not the same kind of change and it is worth being explicit about which is
//! which, because one of them cost nothing and the other one moves the
//! canvas.
//!
//! **`.group { padding: 0 13px }` — free.** [`plan::GROUP_PADDING`] has
//! budgeted 6 pt per side in [`plan::group_width`] since the day the planner
//! was written, and its own doc comment recorded that the renderer never drew
//! it. The band was therefore reserving the space and then spending it as an
//! accidental margin *outside* the group boundary: measured in the running
//! application at 1,100 pt, the Markup tab's Text-markup group box began at
//! x = 322.5 and its first control began at 322.5 as well. Controls sat flush
//! against the group edge and against the rule dividing them from the next
//! group. [`captioned_group`] now insets its body by that same constant, so
//! **no group's planned width changed and no group moved into the overflow
//! menu** — the arithmetic was always right and only the ink was wrong. See
//! [`plan::GROUP_PADDING`] for why 6 pt is the mockup's 13 px rather than a
//! disagreement with it: the mockup's divider is a zero-width `border-right`
//! and this build's is a real `ui.separator()`, so 6 + 14 + 6 lands on the
//! mockup's 26 px from the other direction.
//!
//! **`.band { padding: … 4px }` — not free, and R128 governs it.**
//! [`BAND_PADDING_BOTTOM`] is a real four points of extra ribbon, and the
//! ribbon sits directly above the canvas, so it is added to [`band_height`]'s
//! **derivation** rather than being allowed to fall out of what a group drew.
//! The height stays a function of the theme, the font and two constants; it
//! is still identical on every tab, still identical when every group is in
//! the overflow menu, and `the_band_is_the_same_height_on_every_tab` and
//! `the_band_keeps_its_height_at_widths_where_every_group_overflows` still
//! say so.
//!
//! ## Why the height is fixed, which is the part that is not taste
//!
//! `PROJECT_PLAN.md`'s **R128**. A content-driven height adjacent to a
//! fit-to-viewport zoom is a feedback loop — measured at 230 % → 224 % →
//! 215 % zoom drift — and the ribbon sits in the top panel directly above
//! the canvas. A band that were one row tall on File and two on Markup
//! would therefore change the canvas's rectangle on **every tab click**,
//! and a fit-to-page zoom would chase it.
//!
//! So the height is computed from the theme and the font and **nothing
//! else**: not from how many rows this tab's widest group happened to need,
//! not from how many groups fitted, not from whether the overflow
//! affordance is showing. [`render_band`] reserves it before it draws, and
//! reserves it even when the plan puts *every* group in the menu — which is
//! the case a height derived from drawn content would silently get wrong.
//! `the_band_is_the_same_height_on_every_tab` asserts it, and asserts that
//! the measurement happened rather than that it was vacuously absent.
//!
//! Note what is *not* claimed: a [`crate::theme::Preset`] change does move
//! the band, through `control_height`. That is a deliberate, global, one-off
//! event and not something a tab click can cause, which is the distinction
//! R128 is actually about.
//!
//! # Why the caption is *beneath* the controls, centred
//!
//! Also carried from the salvage source, whose comment records what the
//! alternative looked like when captured from the running application:
//!
//! > ```text
//! > File [Open…] [Save a copy…] Document [Properties] Clipboard [Copy…] …
//! > ```
//! >
//! > — a ~26 px strip in which the captions read as just more small
//! > controls and **the grouping is invisible**.
//!
//! An inline caption is not a smaller version of a ribbon; it is a
//! toolbar with some extra words in it. The one structural cue a ribbon
//! has is a labelled block of related controls, and putting the label
//! beside the block instead of under it removes that cue entirely.
//!
//! Centring needs the row's measured width, which in immediate mode
//! exists only *after* the row is emitted — hence measure-then-allocate
//! rather than a `vertical_centered` wrapper, which would justify to the
//! whole remaining band and scatter the captions across the window.
//!
//! # Overflow
//!
//! The arithmetic lives in [`super::plan`], which explains at length why
//! it is a separate pure module. What happens here is the second half of
//! the enforcement: the overflow control's rectangle is computed **from
//! the band's right edge, before any group is drawn**, and the groups are
//! given a child `Ui` whose `max_rect` stops where that reservation
//! begins. Nothing the group loop does can reach it, because the group
//! loop is not laying out in that space.
//!
//! ## ★ "The band's right edge" is not `available_rect_before_wrap()`
//!
//! That sentence hid a defect for the whole of this module's life, and it
//! is worth spelling out because the wrong version reads perfectly.
//!
//! `egui`'s `Region::expand_to_include_rect` grows a `Ui`'s **`max_rect`**,
//! not only its `min_rect`, whenever a child widget lays out beyond it.
//! The ribbon draws the tab-strip row *before* the band, in the same
//! vertical `Ui`. When the QAT, the tabs and the mode selector do not fit
//! — which is the entire situation the overflow machinery exists for —
//! that row overflows, the enclosing vertical `Ui`'s `max_rect` silently
//! grows to contain it, and the band that is drawn next asks
//! `available_rect_before_wrap()` and is told it has a width the window
//! never had. Observed, at a 180 pt viewport with real font metrics:
//!
//! ```text
//! screen   [   0.0 ..  180.0 ]
//! max_rect [  -7.1 ..  258.1 ]   ← grown by the row above
//! overflow [ 192.7 ..  258.1 ]   ← reserved from a right edge off-screen
//! ```
//!
//! The reservation arithmetic was correct and the affordance was still
//! unreachable: failure mode #8, arrived at through a `Ui` that lied about
//! its width rather than through an ordering mistake. With no font data
//! installed the row always fitted, so no test could see it.
//!
//! The fix is [`entitled_bounds`]: the band lays out inside the rectangle the
//! ribbon was **handed**, intersected with what is actually on screen
//! (`clip_rect`), and never inside whatever a sibling's overflow grew the
//! parent to. `render_band` takes that rectangle as an argument rather
//! than deriving it, because the only `Ui` that knows it is the one the
//! application passed to [`super::Ribbon::render`], before anything was
//! drawn into it.
//!
//! ## When the band is narrower than the affordance itself
//!
//! Something has to give, and #8 dictates what: not the affordance. The
//! rectangle is clamped into the band (`left = max(band.left, band.right −
//! reserved)`), so the control is always fully on screen and always
//! hit-testable; what gives instead is its **label**, which truncates.
//! And it is disclosed — `ribbon-overflow-affordance-clamped` — because a
//! control silently rendering at less than the size it asked for is
//! exactly the kind of degradation that is invisible until somebody
//! screenshots it.

use egui::{Align, Layout, Rect, RichText, TextStyle, UiBuilder, pos2, vec2};

use crate::manifest::{Group, Item, ItemSize};

use super::collapsed;
use super::control::render_item_at;
use super::ctx::Ctx;
use super::measure::{SEPARATOR_LINE, separator_width};
use super::overflow;
use super::plan::{self, CUSTOM_ITEM_WIDTH, GroupRows};
use super::report;
use super::rhythm::{
    BAND_ROW_SPACING, band_height, band_row_height, caption_font, compressed_control_height,
    rewrap_is_legible, rows_height,
};
use super::sizing;

/// Vertical gap between a group's control row and its caption.
///
/// Small on purpose: the caption must read as belonging to the row above
/// it rather than as a line of its own. The salvage source used the same
/// constant for the same reason.
///
/// ★ **2 → 3 on 2026-09-04**, from `mockups/pdfcer-shell.html`'s
/// `.grp .cap { padding: 3px 0 5px }` — the first figure. One point, and it
/// is here rather than left alone because the operator's instruction was
/// *"exactly like that including sizing"* and because the caption's font
/// went up two points in the same pass: a 9 pt caption 2 pt below its row
/// and an 11 pt caption 2 pt below its row are not the same optical gap.
pub(crate) const CAPTION_GAP: f32 = 3.0;

/// Clear space between the band's captions and whatever the application puts
/// underneath the ribbon.
///
/// `mockups/ribbon.html`'s `.band { padding: 8px 10px 4px }` — the third
/// figure. Measured in the running application before it existed: the group
/// captions ended at y = 103 and the dock's tab bar began at y = 105.3, so a
/// 10 pt caption drawn `weak()` and `small()` was separated from the panel
/// header below it by rather less than a line of its own leading. The caption
/// is the one piece of text that says what a block of controls is *for* (see
/// this module's header on why it is beneath the controls at all), and a
/// caption sitting on the seam reads as a label for the thing below it.
///
/// # Why this is added to [`band_height`] and not to the group loop
///
/// `PROJECT_PLAN.md`'s **R128**: the band's height must be a function of the
/// theme and the font and of nothing a tab can vary. Padding drawn as "space
/// after the last group" would be exactly such a variation — a tab whose
/// groups all went into the overflow menu draws no group and would get no
/// padding, so the band would be 4 pt shorter on that tab than on the one
/// beside it and the canvas below would move on a tab click. Folding it into
/// the derivation instead keeps one number, reserved before anything is
/// drawn, spent identically whether the band holds five groups or none.
///
/// # Why the mockup's top and side padding are not here
///
/// Only the bottom edge was measured as wrong. The band's top is already
/// separated from the tab strip by [`super::tabs::strip_underline`] and the
/// enclosing layout's own spacing, and the band's horizontal padding is a
/// decision about the *band's* left edge that would shift every group in it —
/// see [`plan::GROUP_PADDING`]'s closing note. Adding either one would be
/// visual churn beyond the defect, and churn is harder to review than the
/// change it is mixed into.
///
/// ★ **4 → 5 on 2026-09-04.** `mockups/ribbon.html` is superseded by
/// `mockups/pdfcer-shell.html`, whose band puts the clearance on the CAPTION
/// rather than on the band — `.grp .cap { padding: 3px 0 5px }`, the second
/// figure — and asks for five points of it. The role is identical (the
/// caption must not sit on the seam with whatever is under the ribbon) and
/// so is the reasoning below; only the number moved.
pub(super) const BAND_PADDING_BOTTOM: f32 = 5.0;

/// The condition-name prefix that marks a command as currently *on*.
///
/// # Why toggles are expressed as a condition rather than as a field
///
/// A ribbon has toggles: "Single page" is either the current page-display
/// mode or it is not, and a control that cannot show which is a control
/// the operator has to test by clicking. But *which* toggle is on is
/// application state, and [`crate::commands::Command`] deliberately holds
/// no state — it is a registration, built once, shared, `Clone`.
///
/// The [`crate::commands::ConditionSet`] already exists, is already
/// republished every frame, and already carries exactly this kind of
/// fact. So a command with id `view.single` renders selected while the
/// condition `selected:view.single` is set. No new type, no new manifest
/// field, no per-frame registry mutation, and the state is inspectable in
/// the same place every other piece of frame state is.
///
/// The prefix uses `:` rather than `.` so it cannot collide with an
/// application's own dotted condition names.
pub const SELECTED_CONDITION_PREFIX: &str = "selected:";

/// The condition name that reports command `id` as currently on.
#[must_use]
pub fn selected_condition(command_id: &str) -> String {
    format!("{SELECTED_CONDITION_PREFIX}{command_id}")
}

/// What one band did, for the frame report and for the caption
/// invariant's test.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BandOutcome {
    /// How many groups were drawn, in the band and in the overflow menu
    /// together.
    pub groups_rendered: usize,
    /// How many of those were drawn **in the band itself**.
    ///
    /// Recorded as the value [`Self::groups_rendered`] had reached when
    /// the band's own loop finished, *not* as the plan's `shown`. The
    /// distinction is the whole reason this field exists: a counter's
    /// earlier value is provably ≤ its later value, so
    /// `groups_rendered − groups_in_band` — "how many are in the menu" —
    /// cannot underflow whatever the plan said and whatever the popup
    /// did. Deriving the same number by subtracting the *plan's* hidden
    /// count mixes a count of what was drawn with a count of what was
    /// intended, and those two disagree on every frame the menu is shut.
    pub groups_in_band: usize,
    /// How many captions were emitted. **Must equal
    /// [`Self::groups_rendered`].**
    pub captions_emitted: usize,
    /// How many groups the plan moved into the overflow menu.
    pub groups_overflowed: usize,
    /// Whether the overflow affordance was drawn.
    pub overflow_visible: bool,
    /// The `egui::Id` of the overflow affordance, when one was drawn.
    ///
    /// Carried out so a test — or a harness — can ask `egui` itself
    /// whether the control is hit-testable. A rectangle proves a thing
    /// was allocated; only a hit test proves it can be reached.
    pub overflow_id: Option<egui::Id>,
}

/// The caption a group will be drawn with — never empty.
///
/// The manifest's caption is optional because a *layer* may omit it (a
/// layer that says `Group(id: "render")` is reordering a group, not
/// blanking its caption). A complete manifest is required to have one by
/// [`crate::manifest::Shell::validate`].
///
/// This is what happens when an application renders a manifest it did not
/// validate. Falling back to the **id** rather than to `""` is the whole
/// point: an empty caption reproduces the exact defect this module exists
/// to prevent, whereas `page_display` in the caption slot is visibly
/// wrong, unmistakably diagnostic, and names the group whose manifest
/// entry needs fixing.
#[must_use]
pub(crate) fn caption_text(group: &Group) -> &str {
    match group.caption.as_deref() {
        Some(c) if !c.trim().is_empty() => c,
        _ if !group.id.is_empty() => &group.id,
        _ => "(unnamed group)",
    }
}

/// The vertical shape every group in a band is drawn to.
///
/// Two numbers rather than one, because they pin two different things and
/// a group that satisfied only the first would still make the band ragged:
///
/// - [`Self::rows`] pins the **captions** to one baseline — the mockup's
///   `justify-content: space-between`. A one-row group is padded out to the
///   height two rows would have taken, so its caption lands where its
///   two-row neighbour's does.
/// - [`Self::total`] pins the **band** to one height — R128. It closes the
///   gap between what the reservation promised and what the caption
///   actually measured, so that a band showing five groups and a band
///   showing none (everything in the overflow menu) come out identical
///   rather than identical-to-within-a-caption's-rounding.
///
/// [`Self::NATURAL`] — both zero — means "as tall as your content", which
/// is what the overflow menu wants: the band's height is a fact about the
/// band, and padding a popup entry out to it would put a hole under every
/// one-row group in the menu.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct GroupBox {
    /// Clear space **above** the first control row —
    /// [`crate::theme::Metrics::ribbon_pad_top`], the mockup's
    /// `.ribbon { padding: 6px … }`.
    ///
    /// Carried on the box rather than emitted by [`render_band`] before the
    /// group loop because a `ui.horizontal` lays its children from one
    /// cursor: an `add_space` there would push the groups *sideways*, not
    /// down. Every group spends it identically, so the rows still start on
    /// one line.
    pub(crate) pad_top: f32,
    /// Height the control rows are padded out to, before the caption.
    ///
    /// Measured from **below** [`Self::pad_top`], so `pad_top + rows` is
    /// where the caption's gap begins.
    pub(crate) rows: f32,
    /// Height the whole group — padding, rows, gap and caption — is padded
    /// out to.
    pub(crate) total: f32,
}

impl GroupBox {
    /// Pad to nothing: the group is as tall as what it drew.
    pub(crate) const NATURAL: Self = Self {
        pad_top: 0.0,
        rows: 0.0,
        total: 0.0,
    };
}

/// The rectangle a ribbon row is entitled to lay out in.
///
/// Shared by the band and by [`super::strip`], because both rows have the
/// same obligation and the same trap: whatever a row is *offered* by the
/// layout is not necessarily a width the window has.
///
/// Three candidates, and the row gets the **narrowest** of them, because
/// each is an upper bound on where a control can be both drawn and
/// clicked:
///
/// 1. `ui.available_rect_before_wrap()` — where this `Ui`'s cursor is now.
///    Supplies the top edge and the left edge in the ordinary case.
/// 2. `entitled` — the rectangle the application handed
///    [`super::Ribbon::render`], captured **before** anything was drawn
///    into it. This is the one that matters: see the module header on
///    `max_rect` growth. Nothing a sibling row does can inflate it,
///    because it was read before the sibling existed.
/// 3. `ui.clip_rect()` — what is on screen. `egui` never grows a clip rect
///    to fit overflowing content, so it is the honest answer to "would the
///    operator see a pixel painted here", which is what failure mode #8 is
///    ultimately about.
///
/// Only the horizontal extent is negotiated. The vertical extent is the
/// caller's — [`render_band`] replaces the bottom edge with
/// `top + `[`band_height`] immediately, and clamping the height to the clip
/// rect instead would make a partially-scrolled ribbon lay its captions out
/// differently from an unscrolled one.
///
/// A degenerate result (right ≤ left) is returned as a zero-width rect at
/// the left edge rather than as an inverted one: [`plan::plan_band`] reads
/// zero as "nothing fits, everything goes to the menu", which is the safe
/// answer, whereas an inverted rect would produce a negative width and a
/// nonsense plan.
pub(crate) fn entitled_bounds(ui: &egui::Ui, entitled: Rect) -> Rect {
    let cursor = ui.available_rect_before_wrap();
    let clip = ui.clip_rect();
    let left = cursor.left().max(entitled.left()).max(clip.left());
    let right = cursor
        .right()
        .min(entitled.right())
        .min(clip.right())
        .max(left);
    Rect::from_min_max(pos2(left, cursor.top()), pos2(right, cursor.bottom()))
}

/// Draw the band for one tab: its groups, left to right, with a vertical
/// rule between them, and an overflow affordance if they do not fit.
///
/// `entitled` is the rectangle the application handed the ribbon, read
/// before any of the ribbon was drawn. It is a parameter rather than
/// something this function derives because by the time the band runs, the
/// `Ui` it is given can no longer report it — see [`entitled_bounds`] and the
/// module header.
pub(crate) fn render_band(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    groups: &[Group],
    entitled: Rect,
) -> BandOutcome {
    let mut outcome = BandOutcome::default();

    let gutter = ctx.theme.metrics.gutter;
    let separator = separator_width(ui);
    // ★★★ A GROUP WITH NOTHING LEFT IS NOT DRAWN AT ALL —
    // `RIBBON_SCALING.md` §5.3, and R9.
    //
    // An item can be hidden by its `visible_when`; a group all of whose items
    // are hidden is a caption over an empty rectangle, and its separator is a
    // rule between two things with nothing between them. Filtering here rather
    // than inside `group_body` is what makes the space **reclaimed**: the
    // planner never sees the group, so no width is reserved for it, no
    // separator is drawn beside it, and the groups to its right move left.
    //
    // That is the operator's second ask in `O31` — *"shift the space used
    // depending on what exists"* — and it is what lets one tab definition
    // serve Read, Review and Edit rather than three near-identical tabs.
    let groups: Vec<&Group> = groups
        .iter()
        .filter(|g| g.items().iter().any(|i| sizing::visible(i, ctx.conditions)))
        .collect();
    let measured: Vec<(GroupRows, f32)> =
        groups.iter().map(|g| measure_group(ui, ctx, g)).collect();

    // ★★★ S3 — THE COLLAPSE LADDER, run before the overflow planner.
    //
    // The order is the whole point and it is Word's, measured rather than
    // assumed: at 800 pt Word's Font and Paragraph are single captioned
    // buttons and every group is still ON the band; the scroll affordance does
    // not appear until 460. A surface that overflowed first would be hiding
    // commands into a menu while the space to show them, collapsed, was still
    // there. See `plan::collapse`'s header for the three photographs this
    // ordering comes from.
    // ★ When the theme's numbers cannot fit three legible rows into the band's
    // fixed row area, the re-wrapped measurement is simply the natural one — so
    // `Candidate::gains_from` sees no gain, the ladder skips the rung, and no
    // group is ever drawn with clipped icons. The feature disables itself on a
    // theme it cannot serve rather than degrading.
    let rewrap_ok = rewrap_is_legible(ui, ctx);
    let rewrapped: Vec<(GroupRows, f32)> = groups
        .iter()
        .zip(&measured)
        .map(|(g, natural)| {
            if rewrap_ok {
                measure_group_rows(ui, ctx, g, plan::MAX_GROUP_ROWS)
            } else {
                natural.clone()
            }
        })
        .collect();
    let candidates: Vec<plan::collapse::Candidate> = groups
        .iter()
        .zip(&measured)
        .zip(&rewrapped)
        .map(
            |((g, (_, natural)), (_, narrow))| plan::collapse::Candidate {
                natural: *natural,
                rewrapped: *narrow,
                collapsed: collapsed::width(ui, g),
                priority: g.collapse,
            },
        )
        .collect();
    // S4: the reservation is now a scroll ARROW, not a `⏷ N more` dropdown.
    // Same discipline — taken from the band's edge before a single group is
    // laid out, so the affordance can never be the thing that gets squeezed
    // out — and a much smaller number, because a chevron does not carry a
    // count. See `overflow`'s header for what was traded away.
    let reserve = overflow::arrow_width(ctx);

    ui.horizontal(|ui| {
        // ★ R128. The band's height is reserved here, from the theme, before
        // anything is drawn and regardless of what the plan is about to
        // decide — so a tab whose groups all went into the overflow menu is
        // exactly as tall as one whose groups all fitted. A height taken
        // from what was drawn would differ between those two, and the
        // difference would move the canvas underneath. See the module
        // header.
        let height = band_height(ui, ctx);
        let box_ = GroupBox {
            pad_top: ctx.theme.metrics.ribbon_pad_top,
            rows: rows_height(ui, ctx),
            total: height,
        };
        ui.set_min_height(height);

        // The vertical extent is the band's own, not whatever the enclosing
        // `Ui` had left over: `entitled_bounds` negotiates width alone and
        // hands back the caller's bottom edge.
        let offered = entitled_bounds(ui, entitled);
        let full = Rect::from_min_max(
            offered.min,
            pos2(offered.right(), offered.top() + height.max(0.0)),
        );
        // The ladder needs the band's real width, which is only known here —
        // `entitled_bounds` negotiates it — so the mask is computed inside the
        // horizontal rather than beside the measurement. Note it is computed
        // from `full.width()` and from nothing that the previous frame decided:
        // the collapsed set is a pure function of the width, which is what
        // makes widening monotonic and keeps the band from flickering at any
        // particular size. See `plan::collapse`'s header.
        let states = plan::collapse::fit(&candidates, full.width(), separator);
        let widths = plan::collapse::widths_after(&candidates, &states);

        // ★★★ S4 — WHERE THE BAND IS SCROLLED TO, clamped before anything is
        // drawn.
        //
        // The remembered index is an INPUT to layout, so a stale one — left
        // behind by a widened window — would leave blank space at the band's
        // right edge with nothing for the operator to press. `overflow::clamp`
        // answers from the offered width and the group widths alone; it never
        // reads what was drawn, which is what keeps this from becoming the
        // measurement-feeding-its-own-size loop this project has paid for three
        // times.
        let scrolled =
            overflow::first(ui, ctx, tab_id).min(overflow::clamp(&widths, full.width(), separator));
        overflow::set_first(ui, ctx, tab_id, scrolled);

        let visible_widths = &widths[scrolled.min(widths.len())..];
        let band_plan = plan::plan_band(full.width(), visible_widths, separator, reserve);

        // ★ The reservation, taken from the right edge BEFORE any group
        // is drawn. `overflow_rect` is computed from `full.right()` and
        // from nothing the group loop can influence, so failure mode #8
        // — the overflow control being the thing that gets squeezed out —
        // is not reachable from here. See `plan`'s module header.
        //
        // `left` is clamped into the band. When the band is narrower than
        // the affordance the subtraction alone would put the control's
        // left edge off screen, which is #8 again with the affordance
        // present-but-unreachable rather than absent. Clamping spends the
        // shortfall on the label instead — see the module header.
        let overflow_rect = band_plan.has_overflow().then(|| {
            let desired_left = full.right() - band_plan.overflow_width;
            let left = desired_left.max(full.left());
            if left > desired_left {
                crate::verify::event("ribbon-overflow-affordance-clamped")
                    .kv("tab", tab_id)
                    .kv("band_width", format!("{:.1}", full.width()))
                    .kv("reserved", format!("{:.1}", band_plan.overflow_width))
                    .emit();
            }
            Rect::from_min_max(
                pos2(left, full.top()),
                pos2(full.right(), full.top() + ctx.theme.metrics.control_height),
            )
        });

        // ★★★ THE LEFT ARROW'S RESERVATION, taken before any group is laid
        // out — exactly as the right one's is, and for the identical reason.
        //
        // Added 2026-08-25 after the sweep below found the defect: `groups_rect`
        // began at `full.min`, the left arrow's rect is `full.min ..
        // full.left() + reserve`, and the arrow is drawn AFTER the groups. They
        // overlapped precisely, and at 237 pt `ribbon.group.view.window` was
        // measured at `[[0.0 31.0] - [169.5 100.0]]` under an arrow occupying
        // `0 .. 24`.
        //
        // What that costs is worse than an overlap: **the arrow wins the hit
        // test**, so on a scrolled band the leading control is unreachable and
        // clicking it scrolls the ribbon instead. A control drawn normally,
        // looking normal, doing something else entirely.
        //
        // ★ Note the asymmetry that hid it. The right-hand affordance has had
        // `no_visible_group_overlaps_the_overflow_affordance` since the band was
        // written, and that test sweeps every width — but it renders a FRESH,
        // UNSCROLLED band each time, and there is no width at which an
        // unscrolled band draws a left arrow. The guard existed, swept
        // correctly, and could not see this by construction.
        let left_reserve = if scrolled > 0 { reserve } else { 0.0 };
        let groups_rect = Rect::from_min_max(
            pos2((full.left() + left_reserve).min(full.right()), full.top()),
            pos2(
                (full.left() + left_reserve + band_plan.group_budget).min(full.right()),
                full.bottom(),
            ),
        );

        ui.scope_builder(
            UiBuilder::new()
                .id_salt("egui-shell-ribbon-groups")
                .max_rect(groups_rect)
                .layout(Layout::left_to_right(Align::Min)),
            |ui| {
                ui.set_max_width(groups_rect.width());
                for (offset, group) in groups
                    .iter()
                    .skip(scrolled)
                    .take(band_plan.shown)
                    .enumerate()
                {
                    let index = scrolled + offset;
                    // Separator BEFORE the group rather than after, so the
                    // band never ends with a trailing rule and the first
                    // group never starts with one.
                    if offset > 0 {
                        ui.separator();
                    }
                    match states[index] {
                        plan::collapse::State::Collapsed => collapsed::render(
                            ui,
                            ctx,
                            tab_id,
                            group,
                            gutter,
                            &measured[index].0,
                            box_,
                            &mut outcome,
                        ),
                        // ★ The row split handed to the renderer MUST be the
                        // one the ladder priced. Passing `measured` here while
                        // the plan spent `rewrapped`'s width is the exact shape
                        // of the plan/renderer disagreement `GroupRows`' own
                        // doc comment warns about, and it clips a group.
                        state => captioned_group(
                            ui,
                            ctx,
                            tab_id,
                            group,
                            gutter,
                            if state == plan::collapse::State::Rewrapped {
                                &rewrapped[index].0
                            } else {
                                &measured[index].0
                            },
                            box_,
                            &mut outcome,
                        ),
                    }
                }
            },
        );

        // Snapshotted here, between the band's loop and the menu's, so the
        // "how many are in the menu" subtraction downstream is a counter
        // minus its own earlier value. See `BandOutcome::groups_in_band`.
        outcome.groups_in_band = outcome.groups_rendered;

        // The arrows. Right when there is anything past the last group drawn,
        // left when the band has been scrolled off its start. Neither is drawn
        // disabled: R9 reserves greying for *temporarily* unavailable, and an
        // arrow with nowhere to go is not that — it is an arrow that does not
        // apply, and an unavailable capability renders nothing.
        let past_end = scrolled + band_plan.shown < groups.len();
        outcome.groups_overflowed = groups.len() - band_plan.shown.min(groups.len());
        outcome.overflow_visible = past_end || scrolled > 0;

        if let Some(rect) = overflow_rect
            && past_end
        {
            let response = overflow::arrow(
                ui,
                ctx,
                overflow::Direction::Right,
                rect,
                plan::overflow_label(groups.len() - (scrolled + band_plan.shown)),
            );
            // The id a driven check clicks. Carried across from the dropdown
            // unchanged — see `overflow::arrow`.
            outcome.overflow_id = Some(response.id);
            if response.clicked() {
                overflow::set_first(ui, ctx, tab_id, scrolled + 1);
            }
        }
        if scrolled > 0 {
            let rect = Rect::from_min_max(
                full.min,
                pos2(
                    (full.left() + reserve).min(full.right()),
                    full.top() + ctx.theme.metrics.control_height,
                ),
            );
            if overflow::arrow(
                ui,
                ctx,
                overflow::Direction::Left,
                rect,
                plan::overflow_label(scrolled),
            )
            .clicked()
            {
                overflow::set_first(ui, ctx, tab_id, scrolled - 1);
            }
        }
    });

    // The invariant, restated where it can fail loudly in a debug build.
    // The release-mode guarantee is structural (there is one drawing
    // path and it emits the caption itself); this is the tripwire for an
    // edit that adds a second one.
    debug_assert_eq!(
        outcome.groups_rendered, outcome.captions_emitted,
        "a ribbon group was drawn without a caption — every group must go \
         through `captioned_group`, which is the only function that draws one"
    );
    outcome
}

/// **The only function in this crate that draws a ribbon group.**
///
/// Lays one group out as Office lays one out: its controls in up to
/// [`plan::GROUP_ROWS`] rows, its **caption beneath them, centred on the
/// widest row**. The body is a closure rather than a predicate for the
/// reason the salvage source records: to put the caption *under* the
/// controls, the controls must be emitted inside a vertical container that
/// is still open when the caption is written, and a predicate returning
/// `bool` has already returned before the body runs.
///
/// See the module header for why this being the only such function is the
/// point.
///
/// # `rows`
///
/// The split [`plan::wrap_group`] decided, handed in rather than recomputed
/// — see [`GroupRows`] for why the plan and the renderer must not each own
/// a copy of that arithmetic.
///
/// # `box_`
///
/// The heights this group is padded out to — see [`GroupBox`], and
/// [`GroupBox::NATURAL`] for the overflow menu, where neither applies.
///
/// The padding is measured against the `Ui`'s own **cursor** rather than
/// predicted, so a control that turned out taller than
/// [`crate::theme::Metrics::control_height`] shortens the gap instead of
/// pushing the caption out of the band. The cursor, specifically, and not
/// `min_rect`: `egui` advances the cursor past a laid-out rect by
/// `item_spacing`, so the two differ by exactly one gap after every row and
/// padding against the wrong one leaves each group a gap taller than the
/// height the band reserved for it. That is a 3 pt discrepancy that shows
/// up only when a tab has *no* group in the band to compare against — which
/// is to say, only in the R128 case.
///
/// # The horizontal inset
///
/// The mockup's `.group { padding: 0 13px }`, drawn at
/// [`plan::GROUP_PADDING`] — the width [`plan::group_width`] has budgeted for
/// it all along, so this inset is paid for out of a reservation that already
/// existed and costs the band nothing. See the module header.
///
/// The reported rectangle **includes** the inset, deliberately: the group box
/// is what the operator perceives as the group, and a report that named only
/// the content would make "is there padding" unanswerable from outside the
/// process, which is the question that produced this change.
#[allow(clippy::too_many_arguments)]
pub(crate) fn captioned_group(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    group: &Group,
    gutter: f32,
    rows: &GroupRows,
    box_: GroupBox,
    outcome: &mut BandOutcome,
) {
    outcome.groups_rendered += 1;

    // ★ The group's own horizontal padding — `.group { padding: 0 13px }`.
    //
    // `horizontal_top` rather than `horizontal`: the latter is
    // `left_to_right(Align::Center)` and would centre a group vertically
    // within the band's reserved height, which is precisely the pinning the
    // `box_` arithmetic below exists to do by hand. `horizontal_top` is the
    // same layout with `Align::Min`, i.e. exactly what a bare `ui.vertical`
    // in the band's own `left_to_right(Align::Min)` did before this wrapper
    // existed.
    //
    // `item_spacing.x = 0` because `egui` advances the cursor past a
    // laid-out rect by `item_spacing` and `add_space` adds to that: without
    // this the trailing pad would be `GROUP_PADDING + item_spacing.x` and
    // the group would be one gutter wider than the plan budgeted for it.
    // The leading pad has no such term (nothing has been laid out yet), so
    // an asymmetric group is exactly what forgetting this line produces.
    // Setting `.x` alone leaves `item_spacing.y` — which `rows_height`
    // budgets against — untouched.
    let whole = ui
        .horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.add_space(plan::GROUP_PADDING);
            group_body(ui, ctx, tab_id, group, gutter, rows, box_, outcome);
            ui.add_space(plan::GROUP_PADDING);
        })
        .response
        .rect;

    ctx.reporter
        .report(whole, || report::group(tab_id, &group.id));
}

/// The inside of a group: its control rows, then its caption, padded to
/// [`GroupBox`].
///
/// Split out of [`captioned_group`] only so the horizontal inset above reads
/// as one line rather than as a closure wrapping a closure. **It is not a
/// second drawing path** — [`captioned_group`] is the only caller and the
/// only function that can reach it, so the module header's invariant ("every
/// group goes through one closure, which emits the caption itself") is
/// unchanged: `outcome.captions_emitted` is still incremented on a line that
/// cannot be reached without the label having been drawn.
#[allow(clippy::too_many_arguments)]
fn group_body(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab_id: &str,
    group: &Group,
    gutter: f32,
    rows: &GroupRows,
    box_: GroupBox,
    outcome: &mut BandOutcome,
) {
    ui.vertical(|ui| {
        // ★★★ EVERY VERTICAL GAP IN A GROUP IS EXPLICIT — 2026-09-05.
        //
        // The group's own column had the theme's `item_spacing.y` (4 pt at
        // `Quiet`), which `egui` inserts after **every** laid-out rect —
        // including the last row, before the padding below computes. With two
        // 24 pt rows there was twelve points of slack in the 68 pt area and the
        // stray gap disappeared into it. With three rows there is none, so the
        // rows overshot their own budget by exactly one `item_spacing` and the
        // caption was drawn 3 pt into the clearance the band reserves beneath
        // it — measured, and reported by
        // `the_band_leaves_clear_space_beneath_its_captions` as 2.03 pt against
        // 5.
        //
        // Zeroing it here is not tidying: it makes [`band_height`]'s five-term
        // sum the *only* description of the band's vertical rhythm. Every gap
        // that remains — [`GroupBox::pad_top`], [`BAND_ROW_SPACING`] between
        // the control rows, [`CAPTION_GAP`] — is spent by a named line in this
        // function, so a reader can add the terms up and get the number the
        // height test asserts. A spacing the framework inserts on our behalf is
        // a term in that sum that nothing names.
        ui.spacing_mut().item_spacing.y = 0.0;
        // The controls FIRST: the widest row's width is what the
        // caption is then centred within, and that width only exists
        // after the rows have been emitted.
        // ★ The same partition `measure_group` planned against, recomputed
        // from the same inputs rather than threaded through: two call sites
        // agreeing by construction beats two call sites agreeing by a
        // parameter somebody can forget to pass.
        let (large, items) = partition(ctx, group);
        let top = ui.cursor().top();
        // ★ The band's top padding, spent here so it lands *above the rows*
        // rather than beside the first group — see [`GroupBox::pad_top`].
        // `top` is read before it, so the caption arithmetic at the bottom of
        // this function measures from the band's own top edge and the padding
        // is simply the first thing inside the budget.
        ui.add_space(box_.pad_top);
        let mut widest = 0.0_f32;
        let content = ui
            .horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gutter;
                // The Large run leads, at the full height of the row area, so
                // a Large control is exactly as tall as the two rows beside
                // it rather than as tall as its own content.
                for item in &large {
                    render_item_at(ui, ctx, tab_id, &group.id, item, ItemSize::Large, box_.rows);
                }
                if !items.is_empty() {
                    ui.vertical(|ui| {
                        // ★★★ A RE-WRAPPED GROUP DIVIDES THE SAME ROW AREA INTO
                        // MORE ROWS — the band's height does not move.
                        //
                        // `Theme::apply` pins `spacing.interact_size.y` to
                        // `control_height`, which is exactly what makes every
                        // control one row tall and, left alone, would make three
                        // rows half again as tall as two. Overriding it here —
                        // on this group's own `Ui`, for this frame — is the
                        // whole of S5's rendering side.
                        //
                        // Deriving the pitch from `rows.counts.len()` rather
                        // than from a flag is deliberate: the renderer then
                        // cannot disagree with the plan about how many rows
                        // there are, which is the failure `GroupRows`' own doc
                        // comment was written to prevent.
                        {
                            let h = if rows.counts.len() > plan::GROUP_ROWS {
                                compressed_control_height(ui, ctx).max(ctx.theme.metrics.icon_pts)
                            } else {
                                band_row_height(ui, ctx)
                            };
                            ui.spacing_mut().item_spacing.y = BAND_ROW_SPACING;
                            ui.spacing_mut().interact_size.y = h;
                            // ★ `interact_size` is a FLOOR, not a ceiling. A
                            // button is as tall as its own text plus
                            // `button_padding.y` either side, and with the
                            // shipped theme that is 14 + 2×4 = 22 pt — taller
                            // than the 16.7 pt row we just asked for, so the
                            // band grew anyway and the height test said so
                            // (112 pt against 104). Compressing the row means
                            // compressing the padding that sets its floor.
                            //
                            // ★ And the floor is the ICON, not the text. A
                            // ribbon button is an `Atom` icon of `icon_pts`
                            // beside an optional label, so its content height
                            // is `max(icon, text)` — 16 against 14 with the
                            // shipped theme. Deriving the padding from the text
                            // alone left every row two points too tall and the
                            // band still grew, by exactly the eight points the
                            // height test reported the second time.
                            let content = ui
                                .text_style_height(&TextStyle::Button)
                                .max(ctx.theme.metrics.icon_pts);
                            ui.spacing_mut().button_padding.y = ((h - content) / 2.0).max(0.0);
                        }
                        let mut at = 0_usize;
                        for &count in &rows.counts {
                            let end = (at + count).min(items.len());
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gutter;
                                for item in &items[at..end] {
                                    let size = effective_size(ctx, item);
                                    render_item_at(
                                        ui, ctx, tab_id, &group.id, item, size, box_.rows,
                                    );
                                }
                            });
                            at = end;
                        }
                    });
                }
            })
            .response
            .rect;
        widest = widest.max(content.width());

        // ★ The caption is pinned to the bottom of the band's row area,
        // not to the bottom of whatever this group happened to draw. A
        // one-row group and a two-row group therefore caption on the
        // same baseline — `justify-content: space-between`, and the
        // reason the mockup's band reads as staged rather than ragged.
        ui.add_space((box_.pad_top + box_.rows - (ui.cursor().top() - top)).max(0.0));
        ui.add_space(CAPTION_GAP);

        // ★ `.size(…)` rather than `.small()`, since 2026-09-04. The mockup's
        // `.grp .cap { font-size: 11px }` is two points above `egui`'s
        // `TextStyle::Small` (9 pt), and a caption that small under an 11 pt
        // band reads as a footnote rather than as the name of the block above
        // it. `.weak()` is kept: `.cap { color: var(--ink-quiet) }` is
        // `Palette::text_muted`, which is what `weak` resolves to — and it is
        // NOT `.strong()`, so `check-strong-text` has nothing to say here.
        let caption = ui
            .allocate_ui_with_layout(vec2(widest, 0.0), Layout::top_down(Align::Center), |ui| {
                ui.label(
                    RichText::new(caption_text(group))
                        .weak()
                        .size(ctx.theme.metrics.ribbon_caption_pts),
                )
            })
            .inner;

        // Counted here, one line after the label that cannot be
        // reached without emitting it.
        outcome.captions_emitted += 1;
        ctx.reporter
            .report(caption.rect, || report::group_caption(tab_id, &group.id));

        // ★ And out to the band's own height. `allocate_space` rather
        // than `add_space`, because only an allocation grows a `Ui`'s
        // `min_rect` — `add_space` moves the cursor, which is what the
        // padding above it wanted and is exactly not what this wants.
        // Zero-width, so it changes nothing horizontally.
        //
        // What this closes: `band_height` predicts the caption's height
        // from [`caption_height`]'s row height, and the label allocates
        // whatever its galley measured. The two agree in every font this
        // crate has been run against, and if they ever stop agreeing the
        // band would be a fraction taller with groups in it than
        // without — R128 by a hair rather than by a row.
        // `the_band_keeps_its_height_at_widths_where_every_group_overflows`
        // is the tripwire.
        let drawn = ui.cursor().top() - top;
        if box_.total > drawn {
            ui.allocate_space(vec2(0.0, box_.total - drawn));
        }
    });
}

// ---------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------

/// **How one group wraps, and the width it will occupy once it has.**
///
/// The two answers come from one call because they are one decision: a
/// group's width *is* its widest row, and its widest row is decided by the
/// wrap. Returning them together is what makes it impossible for
/// [`render_band`] to plan against one split and draw another — the failure
/// that would show up as a clipped group and never as anything a reader
/// would recognise as a bug.
fn measure_group(ui: &egui::Ui, ctx: &Ctx<'_>, group: &Group) -> (GroupRows, f32) {
    measure_group_rows(ui, ctx, group, plan::GROUP_ROWS)
}

/// The same measurement, at a stated row ceiling.
///
/// ★ S5's whole implementation on the measuring side. `wrap_group` already
/// searches for the **narrowest** packing that fits within the row limit it is
/// handed, so asking it for three rows instead of two returns the three-row
/// layout when that is narrower and the two-row one when it is not — no new
/// packing logic, and no way for the two answers to disagree about how a group
/// wraps.
///
/// Called twice per group per frame, which is affordable: it is arithmetic over
/// a list of item widths that were going to be measured anyway, and the
/// alternative — caching last frame's answer — is the feedback-loop shape this
/// project has paid for three times.
fn measure_group_rows(
    ui: &egui::Ui,
    ctx: &Ctx<'_>,
    group: &Group,
    max_rows: usize,
) -> (GroupRows, f32) {
    let (large, rest) = partition(ctx, group);
    let widths: Vec<f32> = rest
        .iter()
        .map(|item| measure_item(ui, ctx, item))
        .collect();
    let rows = plan::wrap_group(
        &widths,
        ctx.theme.metrics.gutter,
        max_rows,
        plan::GROUP_WRAP_WIDTH,
        // ★ The manifest's own answer, `OPERATOR_REQUESTS.md` O97. `None` is
        // every group that has not asked, which is almost all of them.
        group.preferred_rows().map(|rows| rows as usize),
    );
    // The Large run leads, then a gutter, then the wrapped rows — the same
    // order `group_body` draws them in, and it must be, or the band plans
    // against one width and clips at another.
    let gutter = ctx.theme.metrics.gutter;
    let mut lead = 0.0_f32;
    for item in &large {
        if lead > 0.0 {
            lead += gutter;
        }
        lead += measure_item(ui, ctx, item);
    }
    if lead > 0.0 && !rest.is_empty() {
        lead += gutter;
    }
    // ★★ Measured in the font the caption is actually DRAWN in, since
    // 2026-09-04. It read `&TextStyle::Small` until the mockup pass raised the
    // caption to `Metrics::ribbon_caption_pts` (9 pt → 11), and a group
    // measured against a 9 pt caption and drawn with an 11 pt one is a group
    // whose caption is wider than the box the planner reserved for it — the
    // caption is centred on the group's width, so the overflow is silent and
    // symmetric, half a word past each edge and over the separator rule.
    //
    // This is the same "one decision written twice" trap `sizing`'s header
    // warns about, arriving through a font rather than through arithmetic.
    let caption = ui.ctx().fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(
                caption_text(group).to_owned(),
                caption_font(ctx),
                egui::Color32::PLACEHOLDER,
            )
            .size()
            .x
    });
    // ★ `lead + rows.width` is the whole of what the controls occupy: the
    // Large run, then the wrapped rows beside it. With no Large run this is
    // exactly what the planner computed before sizes existed, so every width
    // test that pins the old arithmetic still pins it.
    let width = plan::group_width(lead + rows.width, caption);
    (rows, width)
}

/// **Which items lead the group, and which wrap into its rows** — the visible
/// ones only.
///
/// Two rules in one pass, because both change what the group measures and a
/// second pass could apply one of them and not the other:
///
/// * an item whose `visible_when` does not hold is **dropped before
///   measurement**, so its space is reclaimed rather than reserved for a
///   control that never draws;
/// * a `Large` item **leads**. See [`sizing`]'s header for why a Large control
///   cannot live inside the row wrap, and why leading costs nothing an author
///   wanted.
fn partition<'a>(ctx: &Ctx<'_>, group: &'a Group) -> (Vec<&'a Item>, Vec<&'a Item>) {
    let mut large = Vec::new();
    let mut rest = Vec::new();
    for item in group.items() {
        if !sizing::visible(item, ctx.conditions) {
            continue;
        }
        if effective_size(ctx, item) == ItemSize::Large {
            large.push(item);
        } else {
            rest.push(item);
        }
    }
    (large, rest)
}

/// The size an item will actually render at — the manifest's ask, after
/// [`sizing::resolved`] has had its say about whether `Small` was earned.
///
/// A separator, a custom item and an unregistered command have one
/// presentation each and report the default.
fn effective_size(ctx: &Ctx<'_>, item: &Item) -> ItemSize {
    match item {
        Item::Command { id, size, .. } => {
            ctx.registry.get(id).map_or(ItemSize::Medium, |command| {
                sizing::resolved(command, *size, ctx.icons.is_some())
            })
        }
        Item::Separator | Item::Custom { .. } => ItemSize::Medium,
    }
}

/// The width one item will occupy.
///
/// # Two corrections that only matter once text has a width
///
/// **The icon/label gap is `icon_spacing`, not `gutter`.** A control with
/// both halves is drawn as an `egui::Atoms` row, and `AtomLayout` spaces
/// its atoms by `ui.spacing().icon_spacing`
/// (`egui-0.35.0/src/atomics/atom_layout.rs`), which this crate's theme
/// does not set and therefore leaves at `egui`'s 4 pt. Estimating the gap
/// as the theme's `gutter` agrees with that by coincidence at the compact
/// density and **under**-estimates by 4 pt per control at the comfortable
/// one. Under-estimating is the dangerous direction — it is the direction
/// that lets a group spill into space the band has already promised to
/// something else — so the estimate asks `egui` for the number `egui` will
/// use.
///
/// **A separator inside a group costs its line, not its line plus two
/// gaps.** [`separator_width`] is the cost of a rule *between two groups*,
/// which includes the `item_spacing` either side of it. Inside a group,
/// [`plan::group_width`] already adds one gutter between every adjacent
/// pair — including the pairs the separator forms with its neighbours — so
/// charging the full inter-group figure here counts those two gaps twice.
/// It over-estimated rather than under-estimated, so it hid a group early
/// instead of clipping one, which is why nothing caught it; it was still
/// wrong by 2 × `gutter` for every separator in the manifest.
fn measure_item(ui: &egui::Ui, ctx: &Ctx<'_>, item: &Item) -> f32 {
    match item {
        Item::Separator => SEPARATOR_LINE,
        Item::Custom { .. } => CUSTOM_ITEM_WIDTH,
        Item::Command { id, size, .. } => match ctx.registry.get(id) {
            // An unknown id draws nothing (see `Ctx::command`), so it must
            // also measure nothing — otherwise the band reserves space for
            // a control that will not appear and the plan is wrong by
            // exactly the width of every stale reference in the manifest.
            None => 0.0,
            // ★ Through `sizing::width`, which is the one place the three
            // sizes are turned into a number, and whose every branch has a
            // matching branch in `render_command`. Measuring here and drawing
            // there from two different rules is how a band that claims to fit
            // clips its last group.
            Some(command) => sizing::width(
                ui,
                ctx,
                command,
                sizing::resolved(command, *size, ctx.icons.is_some()),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **★ A caption is never empty, whatever the manifest says.**
    ///
    /// The fallback chain — caption → id → a literal — is what makes
    /// "every rendered group emits a caption" a *total* claim rather than
    /// one that holds only for validated manifests. An unvalidated
    /// manifest is exactly the input this module has to survive, because
    /// the defect it exists to prevent is a band of unlabelled controls
    /// and an empty caption reproduces it precisely.
    ///
    /// Falling back to the id is also diagnostic: `page_display` sitting
    /// in a caption slot names the manifest entry that needs fixing,
    /// where a blank names nothing.
    #[test]
    fn a_caption_is_never_empty_even_for_an_unvalidated_group() {
        assert_eq!(caption_text(&Group::new("render", "Render")), "Render");
        assert_eq!(
            caption_text(&Group::patch("page_display")),
            "page_display",
            "a caption-less group must announce its own id, not a blank"
        );
        let blank = Group {
            id: "window".to_owned(),
            caption: Some("   ".to_owned()),
            items: None,
            collapse: None,
            prefer_rows: None,
        };
        assert_eq!(
            caption_text(&blank),
            "window",
            "whitespace is as invisible as an empty string and must fall through"
        );
        let nameless = Group {
            id: String::new(),
            caption: None,
            items: None,
            collapse: None,
            prefer_rows: None,
        };
        assert_eq!(caption_text(&nameless), "(unnamed group)");

        // Total claim: no input produces an empty caption.
        for g in [
            Group::new("a", "A"),
            Group::patch("b"),
            blank,
            nameless,
            Group {
                id: String::new(),
                caption: Some(String::new()),
                items: None,
                collapse: None,
                prefer_rows: None,
            },
        ] {
            assert!(!caption_text(&g).trim().is_empty(), "{g:?}");
        }
    }

    /// The selected-condition convention is stable and cannot collide
    /// with an application's own dotted condition names.
    ///
    /// The `:` is load-bearing: with a `.` an application could
    /// accidentally define a real condition called
    /// `selected.view.single` and turn a toggle on from a distance.
    #[test]
    fn the_selected_condition_name_is_namespaced() {
        assert_eq!(selected_condition("view.single"), "selected:view.single");
        assert!(selected_condition("x").starts_with(SELECTED_CONDITION_PREFIX));
        assert!(
            !SELECTED_CONDITION_PREFIX.contains('.'),
            "a dotted prefix could collide with an application's own condition names"
        );
    }
}
