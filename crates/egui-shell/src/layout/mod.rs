//! # `layout` — persistence, named workspaces, and scoped reset
//!
//! `MODES_AND_PANELS.md` Part 2's capability table calls this
//! **"2–3 days — highest value per hour on this list"**, and its
//! recommended build order puts it first with a one-line justification
//! that is the whole argument for this module existing before anything
//! else in the dock is made more flexible:
//!
//! > **(f) persistence** — cheapest, highest value, unblocks everything.
//! > *A rearrangeable layout that forgets itself each restart is worse
//! > than a fixed one.*
//!
//! Worse, not merely less good. A fixed layout costs an operator nothing
//! after the first day; a rearrangeable one that forgets charges them the
//! rearrangement every single session, and teaches them not to bother.
//!
//! ## What is persisted
//!
//! A [`LayoutDocument`]: a schema number, the live arrangement, and any
//! number of named [`Workspace`]s. In RON, beside the manifest, using the
//! same serializer for the same reasons — real enums, comments, trailing
//! commas — all of which matter for a file an operator may open.
//!
//! ```ron
//! LayoutDocument(
//!     schema: 1,
//!     active: DockLayout(
//!         left: SideLayout(
//!             columns: [ Column(
//!                 stacks: [ Stack(tabs: ["pages", "layers"], active: 0, share: 1.0) ],
//!                 share: 1.0,
//!             ) ],
//!             width_pts: 280.0,
//!             visible: true,
//!         ),
//!         right: SideLayout(columns: [], width_pts: 280.0, visible: false),
//!     ),
//!     workspaces: [
//!         Workspace(name: "Review", layout: DockLayout( /* … */ )),
//!     ],
//! )
//! ```
//!
//! Every identifier in that file is either a number, a keyword, or a
//! panel id **the application chose**. There is no generated handle, so
//! there is nothing that can dangle and nothing to remap on load. See
//! [`crate::dock::model`]'s header for why the schema is owned rather
//! than borrowed from a layout engine.
//!
//! ## ★ Fail-soft, per item — the contract, and its exact limits
//!
//! > Missing file, parse failure, unknown panel id, or a panel the
//! > application no longer registers → **that item is dropped with a
//! > structured reason and the rest still loads. Never a dialog, never a
//! > wholesale reset.**
//!
//! Concretely, what each input produces:
//!
//! | Input | Result | Reported as |
//! |---|---|---|
//! | No file | the default arrangement | [`LayoutSkipReason::FileMissing`] — **not** noteworthy; a first run is not a failure |
//! | Unreadable file | the default arrangement | [`LayoutSkipReason::Unreadable`] |
//! | Broken syntax | the default arrangement | [`LayoutSkipReason::ParseFailed`] |
//! | A **newer** schema | the default arrangement | [`LayoutSkipReason::UnsupportedSchema`] |
//! | A missing field | that field's default, everything else kept | nothing — `#[serde(default)]` throughout |
//! | An **unknown** field | ignored, everything else kept | nothing — a newer build's file still loads here |
//! | A panel the application does not register | **that tab** dropped | [`LayoutSkipReason::UnknownPanel`] |
//! | A panel mounted twice | the **second** mount dropped | [`LayoutSkipReason::DuplicatePanel`] |
//! | A compartment left with no tabs | that compartment dropped | [`LayoutSkipReason::EmptyContainer`] |
//! | A share or width that is not a number | replaced with a default | [`LayoutSkipReason::InvalidSize`] |
//! | An active index past the end | the last tab selected | [`LayoutSkipReason::ActiveOutOfRange`] |
//! | One unparseable **workspace** | that workspace dropped, the others kept | [`LayoutSkipReason::WorkspaceName`] / per-item reasons |
//!
//! **The one honest limit**, stated rather than implied away: a file
//! whose *syntax* is broken cannot be partially recovered, because there
//! is no point at which a parser can say which half was meant. That is
//! the single wholesale case, it falls back to the default arrangement
//! rather than to an error, and it is disclosed. Everything past the
//! parser is per item.
//!
//! Two design choices carry most of the weight and are worth naming
//! because neither is obvious:
//!
//! 1. **Every field is `#[serde(default)]` and unknown fields are
//!    ignored.** That means a file written by a *newer* build with an
//!    extra field still loads in an older one, and a file written by an
//!    *older* build with a missing field still loads in a newer one. Only
//!    a deliberate schema bump refuses, and it refuses loudly. Without
//!    this, every field ever added becomes a day on which everybody's
//!    layout resets.
//! 2. **A panel the application does not register is dropped, and that is
//!    the healthy path, not the error path.** `SHELL_FRAMEWORK.md` §5b:
//!    *a capability's presence is expressed by registering it, and by
//!    nothing else.* A build compiled without OCR does not register an
//!    OCR panel, so a saved layout that mounts one loses that tab and
//!    keeps everything else — with no `#[cfg]` anywhere in this crate.
//!
//! ## What this module does **not** do
//!
//! It does not decide *when* to save, and it does not choose a path. An
//! application saves on [`crate::dock::DockFrameReport::layout_changed`]
//! and picks its own location — which for this project means a named
//! partition of the distribution folder rather than a platform app-data
//! directory. The previous implementation records the reasoning:
//! `eframe`'s own persistence *"writes to a platform app-data directory,
//! contradicting decision 003's single-folder-portable posture"*. A
//! shell that chose the path would have made that decision for every
//! application that ever uses it.
//!
//! ## Reset, and why it has scopes
//!
//! See [`ResetScope`]. `RIBBON_IA.md`'s reasoning is short and decisive:
//! *"an operator who only wanted the right dock back must not lose their
//! left one."*

pub mod reset;
pub mod skip;
pub mod workspaces;

use serde::{Deserialize, Serialize};

use crate::dock::model::{DockLayout, DockSide, PanelCatalog, PanelId, SideLayout};

pub use reset::ResetScope;
pub use skip::{LayoutSite, LayoutSkip, LayoutSkipReason, LoadReport};
pub use workspaces::{Unseen, Workspace};

/// Why a layout could not be written.
///
/// Reading never fails — see this module's header — so this is a
/// write-side error only, and it is a real `Result` because a failed save
/// is something the operator must be told about: they are about to close
/// an application believing their arrangement is safe.
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    /// The document could not be rendered as RON.
    #[error("the layout could not be written as text: {0}")]
    Serialize(#[from] ron::Error),
    /// The file could not be written.
    #[error("the layout file could not be written: {0}")]
    Io(#[from] std::io::Error),
}

/// The whole persisted layout state: the live arrangement and every named
/// workspace.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutDocument {
    /// The schema version this document was written against.
    ///
    /// `0` means "unstated", which is what a hand-written file will
    /// usually be, and is treated as the current version. A value
    /// **above** [`Self::SCHEMA`] means the file came from a newer build
    /// and the whole document is skipped with a disclosure rather than
    /// guessed at — a field this build does not understand may be the one
    /// that changes what the rest of the file means. Identical posture,
    /// and identical wording, to [`crate::manifest::Shell::schema`].
    pub schema: u32,
    /// The arrangement in force.
    pub active: DockLayout,
    /// Named arrangements the operator can return to.
    pub workspaces: Vec<Workspace>,
    /// ★★ **The mode in force when this was last written**, if the host has
    /// modes at all.
    ///
    /// # Why it lives here and not in the host's settings
    ///
    /// Because it is the same fact as the rest of this file. A mode *is* an
    /// arrangement — [`Self::active`] holds the layout of whichever mode was on
    /// screen, and without this field the next launch restores that layout
    /// under a **different** mode's name. The two halves of one answer were
    /// being stored in different places, and only one of them was being stored.
    ///
    /// # Why `Option<String>` and not a required field
    ///
    /// Three states, all real. `None` from a file written before this field
    /// existed, `None` from a host with no modes, and `Some` from a host that
    /// has them. All three mean *"start in whatever the host considers first"*
    /// except the last, and a host that reads this must treat an id it does not
    /// recognise the same way — a manifest can be edited between runs, and a
    /// mode that has been renamed away must not leave the shell with no mode.
    ///
    /// ★ The shell learns nothing about what a mode *means* by holding this.
    /// It is an opaque id the host wrote and the host reads back, which is what
    /// keeps R7 true: `egui-shell` has modes, and knows nothing about reading,
    /// reviewing or editing anything.
    pub active_mode: Option<String>,
}

/// A load's result: the document, and everything that had to be skipped.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    /// The document, already sanitized and safe to draw.
    pub document: LayoutDocument,
    /// What could not be carried across. **Deal with this** — see
    /// [`LoadReport::is_noteworthy`].
    pub report: LoadReport,
}

impl LayoutDocument {
    /// The newest schema this build understands.
    pub const SCHEMA: u32 = 1;

    /// A document holding one arrangement and no workspaces.
    #[must_use]
    pub fn new(active: DockLayout) -> Self {
        Self {
            schema: Self::SCHEMA,
            active,
            workspaces: Vec::new(),
            active_mode: None,
        }
    }

    /// Render as RON, one field per line, for a file a person may read.
    ///
    /// # Errors
    ///
    /// [`LayoutError::Serialize`] if the document cannot be rendered,
    /// which in practice means a non-finite `f32` reached the writer —
    /// [`crate::dock::DockLayout::normalize`] removes those, and
    /// `a_normalized_layout_always_serializes` is the test that says the
    /// two agree.
    pub fn to_ron_pretty(&self) -> Result<String, LayoutError> {
        Ok(ron::ser::to_string_pretty(
            self,
            ron::ser::PrettyConfig::default(),
        )?)
    }

    /// Write to a path, creating parent directories as needed.
    ///
    /// # Errors
    ///
    /// [`LayoutError::Serialize`] or [`LayoutError::Io`].
    pub fn save_to_path(&self, path: impl AsRef<std::path::Path>) -> Result<(), LayoutError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_ron_pretty()?)?;
        Ok(())
    }

    /// Parse a document from text, dropping what it cannot use.
    ///
    /// **Never fails.** `fallback` is the application's built-in default
    /// arrangement, used when the text cannot be parsed at all; pass the
    /// same value the application would use on a fresh profile.
    ///
    /// `catalog` is what makes a stale panel id detectable. Pass
    /// [`crate::dock::AnyPanel`] only in tooling that has no registry — in
    /// an application it would disable the check that turns a mount for a
    /// compiled-out capability into a disclosed skip rather than an empty
    /// compartment.
    #[must_use]
    pub fn from_ron(text: &str, fallback: &DockLayout, catalog: &dyn PanelCatalog) -> Loaded {
        let mut report = LoadReport::default();
        let mut document = match ron::from_str::<LayoutDocument>(text) {
            Ok(d) => d,
            Err(e) => {
                report.push(
                    LayoutSite::Document,
                    LayoutSkipReason::ParseFailed {
                        detail: e.to_string(),
                    },
                );
                LayoutDocument::new(fallback.clone())
            }
        };

        if document.schema > Self::SCHEMA {
            report.push(
                LayoutSite::Document,
                LayoutSkipReason::UnsupportedSchema {
                    found: document.schema,
                    supported: Self::SCHEMA,
                },
            );
            document = LayoutDocument::new(fallback.clone());
        }
        document.schema = Self::SCHEMA;

        sanitize(&mut document.active, catalog, &Scope::Active, &mut report);

        // A sanitized-to-nothing active arrangement falls back, because a
        // dock with no panels at all is indistinguishable from a broken
        // application. Every individual drop has already been disclosed,
        // so this adds no new reason — it is the consequence of the ones
        // already reported.
        if document.active.panels().next().is_none() {
            document.active = fallback.clone();
            sanitize(&mut document.active, catalog, &Scope::Active, &mut report);
        }

        workspaces::sanitize_all(&mut document.workspaces, catalog, &mut report);
        Loaded { document, report }
    }

    /// Read a document from a path, dropping what it cannot use.
    ///
    /// **Never fails.** A missing file yields the fallback and a
    /// [`LayoutSkipReason::FileMissing`], which
    /// [`LoadReport::is_noteworthy`] deliberately does not count as worth
    /// telling anybody about: a first run is not a failure.
    #[must_use]
    pub fn load_from_path(
        path: impl AsRef<std::path::Path>,
        fallback: &DockLayout,
        catalog: &dyn PanelCatalog,
    ) -> Loaded {
        let mut report = LoadReport::default();
        match std::fs::read_to_string(path.as_ref()) {
            Ok(text) => Self::from_ron(&text, fallback, catalog),
            Err(e) => {
                let reason = if e.kind() == std::io::ErrorKind::NotFound {
                    LayoutSkipReason::FileMissing
                } else {
                    LayoutSkipReason::Unreadable {
                        detail: e.to_string(),
                    }
                };
                report.push(LayoutSite::Document, reason);
                let mut document = LayoutDocument::new(fallback.clone());
                sanitize(&mut document.active, catalog, &Scope::Active, &mut report);
                Loaded { document, report }
            }
        }
    }
}

/// Which part of a document a sanitization pass is walking.
///
/// A layout inside a named workspace reports its problems against the
/// **workspace**, not against a column index, because *"workspace
/// `Review`: `signatures` is not a panel this build offers"* is what an
/// operator can act on, whereas *"the left dock, column 0, compartment 1"*
/// is ambiguous between the live arrangement and four saved ones.
pub(crate) enum Scope<'a> {
    /// The live arrangement.
    Active,
    /// One named workspace.
    Workspace(&'a str),
}

impl Scope<'_> {
    /// The site to report a whole-side problem against.
    fn side(&self, side: DockSide) -> LayoutSite {
        match self {
            Scope::Active => LayoutSite::Side { side },
            Scope::Workspace(name) => LayoutSite::Workspace {
                name: (*name).to_owned(),
            },
        }
    }

    /// The site to report a per-stack problem against.
    fn stack(&self, side: DockSide, column: usize, index: usize) -> LayoutSite {
        match self {
            Scope::Active => LayoutSite::Stack {
                side,
                column,
                index,
            },
            Scope::Workspace(name) => LayoutSite::Workspace {
                name: (*name).to_owned(),
            },
        }
    }

    /// The site to report a per-tab problem against.
    fn tab(&self, side: DockSide, column: usize, stack: usize, panel: &PanelId) -> LayoutSite {
        match self {
            Scope::Active => LayoutSite::Tab {
                side,
                column,
                stack,
                panel: panel.clone(),
            },
            Scope::Workspace(name) => LayoutSite::Workspace {
                name: (*name).to_owned(),
            },
        }
    }
}

/// Walk one arrangement, repairing every invariant and reporting each
/// repair.
///
/// This is [`crate::dock::DockLayout::normalize`] with a voice. The two
/// must agree — a load that repaired something `normalize` would not, or
/// vice versa, would mean an arrangement that changes shape between being
/// loaded and being drawn — and
/// `sanitizing_leaves_nothing_for_normalize_to_do` is the test that holds
/// them together.
pub(crate) fn sanitize(
    layout: &mut DockLayout,
    catalog: &dyn PanelCatalog,
    scope: &Scope<'_>,
    report: &mut LoadReport,
) {
    let mut seen: std::collections::BTreeSet<PanelId> = std::collections::BTreeSet::new();

    for side in DockSide::ALL {
        let s = layout.side_mut(side);

        if !s.width_pts.is_finite() || s.width_pts <= 0.0 {
            report.push(
                scope.side(side),
                LayoutSkipReason::InvalidSize { value: s.width_pts },
            );
            s.width_pts = SideLayout::default().width_pts;
        }

        for ci in 0..s.columns.len() {
            let share = s.columns[ci].share;
            if !share.is_finite() || share <= 0.0 {
                report.push(
                    scope.side(side),
                    LayoutSkipReason::InvalidSize { value: share },
                );
                s.columns[ci].share = 1.0;
            }

            for si in 0..s.columns[ci].stacks.len() {
                let share = s.columns[ci].stacks[si].share;
                if !share.is_finite() || share <= 0.0 {
                    report.push(
                        scope.stack(side, ci, si),
                        LayoutSkipReason::InvalidSize { value: share },
                    );
                    s.columns[ci].stacks[si].share = 1.0;
                }

                // Which panel was selected, BEFORE anything is dropped.
                // Keeping the identity rather than the index is what lets
                // the selection survive a tab being removed from in front
                // of it — dropping one unregistered panel must not
                // silently change which panel the operator was looking
                // at.
                let stack = &mut s.columns[ci].stacks[si];
                let was_active = stack.active;
                let selected = stack.tabs.get(was_active).cloned();
                let count = stack.tabs.len();

                let mut kept: Vec<PanelId> = Vec::with_capacity(count);
                for panel in std::mem::take(&mut stack.tabs) {
                    if !catalog.contains(panel.as_str()) {
                        report.push(
                            scope.tab(side, ci, si, &panel),
                            LayoutSkipReason::UnknownPanel {
                                panel: panel.clone(),
                            },
                        );
                        continue;
                    }
                    if !seen.insert(panel.clone()) {
                        report.push(
                            scope.tab(side, ci, si, &panel),
                            LayoutSkipReason::DuplicatePanel {
                                panel: panel.clone(),
                            },
                        );
                        continue;
                    }
                    kept.push(panel);
                }

                if was_active >= count && count > 0 {
                    report.push(
                        scope.stack(side, ci, si),
                        LayoutSkipReason::ActiveOutOfRange {
                            was: was_active,
                            len: count,
                        },
                    );
                }

                stack.active = selected
                    .and_then(|p| kept.iter().position(|k| *k == p))
                    .unwrap_or_else(|| was_active.min(kept.len().saturating_sub(1)));
                stack.tabs = kept;
            }

            let before = s.columns[ci].stacks.len();
            s.columns[ci].stacks.retain(|st| !st.tabs.is_empty());
            for _ in s.columns[ci].stacks.len()..before {
                report.push(scope.side(side), LayoutSkipReason::EmptyContainer);
            }
        }

        let before = s.columns.len();
        s.columns.retain(|c| !c.stacks.is_empty());
        for _ in s.columns.len()..before {
            report.push(scope.side(side), LayoutSkipReason::EmptyContainer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{AnyPanel, Column, PanelInfo, PanelRegistry, Stack};

    /// A registry holding exactly the panels this build "offers".
    fn registry(ids: &[&str]) -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for id in ids {
            r.register(PanelInfo::new(*id, *id));
        }
        r
    }

    fn fallback() -> DockLayout {
        DockLayout::new(SideLayout::single("pages"), SideLayout::none())
    }

    fn rich() -> DockLayout {
        DockLayout::new(
            SideLayout::new([
                Column::new([Stack::new("pages"), Stack::tabbed(["layers", "bookmarks"])]),
                Column::new([Stack::new("tools")]),
            ])
            .with_width(320.0),
            SideLayout::single("objects").with_width(240.0),
        )
    }

    /// ★ **A layout round-trips through text unchanged.**
    ///
    /// The property everything else in this module depends on. Asserted
    /// against a *rich* arrangement — two columns, a tabbed stack, a
    /// non-default width, both sides — because a round trip over a
    /// one-panel layout is satisfied by a serializer that drops almost
    /// everything.
    #[test]
    fn a_rich_layout_round_trips_through_text_unchanged() {
        let document = LayoutDocument::new(rich());
        let text = document.to_ron_pretty().expect("serializes");
        let loaded = LayoutDocument::from_ron(&text, &fallback(), &AnyPanel);
        assert!(loaded.report.is_empty(), "{:?}", loaded.report.skips());
        assert_eq!(loaded.document, document);
    }

    /// ★ **A missing file is a first run, not a failure.**
    #[test]
    fn a_missing_file_yields_the_default_and_says_so_quietly() {
        // temp-path-exempt: nothing is ever created here. The test wants a
        // path that does NOT exist, and every process wanting the same
        // non-existent path is not a collision.
        let path = std::env::temp_dir().join("egui-shell-no-such-layout-file.ron");
        let _ = std::fs::remove_file(&path);
        let loaded = LayoutDocument::load_from_path(&path, &fallback(), &AnyPanel);
        assert_eq!(loaded.document.active, fallback());
        assert_eq!(loaded.report.len(), 1);
        assert!(!loaded.report.is_noteworthy(), "a first run is not news");
        assert!(matches!(
            loaded.report.skips()[0].reason,
            LayoutSkipReason::FileMissing
        ));
    }

    /// ★ **Broken syntax falls back and discloses — never a dialog,
    /// never silence.**
    #[test]
    fn broken_syntax_falls_back_with_a_reason_that_names_the_position() {
        let loaded = LayoutDocument::from_ron("LayoutDocument( schema: ", &fallback(), &AnyPanel);
        assert_eq!(loaded.document.active, fallback());
        assert!(loaded.report.is_noteworthy());
        let text = loaded.report.skips()[0].to_string();
        assert!(text.contains("not readable"), "{text}");
    }

    /// A file from a newer build is refused as a whole, loudly, rather
    /// than half-applied.
    #[test]
    fn a_newer_schema_is_refused_rather_than_guessed_at() {
        let text = "LayoutDocument(schema: 99)";
        let loaded = LayoutDocument::from_ron(text, &fallback(), &AnyPanel);
        assert_eq!(loaded.document.active, fallback());
        assert!(matches!(
            loaded.report.skips()[0].reason,
            LayoutSkipReason::UnsupportedSchema {
                found: 99,
                supported: 1
            }
        ));
    }

    /// ★ **A missing field is a default, and an unknown field is
    /// ignored** — the two halves of surviving a version change in either
    /// direction.
    ///
    /// Without the first, every field ever added is a day on which
    /// everybody's layout resets. Without the second, a file touched by a
    /// newer build is unreadable by the one the operator rolled back to.
    #[test]
    fn a_file_from_another_version_still_loads() {
        // No `schema`, no `workspaces`, no `share` on the stack, and a
        // field this build has never heard of.
        let text = r#"LayoutDocument(
            active: (
                left: (
                    columns: [ ( stacks: [ ( tabs: ["pages"] ) ] ) ],
                    lighting: "warm",
                ),
            ),
        )"#;
        let loaded = LayoutDocument::from_ron(text, &fallback(), &AnyPanel);
        assert!(
            loaded.document.active.contains(&PanelId::new("pages")),
            "{:?}",
            loaded.report.skips()
        );
        assert!(loaded.report.is_empty(), "{:?}", loaded.report.skips());
    }

    /// ★ **A panel this build does not offer loses its tab and nothing
    /// else.**
    ///
    /// The `SHELL_FRAMEWORK.md` §5b case: a capability compiled out
    /// registers no panel, so its saved mount is dropped — with the
    /// operator's arrangement of everything else intact, and with no
    /// `#[cfg]` anywhere in this crate.
    #[test]
    fn a_panel_this_build_does_not_offer_loses_its_tab_and_nothing_else() {
        let document = LayoutDocument::new(rich());
        let text = document.to_ron_pretty().expect("serializes");
        let catalog = registry(&["pages", "layers", "tools", "objects"]); // no `bookmarks`
        let loaded = LayoutDocument::from_ron(&text, &fallback(), &catalog);

        assert!(!loaded.document.active.contains(&PanelId::new("bookmarks")));
        for kept in ["pages", "layers", "tools", "objects"] {
            assert!(
                loaded.document.active.contains(&PanelId::new(kept)),
                "{kept} was lost along with the unregistered panel"
            );
        }
        assert_eq!(loaded.report.len(), 1);
        assert!(matches!(
            &loaded.report.skips()[0].reason,
            LayoutSkipReason::UnknownPanel { panel } if panel.as_str() == "bookmarks"
        ));
        // And the arrangement is otherwise identical — the columns, the
        // widths and the second side all survived.
        assert_eq!(loaded.document.active.left.width_pts, 320.0);
        assert_eq!(loaded.document.active.left.columns.len(), 2);
    }

    /// Losing the only panel of a compartment prunes the compartment, and
    /// says so as a second, connectable fact.
    #[test]
    fn a_compartment_emptied_by_a_dropped_panel_is_pruned_and_reported() {
        let document = LayoutDocument::new(rich());
        let text = document.to_ron_pretty().expect("serializes");
        let catalog = registry(&["pages", "layers", "bookmarks", "objects"]); // no `tools`
        let loaded = LayoutDocument::from_ron(&text, &fallback(), &catalog);
        assert_eq!(
            loaded.document.active.left.columns.len(),
            1,
            "the column holding only `tools` should be gone"
        );
        assert!(
            loaded
                .report
                .skips()
                .iter()
                .any(|s| matches!(s.reason, LayoutSkipReason::EmptyContainer)),
            "the pruning was not disclosed: {:?}",
            loaded.report.skips()
        );
    }

    /// Dropping a tab from in front of the selected one does not change
    /// which panel is selected — the selection follows the **panel**, not
    /// the index.
    #[test]
    fn a_dropped_tab_does_not_move_the_selection_to_a_different_panel() {
        let mut layout = DockLayout::new(
            SideLayout::new([Column::new([Stack::tabbed(["a", "b", "c"])])]),
            SideLayout::none(),
        );
        layout.activate(&PanelId::new("c"));
        let text = LayoutDocument::new(layout)
            .to_ron_pretty()
            .expect("serializes");
        let loaded = LayoutDocument::from_ron(&text, &fallback(), &registry(&["a", "c"]));
        assert!(
            loaded.document.active.is_active(&PanelId::new("c")),
            "the selection moved when `b` was dropped"
        );
    }

    /// A panel mounted twice keeps its first mount and discloses the
    /// second.
    #[test]
    fn a_panel_mounted_twice_is_reported_and_deduplicated() {
        let text = r#"LayoutDocument(active: (
            left:  (columns: [(stacks: [(tabs: ["pages"])])]),
            right: (columns: [(stacks: [(tabs: ["pages"])])]),
        ))"#;
        let loaded = LayoutDocument::from_ron(text, &fallback(), &AnyPanel);
        assert_eq!(loaded.document.active.panels().count(), 1);
        assert!(
            loaded
                .document
                .active
                .left
                .panels()
                .any(|p| p.as_str() == "pages")
        );
        assert!(loaded.report.skips().iter().any(
            |s| matches!(&s.reason, LayoutSkipReason::DuplicatePanel { panel }
                                  if panel.as_str() == "pages")
        ));
    }

    /// An out-of-range active index selects the last tab and says so.
    #[test]
    fn an_out_of_range_active_index_is_reported_and_clamped() {
        let text = r#"LayoutDocument(active: (left: (columns: [(stacks: [
            (tabs: ["a", "b"], active: 9)
        ])])))"#;
        let loaded = LayoutDocument::from_ron(text, &fallback(), &AnyPanel);
        assert!(loaded.document.active.is_active(&PanelId::new("b")));
        assert!(matches!(
            loaded.report.skips()[0].reason,
            LayoutSkipReason::ActiveOutOfRange { was: 9, len: 2 }
        ));
    }

    /// A share or width that is not a usable number is replaced and
    /// disclosed, rather than producing a compartment nobody can see.
    #[test]
    fn an_unusable_size_is_replaced_and_reported() {
        let text = r#"LayoutDocument(active: (left: (
            columns: [(stacks: [(tabs: ["a"], share: -3.0)], share: 0.0)],
            width_pts: -1.0,
        )))"#;
        let loaded = LayoutDocument::from_ron(text, &fallback(), &AnyPanel);
        assert!(loaded.document.active.left.width_pts > 0.0);
        assert!(loaded.document.active.left.columns[0].share > 0.0);
        assert!(loaded.document.active.left.columns[0].stacks[0].share > 0.0);
        assert_eq!(
            loaded
                .report
                .skips()
                .iter()
                .filter(|s| matches!(s.reason, LayoutSkipReason::InvalidSize { .. }))
                .count(),
            3
        );
    }

    /// ★ **An arrangement sanitized to nothing falls back rather than
    /// leaving an empty dock.**
    ///
    /// A build with none of the saved panels — every capability compiled
    /// out, or an application that renamed all of its ids — must not
    /// present a dock with nothing in it, which is indistinguishable from
    /// a broken application. Every individual drop has already been
    /// disclosed, so the fallback adds no new reason.
    #[test]
    fn an_arrangement_left_with_nothing_falls_back_to_the_default() {
        let document = LayoutDocument::new(rich());
        let text = document.to_ron_pretty().expect("serializes");
        let loaded = LayoutDocument::from_ron(&text, &fallback(), &registry(&["pages"]));
        assert!(loaded.document.active.contains(&PanelId::new("pages")));
        assert!(loaded.report.is_noteworthy());
    }

    /// ★ **Sanitizing and normalizing agree.**
    ///
    /// They are two implementations of one set of invariants — one with a
    /// voice, one without — and an arrangement that changed shape between
    /// being loaded and being drawn would be a defect no test of either
    /// alone could find.
    #[test]
    fn sanitizing_leaves_nothing_for_normalize_to_do() {
        let text = r#"LayoutDocument(active: (
            left: (columns: [
                (stacks: [(tabs: ["a", "ghost", "a"], active: 5), (tabs: [])], share: 0.0),
                (stacks: []),
            ], width_pts: 0.0),
            right: (columns: [(stacks: [(tabs: ["b"])])]),
        ))"#;
        let loaded = LayoutDocument::from_ron(text, &fallback(), &registry(&["a", "b"]));
        assert!(
            loaded.document.active.is_normalized(),
            "sanitize left work for normalize: {:?}",
            loaded.document.active
        );
    }

    /// A normalized arrangement always serializes, so a save cannot fail
    /// for a reason the model was supposed to have removed.
    #[test]
    fn a_normalized_layout_always_serializes() {
        let mut layout = rich();
        layout.left.width_pts = f32::NAN;
        layout.normalize();
        assert!(LayoutDocument::new(layout).to_ron_pretty().is_ok());
    }

    /// A save followed by a load reproduces the document, through a real
    /// file.
    #[test]
    fn a_document_survives_a_trip_through_a_file() {
        // The pid keeps two concurrent `cargo test` processes out of each
        // other's scratch directory; see `tools/gates/check-test-temp-paths.py`.
        let dir =
            std::env::temp_dir().join(format!("egui-shell-layout-tests-{}", std::process::id()));
        let path = dir.join("round-trip.ron");
        let _ = std::fs::remove_file(&path);
        let document = LayoutDocument::new(rich());
        document.save_to_path(&path).expect("writes");
        let loaded = LayoutDocument::load_from_path(&path, &fallback(), &AnyPanel);
        assert_eq!(loaded.document, document);
        assert!(loaded.report.is_empty());
        let _ = std::fs::remove_file(&path);
    }
}
