//! # `ribbon::plan::collapse` — the compaction ladder
//!
//! **The re-wrap and collapse rungs of `RIBBON_SCALING.md` §1's ladder, as one
//! mechanism.** When a band does not
//! fit, this decides how compact each group must be drawn — and in what order
//! the compactions are applied — before anything is pushed into the overflow
//! affordance at all.
//!
//! ## What Word actually does, measured rather than remembered
//!
//! The specification for this file is a series of photographs taken at twelve
//! window widths by `tools/word-ribbon-study.ps1` (`RIBBON_SCALING.md` §3 —
//! the images themselves are build output and are not committed), because a
//! ribbon's
//! scaling rules are implemented inside the Office UI framework and exposed
//! through no API — `Application.CommandBars` is the 2003 toolbar surface and
//! says nothing about any of this. Four readings decide everything here:
//!
//! | width | what the Home tab shows |
//! |---:|---|
//! | 1900 | everything expanded; **Font is two rows** |
//! | 1300 | still all expanded, Font and Paragraph on two rows |
//! | 1000 | **Font is THREE rows**; Paragraph has collapsed to a button |
//! | 460 | Font, Paragraph, Styles, Editing all collapsed; **Clipboard still expanded**; a `›` scroll arrow at the band's right edge |
//!
//! Four facts follow, and every one is counter-intuitive enough to be worth
//! stating.
//!
//! 1. **Groups re-wrap onto MORE rows as the window narrows** — Font is two
//!    rows at 1900 and three at 1000, the size stepper and the case/clear
//!    controls dropping to a third row on the way.
//!
//!    This is the reading the series is easiest to get backwards, and the way
//!    to get it backwards is to read two frames out of the twelve. Compare
//!    1300 with 800 and the reflow is in neither photograph, because by 800
//!    the group has already collapsed. *Sampling either side of a transition
//!    and concluding there is no transition* is the shape of that mistake;
//!    `D:/dev/rag/egui/` carries it as a finding, because it is repeatable and
//!    it is not carelessness — the two endpoints agreeing is exactly what
//!    makes the conclusion feel safe.
//!
//! 2. **Re-wrapping comes before collapsing, and no group may decline it.**
//!    Re-wrapping hides nothing: every control stays on the band, labelled.
//!    Collapsing hides all of them behind a popup. So it is always better to
//!    re-wrap five groups than to collapse one, and the ladder exhausts the
//!    re-wrap rung completely before it begins the collapse rung.
//!
//! 3. **A group can decline to collapse, and the one that declines is not the
//!    small one.** Clipboard is wider than Editing and outlives it at every
//!    width down to 460. The property being selected on is *importance*, which
//!    is editorial, which is why it is [`crate::manifest::Group::collapse`] in
//!    the manifest and not a heuristic here.
//!
//! 4. **Collapsing comes before scrolling.** The `›` arrow appears at 460 and
//!    not at 800, by which point four groups have already collapsed. A surface
//!    that scrolled first would be hiding commands while the space to show
//!    them, compacted, was still there.
//!
//! ## The `plan_band` invariants, restated
//!
//! `RIBBON_SCALING.md` names this stage's real cost: `super::plan_band`'s
//! invariants have to hold in a world where a group can **shrink** instead of
//! vanishing.
//!
//! **`the_visible_groups_are_a_prefix_and_nothing_is_lost`.** True, and it
//! says something stronger for it. The shown groups remain a prefix of the
//! manifest order; what changes is that a group in that prefix may be present
//! in a compacted form. Nothing is lost in any of them — a re-wrapped group
//! draws every control, and a collapsed group's are all reachable through its
//! own popup, exactly as a hidden group's are through the overflow affordance.
//! The count that must be conserved is `shown + hidden == n` regardless of how
//! many of the shown are compacted, and [`fit`] never changes `n`.
//!
//! **`widening_the_band_never_hides_a_group_that_was_visible`.** This is the
//! one that needs real care, because the obvious implementation breaks it.
//! Monotonicity has **three** rungs and all must hold: widening never
//! re-wraps a group that was natural, never collapses one that was re-wrapped,
//! and never hides one that was visible in any form.
//!
//! [`fit`] gets this by construction rather than by testing for it: it always
//! starts from *everything natural* and advances forward in a fixed order until
//! it fits. **It never starts from the previous frame's answer.** So the state
//! of a group at width `w` is a pure function of `w`, it only ever moves back
//! up the ladder as `w` grows, and there is no hysteresis to tune.
//!
//! That is also why this is a free function over slices rather than a method
//! on something that remembers. **A layout that remembers what it did last
//! frame is how a ribbon acquires a width at which it flickers** — a
//! measurement fed back into the size that produced it, which is the loop R128
//! forbids.

/// How compact one group is being drawn.
///
/// Ordered widest-first, which is the order the ladder advances through and the
/// order [`State::next`] returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum State {
    /// The group's own natural row split — at most [`super::GROUP_ROWS`] rows.
    /// What every group gets when the band has room.
    Natural,
    /// Re-wrapped onto up to [`super::MAX_GROUP_ROWS`] rows, which is narrower.
    ///
    /// **Non-destructive**: every control is still drawn, still labelled,
    /// still on the band. That is why this rung comes before collapsing and why
    /// no group is exempt from it.
    Rewrapped,
    /// Drawn as a single captioned button, its contents in a popup.
    ///
    /// **Destructive** — the controls are no longer on the band — which is why
    /// a group may decline it. See [`crate::manifest::Group::collapse`].
    Collapsed,
}

impl State {
    /// The next rung down, or `None` at the bottom.
    #[allow(dead_code)] // Used by the ordering documentation and by tests.
    fn next(self) -> Option<Self> {
        match self {
            Self::Natural => Some(Self::Rewrapped),
            Self::Rewrapped => Some(Self::Collapsed),
            Self::Collapsed => None,
        }
    }
}

/// One group, as the ladder needs to see it.
///
/// Deliberately not the manifest's `Group`: the ladder needs three widths and a
/// priority, and keeping it that way is what makes it testable without building
/// a manifest, a registry and a font.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Candidate {
    /// What the group costs at its natural row split.
    pub natural: f32,
    /// What it costs re-wrapped onto up to [`super::MAX_GROUP_ROWS`] rows.
    ///
    /// May equal [`Self::natural`] — a group of three items has nowhere to go —
    /// and the ladder gains nothing by advancing such a group, which [`fit`]
    /// detects by measuring rather than by asking.
    pub rewrapped: f32,
    /// What it costs as a single captioned button with a chevron.
    pub collapsed: f32,
    /// Its rung on the collapse ladder, or `None` for a group that never
    /// collapses.
    ///
    /// This gates **collapsing only**. Every group re-wraps, including one
    /// that declines to collapse: Word's Clipboard never collapses, and Font,
    /// which does, still re-wraps from two rows to three on the way there.
    /// Re-wrapping hides nothing, so there is nothing to decline.
    pub priority: Option<u32>,
}

impl Candidate {
    /// This group's width in a given state.
    fn width(self, state: State) -> f32 {
        match state {
            State::Natural => self.natural,
            State::Rewrapped => self.rewrapped,
            State::Collapsed => self.collapsed,
        }
    }

    /// Whether advancing to `state` is worth anything at all.
    ///
    /// Measured, not assumed. A group whose three-row layout is no narrower
    /// than its natural one — anything with few enough items — would otherwise
    /// consume a rung of the ladder, change nothing, and make the band appear
    /// to stall at one width and jump at the next.
    fn gains_from(self, state: State) -> bool {
        match state {
            State::Collapsed => self.priority.is_some(),
            _ => self.width(state) < self.width(State::Natural),
        }
    }
}

/// **How compact each group must be for the band to fit.**
///
/// Returns a state per group, parallel to `groups`. A pure function of its
/// inputs, which is the whole of the monotonicity argument in this module's
/// header.
///
/// # The ladder
///
/// 1. Everything at [`State::Natural`]. If it fits, stop — a band that
///    compacted a group it had room for would be making itself harder to read
///    for no gain.
/// 2. **Re-wrap** every group that gets narrower by it, in manifest order.
///    This rung is exhausted before the next one begins.
/// 3. **Collapse** in authored priority order, skipping any group that
///    declines.
/// 4. Stop when it fits, or when the ladder is exhausted — at which point what
///    is left over goes to [`super::plan_band`] and its overflow affordance,
///    which was always going to be the last resort.
///
/// Each step re-measures rather than subtracting a precomputed saving,
/// because the two are not the same once separators are involved, and the
/// difference is exactly the kind of one-group-too-many error that shows up
/// only at a single window width.
pub(crate) fn fit(groups: &[Candidate], available: f32, separator: f32) -> Vec<State> {
    let mut states = vec![State::Natural; groups.len()];
    if groups.is_empty() {
        return states;
    }

    // A non-finite or negative width is not a width. Zero makes the degenerate
    // case the safe one — compact everything that may compact — rather than the
    // dangerous one, an infinite budget that compacts nothing and clips.
    let available = if available.is_finite() {
        available.max(0.0)
    } else {
        0.0
    };
    let separator = if separator.is_finite() {
        separator.max(0.0)
    } else {
        0.0
    };

    if width_of(groups, &states, separator) <= available {
        return states;
    }

    // Rung 2 — re-wrap, in manifest order. No priority: see `Candidate::
    // priority`. Left to right is what a reader watching the band narrow would
    // predict.
    for i in 0..groups.len() {
        if groups[i].gains_from(State::Rewrapped) {
            states[i] = State::Rewrapped;
            if width_of(groups, &states, separator) <= available {
                return states;
            }
        }
    }

    // Rung 3 — collapse, in authored priority order. `(priority, index)` with a
    // stable sort gives manifest order within a priority for free.
    let mut ladder: Vec<(u32, usize)> = groups
        .iter()
        .enumerate()
        .filter_map(|(i, g)| g.priority.map(|p| (p, i)))
        .collect();
    ladder.sort_by_key(|(p, _)| *p);

    for (_, i) in ladder {
        states[i] = State::Collapsed;
        if width_of(groups, &states, separator) <= available {
            break;
        }
    }
    states
}

/// What the band costs with these states applied.
///
/// `n` groups carry `n − 1` separators, and a compacted group still occupies a
/// slot — it is narrower, not absent. Conflating "compacted" with "gone" is the
/// mistake that would break the conservation half of
/// `the_visible_groups_are_a_prefix_and_nothing_is_lost`.
fn width_of(groups: &[Candidate], states: &[State], separator: f32) -> f32 {
    let sum: f32 = groups.iter().zip(states).map(|(g, &s)| g.width(s)).sum();
    sum + separator * (groups.len().saturating_sub(1)) as f32
}

/// The widths [`super::plan_band`] should be given, once the ladder has run.
///
/// A convenience so the caller does not re-derive the same `match` in a third
/// place — the states and the widths must agree, and the cheapest way to
/// guarantee that is to produce them together.
pub(crate) fn widths_after(groups: &[Candidate], states: &[State]) -> Vec<f32> {
    groups
        .iter()
        .zip(states)
        .map(|(g, &s)| g.width(s))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three groups that each get meaningfully narrower at every rung.
    fn trio() -> Vec<Candidate> {
        vec![
            Candidate {
                natural: 100.0,
                rewrapped: 80.0,
                collapsed: 40.0,
                priority: Some(2),
            },
            Candidate {
                natural: 200.0,
                rewrapped: 140.0,
                collapsed: 40.0,
                priority: Some(1),
            },
            Candidate {
                natural: 150.0,
                rewrapped: 120.0,
                collapsed: 40.0,
                priority: Some(3),
            },
        ]
    }

    /// **The rungs are ordered widest-first**, which every `<=` comparison in
    /// the monotonicity sweep depends on.
    #[test]
    fn the_ladder_is_ordered_widest_first() {
        assert!(State::Natural < State::Rewrapped);
        assert!(State::Rewrapped < State::Collapsed);
        assert_eq!(State::Natural.next(), Some(State::Rewrapped));
        assert_eq!(State::Collapsed.next(), None);
    }

    /// **A band with room compacts nothing.** The first rung of the ladder is
    /// not standing on it.
    #[test]
    fn a_band_that_fits_is_left_alone() {
        // 100 + 200 + 150 + 2 separators of 8 = 466
        assert_eq!(
            fit(&trio(), 500.0, 8.0),
            vec![State::Natural, State::Natural, State::Natural]
        );
    }

    /// **Re-wrapping is exhausted before anything collapses.**
    ///
    /// The rung order is the whole of the ladder, and it is the property most
    /// likely to be broken by an "optimisation" that collapses the widest group
    /// first because that converges faster. It does converge faster and it is
    /// wrong: re-wrapping keeps every control on the band and collapsing hides
    /// them all, so five re-wraps beat one collapse however the arithmetic
    /// comes out.
    #[test]
    fn every_group_rewraps_before_any_group_collapses() {
        // Natural 466. All re-wrapped: 80 + 140 + 120 + 16 = 356.
        assert_eq!(
            fit(&trio(), 360.0, 8.0),
            vec![State::Rewrapped, State::Rewrapped, State::Rewrapped],
            "a width reachable by re-wrapping alone must not collapse anything"
        );
    }

    /// …and one re-wrap is enough when one is enough.
    #[test]
    fn the_rewrap_rung_stops_as_soon_as_it_fits() {
        // Re-wrapping only the first: 80 + 200 + 150 + 16 = 446.
        assert_eq!(
            fit(&trio(), 450.0, 8.0),
            vec![State::Rewrapped, State::Natural, State::Natural]
        );
    }

    /// **Collapsing begins only once re-wrapping has run out**, and then goes
    /// in priority order — and the groups already re-wrapped stay re-wrapped.
    #[test]
    fn collapsing_starts_after_the_rewrap_rung_and_follows_priority() {
        // Fully re-wrapped is 356; collapsing priority 1 (index 1) → 256.
        assert_eq!(
            fit(&trio(), 300.0, 8.0),
            vec![State::Rewrapped, State::Collapsed, State::Rewrapped]
        );
    }

    /// **A group with no priority never collapses — but it still re-wraps.**
    /// The Clipboard case, and the distinction between the two rungs.
    #[test]
    fn a_group_that_declines_to_collapse_still_rewraps() {
        let g = vec![
            Candidate {
                natural: 300.0,
                rewrapped: 220.0,
                collapsed: 40.0,
                priority: None,
            },
            Candidate {
                natural: 200.0,
                rewrapped: 150.0,
                collapsed: 40.0,
                priority: Some(1),
            },
        ];
        let states = fit(&g, 10.0, 8.0);
        assert_eq!(
            states[0],
            State::Rewrapped,
            "declining to COLLAPSE is not declining to re-wrap: re-wrapping \
             hides nothing, so there is nothing for a group to decline"
        );
        assert_eq!(states[1], State::Collapsed);
    }

    /// **A group that gains nothing by re-wrapping is not re-wrapped.**
    ///
    /// Otherwise it consumes a rung, changes no width, and the band appears to
    /// stall at one width and jump at the next.
    #[test]
    fn a_group_with_nowhere_to_reflow_is_skipped() {
        let g = vec![
            // Three items on one row: the three-row layout is the same width.
            Candidate {
                natural: 90.0,
                rewrapped: 90.0,
                collapsed: 30.0,
                priority: None,
            },
            Candidate {
                natural: 300.0,
                rewrapped: 180.0,
                collapsed: 40.0,
                priority: None,
            },
        ];
        assert_eq!(
            fit(&g, 280.0, 8.0),
            vec![State::Natural, State::Rewrapped],
            "a group whose re-wrapped width equals its natural one must be left \
             at Natural, so every step of the ladder means something"
        );
    }

    /// **Running out of ladder is not a failure.** Everything compactable is
    /// compacted and the remainder is left to the overflow affordance.
    #[test]
    fn exhausting_the_ladder_leaves_the_rest_to_the_overflow_affordance() {
        assert_eq!(
            fit(&trio(), 1.0, 8.0),
            vec![State::Collapsed, State::Collapsed, State::Collapsed]
        );
    }

    /// **Equal priorities collapse left to right.**
    #[test]
    fn ties_break_on_manifest_order() {
        let g = vec![
            Candidate {
                natural: 100.0,
                rewrapped: 100.0,
                collapsed: 10.0,
                priority: Some(5),
            },
            Candidate {
                natural: 100.0,
                rewrapped: 100.0,
                collapsed: 10.0,
                priority: Some(5),
            },
        ];
        // Neither gains from re-wrapping, so the collapse rung runs. One
        // collapsed: 10 + 100 + 8 = 118.
        assert_eq!(fit(&g, 120.0, 8.0), vec![State::Collapsed, State::Natural]);
    }

    /// **Widening never compacts further, on any rung.** The restated
    /// monotonicity invariant, swept rather than spot-checked.
    ///
    /// For every pair of adjacent widths, no group's state may move DOWN the
    /// ladder as the band grows. A hysteresis bug — the classic consequence of
    /// planning from the previous frame's answer instead of from scratch —
    /// shows up here as a width at which some group compacts again.
    ///
    /// The sweep is one point at a time, deliberately. A claim about what
    /// happens across a range that is checked at two widths is a claim about
    /// two widths: endpoints agreeing is not evidence about what happens
    /// between them.
    #[test]
    fn widening_the_band_never_compacts_a_group_further() {
        let g = trio();
        let mut prev: Option<Vec<State>> = None;
        for w in 0..600 {
            let states = fit(&g, w as f32, 8.0);
            if let Some(prev) = &prev {
                for (i, (&now, &before)) in states.iter().zip(prev).enumerate() {
                    assert!(
                        now <= before,
                        "group {i} moved from {before:?} to {now:?} when the band GREW from {} to {w}, which means it flickers between the two",
                        w - 1
                    );
                }
            }
            prev = Some(states);
        }
    }

    /// **The states and the widths agree**, because they are produced together.
    #[test]
    fn widths_after_matches_the_states() {
        let g = trio();
        let states = vec![State::Natural, State::Rewrapped, State::Collapsed];
        assert_eq!(widths_after(&g, &states), vec![100.0, 140.0, 40.0]);
    }

    /// A degenerate width does not produce a degenerate band: it compacts
    /// everything that may compact, rather than nothing.
    #[test]
    fn a_nonsense_width_is_treated_as_zero_not_as_infinity() {
        assert_eq!(
            fit(&trio(), f32::NAN, 8.0),
            vec![State::Collapsed, State::Collapsed, State::Collapsed]
        );
    }

    /// No groups, no work, no panic.
    #[test]
    fn an_empty_band_compacts_nothing() {
        assert!(fit(&[], 100.0, 8.0).is_empty());
    }
}
