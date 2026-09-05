//! `text::panels::layers` — **every sentence pdfcer says about which layer a
//! selection is on**, in one module.
//!
//! # Why one module, when the sentences appear on two surfaces
//!
//! The answer is shown in two places and they are deliberately not the same
//! width:
//!
//! | surface | form | why it exists |
//! |---|---|---|
//! | the **Layers panel**, under the count | the long form, a whole sentence | the panel is where the operator went to ask |
//! | the **status bar**, appended to the selection line | a short clause | ★★★ **the canvas is the primary surface, never a panel** — clicking the object must reach the answer with no panel open |
//!
//! Two surfaces saying one thing is `DEFECTS.md` D5's shape exactly: *the same
//! concept implemented in two places, and the copies drift.* The drift here
//! would be invisible and expensive — a bar reading *"not on a layer"* beside
//! a panel reading *"pdfcer could not tell"* is a contradiction the operator
//! can only resolve by deciding the program is broken.
//!
//! ⇒ So both forms are generated **here**, from the same
//! [`Membership`](crate::panels::layers::highlight::Membership), by two
//! functions whose matches are exhaustive. A new state cannot be added to that
//! enum without both of these failing to compile, which is the mechanism this
//! project keeps re-learning it needs: **a hand-written list inside a
//! completeness sweep is not a sweep.** `check-ui-strings.sh` would have said
//! nothing about a missing arm; the compiler says everything.
//!
//! # ★★ The register: a measurement, never a hedge
//!
//! *"This may be part of a larger object"* is a disclaimer, and a disclaimer
//! printed on every selection teaches the operator to skip the line. *"This
//! object holds 1,194 parts"* is a number he can act on, and it is silent on
//! the object that holds one. Every sentence below is written to that rule —
//! see [`layer_selection_granularity`], which is the one that carries his own
//! finding back to him.
//!
//! # ★★ Rule 4 lives in what these sentences are FOR
//!
//! None of this is drawn on the drawing. An inference the operator cannot see
//! — an unresolvable `/OC` section, a group the document never listed, an
//! object nested deeper than the leaf list can see through — still owes a
//! report, and this is where that report is worded. **Render normally; report
//! separately. Both.**

use crate::panels::layers::highlight::{Membership, Unresolved};

/// Where the answer's optional-content group sits **relative to the list the
/// panel is actually drawing**.
///
/// # ★★★ Why this is not simply `Option<&str>`
///
/// Because a highlight that lands on a row nobody can see is
/// indistinguishable from no highlight at all, and this panel has **two**
/// independent ways for that to happen:
///
/// | | what the operator sees | what they conclude |
/// |---|---|---|
/// | the row is on screen and plated | the answer | correct |
/// | the row exists but the **search** has filtered it out | nothing | *"selecting an object does not highlight the layer"* |
/// | there is **no row** — an OCMD (§8.11.2.2), or an OCG the default configuration omits | nothing | the same, and wrongly |
///
/// The second and third were both silent before this type existed. They are
/// different facts with different remedies — clear the search; or accept that
/// the document's own configuration does not list this group — so they get
/// different sentences rather than one apology covering both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowOfAnswer<'a> {
    /// The answer is not a group, so there is no row to look for.
    NotAGroup,
    /// A row for it is in the list on screen. **The plate is the statement**
    /// and no sentence is owed — saying it twice would make the panel narrate
    /// its own highlight.
    OnScreen,
    /// The document lists this group and the operator's search query has
    /// narrowed it out of view. Carries the name, because the whole point of
    /// the sentence is to name what the search is hiding.
    HiddenBySearch(&'a str),
    /// **No row exists at all.** Either the `/OC` names an OCMD — a visibility
    /// *expression* over several groups, which is not one row to emphasise —
    /// or it names an OCG the document's default configuration never
    /// registered. Both are real files; neither is a pdfcer defect; both look
    /// exactly like a broken highlight if nothing is said.
    NotListed,
}

/// **What the panel says when the selected mark is on no layer.**
///
/// Shown for `Membership::None` and for nothing else.
///
/// ## ★★★ Why this sentence exists, when silence would be simpler
///
/// It is the disambiguating half of a pair. A selection either highlights a
/// row or does not, and "does not" has several causes the operator cannot
/// otherwise tell apart — genuinely unlayered, unresolvable, nested too deep,
/// spanning two layers. This line is said only in the first case, so its
/// *presence* is the answer and its absence is not a claim.
///
/// ## ★★ "Not on a layer", not "on no layer"
///
/// The operator's mental model is that things are *put on* layers. "Not on a
/// layer" describes the mark; "on no layer" describes a set, and reads like
/// the beginning of a fault report. The sentence is a statement about the
/// document, as ordinary as a layer's name, so it is phrased as one.
///
/// ## ★ It says "selected", not "this object"
///
/// Because it covers an annotation — a stamp, a cloud, a note, a dimension —
/// as well as a content object, and the two need one sentence rather than two
/// nearly identical ones.
#[must_use]
pub fn layer_selection_unlayered() -> &'static str {
    "What you have selected is not on a layer."
}

/// **The panel's long form** — the whole sentence, or `None` when nothing is
/// owed.
///
/// # The two states that owe nothing, and they owe nothing for opposite
/// reasons
///
/// * `NothingSelected` — there is no question. A status bar or a panel that
///   narrates the absence of a thing spends a permanent line on the most
///   common state in the program.
/// * `Group` **with its row on screen** — the plate has already said it.
///   Repeating it in prose would make the panel narrate its own highlight, and
///   the operator would read the sentence as a *second* fact.
///
/// # ★ Every other state owes one, including the unknowns
///
/// That is a reversal, and it is deliberate. `Unknown` used to be silent, on
/// the argument that a line reading *"pdfcer cannot tell you which layer this
/// is on"* would be a **permanent apology** — true, while `pdfcer-core` could
/// not answer for any content object at all. `Pass 250.0` retired that
/// premise: the unknowns below are now rare, specific and individually
/// actionable, and withholding them would be hiding an inference the operator
/// cannot see. See `Unresolved`'s own doc comment for the full argument.
#[must_use]
pub fn layer_selection_report(m: Membership, row: RowOfAnswer<'_>) -> Option<String> {
    Some(match (m, row) {
        (Membership::NothingSelected, _) => return None,
        (Membership::Group(_), RowOfAnswer::OnScreen) => return None,
        (Membership::Group(_), RowOfAnswer::HiddenBySearch(name)) => format!(
            "What you have selected is on \"{name}\", which your search has narrowed out of the \
             list."
        ),
        // ★ The `NotAGroup` pairing is unreachable — the panel derives `row`
        // from the same `Membership` — and it is answered rather than
        // `unreachable!()`d, because a panic in a readout is a worse outcome
        // than a slightly vague sentence, and because the pairing is not
        // enforced by a type.
        (Membership::Group(_), RowOfAnswer::NotListed | RowOfAnswer::NotAGroup) => {
            "What you have selected is on an optional-content group this document does not list \
             as a layer. That is legal — a group can be referred to by page content without \
             being registered, and a membership can name a combination of groups rather than \
             one — so there is no row here to highlight."
                .to_owned()
        }
        (Membership::None, _) => layer_selection_unlayered().to_owned(),
        (Membership::Mixed, _) => "What you have selected is on more than one layer, so no \
                                   single row is highlighted."
            .to_owned(),
        (Membership::Unknown(why), _) => unresolved_long(why).to_owned(),
    })
}

/// The long form of each reason pdfcer cannot name the layer.
///
/// Separate from [`layer_selection_report`] so the reasons can be read as a
/// set — they are meant to be *different* from one another, and a reader
/// checking that has them in one place rather than spread through a match with
/// four other arms.
const fn unresolved_long(why: Unresolved) -> &'static str {
    match why {
        Unresolved::PageNotDecomposed => {
            "pdfcer could not read this page's contents, so it cannot say which layer anything on \
             it is on."
        }
        Unresolved::Malformed => {
            "This page marks content as belonging to a layer it does not name, so pdfcer will not \
             say that anything here is on no layer — it may be on the layer that could not be \
             named."
        }
        Unresolved::NestedForm => {
            "What you have selected is drawn from inside nested forms, and pdfcer cannot see \
             which layer the inner one was placed on."
        }
        Unresolved::Stale => {
            "What you have selected has moved since pdfcer last read this page. Click it again."
        }
        Unresolved::OtherPage => {
            "Part of what you have selected is on a page that is not on screen, and pdfcer has \
             not read that page's contents."
        }
    }
}

/// **The status bar's short form** — the same fact, sized for a bar.
///
/// `None` when nothing is owed, on exactly the one state that owes nothing
/// there: nothing selected. Unlike the panel, the bar **does** speak when the
/// row is on screen, because the bar is the surface reached with no panel open
/// and it cannot lean on a plate the operator may not be looking at.
///
/// # ★ `name` and the `Membership` are not independent
///
/// `Some(name)` is meaningful only alongside `Group`, and a caller with a
/// group it could not name passes `None` — which is the OCMD and
/// unregistered-OCG case, and gets its own words rather than an empty pair of
/// quotes.
#[must_use]
pub fn layer_clause(m: Membership, name: Option<&str>) -> Option<String> {
    Some(match m {
        Membership::NothingSelected => return None,
        Membership::Group(_) => match name {
            Some(name) => format!("on layer \"{name}\""),
            None => "on a group this document does not list".to_owned(),
        },
        Membership::None => "not on a layer".to_owned(),
        Membership::Mixed => "on several layers".to_owned(),
        Membership::Unknown(why) => format!("layer not known — {}", unresolved_short(why)),
    })
}

/// The short form of each reason, for the bar.
///
/// ★ Every one of these is a **cause**, not an apology. *"layer not known"*
/// alone would be the hedge this catalog's header forbids; the clause after
/// the dash is what tells the operator whether to look at their file, their
/// selection, or pdfcer.
const fn unresolved_short(why: Unresolved) -> &'static str {
    match why {
        Unresolved::PageNotDecomposed => "this page would not read",
        Unresolved::Malformed => "this page names a layer it does not declare",
        Unresolved::NestedForm => "it is inside nested forms",
        Unresolved::Stale => "the page has changed since you clicked",
        Unresolved::OtherPage => "part of it is on another page",
    }
}

/// Append a layer clause to the status bar's selection line.
///
/// Appended rather than given a line of its own, because it is a fact about
/// **the same selection** — the same reasoning `status::selected` applies to
/// its depth clause. A second label would read as a second subject.
#[must_use]
pub fn selection_with_layer(line: &str, clause: &str) -> String {
    format!("{line} · {clause}")
}

/// ★★★ **The operator's own finding, said back to him: the unit of selection
/// is not his.**
///
/// He measured it on his own drawing — *one PDF path object holds 6,681
/// anchors across half his sheet* — and the largest object on `SW41177.pdf`
/// p1 holds **1,194 subpaths** over 550 × 500 pt. He clicks a circle; pdfcer
/// selects the object the circle is one subpath of.
///
/// # Why the layer answer is still exact, and why that is not enough
///
/// `/OC` membership belongs to a marked-content section, which wraps *paint
/// operators*. A `BDC /OC` cannot begin in the middle of a subpath, so every
/// part of one object shares one membership by construction. The relation has
/// **no finer form to be exact at** — descending to the Part or Point rung
/// cannot refine it.
///
/// So the sentence *"this is on layer Grid"* is true. Said about something the
/// operator believes is a single circle, it is a claim he will apply to the
/// circle, and on his files it is a claim about a thousand other curves too.
///
/// ⇒ This line states the granularity as a **measurement** rather than
/// implying a precision that does not exist. It is off-canvas, per Rule 4, and
/// it is silent on the overwhelmingly common object that holds one part.
#[must_use]
pub fn layer_selection_granularity(parts: usize) -> String {
    format!(
        "The object you selected holds {parts} separate parts. The layer belongs to the whole \
         object, not to the part under your pointer."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::object::ObjId;

    fn group() -> Membership {
        Membership::Group(ObjId::new(4, 0))
    }

    /// Every state of the answer, named **once**, so the tests below cannot
    /// quietly stop covering one.
    ///
    /// ★ The `match` beneath it is the mechanism: adding a variant to
    /// `Membership` makes this file fail to compile, which is what a
    /// hand-written array in a completeness test cannot do. This project has
    /// shipped four defects into exactly that gap (`RESUME.md`, three
    /// separate recurrences), and the fix each time was to make the compiler
    /// hold the list.
    fn every_state() -> Vec<Membership> {
        let all = vec![
            Membership::NothingSelected,
            group(),
            Membership::None,
            Membership::Mixed,
            Membership::Unknown(Unresolved::PageNotDecomposed),
            Membership::Unknown(Unresolved::Malformed),
            Membership::Unknown(Unresolved::NestedForm),
            Membership::Unknown(Unresolved::Stale),
            Membership::Unknown(Unresolved::OtherPage),
        ];
        // The exhaustiveness guard. It computes nothing; it fails to compile
        // when a variant is added and this list was not updated.
        for m in &all {
            match m {
                Membership::NothingSelected
                | Membership::Group(_)
                | Membership::None
                | Membership::Mixed
                | Membership::Unknown(
                    Unresolved::PageNotDecomposed
                    | Unresolved::Malformed
                    | Unresolved::NestedForm
                    | Unresolved::Stale
                    | Unresolved::OtherPage,
                ) => {}
            }
        }
        all
    }

    /// **Exactly two states owe the panel nothing**, and both for stated
    /// reasons: there is no question, or the plate has already answered it.
    #[test]
    fn the_panel_is_silent_only_where_silence_is_the_answer() {
        for m in every_state() {
            let row = if matches!(m, Membership::Group(_)) {
                RowOfAnswer::OnScreen
            } else {
                RowOfAnswer::NotAGroup
            };
            let said = layer_selection_report(m, row);
            match m {
                Membership::NothingSelected | Membership::Group(_) => {
                    assert!(said.is_none(), "{m:?} should say nothing in the panel");
                }
                _ => assert!(said.is_some(), "{m:?} owes the panel a sentence"),
            }
        }
    }

    /// ★★★ **A highlighted row that is off screen still owes words.**
    ///
    /// The failure this forbids is silent and reads as a broken feature: the
    /// operator types in the search, the matching layer is narrowed out, the
    /// plate goes with it, and the panel looks exactly as it would if
    /// selecting an object highlighted nothing at all.
    #[test]
    fn a_group_whose_row_is_not_on_screen_is_reported_in_words() {
        let hidden = layer_selection_report(group(), RowOfAnswer::HiddenBySearch("Grid"))
            .expect("a hidden row owes a sentence");
        assert!(hidden.contains("Grid"), "it must name what is hidden");
        assert!(layer_selection_report(group(), RowOfAnswer::NotListed).is_some());
        assert_ne!(
            hidden,
            layer_selection_report(group(), RowOfAnswer::NotListed).unwrap(),
            "a search-hidden row and an unlisted group are different facts and need different \
             sentences — one is fixed by clearing the search and the other cannot be fixed"
        );
    }

    /// **The five reasons are five sentences**, long and short.
    ///
    /// A hedge repeated five times would satisfy every other test here and
    /// teach the operator that the reason clause carries no information.
    #[test]
    fn each_reason_says_something_the_others_do_not() {
        let reasons = [
            Unresolved::PageNotDecomposed,
            Unresolved::Malformed,
            Unresolved::NestedForm,
            Unresolved::Stale,
            Unresolved::OtherPage,
        ];
        for (i, a) in reasons.iter().enumerate() {
            for b in reasons.iter().skip(i + 1) {
                assert_ne!(unresolved_long(*a), unresolved_long(*b));
                assert_ne!(unresolved_short(*a), unresolved_short(*b));
            }
            assert!(
                unresolved_long(*a).len() > 40,
                "a long form too short to be specific: {}",
                unresolved_long(*a)
            );
            assert!(
                unresolved_short(*a).len() < 60,
                "a short form too long for the bar: {}",
                unresolved_short(*a)
            );
        }
    }

    /// **The bar speaks for every state except "nothing selected"** —
    /// including for a group whose row is on screen, because the bar is
    /// reached with no panel open and cannot lean on a plate.
    #[test]
    fn the_bar_answers_wherever_there_is_a_question() {
        for m in every_state() {
            let said = layer_clause(m, Some("Grid"));
            if matches!(m, Membership::NothingSelected) {
                assert!(said.is_none());
            } else {
                assert!(said.is_some(), "{m:?} owes the bar a clause");
            }
        }
    }

    /// ★★ **A group pdfcer cannot name does not render as empty quotes.**
    ///
    /// `on layer ""` is the shape of a placeholder, and R9 forbids one. The
    /// unnamed case is a different sentence, not the same sentence with a hole
    /// in it.
    #[test]
    fn an_unnamed_group_gets_its_own_words() {
        let named = layer_clause(group(), Some("Grid")).unwrap();
        let unnamed = layer_clause(group(), None).unwrap();
        assert!(named.contains("Grid"));
        assert!(
            !unnamed.contains("\"\""),
            "an empty pair of quotes: {unnamed}"
        );
        assert_ne!(named, unnamed);
    }

    /// **"Not on a layer" and "layer not known" are different clauses**, on
    /// the bar as well as in the panel.
    ///
    /// The whole three-valued design collapses if the two surfaces that carry
    /// it ever spell them the same.
    #[test]
    fn the_bar_keeps_the_distinction_the_type_exists_for() {
        let none = layer_clause(Membership::None, None).unwrap();
        let unknown = layer_clause(Membership::Unknown(Unresolved::NestedForm), None).unwrap();
        assert_ne!(none, unknown);
    }

    /// **The granularity line is a count, not a hedge.**
    #[test]
    fn the_granularity_line_carries_the_number() {
        let said = layer_selection_granularity(1194);
        assert!(
            said.contains("1194"),
            "the measurement must be in it: {said}"
        );
    }

    /// The clause is appended to the line rather than replacing it.
    #[test]
    fn the_bar_keeps_what_it_already_said() {
        let line = selection_with_layer("Selected: Path · 10.0 × 10.0 pt", "on layer \"Grid\"");
        assert!(line.starts_with("Selected: Path"));
        assert!(line.ends_with("on layer \"Grid\""));
    }
}
