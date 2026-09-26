//! `text::panels::layers` — **every sentence pdfcer says about which layer a
//! selection is on**, in one module.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/layers.md`.

use crate::layermembership::{Membership, Unresolved};

/// Where the answer's optional-content group sits **relative to the list the
/// panel is actually drawing**.
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
#[must_use]
pub fn layer_selection_unlayered() -> &'static str {
    "What you have selected is not on a layer."
}

/// **The panel's long form** — the whole sentence, or `None` when nothing is
/// owed.
#[must_use]
pub fn layer_selection_report(m: Membership, row: RowOfAnswer<'_>) -> Option<String> {
    Some(match (m, row) {
        (Membership::NothingSelected, _) => return None,
        (Membership::Group(_), RowOfAnswer::OnScreen) => return None,
        (Membership::Group(_), RowOfAnswer::HiddenBySearch(name)) => format!(
            "What you have selected is on \"{name}\", which your search has narrowed out of the \
             list."
        ),
        // The `NotAGroup` pairing is unreachable — the panel derives `row`
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
#[must_use]
pub fn selection_with_layer(line: &str, clause: &str) -> String {
    format!("{line} · {clause}")
}

/// **The operator's own finding, said back to him: the unit of selection
/// is not his.**
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

    /// **A highlighted row that is off screen still owes words.**
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

    /// **A group pdfcer cannot name does not render as empty quotes.**
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
