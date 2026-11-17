//! # `text::grouping` — every sentence Group and Ungroup can owe
//!
//! Two kinds of thing group, and the words keep them apart: **page objects**
//! become one placed drawing (a form XObject), **markups** become one comment
//! thread led by a primary (ISO 32000-1 §12.5.6.2). A ce dimension is neither:
//! it groups through its ce-dimension group.

/// Why Group or Ungroup changed nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupRefusal {
    /// Fewer than two things were selected to group.
    TooFew,
    /// A page object is under a different clip, layer or tagged section.
    DifferentScope,
    /// A page object sets a clip or tagged section in its own bytes.
    Entangled,
    /// The transform at the group's position does not invert.
    NoBox,
    /// The selected page object is not a form XObject.
    NotADrawing,
    /// The form is a transparency group (§11.6.6).
    BlendsAsOne,
    /// The form carries `/OC` (§8.11.3.2).
    OnALayer,
    /// The form's content cannot stand in the page's stream.
    CannotInline,
    /// A selected annotation is not a markup annotation.
    NotMarkup,
    /// A selected annotation is a ce dimension.
    CeDimension,
    /// A selected markup is a reply or already a group member.
    AlreadyLinked,
    /// A selected markup is the primary of another group.
    LeadsAGroup,
    /// The selected markup is in no group.
    NotGrouped,
    /// The selected markups are on different pages.
    DifferentPages,
    /// Anything else the engine refused.
    Other,
}

impl GroupRefusal {
    /// The sentence.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::TooFew => "Select two or more things to group them.",
            Self::DifferentScope => {
                "One of the selected objects is drawn inside a different clipping region, layer \
                 or tagged section from the others, so they cannot be gathered into one group \
                 without changing how the page draws. Nothing was grouped."
            }
            Self::Entangled => {
                "One of the selected objects sets up a clipping region or tagged section of its \
                 own, so it cannot go into a group. Nothing was grouped."
            }
            Self::NoBox => {
                "The page is squashed flat where the group would sit, so the group would have no \
                 box. Nothing was grouped."
            }
            Self::NotADrawing => {
                "The selected object is not a group or a placed drawing, so there is nothing to \
                 ungroup."
            }
            Self::BlendsAsOne => {
                "This drawing blends with the page as one unit. Taken apart, its pieces would \
                 blend differently, so it stays whole."
            }
            Self::OnALayer => {
                "This drawing belongs to a layer. Taken apart, its pieces would show even with \
                 the layer off, so it stays whole."
            }
            Self::CannotInline => {
                "This drawing's content cannot be moved onto the page as it stands, so it stays \
                 whole."
            }
            Self::NotMarkup => {
                "One of the selected items is not a markup, so it cannot join a group. Nothing \
                 was grouped."
            }
            Self::CeDimension => {
                "A ce dimension is grouped through its ce-dimension group, not with markups. \
                 Nothing was grouped."
            }
            Self::AlreadyLinked => {
                "One of the selected markups is a reply to another, or is already in a group. \
                 Ungroup it first. Nothing was grouped."
            }
            Self::LeadsAGroup => {
                "One of the selected markups already leads a group of its own. Ungroup that \
                 group first. Nothing was grouped."
            }
            Self::NotGrouped => "The selected markup is not in a group.",
            Self::DifferentPages => {
                "Markups on different pages cannot be grouped. Nothing was grouped."
            }
            Self::Other => "That could not be grouped or ungrouped. Nothing changed.",
        }
    }

    /// Every refusal, for tests.
    #[must_use]
    pub const fn all() -> [Self; 15] {
        [
            Self::TooFew,
            Self::DifferentScope,
            Self::Entangled,
            Self::NoBox,
            Self::NotADrawing,
            Self::BlendsAsOne,
            Self::OnALayer,
            Self::CannotInline,
            Self::NotMarkup,
            Self::CeDimension,
            Self::AlreadyLinked,
            Self::LeadsAGroup,
            Self::NotGrouped,
            Self::DifferentPages,
            Self::Other,
        ]
    }
}

/// **The group took the depth of its front-most member**, so a member that was
/// behind an unselected object in between now draws over it. Owed only when
/// the members were not next to each other in paint order.
#[must_use]
pub const fn group_gathered_at_the_front() -> &'static str {
    "The group sits where its front-most object was, so any object that was between its \
     members is now drawn under all of them."
}

/// **The drawing's frame was kept as a clip around its pieces.**
#[must_use]
pub const fn ungrouped_keeps_its_frame() -> &'static str {
    "The drawing's frame was kept as a clipping region around its pieces, so they draw exactly \
     as the drawing did."
}

/// **Resources that clashed with the page's took a new name.**
#[must_use]
pub fn ungrouped_renamed(count: usize) -> String {
    if count == 1 {
        "One font or other resource the drawing used shares its name with a different one on \
         this page, so it was given a new name here. Nothing looks different."
            .to_owned()
    } else {
        format!(
            "{count} fonts or other resources the drawing used share their names with different \
             ones on this page, so they were given new names here. Nothing looks different."
        )
    }
}

/// **What other readers do with a markup group**: the comment, author and
/// colour shown for the group are the primary's.
#[must_use]
pub fn markups_grouped(count: usize) -> String {
    format!(
        "{count} markups are now one group. Other PDF readers show the group's comment and \
         author from the markup you selected last; the others keep their own in the file and \
         get them back when ungrouped."
    )
}

/// **A markup group was dissolved**; each member's own comment is in force
/// again.
#[must_use]
pub fn markups_ungrouped(count: usize) -> String {
    if count == 1 {
        "One markup left the group and shows its own comment and author again.".to_owned()
    } else {
        format!("{count} markups left the group and show their own comments and authors again.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every refusal has a sentence, and no two share one.**
    #[test]
    fn every_refusal_has_its_own_sentence() {
        let mut lines: Vec<&str> = GroupRefusal::all().iter().map(|r| r.line()).collect();
        assert!(lines.iter().all(|l| l.ends_with('.') && l.len() > 20));
        let total = lines.len();
        lines.sort_unstable();
        lines.dedup();
        assert_eq!(lines.len(), total);
    }

    /// **A ce dimension is never called a bare dimension.**
    #[test]
    fn the_ce_dimension_sentence_says_ce() {
        let line = GroupRefusal::CeDimension.line();
        assert_eq!(
            line.matches("dimension").count(),
            line.matches("ce-dimension").count() + line.matches("ce dimension").count()
        );
    }

    /// **Counts read as numbers, and one reads as a word.**
    #[test]
    fn counts_are_worded() {
        assert!(ungrouped_renamed(1).starts_with("One "));
        assert!(ungrouped_renamed(3).starts_with("3 "));
        assert!(markups_ungrouped(1).starts_with("One "));
        assert!(markups_grouped(4).starts_with("4 "));
    }
}
