//! # `status::fitting` — what the bar can afford to show when the window is narrow
//!
//! ## The defect this exists for
//!
//! Found 2026-08-26 by `ui-verify ui_scale_resizes_the_chrome`, in the first
//! driven run of this project's history in which every check launched. At
//! `ui_scale = 1.80` in an 1100 × 800 window — 611 × 444 **points** — five of
//! the status bar's declared regions lay outside the client area:
//!
//! ```text
//! status-group:page    457.4 .. 603.1     ok
//! status-group:zoom    326.7 .. 435.4     ok
//! status-group:fit       6.6 .. 304.7     ok, and 298 pt wide — half the bar
//! status-group:find    -54.2 ..  -15.4    ★ off the left edge
//! status-group:filter -127.5 ..  -76.2    ★ off the left edge
//! ```
//!
//! **Find and the selection filter were unreachable**, and `status-group:notes`
//! sat at 8 .. 114, underneath the fit group, so the left-hand notes and the
//! right-hand cluster were drawn on top of each other.
//!
//! ★ It is the redaction-apply defect's shape — a control declared outside the
//! body it lives in — reached by *scaling* rather than by adding copy, which is
//! why no layout test saw it. Every test in this crate measures at
//! `ui_scale = 1.0`.
//!
//! ## ★★ Why the existing argument did not hold, and it is worth reading
//!
//! `status.rs` already argued the case, correctly, for the mechanism it chose:
//!
//! > *"A right-to-left layout cannot get that wrong; it simply runs out of
//! > room, and the notes on the left are what yields."*
//!
//! Right-to-left **is** the right layout, and the alternative it rejected —
//! `right − width`, which goes negative the moment the bar is narrower than its
//! content — is genuinely worse. What the argument missed is that *running out
//! of room* is not a graceful state. egui does not clip a `Layout` to its
//! parent; a right-to-left run whose content exceeds the available width simply
//! continues past the left edge into negative coordinates. The notes do not
//! yield, they are **overdrawn**.
//!
//! So the layout was never wrong. What was missing was anybody asking *does
//! this fit* before adding the next thing.
//!
//! ## The rule, and where it comes from
//!
//! **Shed from the low-priority end until the rest fits.** That is what a
//! status bar does everywhere it is done well — Word drops items as the window
//! narrows, VS Code hides them by declared priority, and browsers do the same
//! with their own. Nothing announces it, and nothing should: a status bar is a
//! summary surface, and an overflow chevron on one is a control about a control.
//!
//! ★ **Relative order survives any subset**, which is what makes shedding by
//! priority safe rather than disruptive. A right-to-left run draws whatever it
//! is given in the order it is given; removing an item from the middle closes
//! the gap without moving anything past anything else. So `Filter · Zoom ·
//! Page` reads left to right exactly as `Filter · Find · Fit · Zoom · Page`
//! does, minus two. `status::fit`'s warning about not reordering controls the
//! operator has learned is about the four fit buttons **among themselves**; it
//! does not bind here.
//!
//! ## ★★★ THE CLAUSE THAT DECIDES EVERYTHING: nothing may shed its only home
//!
//! This is what makes the design legitimate rather than convenient, and it is
//! **enforced, not asserted** — [`SHED_ORDER`] names each group this module may
//! drop beside a command that still reaches it, and
//! [`tests::nothing_sheddable_loses_its_last_route`] resolves every one through
//! the real command registry.
//!
//! ★★ **That test immediately refused the first design, and it was right.**
//! The obvious rule — shed from the low-priority end, which in this layout is
//! the left — would have dropped the **selection filter** first, since it is
//! leftmost. The filter has **no ribbon command, no menu entry and no
//! shortcut**: it exists only on this bar. Shedding it makes it unreachable,
//! which is the very defect being fixed, moved from 1.80 scale to 1.60.
//!
//! Checking the rest against the registry left exactly two groups that may go:
//!
//! | group | pt | may be shed? | why |
//! |---|---:|---|---|
//! | Fit | 298.1 | **yes** | all four buttons are View ▸ Zoom commands |
//! | Find | 38.8 | **yes** | `edit.find`, and `Ctrl+F` |
//! | Zoom | 108.7 | no | `+`/`−` have no command; the readout has no other home |
//! | Filter | 51.3 | no | **no other home at all** |
//! | Page | 145.7 | no | the only answer to *which page am I on* |
//!
//! And it is enough: at the 611 pt width that found the defect, dropping the
//! fit group alone takes the cluster from 666 pt to 362 pt.
//!
//! ★ Two of the five being unsheddable is a finding about the **ribbon**, not
//! about this module, and it is recorded rather than worked around: the
//! selection filter and the zoom stepper are status-bar-only capabilities. If
//! either acquires a home, it can join the list — one line, and the test will
//! confirm it.
//!
//! ## Measuring: last frame's rect, not a recomputed estimate
//!
//! [`Widths`] remembers what each group actually occupied on the previous frame,
//! taken from the same rect the group already publishes for the harness. The
//! alternative — a `min_width()` per group that re-measures its own labels —
//! is a second implementation of egui's layout that would drift from the first
//! the day a separator's padding changed, and it would drift *silently*, in the
//! direction of a bar that thinks it fits and does not.
//!
//! The cost is one frame: a window resized in a single step shows the old
//! decision once before correcting. During a drag that is invisible, and on the
//! first frame after start-up the bar simply shows everything, which is the
//! behaviour it had before this module existed.

use std::collections::BTreeMap;

/// The groups of the bar's fixed right-hand cluster, in the order
/// `status::bar` adds them — which is right to left on screen, and
/// most-essential first.
///
/// A plain enum rather than the region-name strings, so a caller cannot ask
/// about a group that does not exist and the exhaustiveness of [`SHED_ORDER`] is
/// the compiler's problem rather than a reviewer's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    /// `◀ n / N ▶`. **Never shed** — see the module header.
    Page,
    /// `− 100 % +`.
    Zoom,
    /// `Actual size · Fit width · Fit height · Fit page`. The widest of the
    /// five by a factor of three, and therefore the one whose absence buys the
    /// most.
    Fit,
    /// The Find toggle.
    Find,
    /// The selection filter.
    Filter,
}

impl Group {
    /// Every group, in the order the bar adds them.
    pub const ORDER: [Self; 5] = [Self::Page, Self::Zoom, Self::Fit, Self::Find, Self::Filter];

    /// The region name this group publishes its rect under, which is also the
    /// key its width is remembered by.
    ///
    /// Deliberately the **published** name rather than a private key: the width
    /// is remembered from the rect the harness reads, so one name for both
    /// makes it impossible for the two to describe different widgets.
    pub const fn region(self) -> &'static str {
        match self {
            // ui-text-exempt: trace region names, never displayed
            Self::Page => "status-group:page",
            Self::Zoom => "status-group:zoom",
            Self::Fit => "status-group:fit",
            Self::Find => "status-group:find",
            Self::Filter => "status-group:filter",
        }
    }
}

/// ★★★ **The groups this module may drop, in the order it drops them, each
/// beside a command that still reaches it.**
///
/// Ordered by *what dropping it buys against what it costs* — the fit group is
/// 298 points, nearly half the bar at the width that found the defect, and its
/// four buttons are all View ▸ Zoom commands; Find is 39 points and is a
/// keystroke away. Everything not in this list is undroppable, and the module
/// header's table says why for each.
///
/// The second field is a command id and it is **checked against the real
/// registry** by [`tests::nothing_sheddable_loses_its_last_route`], which is
/// what stopped the first version of this list shedding a control that has no
/// other home anywhere in the program.
pub const SHED_ORDER: &[(Group, &str)] = &[
    // ui-text-exempt: command ids, never displayed
    (Group::Fit, "view.zoom_fit_page"),
    (Group::Find, "edit.find"),
];

/// What each group occupied on the previous frame, in points.
///
/// A `BTreeMap` over five keys rather than a struct of five `f32`s: the map is
/// **partial**, and that is the point. A group that has never been drawn — no
/// document open, or the first frame after start-up — has no entry, and
/// [`affordable`] treats an unknown width as *"show it and find out"*, which is
/// exactly right for a bar that has not yet been measured.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Widths(BTreeMap<Group, f32>);

impl Widths {
    /// Remember what `group` occupied.
    ///
    /// Non-finite and negative widths are dropped rather than stored: egui can
    /// report a degenerate rect for a widget laid out in a zero-width parent,
    /// and a `NaN` in here would poison every comparison in [`affordable`] into
    /// answering `false` — which would shed the whole cluster permanently, from
    /// one bad frame.
    pub fn record(&mut self, group: Group, width: f32) {
        if width.is_finite() && width >= 0.0 {
            self.0.insert(group, width);
        }
    }

    /// What `group` occupied last time, if it has ever been drawn.
    #[must_use]
    pub fn get(&self, group: Group) -> Option<f32> {
        self.0.get(&group).copied()
    }
}

/// How much room a separator between two groups needs, in points.
///
/// egui's `ui.separator()` is a fixed spacing plus a hairline and does not vary
/// with content, so unlike the groups themselves it is a constant rather than a
/// measurement. Six points is `Spacing::item_spacing.x` (4) plus the rule (2)
/// at this crate's theme; it is deliberately a slight **over**-estimate, which
/// biases the decision towards shedding one group too early rather than one too
/// late. Too early costs a control that is reachable elsewhere; too late puts
/// it at negative x, which is the defect.
const SEPARATOR_PTS: f32 = 6.0;

/// ★ **Which of the cluster's groups fit in `available` points.**
///
/// Returns them in [`Group::ORDER`] — the order `status::bar` must add them —
/// with the undroppable ones always present and the droppable ones removed, in
/// [`SHED_ORDER`], until the rest fits.
///
/// # It can return more than fits, and that is deliberate
///
/// If every droppable group is gone and the remainder still overflows, the
/// remainder is returned anyway. There is nothing left this function is allowed
/// to drop — each of the three is the operator's only route to what it says —
/// and a window that narrow is past what any shedding rule can rescue. A bar
/// that overflows by a little is a better outcome than one that has thrown away
/// the page number, and the alternative is to start making a capability
/// unreachable, which is the defect this module exists to prevent.
#[must_use]
pub fn affordable(available: f32, widths: &Widths) -> Vec<Group> {
    let mut shown: Vec<Group> = Group::ORDER.to_vec();
    if !available.is_finite() {
        // A non-finite available width means the parent is mid-layout or
        // degenerate. Show everything: that is the pre-2026-08-26 behaviour,
        // and it is the right fallback because it fails towards *visible*.
        return shown;
    }
    for (group, _) in SHED_ORDER {
        if measured_width(&shown, widths) <= available {
            break;
        }
        shown.retain(|g| g != group);
    }
    shown
}

/// What `shown` would occupy: the groups' own widths plus a separator between
/// each adjacent pair.
///
/// A group with no remembered width contributes **nothing**, which is what
/// makes the bootstrap work: on the first frame nothing has been measured, the
/// total is zero, and everything is shown — so everything gets measured. See
/// [`Widths`].
fn measured_width(shown: &[Group], widths: &Widths) -> f32 {
    let known: Vec<f32> = shown.iter().filter_map(|g| widths.get(*g)).collect();
    // ★ Separators are counted between MEASURED groups, not between shown ones.
    //
    // Counting them per shown group broke the bootstrap and the test caught it:
    // with nothing yet measured the sum is zero but four separators are 24 pt,
    // so a bar narrower than that shed two groups on its very first frame —
    // before either had ever been drawn, and therefore before either could
    // acquire the width that would have let it back. A rule that cannot
    // bootstrap is not a rule.
    let separators = SEPARATOR_PTS * (known.len().saturating_sub(1)) as f32;
    known.iter().sum::<f32>() + separators
}

/// Where a shed group is still reachable, if this module may shed it.
///
/// The read side of [`SHED_ORDER`], used by [`trace_shed`] so a diagnostic
/// naming a dropped control also names where it went. `None` for a group that
/// is never shed.
#[must_use]
pub fn still_reachable_at(group: Group) -> Option<&'static str> {
    SHED_ORDER
        .iter()
        .find(|(g, _)| *g == group)
        .map(|(_, command)| *command)
}

/// ★★ **Say what the bar dropped, and where it still is.**
///
/// Emitted on change only, from `status::bar`, once the decision is made.
///
/// # Why this is traced when nothing else about the bar's layout is
///
/// Because **absence is not evidence**, and a driven check has nothing else to
/// read. A shed group publishes no `ui_rect` — but neither does a group that is
/// merely scrolled out of view, nor one whose widget failed to build, nor one
/// the mode does not offer. Four causes, one symptom, and a harness reading
/// only the region list cannot tell them apart.
///
/// So the bar states its own decision. `status-shed groups=none` is the
/// ordinary case and says the window is wide enough; anything else names what
/// went and where an operator can still reach it, which is the fact the
/// shedding rule's legitimacy rests on.
pub fn trace_shed(shown: &[Group]) {
    let dropped: Vec<Group> = Group::ORDER
        .into_iter()
        .filter(|g| !shown.contains(g))
        .collect();
    crate::diag::trace_changed(SHED_SLOT, move || {
        if dropped.is_empty() {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            return "status-shed groups=none".to_owned();
        }
        let named: Vec<String> = dropped
            .iter()
            .map(|g| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "{}@{}",
                    g.region(),
                    still_reachable_at(*g).unwrap_or("nowhere")
                )
            })
            .collect();
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("status-shed groups={}", named.join(","))
    });
}

/// The de-duplication slot for [`trace_shed`]. A line per frame at sixty hertz
/// is not a diagnostic.
const SHED_SLOT: &str = "status-shed"; // ui-text-exempt: a trace slot key, never displayed

#[cfg(test)]
mod tests {
    use super::*;

    /// A cluster whose five groups measure the widths the defect was found at.
    ///
    /// Taken from the trace of the failing run rather than invented, so the
    /// thresholds below are the real ones: page 145.7, zoom 108.7, fit 298.1,
    /// find 38.8, filter 51.3 — measured at `ui_scale = 1.80` in a 611 pt bar.
    fn measured() -> Widths {
        let mut w = Widths::default();
        w.record(Group::Page, 145.7);
        w.record(Group::Zoom, 108.7);
        w.record(Group::Fit, 298.1);
        w.record(Group::Find, 38.8);
        w.record(Group::Filter, 51.3);
        w
    }

    /// ★★★ **The defect, as an assertion.**
    ///
    /// 611 points is the client width the failing run reported. The whole
    /// cluster needs 145.7 + 108.7 + 298.1 + 38.8 + 51.3 = 642.6 plus four
    /// separators = 666.6 — so it cannot fit, and before this module it did not
    /// fit *and was drawn anyway*, at negative x.
    ///
    /// Dropping the fit group alone takes it to 362.5, which is what the
    /// assertion below checks: the biggest, least essential group goes, and
    /// nothing else has to.
    #[test]
    fn at_the_scale_that_found_the_defect_the_cluster_sheds_rather_than_overflows() {
        let widths = measured();
        let shown = affordable(611.0, &widths);
        assert!(
            !shown.contains(&Group::Fit),
            "the fit group is 298 pt of a 611 pt bar and must be the first to go"
        );
        assert!(
            shown.contains(&Group::Find),
            "dropping the fit group is enough on its own; Find should not also go"
        );
        assert!(
            measured_width(&shown, &widths) <= 611.0,
            "what is shown must fit: {} pt of 611, showing {shown:?}",
            measured_width(&shown, &widths)
        );
    }

    /// A wide bar shows everything, which is the property that stops this being
    /// a regression at the size every operator actually uses.
    #[test]
    fn a_wide_bar_sheds_nothing() {
        assert_eq!(affordable(2000.0, &measured()), Group::ORDER.to_vec());
    }

    /// ★★ **Relative order is preserved under every subset.**
    ///
    /// The property that keeps the bar's controls in the positions the operator
    /// learned. Shedding is by priority, not by position, so the result is not
    /// a prefix — but whatever survives must still read in the same order.
    /// Swept across every width from nothing to generous, because the
    /// interesting failures are at the boundaries and picking two points either
    /// side of a transition looks exactly like no transition at all.
    #[test]
    fn every_width_keeps_the_groups_in_order() {
        let widths = measured();
        let mut width = 0.0_f32;
        while width < 1200.0 {
            let shown = affordable(width, &widths);
            let expected: Vec<Group> = Group::ORDER
                .into_iter()
                .filter(|g| shown.contains(g))
                .collect();
            assert_eq!(
                shown, expected,
                "at {width} pt the bar showed {shown:?}, which is out of order"
            );
            width += 3.0;
        }
    }

    /// ★★★ **The two groups with no other home are never shed, at any width.**
    ///
    /// The clause the whole design rests on, swept rather than sampled. The
    /// selection filter has no ribbon command, no menu entry and no shortcut —
    /// it exists only on this bar — and the zoom stepper's `+`/`−` have no
    /// command either. Dropping either would move the very defect this module
    /// fixes from 1.80 scale to a narrower one, which is not a fix.
    #[test]
    fn a_group_with_no_other_home_survives_every_width() {
        let widths = measured();
        let mut width = 0.0_f32;
        while width < 1200.0 {
            let shown = affordable(width, &widths);
            for group in [Group::Page, Group::Zoom, Group::Filter] {
                assert!(
                    shown.contains(&group),
                    "{} was shed at {width} pt and has nowhere else to be reached",
                    group.region()
                );
            }
            width += 3.0;
        }
    }

    /// ★★ **Narrowing never puts a control back.**
    ///
    /// A monotonicity property, and the one a hand-written threshold ladder
    /// would break first: as the bar narrows, the set shown must only ever
    /// shrink. A bar that dropped Find at 600 pt and showed it again at 590
    /// would flicker as the window is dragged.
    #[test]
    fn a_narrower_bar_never_shows_more() {
        let widths = measured();
        let mut previous = usize::MAX;
        let mut width = 1200.0_f32;
        while width > 0.0 {
            let count = affordable(width, &widths).len();
            assert!(
                count <= previous,
                "narrowing to {width} pt showed {count} groups, up from {previous}"
            );
            previous = count;
            width -= 3.0;
        }
    }

    /// The page box survives any width, including absurd ones.
    #[test]
    fn the_page_number_is_never_shed() {
        for width in [0.0_f32, 1.0, 50.0, 145.6, f32::NAN, f32::NEG_INFINITY] {
            assert!(
                affordable(width, &measured()).contains(&Group::Page),
                "the page number went missing at {width} pt"
            );
        }
    }

    /// An unmeasured bar shows everything, so a group can acquire a width.
    ///
    /// Without this the first frame would shed every group whose width is
    /// unknown — which is all of them — and they would never be drawn, so they
    /// would never be measured, and the bar would be permanently empty. A
    /// bootstrap that cannot bootstrap.
    #[test]
    fn nothing_is_shed_before_it_has_ever_been_measured() {
        assert_eq!(
            affordable(10.0, &Widths::default()),
            Group::ORDER.to_vec(),
            "a bar with no measurements must show everything, or it can never take one"
        );
    }

    /// A poisoned measurement cannot empty the bar.
    #[test]
    fn a_degenerate_measurement_is_refused_rather_than_stored() {
        let mut w = Widths::default();
        w.record(Group::Fit, f32::NAN);
        w.record(Group::Find, -5.0);
        assert_eq!(w.get(Group::Fit), None, "a NaN width must not be stored");
        assert_eq!(
            w.get(Group::Find),
            None,
            "a negative width must not be stored"
        );
    }

    /// ★★★ **Nothing this module may shed loses its last route.**
    ///
    /// The clause that makes shedding legitimate, checked against the real
    /// command registry rather than against a comment. A group whose ribbon
    /// home was renamed or deleted would fail here — loudly, in a unit test —
    /// rather than quietly becoming unreachable at a UI scale nobody on this
    /// project runs at.
    #[test]
    fn nothing_sheddable_loses_its_last_route() {
        // The real registry, built exactly as start-up builds it. Not a list
        // of ids restated here: a restatement is what goes stale.
        let mut registry = egui_shell::commands::CommandRegistry::default();
        crate::shell::commands::register(&mut registry);
        let ids: std::collections::HashSet<&str> = registry.ids().collect();
        for (group, command) in SHED_ORDER {
            assert!(
                ids.contains(command),
                "`{}` may be shed from the status bar when the window is narrow, and its stated \
                 remaining route `{command}` is not a registered command. Either the command was \
                 renamed — fix the id here — or the group's only other home was deleted, in which \
                 case it must be removed from SHEDDABLE and the bar must stop dropping it.",
                group.region()
            );
        }
        assert!(
            !SHED_ORDER.iter().any(|(g, _)| *g == Group::Page),
            "the page box has no other home and must never be sheddable"
        );
    }
}
