//! Named workspaces — save, load, list, delete.
//!
//! # What a workspace is, and what it deliberately is not
//!
//! A workspace is **a name and an arrangement**. That is the whole type.
//!
//! `MODES_AND_PANELS.md` treats a mode selector and flexible panel areas
//! as one system — *"A mode is a named workspace plus a capability
//! set"* — and its capability **(g)** is this module: *"Capability (g) is
//! what the modes are: Read, Review and Edit are three registered
//! workspaces."*
//!
//! So the mode names live in the **application's** configuration, not
//! here. Nothing in this file knows them, and nothing in this crate ever
//! will: `SHELL_FRAMEWORK.md` §4 states the same rule for the manifest —
//! *"Read/Review/Edit is a configuration; nothing in the crate knows
//! those names"* — and a workspace store that shipped three magic names
//! would be the same violation wearing a different hat. An application
//! that wants three modes registers three workspaces; one that wants
//! eleven registers eleven; one that wants none never calls this
//! module.
//!
//! # Why this is table stakes rather than a luxury
//!
//! The peer table in `MODES_AND_PANELS.md` is unambiguous: Photoshop,
//! Krita and Affinity all ship named layouts, and the benchmark product
//! does not — with the consequence that its community's workaround is
//! *"copying a state file aside and back, awkward because the file is
//! rewritten on every exit"*. Failure mode #12 pairs that with the
//! missing in-app reset and rules on both: *"Both are table stakes, not
//! luxuries."*
//!
//! Note what that workaround actually is: an operator hand-rolling this
//! module out of file copies, and losing to a race with the application's
//! own writer. Two functions here retire it.
//!
//! # The one rule that keeps the store honest
//!
//! **Names are unique and are matched exactly.** Saving over an existing
//! name replaces it in place — keeping its position in the list, so a
//! workspace does not jump to the end of a menu every time it is
//! updated — and loading a name that is not there returns `None` rather
//! than a nearest match. A store that guessed would make "load Review"
//! into a command whose effect depends on what else is saved.

use serde::{Deserialize, Serialize};

use super::skip::{LayoutSite, LayoutSkipReason, LoadReport};
use super::{LayoutDocument, Scope, sanitize};
use crate::dock::model::{DockLayout, PanelCatalog, PanelId};

/// One named arrangement.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    /// What the operator called it.
    ///
    /// Free text, and deliberately not an id: a workspace is named by a
    /// person, for a person, and forcing `review_layout_2` on them buys
    /// nothing. Uniqueness is enforced on load; see the module header.
    pub name: String,
    /// The arrangement it restores.
    pub layout: DockLayout,
    /// **Which panels existed when this arrangement was saved.**
    ///
    /// Not the panels it *mounts* — the panels the application had
    /// registered at the moment it was written, mounted or not. That
    /// distinction is the whole point: it is what lets a later load tell
    /// *"the operator closed this"* from *"this did not exist yet"*, two
    /// states that are otherwise identical in a saved layout.
    ///
    /// # The failure this prevents
    ///
    /// A consumer that adds a panel in a new release ships it **invisible
    /// to everyone who already uses the program**. Their saved layout does
    /// not mount it, so it is not shown; and it is not shown, so nobody
    /// finds out. Worse, the usual advice for a portable build — *replace
    /// the binary, keep your settings* — is exactly what preserves the
    /// stale layout, so the more carefully an operator upgrades, the more
    /// reliably they miss the new feature.
    ///
    /// Neither obvious fix works. Forcing the default arrangement over a
    /// remembered one discards the operator's own work, which is the
    /// entire feature this module exists to provide. Leaving it means
    /// every panel added from now on is born hidden.
    ///
    /// # `None` means "written before anyone recorded this"
    ///
    /// Deliberately an `Option`, not an empty `Vec`. An empty set is a
    /// real, different answer — *"no panels were registered"* — and
    /// conflating the two would make every panel look new to every old
    /// file, re-opening panels the operator had deliberately closed. That
    /// is a worse bug than the one being fixed, because it undoes a
    /// decision the operator actually made.
    ///
    /// [`LayoutDocument::unseen_panels`] reports the distinction rather
    /// than resolving it: what to do about `None` is a product decision
    /// about a specific application's upgrade path, and this crate does
    /// not have the standing to make it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_panels: Option<Vec<PanelId>>,
}

impl Workspace {
    /// Name an arrangement.
    #[must_use]
    pub fn new(name: impl Into<String>, layout: DockLayout) -> Self {
        Self {
            name: name.into(),
            layout,
            known_panels: None,
        }
    }

    /// Record which panels existed when this arrangement was saved.
    ///
    /// A consumer calls this with **every id it has registered**, not with
    /// the ids the layout mounts. See [`Self::known_panels`].
    #[must_use]
    pub fn knowing(mut self, registered: impl IntoIterator<Item = PanelId>) -> Self {
        let mut ids: Vec<PanelId> = registered.into_iter().collect();
        // Sorted and deduplicated so the serialized form is stable: a file
        // that reordered its own list every save would produce a diff on
        // every write and make a real change impossible to spot in one.
        ids.sort();
        ids.dedup();
        self.known_panels = Some(ids);
        self
    }
}

/// What a workspace can say about panels registered *now*.
///
/// Returned by [`LayoutDocument::unseen_panels`], which reports rather than
/// decides — see [`Workspace::known_panels`] for why the decision is the
/// application's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unseen {
    /// The workspace predates the record, so nothing can be concluded.
    ///
    /// **Not the same as `New(vec![])`.** That would say "every registered
    /// panel was already known", which is a claim; this says there is no
    /// evidence either way. An application deciding what to do here is
    /// choosing a one-time upgrade policy, and the two cases want opposite
    /// answers: `New(vec![])` means do nothing, `Unknown` means decide.
    Unknown,
    /// These registered panels did not exist when the workspace was saved.
    ///
    /// Empty when the application has registered nothing the workspace had
    /// not already seen — the ordinary case on every launch after the
    /// first.
    New(Vec<PanelId>),
}

impl LayoutDocument {
    /// Save `layout` under `name`, replacing any workspace already using
    /// that name **in place**.
    ///
    /// Returns whether an existing workspace was replaced, so an
    /// application can offer "overwrite?" *after* the fact in a status
    /// surface rather than asking before it — the same posture the rest
    /// of this crate takes towards modal interruptions.
    ///
    /// Replacing in place rather than appending is not cosmetic: a
    /// workspace that moved to the end of the list every time it was
    /// updated would reorder the operator's own menu behind their back,
    /// and a menu whose order changes when you use it is a menu you
    /// cannot build muscle memory for.
    pub fn save_workspace(&mut self, name: impl Into<String>, layout: DockLayout) -> bool {
        let name = name.into();
        let mut layout = layout;
        layout.normalize();
        if let Some(existing) = self.workspaces.iter_mut().find(|w| w.name == name) {
            existing.layout = layout;
            return true;
        }
        self.workspaces.push(Workspace::new(name, layout));
        false
    }

    /// The arrangement saved under `name`, if any.
    ///
    /// Returns a reference; the caller clones it into
    /// [`crate::dock::DockState::set_layout`]. Deliberately **not** a
    /// method that applies it: the store does not own the live state, and
    /// a function here that reached into a `DockState` would be a second
    /// path by which the arrangement changes.
    #[must_use]
    pub fn workspace(&self, name: &str) -> Option<&DockLayout> {
        self.workspaces
            .iter()
            .find(|w| w.name == name)
            .map(|w| &w.layout)
    }

    /// **Which currently-registered panels a saved workspace has never
    /// seen.**
    ///
    /// `registered` is every panel id the application has registered right
    /// now — not the ids the layout mounts, and not the ids it *would*
    /// mount by default. The comparison is against what EXISTED, which is
    /// the only thing that separates "closed on purpose" from "did not
    /// exist yet".
    ///
    /// Returns [`Unseen::Unknown`] for a workspace saved before the record
    /// existed, and for a name that is not in the store at all — in both
    /// cases the honest answer is that this document cannot say. A caller
    /// that wants to distinguish them can check
    /// [`Self::workspace`] first.
    ///
    /// # This reports; it does not act
    ///
    /// Consistent with [`Self::workspace`] returning a reference rather
    /// than applying it: the store does not own the live state, and a
    /// method here that mounted a panel would be a second path by which
    /// the arrangement changes. What to do with the answer — mount it,
    /// mention it in a status line, ignore it — is the application's, and
    /// it is a product decision rather than a framework one.
    #[must_use]
    pub fn unseen_panels(&self, name: &str, registered: &[PanelId]) -> Unseen {
        let Some(workspace) = self.workspaces.iter().find(|w| w.name == name) else {
            return Unseen::Unknown;
        };
        let Some(known) = &workspace.known_panels else {
            return Unseen::Unknown;
        };
        Unseen::New(
            registered
                .iter()
                .filter(|id| !known.contains(id))
                .cloned()
                .collect(),
        )
    }

    /// Stamp `name` with the panels registered now, without touching its
    /// arrangement.
    ///
    /// For the moment **after** an application has acted on an
    /// [`Unseen`] answer: having decided what to do about the new panels,
    /// it records that it has seen them, so the next launch reports
    /// `New(vec![])` rather than offering the same ones again.
    ///
    /// Separate from [`Self::save_workspace`] because the two happen at
    /// different times and for different reasons — saving is the operator
    /// rearranging something, stamping is the application acknowledging a
    /// release. Folding them together would mean an application could only
    /// record what it had seen by also rewriting a layout it had no reason
    /// to touch.
    ///
    /// Returns whether the workspace existed.
    pub fn mark_panels_seen(&mut self, name: &str, registered: &[PanelId]) -> bool {
        let Some(workspace) = self.workspaces.iter_mut().find(|w| w.name == name) else {
            return false;
        };
        let mut ids = registered.to_vec();
        ids.sort();
        ids.dedup();
        workspace.known_panels = Some(ids);
        true
    }

    /// Every workspace name, in the order they were first saved.
    #[must_use]
    pub fn workspace_names(&self) -> Vec<&str> {
        self.workspaces.iter().map(|w| w.name.as_str()).collect()
    }

    /// Delete the workspace called `name`.
    ///
    /// Returns whether one was removed. Deleting something that is not
    /// there is not an error — a second click on a delete command, or a
    /// stale menu, must not produce a failure the operator has to read.
    pub fn delete_workspace(&mut self, name: &str) -> bool {
        let before = self.workspaces.len();
        self.workspaces.retain(|w| w.name != name);
        self.workspaces.len() != before
    }

    /// Rename a workspace, refusing if the new name is taken or empty.
    ///
    /// Refusing rather than merging: a rename that silently absorbed
    /// another workspace would destroy an arrangement the operator did
    /// not mention.
    pub fn rename_workspace(&mut self, from: &str, to: impl Into<String>) -> bool {
        let to = to.into();
        if to.trim().is_empty() || self.workspaces.iter().any(|w| w.name == to) {
            return false;
        }
        match self.workspaces.iter_mut().find(|w| w.name == from) {
            Some(w) => {
                w.name = to;
                true
            }
            None => false,
        }
    }
}

/// Sanitize every workspace in a loaded document, dropping the ones that
/// cannot survive and repairing the ones that can.
///
/// **Per workspace**, which is the whole point: one saved arrangement
/// naming a panel this build does not offer loses that tab; one with no
/// name at all is dropped; a second one claiming a name already used is
/// dropped; and every other workspace in the file is untouched. That is
/// the per-item promise applied at the granularity an operator thinks in
/// — *"my Review layout came back and my Proofing one did not"* is a
/// sentence they can act on.
pub(crate) fn sanitize_all(
    workspaces: &mut Vec<Workspace>,
    catalog: &dyn PanelCatalog,
    report: &mut LoadReport,
) {
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut kept: Vec<Workspace> = Vec::with_capacity(workspaces.len());

    for mut workspace in std::mem::take(workspaces) {
        if workspace.name.trim().is_empty() {
            report.push(
                LayoutSite::Document,
                LayoutSkipReason::WorkspaceName {
                    name: String::new(),
                },
            );
            continue;
        }
        if !seen.insert(workspace.name.clone()) {
            report.push(
                LayoutSite::Document,
                LayoutSkipReason::WorkspaceName {
                    name: workspace.name.clone(),
                },
            );
            continue;
        }

        let name = workspace.name.clone();
        sanitize(
            &mut workspace.layout,
            catalog,
            &Scope::Workspace(&name),
            report,
        );

        // A workspace sanitized to nothing is dropped rather than kept as
        // an empty arrangement. Restoring it would give the operator a
        // dock with no panels and no explanation; the per-panel skips
        // already recorded say why it went.
        if workspace.layout.panels().next().is_none() {
            report.push(
                LayoutSite::Workspace { name },
                LayoutSkipReason::EmptyContainer,
            );
            continue;
        }

        kept.push(workspace);
    }

    *workspaces = kept;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{
        AnyPanel, Column, PanelId, PanelInfo, PanelRegistry, SideLayout, Stack,
    };

    fn registry(ids: &[&str]) -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for id in ids {
            r.register(PanelInfo::new(*id, *id));
        }
        r
    }

    fn one(panel: &str) -> DockLayout {
        DockLayout::new(SideLayout::single(panel), SideLayout::none())
    }

    /// Save, load, list, delete — the four operations, in one pass.
    #[test]
    fn the_four_operations_do_what_they_say() {
        let mut doc = LayoutDocument::new(one("pages"));
        assert!(doc.workspace_names().is_empty());

        assert!(!doc.save_workspace("Reading", one("pages")));
        assert!(!doc.save_workspace("Marking up", one("layers")));
        assert_eq!(doc.workspace_names(), vec!["Reading", "Marking up"]);

        assert!(
            doc.workspace("Reading")
                .expect("saved")
                .contains(&PanelId::new("pages"))
        );
        assert!(doc.workspace("Nothing").is_none(), "no nearest match");

        assert!(doc.delete_workspace("Reading"));
        assert!(
            !doc.delete_workspace("Reading"),
            "a second delete is not an error"
        );
        assert_eq!(doc.workspace_names(), vec!["Marking up"]);
    }

    /// ★ **Saving over a name replaces it in place**, so the operator's
    /// own menu does not reorder itself every time they use it.
    #[test]
    fn saving_over_a_name_keeps_its_position_in_the_list() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.save_workspace("A", one("pages"));
        doc.save_workspace("B", one("layers"));
        doc.save_workspace("C", one("tools"));

        assert!(
            doc.save_workspace("A", one("objects")),
            "reported as a replace"
        );
        assert_eq!(doc.workspace_names(), vec!["A", "B", "C"], "order held");
        assert!(
            doc.workspace("A")
                .expect("still there")
                .contains(&PanelId::new("objects"))
        );
    }

    /// A saved arrangement is normalized on the way in, so a workspace
    /// cannot carry a defect the live layout would have repaired.
    #[test]
    fn a_saved_workspace_is_normalized_on_the_way_in() {
        let mut doc = LayoutDocument::new(one("pages"));
        let messy = DockLayout::new(
            SideLayout::new([Column::new([Stack::new("a"), Stack::new("a")])]),
            SideLayout::none(),
        );
        doc.save_workspace("Messy", messy);
        let saved = doc.workspace("Messy").expect("saved");
        assert!(saved.is_normalized());
        assert_eq!(saved.panels().count(), 1);
    }

    /// A rename refuses a collision and an empty name, rather than
    /// absorbing an arrangement the operator did not mention.
    #[test]
    fn a_rename_refuses_a_collision_and_an_empty_name() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.save_workspace("A", one("pages"));
        doc.save_workspace("B", one("layers"));
        assert!(!doc.rename_workspace("A", "B"), "would have destroyed B");
        assert!(!doc.rename_workspace("A", "   "));
        assert!(!doc.rename_workspace("missing", "C"));
        assert!(doc.rename_workspace("A", "Reading"));
        assert_eq!(doc.workspace_names(), vec!["Reading", "B"]);
    }

    /// Workspaces survive a round trip through text with the live
    /// arrangement.
    #[test]
    fn workspaces_round_trip_with_the_document() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.save_workspace("Reading", one("pages"));
        doc.save_workspace("Marking up", one("layers"));
        let text = doc.to_ron_pretty().expect("serializes");
        let loaded = LayoutDocument::from_ron(&text, &one("pages"), &AnyPanel);
        assert!(loaded.report.is_empty(), "{:?}", loaded.report.skips());
        assert_eq!(loaded.document, doc);
    }

    /// ★ **One bad workspace does not cost the others.**
    ///
    /// The per-item promise at the granularity an operator thinks in.
    /// Here: an unnamed one, a duplicate one, one whose only panel this
    /// build does not offer — and two good ones that come back intact.
    #[test]
    fn one_bad_workspace_does_not_cost_the_others() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.workspaces = vec![
            Workspace::new("Reading", one("pages")),
            Workspace::new("", one("pages")),
            Workspace::new("Reading", one("layers")),
            Workspace::new("Signing", one("signatures")),
            Workspace::new("Marking up", one("layers")),
        ];
        let text = doc.to_ron_pretty().expect("serializes");
        let loaded =
            LayoutDocument::from_ron(&text, &one("pages"), &registry(&["pages", "layers"]));

        assert_eq!(
            loaded.document.workspace_names(),
            vec!["Reading", "Marking up"],
            "the good ones survived and kept their order"
        );
        assert!(
            loaded
                .document
                .workspace("Reading")
                .expect("kept")
                .contains(&PanelId::new("pages")),
            "the FIRST `Reading` won, not the duplicate"
        );

        let reasons = loaded.report.skips();
        assert!(reasons.iter().any(|s| matches!(&s.reason,
            LayoutSkipReason::WorkspaceName { name } if name.is_empty())));
        assert!(reasons.iter().any(|s| matches!(&s.reason,
            LayoutSkipReason::WorkspaceName { name } if name == "Reading")));
        assert!(reasons.iter().any(|s| matches!(&s.reason,
            LayoutSkipReason::UnknownPanel { panel } if panel.as_str() == "signatures")));
    }

    /// A workspace that lost every panel is dropped rather than restored
    /// as an empty dock — and the skip names the workspace, so the
    /// operator can connect it to the panel that went.
    #[test]
    fn a_workspace_emptied_by_a_missing_capability_is_dropped_by_name() {
        let mut doc = LayoutDocument::new(one("pages"));
        doc.save_workspace("Signing", one("signatures"));
        let text = doc.to_ron_pretty().expect("serializes");
        let loaded = LayoutDocument::from_ron(&text, &one("pages"), &registry(&["pages"]));
        assert!(loaded.document.workspaces.is_empty());
        assert!(
            loaded.report.skips().iter().any(|s| matches!(
                &s.site,
                LayoutSite::Workspace { name } if name == "Signing"
            )),
            "the skip does not name the workspace: {:?}",
            loaded.report.skips()
        );
    }

    /// ★ **The shell ships no workspace names of its own.**
    ///
    /// `MODES_AND_PANELS.md` makes a mode *a named workspace*, and
    /// `SHELL_FRAMEWORK.md` makes Read/Review/Edit a configuration rather
    /// than a built-in. A default store with three magic names would
    /// quietly weld one application's modes into the framework, which is
    /// the exact class of coupling the purity gate exists to prevent —
    /// and that gate greps for the application crates' names, so it would
    /// not catch this one.
    #[test]
    fn a_fresh_document_ships_no_workspaces_at_all() {
        assert!(LayoutDocument::default().workspaces.is_empty());
        assert!(
            LayoutDocument::new(one("pages"))
                .workspace_names()
                .is_empty()
        );
    }

    // -----------------------------------------------------------------
    // known_panels — telling "closed on purpose" from "did not exist yet"
    // -----------------------------------------------------------------

    fn ids(names: &[&str]) -> Vec<PanelId> {
        names.iter().map(|n| PanelId::new(*n)).collect()
    }

    /// ★ **`Unknown` and `New(vec![])` are different answers.**
    ///
    /// The single most important property here, because collapsing them is
    /// the tempting simplification and it is the one that reintroduces a
    /// worse bug than the one this fixes. `New(vec![])` says *every
    /// registered panel was already known* — act on nothing. `Unknown` says
    /// *this file predates the record* — there is no evidence, decide.
    ///
    /// If `Unknown` were represented as an empty list, every workspace
    /// written before this field existed would report "nothing is new",
    /// and the upgrade case this whole mechanism exists for would silently
    /// do nothing. If instead it were represented as "everything is new",
    /// every panel the operator had deliberately closed would spring back
    /// open — undoing a decision they actually made.
    #[test]
    fn an_unstamped_workspace_is_unknown_not_empty() {
        let mut doc = LayoutDocument::default();
        doc.save_workspace("mode:read", DockLayout::default());
        assert_eq!(
            doc.unseen_panels("mode:read", &ids(&["pages", "bookmarks"])),
            Unseen::Unknown,
            "a workspace saved without a stamp cannot report what it knew"
        );
        assert_ne!(
            doc.unseen_panels("mode:read", &ids(&["pages"])),
            Unseen::New(Vec::new()),
            "Unknown must not be confused with 'nothing is new'"
        );
    }

    /// A stamped workspace names exactly the panels registered since.
    #[test]
    fn a_stamped_workspace_reports_only_what_it_never_saw() {
        let mut doc = LayoutDocument::default();
        doc.save_workspace("mode:read", DockLayout::default());
        doc.mark_panels_seen("mode:read", &ids(&["bookmarks", "layers"]));

        assert_eq!(
            doc.unseen_panels("mode:read", &ids(&["bookmarks", "layers"])),
            Unseen::New(Vec::new()),
            "nothing registered since the stamp"
        );
        assert_eq!(
            doc.unseen_panels("mode:read", &ids(&["bookmarks", "layers", "pages"])),
            Unseen::New(ids(&["pages"])),
            "a panel registered after the stamp is the one reported"
        );
    }

    /// A panel the operator CLOSED is not reported as new.
    ///
    /// The behaviour the whole design is for, stated as a test rather than
    /// left to follow from the definition: `known_panels` records what
    /// EXISTED, not what was mounted, so a panel that was registered and
    /// deliberately left out of the arrangement is known — and stays out.
    #[test]
    fn a_panel_the_operator_closed_stays_closed() {
        let mut doc = LayoutDocument::default();
        // An arrangement mounting nothing at all: the operator closed
        // every panel. All three were registered when they did it.
        doc.save_workspace("mode:read", DockLayout::default());
        doc.mark_panels_seen("mode:read", &ids(&["bookmarks", "layers", "pages"]));
        assert_eq!(
            doc.unseen_panels("mode:read", &ids(&["bookmarks", "layers", "pages"])),
            Unseen::New(Vec::new()),
            "an empty arrangement must not make every panel look new"
        );
    }

    /// An absent workspace answers `Unknown`, not an empty list.
    #[test]
    fn a_workspace_that_is_not_there_is_unknown() {
        let doc = LayoutDocument::default();
        assert_eq!(
            doc.unseen_panels("mode:nope", &ids(&["pages"])),
            Unseen::Unknown
        );
    }

    /// The stamp is sorted and deduplicated, so a save does not churn the
    /// file.
    ///
    /// A list that reordered itself on every write would produce a diff
    /// every time and make a real change impossible to spot in one.
    #[test]
    fn the_stamp_is_stable_regardless_of_registration_order() {
        let mut a = LayoutDocument::default();
        a.save_workspace("w", DockLayout::default());
        a.mark_panels_seen("w", &ids(&["pages", "bookmarks", "pages"]));

        let mut b = LayoutDocument::default();
        b.save_workspace("w", DockLayout::default());
        b.mark_panels_seen("w", &ids(&["bookmarks", "pages"]));

        assert_eq!(a.workspaces, b.workspaces);
    }

    /// `mark_panels_seen` reports an absent workspace rather than creating
    /// one.
    #[test]
    fn stamping_a_missing_workspace_says_so_and_creates_nothing() {
        let mut doc = LayoutDocument::default();
        assert!(!doc.mark_panels_seen("mode:read", &ids(&["pages"])));
        assert!(doc.workspace_names().is_empty());
    }

    /// An old file with no `known_panels` key still loads, and reports
    /// `Unknown`.
    ///
    /// The compatibility property the `#[serde(default)]` buys, asserted
    /// against real serialized text rather than against a constructed
    /// value — a `Default` impl cannot prove that a file written by an
    /// older build parses.
    #[test]
    fn a_file_written_before_this_field_still_loads() {
        let mut old = LayoutDocument::default();
        old.save_workspace("mode:read", DockLayout::default());
        let text = ron::ser::to_string(&old).expect("serializes");
        assert!(
            !text.contains("known_panels"),
            "an unstamped workspace must not write the key at all: {text}"
        );
        let back: LayoutDocument = ron::from_str(&text).expect("parses");
        assert_eq!(
            back.unseen_panels("mode:read", &ids(&["pages"])),
            Unseen::Unknown
        );
    }
}
