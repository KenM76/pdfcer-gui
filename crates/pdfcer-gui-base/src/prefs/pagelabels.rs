//! # `prefs::pagelabels` — what page labels do when pages move
//!
//! Key: `page_labels` (`follow` | `sequence`, default `follow`). One choice
//! that sets the engine's three page-label policies together, so moving,
//! deleting and inserting sheets agree:
//!
//! | Choice | Reorder | Delete | Insert |
//! |---|---|---|---|
//! | `follow` | `FollowPages` | `KeepEach` | `Source` |
//! | `sequence` | `Positional` | `Renumber` | `ContinueRange` |
//!
//! `follow` is the default because a drawing set's labels name sheets
//! ("A-101"), and a sheet that moves must keep its name. `sequence` is the
//! book reading, and what Acrobat's reorder does.

use std::fmt::Write as _;

use pdfcer_core::pageops::{DeletedPageLabels, InsertedPageLabels, ReorderedPageLabels};

use super::printing::KeyOutcome;

/// The file key.
const KEY: &str = "page_labels";

/// What page labels do when pages are moved, deleted or inserted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageLabelPolicy {
    /// Each page keeps the label it showed.
    #[default]
    FollowPages,
    /// Numbering stays in page order.
    InSequence,
}

impl PageLabelPolicy {
    /// Every choice, default first.
    pub const ALL: &'static [Self] = &[Self::FollowPages, Self::InSequence];

    /// The file and trace token.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::FollowPages => "follow",
            Self::InSequence => "sequence",
        }
    }

    /// The choice a token names.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.id() == id)
    }

    /// The engine policy for a reorder.
    #[must_use]
    pub const fn reordered(self) -> ReorderedPageLabels {
        match self {
            Self::FollowPages => ReorderedPageLabels::FollowPages,
            Self::InSequence => ReorderedPageLabels::Positional,
        }
    }

    /// The engine policy for a delete.
    #[must_use]
    pub const fn deleted(self) -> DeletedPageLabels {
        match self {
            Self::FollowPages => DeletedPageLabels::KeepEach,
            Self::InSequence => DeletedPageLabels::Renumber,
        }
    }

    /// The engine policy for an insert.
    #[must_use]
    pub const fn inserted(self) -> InsertedPageLabels {
        match self {
            Self::FollowPages => InsertedPageLabels::Source,
            Self::InSequence => InsertedPageLabels::ContinueRange,
        }
    }
}

/// Claim `key = value` if it is this group's.
pub(super) fn parse_key(pref: &mut PageLabelPolicy, key: &str, value: &str) -> KeyOutcome {
    if key != KEY {
        return KeyOutcome::NotMine;
    }
    match PageLabelPolicy::from_id(value.trim()) {
        Some(choice) => {
            *pref = choice;
            KeyOutcome::Accepted
        }
        None => KeyOutcome::BadValue,
    }
}

/// Append this group's block to a preferences file.
pub(super) fn write_block(pref: PageLabelPolicy, out: &mut String) {
    out.push_str(
        "\n\
         # page_labels: follow or sequence (default follow). When pages are\n\
         # moved, deleted or inserted: follow keeps each page's label (a\n\
         # sheet called A-101 stays A-101); sequence keeps the numbering in\n\
         # page order.\n",
    );
    let _ = writeln!(out, "{KEY} = {}", pref.id()); // ui-text-exempt: a preferences-file line, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_token_round_trips() {
        for choice in PageLabelPolicy::ALL {
            let mut out = String::new();
            write_block(*choice, &mut out);
            let line = out.lines().last().unwrap();
            let (key, value) = line.split_once('=').unwrap();
            let mut read = PageLabelPolicy::default();
            if *choice == read {
                read = PageLabelPolicy::InSequence;
            }
            assert_eq!(
                parse_key(&mut read, key.trim(), value),
                KeyOutcome::Accepted
            );
            assert_eq!(read, *choice);
        }
    }

    #[test]
    fn follow_sets_all_three_engine_policies_to_keep_labels() {
        let p = PageLabelPolicy::FollowPages;
        assert_eq!(p.reordered(), ReorderedPageLabels::FollowPages);
        assert_eq!(p.deleted(), DeletedPageLabels::KeepEach);
        assert_eq!(p.inserted(), InsertedPageLabels::Source);
        let s = PageLabelPolicy::InSequence;
        assert_eq!(s.reordered(), ReorderedPageLabels::Positional);
        assert_eq!(s.deleted(), DeletedPageLabels::Renumber);
        assert_eq!(s.inserted(), InsertedPageLabels::ContinueRange);
    }

    #[test]
    fn an_unknown_token_is_a_bad_value() {
        let mut p = PageLabelPolicy::default();
        assert_eq!(parse_key(&mut p, KEY, "maybe"), KeyOutcome::BadValue);
        assert_eq!(parse_key(&mut p, "other", "follow"), KeyOutcome::NotMine);
    }
}
