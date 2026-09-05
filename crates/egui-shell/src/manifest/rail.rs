//! # `manifest::rail` — the left rail, as serializable data
//!
//! The vertical strip down the outer edge of a dock side: the panel tabs, the
//! navigate selectors, the selection tools, and whatever else an application
//! wants permanently one click away.
//!
//! `OPERATOR_REQUESTS.md` **O123**, his last paragraph, verbatim:
//!
//! > *"What I'd also added in the bar at the left side that we are adding: the
//! > navigate selectors and some other related selection controls (lasso tool
//! > when we implement one, etc) and these will fold up into a drop down arrow
//! > if space becomes scarce."*
//!
//! and **O126**'s addendum:
//!
//! > *"also add rotate pages to that area, and those should be available in
//! > every mode including read."*
//!
//! ## ★★★ Why this is a manifest type and not a callback
//!
//! `SHELL_FRAMEWORK.md` makes the ribbon, the dock, the modes and the keymap
//! one serializable [`super::Shell`] document, and says why in one line:
//! *"a rail that only `pdfcer-gui` knows about breaks it quietly."* A surface
//! whose contents live in a Rust function cannot be customized by an operator
//! file, cannot be merged by [`super::merge`], cannot be validated by
//! [`super::validate`], and cannot be read by a tool that does not link
//! `egui`. Every one of those is a property the other four regions have, and a
//! sixth region that quietly lacks them is how a "serializable shell" stops
//! being one.
//!
//! The worked precedent is [`super::Trailing`], added the same week for
//! *Open in Acrobat*: a region introduced as **data on `Shell`** rather than
//! as a builder callback, carrying [`super::Item`]s so `visible_when` can make
//! a control *absent* rather than greyed.
//!
//! ## ★★ R7 — nothing here knows what a PDF is
//!
//! `tools/gates/check-shell-purity.sh` forbids `egui-shell` naming anything
//! from `pdfcer-*`. A [`RailGroup`] carries an id, an optional caption and a
//! list of command ids. It does not know that `pages` is a page thumbnail
//! list, that `view.tool_hand` pans, or that `pages.rotate_left` writes
//! `/Rotate`. That is exactly the line the gate draws, and it is what lets the
//! same rail type serve an application that has never heard of a document.
//!
//! ## The fold policy is DECLARED, not derived
//!
//! [`RailFold`] is per group, and the reason it is authored rather than
//! computed is `RIBBON_SCALING.md` §3.2's finding about Word, arrived at by
//! photographing it: groups collapse in an **authored** order, not
//! right-to-left, and *Clipboard never collapses at any width*. The same
//! evidence, one surface over. A rail that derived its fold order from
//! position would fold the panel tabs first, and the panel tabs are the
//! rail's entire argument for existing.

use serde::{Deserialize, Serialize};

use super::Item;

/// What happens to a group when the rail runs out of room.
///
/// One value per group, authored in the manifest. See the module header on
/// why this is declared rather than derived from position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RailFold {
    /// ★★ **The floor. This group is drawn at every rung, entire.**
    ///
    /// For pdfcer this is the five panel tabs, and *"all five panels one click
    /// away"* is the only reason the rail exists rather than a horizontal tab
    /// bar. A rail that folds them is strictly worse than the tab stack it
    /// replaced — at that point the honest move is to switch arrangements, not
    /// to keep shrinking.
    ///
    /// The default, because a group whose author did not think about folding
    /// should not silently disappear.
    #[default]
    Never,
    /// The whole group goes behind the chevron together, caption and all.
    ///
    /// For a group of specialist gestures — nothing in it reached by habit,
    /// nothing in it with a keyboard chord.
    Whole,
    /// ★★★ The group collapses to a **single pinned row: whatever is armed**.
    ///
    /// For a set of mutually exclusive modal tools. Folding such a group
    /// entirely would leave the operator holding a tool the rail cannot name,
    /// and naming the armed tool is the job the one-line tool status handed
    /// this strip. The pinned row shows the armed member **even when it was
    /// armed from a ribbon tab that is not open**, which is the case that
    /// makes this variant worth having: the rail is the only permanent
    /// surface, so it is the only one that can answer *what am I holding* at
    /// every moment.
    ///
    /// Which member is "armed" is [`crate::ribbon::band::selected_condition`]
    /// — the same `selected:<id>` convention a ribbon toggle already renders
    /// pressed on. No new state, no second source of truth.
    PinArmed,
}

impl RailFold {
    /// Serde's `skip_serializing_if` predicate for the default.
    #[must_use]
    pub fn is_default(&self) -> bool {
        *self == Self::Never
    }
}

/// One run of rail entries under an optional caption.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RailGroup {
    /// A stable id, used for the region name the group publishes and as the
    /// handle an operator overlay names when it wants to replace this group's
    /// items. Never displayed.
    pub id: String,
    /// The word drawn above the group at the widest rung — `navigate`,
    /// `select`.
    ///
    /// ★ Optional, and absent for the panel tabs on purpose: a caption over
    /// the first group in the strip would be a heading for the whole rail
    /// rather than for that group, which is a different claim.
    ///
    /// The caption is **the first thing dropped** as room gets scarce — see
    /// [`crate::dock::rail::Rung::Tight`]. It is presentation about a group
    /// whose members are still all one click away, so it is the cheapest thing
    /// on the strip to give up.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// What happens to this group when the rail runs out of room.
    #[serde(skip_serializing_if = "RailFold::is_default")]
    pub fold: RailFold,
    /// The entries, top to bottom.
    ///
    /// [`Item::Command`] only, in practice: the rail's row shape is a picture
    /// with an optional word under it, and [`Item::Separator`] has a group
    /// rule to do its job while [`Item::Custom`] would be a widget in a 52 pt
    /// column. Both are accepted by the type and ignored by the planner, for
    /// [`super::Trailing`]'s stated reason — `Item` is the shared vocabulary,
    /// and a second item type for this region would duplicate `visible_when`,
    /// which every region genuinely does share.
    pub items: Vec<Item>,
}

impl RailGroup {
    /// A group of items under an id, folding as [`RailFold::Never`].
    #[must_use]
    pub fn new(id: impl Into<String>, items: impl IntoIterator<Item = Item>) -> Self {
        Self {
            id: id.into(),
            caption: None,
            fold: RailFold::Never,
            items: items.into_iter().collect(),
        }
    }

    /// Give the group a caption, drawn at the widest rung only.
    #[must_use]
    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Set what happens to this group when room runs short.
    #[must_use]
    pub fn with_fold(mut self, fold: RailFold) -> Self {
        self.fold = fold;
        self
    }
}

/// The rail: a list of groups, top to bottom.
///
/// A newtype rather than a bare `Vec` for [`super::Trailing`]'s reason — the
/// region gets a name a doc comment can hang an argument on, and a
/// present-but-empty rail is treated exactly as an absent one so that an
/// operator customization which removed the last group reclaims the strip
/// instead of leaving a 52 pt column of nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rail(pub Vec<RailGroup>);

impl Rail {
    /// The groups, in drawing order — top to bottom.
    #[must_use]
    pub fn groups(&self) -> &[RailGroup] {
        &self.0
    }

    /// Whether there is nothing to draw.
    ///
    /// True for a rail with no groups **and** for one whose every group is
    /// empty: a caption with no entries under it is the placeholder R9
    /// forbids, and a strip of nothing but captions is that defect repeated.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|g| g.items.is_empty())
    }
}

impl FromIterator<RailGroup> for Rail {
    fn from_iter<I: IntoIterator<Item = RailGroup>>(iter: I) -> Self {
        Rail(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A rail of empty groups is an absent rail, not a column of captions.
    #[test]
    fn a_rail_of_empty_groups_is_empty() {
        let rail: Rail = [
            RailGroup::new("navigate", []).with_caption("navigate"),
            RailGroup::new("select", []).with_caption("select"),
        ]
        .into_iter()
        .collect();
        assert!(rail.is_empty());
        assert!(Rail::default().is_empty());
    }

    /// One populated group is enough to make the rail worth drawing.
    #[test]
    fn one_populated_group_is_not_empty() {
        let rail: Rail = [
            RailGroup::new("select", []),
            RailGroup::new("tabs", [Item::command("view.panel_pages")]),
        ]
        .into_iter()
        .collect();
        assert!(!rail.is_empty());
    }

    /// The default fold is the floor, so a group whose author said nothing
    /// about folding does not silently disappear.
    #[test]
    fn the_default_fold_never_folds() {
        assert_eq!(RailFold::default(), RailFold::Never);
        assert!(RailFold::Never.is_default());
        assert!(!RailFold::Whole.is_default());
        assert!(!RailFold::PinArmed.is_default());
    }

    /// The document round-trips, which is the whole claim of putting the rail
    /// in the manifest rather than in a builder.
    #[test]
    fn a_rail_round_trips_through_ron() {
        let rail: Rail = [
            RailGroup::new("tabs", [Item::command("view.panel_pages")]),
            RailGroup::new("navigate", [Item::command("view.tool_hand")])
                .with_caption("navigate")
                .with_fold(RailFold::PinArmed),
        ]
        .into_iter()
        .collect();
        let text = ron::ser::to_string(&rail).expect("serialize");
        let back: Rail = ron::from_str(&text).expect("deserialize");
        assert_eq!(rail, back);
    }
}
