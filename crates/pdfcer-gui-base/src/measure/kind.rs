//! # `measure::kind` — which dimensioning tool is armed
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/measure/kind.md`.

/// **Which dimensioning tool is armed.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasureKind {
    /// Point A → point B → a third click for the standoff. The ordinary
    /// dimension, and the one an operator reaches for first.
    Linear,
    /// **Toggle objects into a best-fit circle**, then say when the set is
    /// complete. Authors a [`pdfcer_core::dimension::DimensionKind::Circular`]
    /// — a radius or a diameter, which is one display toggle on one fit rather
    /// than two tools (decision 011's value model: `diameter = 2 × radius`, the
    /// same stored geometry).
    ///
    /// # This variant was deleted once, and what brought it back
    ///
    /// Phase 7 shipped Linear and Two-line and deliberately left this one
    /// unarmed. The reason was recorded in three documents and it was a real
    /// one rather than an unfinished corner: **the gesture has no natural
    /// end.** Linear knows it is finished at three clicks and two-line at two,
    /// because both are picks of a fixed arity. A best-fit circle is finished
    /// when the operator says it is — an arc drawn as four separate polyline
    /// objects needs four picks, the same arc drawn as one needs one, and
    /// nothing in the geometry can tell pdfcer which the operator meant. The
    /// only surface available for saying *"done"* was an accept box floating
    /// over the canvas, and decision 024 retired exactly that at the operator's
    /// instruction.
    ///
    /// **So it has two endings.** A **double-click** on the canvas, which is
    /// what every polyline-ish tool in every drawing package uses to say "that
    /// was the last one" and is therefore what working the way other programs
    /// do requires; and
    /// a ribbon command, `measure.finish`, for an operator who does not know
    /// the double-click or whose last pick was awkward to double-click on.
    /// Neither is a floating box, and both reach `circular::commit` — one
    /// commit path, so the two endings cannot author different dimensions.
    ///
    /// # Its pick set is the tool's own and is never the selection
    ///
    /// [`super::pick::CircularPick`]'s docs are explicit (ui-spec §3.1) and this
    /// hosting keeps the line: the objects toggled into a fit live on the pick,
    /// not in [`crate::selectionstate::SelectionState`]. A half-assembled
    /// circle fit is not a selection — no verb on the Format tab means anything
    /// applied to it, Delete least of all — and borrowing the selection to hold
    /// it would arm a destructive control over a set the operator assembled for
    /// a completely different purpose.
    Circular,
    /// **Click around a shape; one number for the whole way round.** The tool
    /// `canvas::measure::perimeter` implements.
    ///
    /// Three endings rather than the usual one or two, and each is a different
    /// sentence the operator might mean: a **double-click** ends it as an open
    /// path (a pipe run, a cable route), a click on the **first vertex** closes
    /// it into a ring (a footprint, a fence line), and `measure.finish` ends it
    /// open for a pick that is awkward to double-click on.
    ///
    /// Its picks are POINTS, so it shares `Linear`'s snap machinery
    /// untouched - which matters, because tracing a building outline means
    /// aiming at the corners of paths that are already on the page.
    Perimeter,
    /// **Click along something; one number for how far it runs.** The perimeter
    /// tool's gesture without the requirement to close the profile.
    ///
    /// # Why this is a second KIND and not a checkbox on Perimeter
    ///
    /// The machinery is identical — the same `canvas::measure::perimeter::PerimeterPick`, the
    /// same snapped point picks, the same preview — and `closed` was already a
    /// flag on the authored dimension. So a reasonable reading is that this
    /// should be a toggle in Tool Options rather than a second control.
    ///
    /// It is a second control because **"Perimeter" says closed**, and an
    /// operator measuring a pipe run, a cable route or a kerb line would never
    /// reach for it. That is not a labelling problem a tooltip fixes: the
    /// ribbon is a list of activities (P2), and *"how long is this run"* and
    /// *"how far around is this shape"* are two activities that happen to share
    /// an implementation. Hiding one inside the other's options makes it
    /// findable only by somebody who already knows it is there.
    ///
    /// What it costs is one enum variant and one ribbon item. What it buys is
    /// that both readings of the gesture are on the tab the operator is already
    /// looking at.
    ///
    /// # The only behavioural difference
    ///
    /// It never closes. Clicking the first vertex again adds a vertex there,
    /// like any other click, because a path that returns to its start is a
    /// perfectly ordinary path — a loop of cable is still cable. Double-click
    /// and `measure.finish` are its endings.
    PathLength,
    /// Pick two lines on the page; the engine authors the dimension between
    /// them — [`pdfcer_core::dimension::TwoLinePlacement`] decides where it
    /// lands, so this side chooses the pair and nothing else.
    TwoLine,
    /// **Pick two points on the drawing to say what its scale is.**
    ///
    /// The calibration gesture: pick two lines or points and say what that
    /// distance represents on the real thing.
    ///
    /// # It authors no dimension, which is why it is a kind and not a verb
    ///
    /// Every other variant ends in `Action::CommitDimension`. This one ends in
    /// a **dialog**: two picks measure a reference length in PDF points, and
    /// the operator then says what that length *is* on the real thing. The
    /// scale falls out of the two, and `EditSession::set_group_scale` records
    /// it against the group.
    ///
    /// It is nevertheless a `MeasureKind` rather than a separate tool, because
    /// everything about the *gesture* is a measure pick: it snaps to content,
    /// it honours the H/V/aligned constraint, it Tab-cycles candidates, and it
    /// clears on page navigation. [`super::scale::ScalePick`]
    /// reuses `LinearPick` **verbatim** for exactly that reason — the reference
    /// line is a linear pick that happens not to be authored.
    ///
    /// # Deliberately absent from [`Self::ALL`]
    ///
    /// `ALL` is the list of kinds the **Measure tab arms with a command**, and
    /// this one is armed from inside the Set-scale dialog instead. See `ALL`'s
    /// own docs for why that distinction is worth the exception, and
    /// `tests::every_variant_is_either_offered_or_deliberately_excluded` for
    /// what stops the exception becoming a hole.
    Scale,
}

impl MeasureKind {
    /// Every variant the **Measure tab offers as a ribbon control**, in the
    /// order it offers them.
    pub const ALL: &'static [Self] = &[
        Self::Linear,
        Self::Circular,
        Self::Perimeter,
        Self::PathLength,
        Self::TwoLine,
    ];

    /// The kinds that are deliberately **not** on the ribbon, with the surface
    /// that arms each one instead.
    ///
    /// Read only by the exhaustiveness test. Its value is the second column:
    /// "excluded" with no destination is indistinguishable from "forgotten".
    #[cfg(test)]
    const ARMED_ELSEWHERE: &'static [(Self, &'static str)] = &[(
        Self::Scale,
        // ui-text-exempt: a test-only note naming the surface that arms this
        // kind. Never displayed — it exists so an excluded variant carries its
        // destination, and the test asserts it is non-empty.
        "the Set-scale dialog's calibrate button",
    )];
}

#[cfg(test)]
mod tests {
    use super::MeasureKind;

    /// **Every variant is either on the ribbon or deliberately excluded.**
    #[test]
    fn every_variant_is_either_offered_or_deliberately_excluded() {
        // No wildcard. This is the assertion; the body is bookkeeping.
        fn classify(k: MeasureKind) -> &'static str {
            match k {
                MeasureKind::Linear
                | MeasureKind::Circular
                | MeasureKind::Perimeter
                | MeasureKind::PathLength
                | MeasureKind::TwoLine => "ribbon",
                MeasureKind::Scale => "elsewhere",
            }
        }

        for k in MeasureKind::ALL {
            assert_eq!(
                classify(*k),
                "ribbon",
                "{k:?} is in ALL, so it must be a ribbon kind"
            );
            assert!(
                !MeasureKind::ARMED_ELSEWHERE.iter().any(|(e, _)| e == k),
                "{k:?} is in BOTH lists — it cannot be armed from the ribbon and not"
            );
        }
        for (k, where_armed) in MeasureKind::ARMED_ELSEWHERE {
            assert_eq!(
                classify(*k),
                "elsewhere",
                "{k:?} is excluded from the ribbon and the classifier disagrees"
            );
            assert!(
                !where_armed.is_empty(),
                "{k:?} is excluded with no surface named — 'excluded' with no destination is indistinguishable from 'forgotten'"
            );
        }
        assert_eq!(
            MeasureKind::ALL.len() + MeasureKind::ARMED_ELSEWHERE.len(),
            6,
            "a variant was added to the enum and to neither list, or counted twice"
        );
    }
}
