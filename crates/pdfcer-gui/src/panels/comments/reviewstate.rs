//! # `panels::comments::reviewstate` — a comment's **review status**
//!
//! `/State` and `/StateModel` (§12.5.6.3, Table 171; 2.0's Table 174), read
//! into a per-reviewer history, shown on the row, filtered beside the existing
//! sort, and recorded through
//! [`pdfcer_core::edit::EditSession::add_review_state`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/comments/reviewstate.md`.

use std::collections::{BTreeMap, BTreeSet};

use pdfcer_core::annot::page_annotations;
use pdfcer_core::edit::ReviewState;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Page;

use crate::text::reviewstate as t;

use super::model::CommentRow;

/// The region the per-row **Record status** control publishes.
pub const REGION_RECORD: &str = "comments.record_status"; // ui-text-exempt: trace region name, never displayed

/// The region the status chooser in the filter strip publishes.
pub const REGION_STATUS_FILTER: &str = "comments.status_filter"; // ui-text-exempt: trace region name, never displayed

/// The seven states pdfcer authors, in the order the chooser offers them.
pub const OFFERED: [ReviewState; 7] = [
    ReviewState::Accepted,
    ReviewState::Rejected,
    ReviewState::Cancelled,
    ReviewState::Completed,
    ReviewState::None,
    ReviewState::Marked,
    ReviewState::Unmarked,
];

/// How far a `/IRT` chain is walked before it is abandoned.
const MAX_CHAIN: usize = 64;

/// **What one `/State` + `/StateModel` pair says**, classified but never
/// normalised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateReading {
    /// Table 171's value **in Table 171's model for that value** — the closed
    /// set [`ReviewState`] holds, which is exactly what pdfcer authors.
    ///
    /// The model has to agree. `/State (Accepted) /StateModel (Marked)` is
    /// **not** this variant: `Accepted` belongs to `Review`
    /// ([`ReviewState::model`] derives it, and `add_review_state` writes the
    /// derived one so *"the one non-conforming combination cannot be
    /// expressed"*), so a file pairing it with `Marked` is saying something
    /// pdfcer would not write. It reads as [`Self::Unmodelled`] — the value is
    /// unrecognised **in the model the file put it in**, which is the honest
    /// statement and the one that keeps the row's offer correct.
    Modelled(ReviewState),
    /// `/StateModel` is `Review` or `Marked` — a vocabulary pdfcer **authors**
    /// — and `/State` is not one of that model's values.
    ///
    /// Named, shown verbatim, and the row still offers to record: pdfcer can
    /// add its own status **in the same model**, which is the entire content of
    /// the `Unmodelled`-versus-`Foreign` distinction here.
    Unmodelled {
        /// `/State`, exactly as the file decoded.
        state: String,
        /// `/StateModel`, exactly as the file decoded.
        model: String,
    },
    /// `/StateModel` is neither of §12.5.6.3's two — a vocabulary pdfcer will
    /// not author.
    ///
    /// Legal: neither edition says *shall be one of*. Both values are shown and
    /// **nothing is offered in that model**; the row says what recording will
    /// do instead, which is to start a `Review` history beside it.
    Foreign {
        /// `/State`, exactly as the file decoded.
        state: String,
        /// `/StateModel`, exactly as the file decoded.
        model: String,
    },
    /// `/State` present, `/StateModel` absent.
    ///
    /// Table 171's **one non-conforming combination** —
    /// `Annotation::state_model`: *"`/StateModel` is 'Required if `State` is
    /// present, otherwise optional'. The converse does not hold … the only
    /// non-conforming combination is `/State` present with this absent."*
    ///
    /// Surfaced rather than repaired, the same way
    /// [`super::model::CommentRow::subtype`] surfaces a missing `/Subtype`.
    /// Guessing the model would decide whether `Marked` means *ticked* or is
    /// an unrecognised `Review` value, and the file gives no basis for either.
    ModelMissing(String),
}

impl StateReading {
    /// **Classify one pair.** `state` is `/State`; `model` is `/StateModel`.
    #[must_use]
    pub fn of(state: &str, model: Option<&str>) -> Self {
        let Some(model) = model else {
            return Self::ModelMissing(state.to_owned());
        };
        // Filtered by the model FIRST, so a `Review` value paired with the
        // `Marked` model reads as unrecognised-in-that-model rather than as
        // modelled — see `Self::Modelled`'s note.
        let known = OFFERED
            .iter()
            .find(|s| s.model() == model && s.as_str() == state);
        match (known, model) {
            (Some(s), _) => Self::Modelled(*s),
            // `model()` is the authority on which models pdfcer authors, so
            // the set is derived from the enum rather than written out again —
            // a third model added to `ReviewState` would land here without an
            // edit.
            (None, m) if OFFERED.iter().any(|s| s.model() == m) => Self::Unmodelled {
                state: state.to_owned(),
                model: m.to_owned(),
            },
            (None, m) => Self::Foreign {
                state: state.to_owned(),
                model: m.to_owned(),
            },
        }
    }

    /// **What the document literally says** in `/State`.
    #[must_use]
    pub fn raw(&self) -> &str {
        match self {
            Self::Modelled(s) => s.as_str(),
            Self::Unmodelled { state, .. }
            | Self::Foreign { state, .. }
            | Self::ModelMissing(state) => state,
        }
    }

    /// **The `/StateModel` pdfcer would author into, given this reading** —
    /// `None` for [`Self::Foreign`] alone.
    #[must_use]
    pub fn authorable_model(&self) -> Option<&str> {
        match self {
            Self::Modelled(s) => Some(s.model()),
            Self::Unmodelled { model, .. } => Some(model),
            Self::Foreign { .. } => None,
            Self::ModelMissing(_) => Some(ReviewState::Accepted.model()),
        }
    }

    /// The sentence the operator reads for this reading.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Modelled(s) => t::state_name(*s).to_owned(),
            Self::Unmodelled { state, model } => t::state_unmodelled(state, model),
            Self::Foreign { state, model } => t::state_foreign(state, model),
            Self::ModelMissing(state) => t::state_without_model(state),
        }
    }
}

/// **One reviewer's current status on one comment**, plus how much history
/// stands behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// `/T` on the state annotation — the reviewer, when the file names one.
    ///
    /// `None` is not *anonymous*: `add_review_state` chains on `/T` equality
    /// (`edit.rs`, `deepest_state_for_author`), so an unnamed status can never
    /// be continued and always stands alone. [`crate::text::reviewstate`]'s
    /// [`crate::text::reviewstate::row_status_unsigned`] says exactly that.
    pub who: Option<String>,
    /// What that reviewer's most recent status says.
    pub reading: StateReading,
    /// **How many statuses this reviewer has recorded on this comment**, the
    /// tip included. `1` for a first status.
    ///
    /// The number [`pdfcer_core::edit::ReviewStateAdded::chain_depth`] reports
    /// after a write, computed here for what is already in the file. Printed
    /// whenever it exceeds one — see the module header's point 2.
    pub depth: usize,
}

/// **Every review status in the document**, indexed two ways.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Statuses {
    /// Reviewed annotation → one entry per reviewer, tip of that reviewer's
    /// chain. Sorted for a stable draw; see [`read`].
    by_target: BTreeMap<ObjId, Vec<Recorded>>,
    /// Annotation that **is** a status → what it says. The index behind the
    /// *this row is a status* caption; see the module header.
    is_status: BTreeMap<ObjId, StateReading>,
}

impl Statuses {
    /// What is recorded on one annotation, newest-per-reviewer.
    #[must_use]
    pub fn on(&self, id: Option<ObjId>) -> &[Recorded] {
        id.and_then(|id| self.by_target.get(&id))
            .map_or(&[], Vec::as_slice)
    }

    /// What this annotation says, when the annotation **is** a status.
    #[must_use]
    pub fn as_status(&self, id: Option<ObjId>) -> Option<&StateReading> {
        id.and_then(|id| self.is_status.get(&id))
    }

    /// **Every distinct `/State` value the chooser should offer**, in the
    /// file's own spelling, alphabetically.
    #[must_use]
    pub fn values(&self) -> Vec<String> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for recorded in self.by_target.values() {
            for r in recorded {
                seen.insert(r.reading.raw());
            }
        }
        seen.into_iter().map(str::to_owned).collect()
    }

    /// **Does one row survive a status filter?**
    #[must_use]
    pub fn keeps(&self, row: &CommentRow, choice: Option<&StatusChoice>) -> bool {
        match choice {
            None => true,
            Some(StatusChoice::Unrecorded) => self.on(row.id).is_empty(),
            Some(StatusChoice::Is(value)) => {
                self.on(row.id).iter().any(|r| r.reading.raw() == value)
            }
        }
    }
}

/// **What the reviewer has narrowed the status to.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusChoice {
    /// Only comments carrying **no status at all**.
    Unrecorded,
    /// Only comments where some reviewer's current status is this exact
    /// `/State` string.
    Is(String),
}

/// **Read every review status in the document.**
#[must_use]
pub fn read<G: ObjectGraph + ?Sized>(graph: &G, pages: &[Page]) -> Statuses {
    let states: Vec<StateAnnot> = pages
        .iter()
        .flat_map(|page| page_annotations(graph, page.id))
        .filter_map(|a| {
            // Both are required, and neither is a judgement: `/State` is what
            // makes this a status at all, and an object id is what lets a
            // `/IRT` — an indirect reference by construction — name it.
            let (id, state) = (a.id?, a.state.as_deref()?);
            Some(StateAnnot {
                id,
                who: a.title.as_deref().map(str::trim).map(str::to_owned),
                irt: a.in_reply_to,
                reading: StateReading::of(state, a.state_model.as_deref()),
            })
        })
        .collect();
    let read = assemble(&states);
    // THE ONLY ORACLE FOR THIS FEATURE THAT A SCREENSHOT CANNOT GIVE.
    //
    // A review status is **invisible on the page** by construction — R8b puts it
    // off-canvas, and the annotation carrying it has an empty `/Contents` — so a
    // driven run of `tools/ui-verify` has nothing to look at except this line
    // and the rectangles the two controls publish. Three numbers, because the
    // three failures they separate all look identical from outside:
    //
    // | field | the question | the failure it catches |
    // |---|---|---|
    // | `states` | how many `/State` annotations does the document carry | the panel read the wrong revision, or read nothing |
    // | `reviewed` | how many COMMENTS have a status | every chain collapsed to its own target, or none reached one |
    // | `values` | how many distinct statuses can be filtered to | the chooser is empty on a document that has statuses |
    //
    // `reviewed` is the one worth having. `states` and `values` would both be
    // right on a build whose `/IRT` walk was broken; `reviewed` is the number
    // that changes when a chain fails to reach the comment it describes, and
    // that is the failure the engine calls invisible.
    //
    // `comments-status`, not `comments-panel`: the panel's own summary line owns
    // that token, and `TraceLog::last(name)` matches the FIRST token, so two
    // lines under one name would shadow each other. Same rule
    // `tools/gates/check-trace-names.py` enforces against the edit funnel.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "comments-status states={} reviewed={} values={}",
            states.len(),
            read.by_target.len(),
            read.values().len()
        )
    });
    read
}

/// **One `/State`-carrying annotation**, reduced to the four facts the chain
/// walk and the row need.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StateAnnot {
    /// The state annotation's own object id.
    id: ObjId,
    /// `/T`, trimmed. `None` when absent — see [`same_author`].
    who: Option<String>,
    /// `/IRT` — what this status is a reply to.
    irt: Option<ObjId>,
    /// What `/State` + `/StateModel` say.
    reading: StateReading,
}

/// Build the two indexes from the document's state annotations.
///
/// Separate from [`read`] so the whole classification can be asserted without a
/// fixture document — [`super::model`]'s seam, at this module's scale.
fn assemble(states: &[StateAnnot]) -> Statuses {
    let mut out = Statuses::default();
    for s in states {
        out.is_status.insert(s.id, s.reading.clone());
        // A tip is a status nobody **by the same name** has replied to.
        // `same_author` guards the whole test: an unsigned status is never
        // anybody's parent, so it can never be superseded.
        if states
            .iter()
            .any(|t| t.irt == Some(s.id) && same_author(t, s))
        {
            continue;
        }
        let (target, depth) = walk_up(states, s);
        let Some(target) = target else {
            continue;
        };
        out.by_target.entry(target).or_default().push(Recorded {
            who: s.who.clone().filter(|w| !w.is_empty()),
            reading: s.reading.clone(),
            depth,
        });
    }

    // Sorted for a STABLE draw. The input is in page-then-`/Annots` order,
    // which is stable in itself — but two reviewers' tips arrive in whatever
    // order each happened to be placed, and a list of statuses that reshuffled
    // between frames reads as the panel flickering. The rule is
    // `super::filter::apply`'s, restated: an unsigned entry sorts LAST, because
    // a reader scanning for a name should not have to read past the rows that
    // carry none.
    for recorded in out.by_target.values_mut() {
        recorded.sort_by_key(|r| {
            let who = r.who.clone().unwrap_or_default();
            (
                who.is_empty(),
                who.to_lowercase(),
                r.reading.raw().to_owned(),
            )
        });
    }
    out
}

/// Whether two state annotations belong to the same reviewer's chain.
fn same_author(a: &StateAnnot, b: &StateAnnot) -> bool {
    a.who.is_some() && a.who == b.who
}

/// Walk a chain from its tip to the annotation it reviews.
fn walk_up(states: &[StateAnnot], tip: &StateAnnot) -> (Option<ObjId>, usize) {
    let mut current = tip;
    let mut depth = 1usize;
    loop {
        let Some(parent_id) = current.irt else {
            return (None, depth);
        };
        let parent = states
            .iter()
            .find(|p| p.id == parent_id && same_author(p, tip));
        match parent {
            Some(p) if depth < MAX_CHAIN => {
                current = p;
                depth += 1;
            }
            // Either the parent is not this reviewer's status — so it is the
            // annotation being reviewed — or the chain is longer than the
            // engine itself will walk, in which case reporting the deepest
            // point reached is the honest answer and matches what
            // `add_review_state` would attach to.
            _ => return (Some(parent_id), depth),
        }
    }
}

/// **Narrow a list of rows by status**, after [`super::filter::apply`] has
/// narrowed and ordered it.
#[must_use]
pub fn narrow(
    rows: Vec<CommentRow>,
    statuses: &Statuses,
    choice: Option<&StatusChoice>,
) -> Vec<CommentRow> {
    rows.into_iter()
        .filter(|r| statuses.keeps(r, choice))
        .collect()
}

/// **The status chooser**, drawn beneath the existing filter strip.
pub fn status_strip(ui: &mut egui::Ui, statuses: &Statuses, chosen: &mut Option<StatusChoice>) {
    let values = statuses.values();
    if values.is_empty() {
        // …and the filter is LIFTED, not merely hidden. A chooser that
        // vanished while still narrowing would leave rows hidden with no
        // control to restore them — the trap version of R9, and the exact
        // failure `super::filter`'s *Show all* exists to prevent.
        *chosen = None;
        return;
    }
    let selected = match chosen.as_ref() {
        None => t::filter_status_label().to_owned(),
        Some(StatusChoice::Unrecorded) => t::filter_status_unrecorded().to_owned(),
        Some(StatusChoice::Is(v)) => v.clone(),
    };
    ui.horizontal_wrapped(|ui| {
        let combo = egui::ComboBox::from_id_salt("comments-filter-status") // ui-text-exempt: internal widget id, never displayed
            .selected_text(selected)
            .show_ui(ui, |ui| {
                ui.selectable_value(chosen, None, t::filter_status_any());
                ui.selectable_value(
                    chosen,
                    Some(StatusChoice::Unrecorded),
                    t::filter_status_unrecorded(),
                );
                for value in &values {
                    ui.selectable_value(chosen, Some(StatusChoice::Is(value.clone())), value);
                }
            })
            .response;
        crate::diag::ui_rect_visible(REGION_STATUS_FILTER, combo.rect, ui.clip_rect());
        combo.on_hover_text(t::filter_status_label());
    });
}

/// **One row's status lines, and the control that records another.**
pub fn row_status(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    statuses: &Statuses,
    ctx: RowStatusCtx<'_>,
) {
    // The row that IS a status, named before anything else on it: an operator
    // looking at a blank comment needs the explanation before they conclude the
    // panel is broken.
    if let Some(reading) = statuses.as_status(comment.id) {
        ui.label(
            egui::RichText::new(t::row_is_a_status(&reading.label()))
                .small()
                .weak(),
        );
    }

    let recorded = statuses.on(comment.id);
    for entry in recorded {
        let status = entry.reading.label();
        let line = match entry.who.as_deref().filter(|w| !w.is_empty()) {
            Some(who) => t::row_status_by(who, &status),
            None => t::row_status_unsigned(&status),
        };
        ui.label(egui::RichText::new(line).small().weak());
        // Only when there IS history. Printing "1 status recorded" beside
        // every row would be a caption with the same information content as
        // nothing at all — this panel's rule for its other four disclosures.
        if entry.depth > 1 {
            ui.label(
                egui::RichText::new(t::row_status_history(entry.depth))
                    .small()
                    .weak(),
            );
        }
    }

    // Said only where the operator ASKED the question. Under the *No status
    // recorded* filter the emptiness is the answer and the caption confirms it;
    // on an unfiltered list it would repeat "nothing has happened here" on
    // forty rows.
    if recorded.is_empty() && ctx.unreviewed_asked {
        ui.label(egui::RichText::new(t::row_status_none()).small().weak());
    }

    // The `Foreign` note, and this is the one place the fourth reading
    // changes what the operator is told. Drawn when EVERY status on the row is
    // in a model pdfcer will not author — if any reading has an authorable
    // model, recording continues a vocabulary that is already here and there is
    // nothing to explain.
    if !recorded.is_empty()
        && recorded
            .iter()
            .all(|r| r.reading.authorable_model().is_none())
    {
        ui.label(
            egui::RichText::new(t::record_other_model_note())
                .small()
                .weak(),
        );
    }

    record_control(ui, comment, statuses, ctx);
}

/// What [`row_status`] needs from the frame around it.
pub struct RowStatusCtx<'a> {
    /// Whether the current mode authors markup —
    /// `crate::canvas::tool::capabilities(..).author_markup`, asked once per
    /// frame by [`super::body`] and passed down.
    pub authoring: bool,
    /// Whether the operator has filtered to **No status recorded**, which is
    /// the only condition under which an unreviewed row says so.
    pub unreviewed_asked: bool,
    /// Whether [`REGION_RECORD`] has already been published this frame. One
    /// name, one row — see that constant.
    pub published: &'a mut bool,
    /// Tallied into `super::note::CommentsUi::writing_controls_drawn`.
    pub controls_drawn: &'a mut u32,
    /// The status one row asked to record, if any. One `Option` rather than a
    /// `Vec` for [`super::RowSink`]'s reason: two rows cannot be pressed in one
    /// frame.
    pub verb: &'a mut Option<(ObjId, ReviewState)>,
}

/// The **Record status** chooser for one row. See [`row_status`] for the five
/// conditions under which it is not drawn.
fn record_control(
    ui: &mut egui::Ui,
    comment: &CommentRow,
    statuses: &Statuses,
    ctx: RowStatusCtx<'_>,
) {
    if !ctx.authoring || comment.is_ce_dimension {
        return;
    }
    let Some(id) = comment.id else {
        return;
    };
    if statuses.as_status(Some(id)).is_some() {
        return;
    }
    let mut chosen: Option<ReviewState> = None;
    let combo = egui::ComboBox::from_id_salt("comments-record-status") // ui-text-exempt: internal widget id, never displayed
        .selected_text(t::record_placeholder())
        .show_ui(ui, |ui| {
            // Grouped by model, with each model's own name above it. The two
            // are different vocabularies rather than seven points on one scale
            // — `Annotation::state`: "a caller that wants the effective state
            // must read both fields together" — and a flat list would invite
            // the operator to read `Marked` as a sixth `Review` value.
            //
            // The headings are `ReviewState::model()`'s own words, which are
            // the strings the file carries, so they are not copy this shell
            // invented and there is nothing for the catalog to hold.
            let mut model: Option<&str> = None;
            for state in OFFERED {
                if model != Some(state.model()) {
                    model = Some(state.model());
                    ui.label(egui::RichText::new(state.model()).small().weak());
                }
                if ui.selectable_label(false, t::state_name(state)).clicked() {
                    chosen = Some(state);
                }
            }
        })
        .response;
    *ctx.controls_drawn += 1;
    // `ui_rect_visible`, not `ui_rect`: these rows live in a `ScrollArea` and a
    // control scrolled out of view still reports a rect. See
    // `super::REGION_EDIT`.
    if !*ctx.published {
        crate::diag::ui_rect_visible(REGION_RECORD, combo.rect, ui.clip_rect());
        *ctx.published = true;
    }
    combo.on_hover_text(t::record_tooltip());
    if let Some(state) = chosen {
        *ctx.verb = Some((id, state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One state annotation, named by object number, exactly as [`read`] would
    /// have reduced it.
    fn annot(
        num: u32,
        title: Option<&str>,
        irt: Option<u32>,
        state: &str,
        model: Option<&str>,
    ) -> StateAnnot {
        StateAnnot {
            id: ObjId::new(num, 0),
            who: title.map(str::trim).map(str::to_owned),
            irt: irt.map(|n| ObjId::new(n, 0)),
            reading: StateReading::of(state, model),
        }
    }

    /// The classification under test, run over hand-built statuses.
    fn statuses(annots: &[StateAnnot]) -> Statuses {
        assemble(annots)
    }

    fn row(num: u32) -> CommentRow {
        CommentRow {
            page_index: 0,
            id: Some(ObjId::new(num, 0)),
            subtype: "Square".to_owned(),
            is_ce_dimension: false,
            note: super::super::model::Note::Absent,
            author: None,
            modified: None,
            suppressed: false,
            appearance_unresolved: false,
            relation: None,
            in_reply_to: None,
        }
    }

    // ---------------------------------------------------------------------
    // The vocabulary: what is recognised, and what is shown rather than
    // normalised.
    // ---------------------------------------------------------------------

    /// **THE CONTROL FOR THE NEGATIVE BELOW.**
    #[test]
    fn every_authored_state_is_recognised_as_modelled() {
        for state in OFFERED {
            assert_eq!(
                StateReading::of(state.as_str(), Some(state.model())),
                StateReading::Modelled(state),
                "{} in the {} model",
                state.as_str(),
                state.model()
            );
        }
        assert_eq!(
            OFFERED.len(),
            7,
            "Table 171 gives seven values in two models"
        );
    }

    /// **An unrecognised value in a model pdfcer authors is SHOWN, not
    /// normalised** — and pdfcer still offers to continue that model.
    #[test]
    fn an_unknown_value_in_a_known_model_is_shown_verbatim() {
        let reading = StateReading::of("Deferred", Some("Review"));
        assert_eq!(
            reading,
            StateReading::Unmodelled {
                state: "Deferred".to_owned(),
                model: "Review".to_owned(),
            }
        );
        assert_eq!(reading.raw(), "Deferred");
        assert!(reading.label().contains("Deferred"), "{}", reading.label());
        assert_eq!(
            reading.authorable_model(),
            Some("Review"),
            "pdfcer authors the Review model and must still offer to record in it"
        );
    }

    /// **An unknown MODEL is foreign, and pdfcer offers nothing in it.**
    ///
    /// The other half of the distinction. `authorable_model` is `None`, which
    /// is the single observable the row's note is keyed off.
    #[test]
    fn an_unknown_model_is_foreign_and_offers_nothing() {
        let reading = StateReading::of("Sealed", Some("Approval"));
        assert_eq!(
            reading,
            StateReading::Foreign {
                state: "Sealed".to_owned(),
                model: "Approval".to_owned(),
            }
        );
        assert_eq!(reading.authorable_model(), None);
        let label = reading.label();
        assert!(
            label.contains("Sealed") && label.contains("Approval"),
            "{label}"
        );
    }

    /// **A Review value in the Marked model is not modelled**, because
    /// `ReviewState::model` derives the pairing and pdfcer cannot express this
    /// one.
    #[test]
    fn a_state_paired_with_the_wrong_model_is_unmodelled() {
        assert_eq!(
            StateReading::of("Accepted", Some("Marked")),
            StateReading::Unmodelled {
                state: "Accepted".to_owned(),
                model: "Marked".to_owned(),
            }
        );
    }

    /// Matching is **exact**: a case variant is a value pdfcer did not decode.
    #[test]
    fn matching_is_case_sensitive() {
        assert!(matches!(
            StateReading::of("accepted", Some("Review")),
            StateReading::Unmodelled { .. }
        ));
    }

    /// `/State` with no `/StateModel` — Table 171's one non-conforming
    /// combination — is surfaced rather than guessed at.
    #[test]
    fn a_state_without_a_model_is_named_as_such() {
        let reading = StateReading::of("Accepted", None);
        assert_eq!(reading, StateReading::ModelMissing("Accepted".to_owned()));
        assert_eq!(reading.raw(), "Accepted");
    }

    /// `None` is a **value**, not the absence of one, and the two are
    /// different entries the operator can act on.
    #[test]
    fn a_state_of_none_is_recorded_and_not_unrecorded() {
        let doc = [annot(2, Some("Ken"), Some(1), "None", Some("Review"))];
        let s = statuses(&doc);
        assert_eq!(s.on(Some(ObjId::new(1, 0))).len(), 1);
        assert!(
            !s.keeps(&row(1), Some(&StatusChoice::Unrecorded)),
            "a withdrawn status is not an unreviewed comment"
        );
        assert!(s.keeps(&row(1), Some(&StatusChoice::Is("None".to_owned()))));
    }

    // ---------------------------------------------------------------------
    // The chain: appended, not set.
    // ---------------------------------------------------------------------

    /// **A SECOND STATUS DOES NOT REPLACE THE FIRST — it chains onto it,
    /// and the panel reports the depth.**
    #[test]
    fn a_reviewers_second_status_chains_and_the_depth_is_reported() {
        let doc = [
            annot(2, Some("Ken"), Some(1), "Rejected", Some("Review")),
            annot(3, Some("Ken"), Some(2), "Accepted", Some("Review")),
        ];
        let s = statuses(&doc);
        let on = s.on(Some(ObjId::new(1, 0)));
        assert_eq!(on.len(), 1, "one entry per reviewer, not one per status");
        assert_eq!(on[0].reading, StateReading::Modelled(ReviewState::Accepted));
        assert_eq!(on[0].depth, 2);
    }

    /// **Two reviewers are two entries, not one winner.**
    #[test]
    fn two_reviewers_are_both_reported() {
        let doc = [
            annot(2, Some("Ken"), Some(1), "Accepted", Some("Review")),
            annot(3, Some("Jo"), Some(1), "Rejected", Some("Review")),
        ];
        let s = statuses(&doc);
        let on = s.on(Some(ObjId::new(1, 0)));
        assert_eq!(on.len(), 2);
        assert_eq!(on[0].who.as_deref(), Some("Jo"), "sorted by name");
        assert_eq!(on[1].who.as_deref(), Some("Ken"));
    }

    /// **An unsigned status never chains**, mirroring the engine's own
    /// `title.as_deref() == Some(author)`.
    #[test]
    fn an_unsigned_status_never_chains() {
        let doc = [
            annot(2, None, Some(1), "Accepted", Some("Review")),
            annot(3, None, Some(2), "Rejected", Some("Review")),
        ];
        let s = statuses(&doc);
        let on = s.on(Some(ObjId::new(1, 0)));
        assert_eq!(on.len(), 1, "only the one whose /IRT points at the comment");
        assert_eq!(on[0].depth, 1);
        assert!(on[0].who.is_none());
        // The second one refers to a state annotation, not to the comment, so
        // it is reported against THAT.
        assert_eq!(s.on(Some(ObjId::new(2, 0))).len(), 1);
    }

    /// A `/IRT` **cycle** terminates rather than hanging, and the walk is
    /// what has to survive it.
    #[test]
    fn an_irt_cycle_terminates() {
        let doc = [
            annot(3, Some("Ken"), Some(4), "Accepted", Some("Review")),
            annot(4, Some("Ken"), Some(5), "Rejected", Some("Review")),
            annot(5, Some("Ken"), Some(4), "Completed", Some("Review")),
        ];
        let s = statuses(&doc);
        // 3 is the only tip: 4 is replied to by both 3 and 5, and 5 by 4.
        // The walk from 3 runs 4 → 5 → 4 → … and stops at the bound, reporting
        // the deepest point it reached — the honest answer, and the same place
        // `add_review_state` would attach to.
        let reported: usize = [4u32, 5]
            .iter()
            .map(|n| s.on(Some(ObjId::new(*n, 0))).len())
            .sum();
        assert_eq!(reported, 1, "the bounded walk reported exactly one target");
        let depth = [4u32, 5]
            .iter()
            .filter_map(|n| s.on(Some(ObjId::new(*n, 0))).first())
            .map(|r| r.depth)
            .next()
            .expect("the walk reported a target");
        assert_eq!(
            depth, MAX_CHAIN,
            "the walk stopped at the bound, not before"
        );
    }

    /// A status with **no `/IRT`** describes nothing, and is still named on
    /// its own row rather than dropped.
    #[test]
    fn a_status_with_no_irt_reaches_no_target_but_is_still_read() {
        let doc = [annot(2, Some("Ken"), None, "Accepted", Some("Review"))];
        let s = statuses(&doc);
        assert!(
            s.by_target.is_empty(),
            "a status with no /IRT reached {:?}",
            s.by_target.keys().collect::<Vec<_>>()
        );
        assert!(
            s.as_status(Some(ObjId::new(2, 0))).is_some(),
            "the row must still be able to say what it is"
        );
    }

    /// **A status annotation knows it is one**, which is what stops the blank
    /// row it creates reading as a defect.
    #[test]
    fn a_status_annotation_is_identifiable_as_one() {
        let doc = [annot(2, Some("Ken"), Some(1), "Accepted", Some("Review"))];
        let s = statuses(&doc);
        assert_eq!(
            s.as_status(Some(ObjId::new(2, 0))),
            Some(&StateReading::Modelled(ReviewState::Accepted))
        );
        assert_eq!(s.as_status(Some(ObjId::new(1, 0))), None);
    }

    // ---------------------------------------------------------------------
    // The filter.
    // ---------------------------------------------------------------------

    /// **No status filter hides nothing** — the default this panel has always
    /// had.
    #[test]
    fn no_status_filter_keeps_every_row() {
        let s = statuses(&[annot(2, Some("Ken"), Some(1), "Accepted", Some("Review"))]);
        let rows = vec![row(1), row(5)];
        assert_eq!(narrow(rows, &s, None).len(), 2);
    }

    /// Filtering to a value keeps the comments carrying it and only those.
    #[test]
    fn filtering_to_a_status_keeps_only_those_comments() {
        let s = statuses(&[
            annot(3, Some("Ken"), Some(1), "Accepted", Some("Review")),
            annot(4, Some("Ken"), Some(2), "Rejected", Some("Review")),
        ]);
        let rows = vec![row(1), row(2), row(5)];
        let kept = narrow(rows, &s, Some(&StatusChoice::Is("Accepted".to_owned())));
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].id, Some(ObjId::new(1, 0)));
    }

    /// **The work-list filter**: the comments nobody has reviewed.
    #[test]
    fn filtering_to_unrecorded_keeps_the_untouched_comments() {
        let s = statuses(&[annot(3, Some("Ken"), Some(1), "Accepted", Some("Review"))]);
        let rows = vec![row(1), row(2), row(5)];
        let kept = narrow(rows, &s, Some(&StatusChoice::Unrecorded));
        assert_eq!(kept.len(), 2);
        assert!(kept.iter().all(|r| r.id != Some(ObjId::new(1, 0))));
    }

    /// **A value pdfcer never heard of is filterable**, in the file's own
    /// spelling — the operator-facing consequence of the engine reading
    /// `/State` verbatim.
    #[test]
    fn an_unmodelled_value_is_offered_by_the_chooser_and_filters() {
        let s = statuses(&[annot(2, Some("Ken"), Some(1), "Deferred", Some("Review"))]);
        assert_eq!(s.values(), vec!["Deferred"]);
        let kept = narrow(
            vec![row(1), row(9)],
            &s,
            Some(&StatusChoice::Is("Deferred".to_owned())),
        );
        assert_eq!(kept.len(), 1);
    }

    /// The chooser offers **tip** values only. A status somebody has since
    /// superseded would be a menu entry that filters to nothing.
    #[test]
    fn the_chooser_does_not_offer_a_superseded_value() {
        let s = statuses(&[
            annot(2, Some("Ken"), Some(1), "Rejected", Some("Review")),
            annot(3, Some("Ken"), Some(2), "Accepted", Some("Review")),
        ]);
        assert_eq!(s.values(), vec!["Accepted"]);
    }

    /// **Any reviewer, not every reviewer.** Two people disagreeing must both
    /// be findable, because a work list's job is to surface what may need
    /// attention.
    #[test]
    fn a_status_filter_matches_if_any_reviewer_holds_it() {
        let s = statuses(&[
            annot(2, Some("Ken"), Some(1), "Accepted", Some("Review")),
            annot(3, Some("Jo"), Some(1), "Rejected", Some("Review")),
        ]);
        assert!(s.keeps(&row(1), Some(&StatusChoice::Is("Rejected".to_owned()))));
        assert!(s.keeps(&row(1), Some(&StatusChoice::Is("Accepted".to_owned()))));
    }

    /// A row with **no object id** carries no status, because nothing can point
    /// at it — and it therefore counts as unreviewed rather than as an error.
    #[test]
    fn a_row_with_no_id_is_unreviewed() {
        let s = statuses(&[annot(2, Some("Ken"), Some(1), "Accepted", Some("Review"))]);
        let mut r = row(1);
        r.id = None;
        assert!(s.on(r.id).is_empty());
        assert!(s.keeps(&r, Some(&StatusChoice::Unrecorded)));
    }

    /// **An empty document has nothing to offer**, which is what lets
    /// [`status_strip`] withhold the chooser under R9.
    #[test]
    fn a_document_with_no_statuses_offers_no_values() {
        assert!(statuses(&[]).values().is_empty());
    }
}
