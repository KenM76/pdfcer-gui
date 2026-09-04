//! # `app::modes` — Read / Review / Edit as named workspaces
//!
//! `MODES_AND_PANELS.md` closes its analysis by identifying the two
//! requests that arrived together — a three-position mode selector, and
//! flexible panel areas — as one system, in one sentence:
//!
//! > **A mode is capability (g).** Read, Review and Edit are three built-in
//! > named workspaces, shipped as defaults, each remembering the operator's
//! > arrangement of it.
//!
//! This module is that sentence, implemented. It binds each mode the
//! **manifest** declares to a named workspace in the layout document that
//! [`crate::app::persistence`] keeps on disk, so that:
//!
//! - each mode starts from a default arrangement suited to what that mode
//!   is *for*;
//! - the operator's own rearrangement of a mode is remembered, per mode;
//! - Read → Edit → Read restores **your Edit**, not a default.
//!
//! ## ★ Three modes are configuration, not a built-in — on both sides
//!
//! `egui-shell`'s workspace store ships **no names at all**, and says why:
//! *"an application that wants three modes registers three workspaces; one
//! that wants eleven registers eleven; one that wants none never calls this
//! module."* `SHELL_FRAMEWORK.md` §4 states the same rule from the
//! manifest's side — *Read/Review/Edit is a configuration, not a built-in*.
//!
//! That rule binds **here** too, and this module honours it in the one way
//! that matters: [`Modes`] holds whatever mode ids
//! `crate::shell::manifest::built_in` declares, in that order, and has no
//! opinion about how many there are or what they are called. Adding a
//! fourth mode to the manifest is one line in the manifest; nothing here
//! changes, and nothing here needs to.
//!
//! ## Why the arrangements themselves live in [`defaults`]
//!
//! `app/modes.rs` was one file until it reached 1,512 lines against the
//! 1,500-line gate (R2). It was split rather than trimmed, for the reason
//! the gate itself gives — *"the right response to this gate firing is to
//! SPLIT THE MODULE, not to shrink the prose"* — and along the seam the file
//! had already drawn between its own two halves. `app/mod.rs` has been split
//! twice under the same rule, into [`crate::app::dispatch`] and
//! [`crate::app::conditions`], and the pattern is deliberately the same one.
//!
//! The two halves answer two different questions:
//!
//! * [`defaults`] answers **what a mode's arrangement *is***. Which panels
//!   Read mounts, on which side, how wide. It is pure: a mode id in, a
//!   [`DockLayout`] out, with no file, no dock and no document anywhere in
//!   its argument lists. It changes when the *information architecture*
//!   changes.
//! * **This file** answers **how an arrangement is *remembered***. Workspace
//!   naming, the adopt-on-mode-change sequence, the debounced write, the
//!   reconciliation that stops a newly shipped panel being born invisible,
//!   and the start-up order. It changes when *persistence* changes.
//!
//! That is a seam and not arithmetic: the last change to [`defaults`] was a
//! taxonomy answer from the operator (Read fills forms), the last change to
//! this file was an upgrade-path mechanism (`Unseen`), and neither would have
//! needed to touch the other. The dependency runs one way — this file calls
//! [`layout_for_build`]; [`defaults`] calls nothing here — which is what makes
//! the split honest rather than a pair of files that both have to be open.
//!
//! [`layout_for`], [`layout_for_build`] and [`ABSENT_PANELS`] are re-exported
//! below, so every path a caller used before the split still resolves.
//!
//! ## What a mode change must **not** do
//!
//! `MODES_AND_PANELS.md` Part 1's behavioural rules, and the first two are
//! the ones this module is accountable for:
//!
//! > 1. **Switching modes never destroys work.** Read ⇄ Edit is a view
//! >    stance, not a save boundary. Unsaved edits survive a trip through
//! >    Read mode untouched.
//! > 2. **The undo stack is not cleared, ever.**
//!
//! That is enforced **structurally** rather than by care:
//! [`Modes::on_mode_changed`] takes a [`DockState`] and a
//! [`LayoutStore`], and neither of them can reach a document, a selection,
//! an edit session or an undo stack. There is no path from this module to
//! any of them, and `switching_modes_touches_neither_the_document_nor_the_selection`
//! asserts the consequence against a real open document anyway — because a
//! later edit that *adds* such a path should fail a test rather than pass a
//! review.
//!
//! ## What this module deliberately does not do
//!
//! - **It does not change which tabs the ribbon shows.** That is the
//!   manifest's `Mode::tabs` and `egui-shell`'s renderer; a mode's tab set
//!   and a mode's panel arrangement are two different things that happen to
//!   share a name.
//! - **It does not set the page-display default.** `MODES_AND_PANELS.md`
//!   Part 1: *"Read defaults to continuous scroll; Review and Edit default
//!   to single page."* That is a `crate::viewer` concern — there is no
//!   continuous-scroll display mode in this build yet — and it is recorded
//!   here only so the next reader knows it is a known, deliberate omission
//!   rather than a missed row of the table.
//! - **It does not decide the default mode.** Part 1 rule 5 makes that a
//!   setting; today `crate::app::PdfcerApp::new` starts in the manifest's
//!   first mode and [`start`] adopts whatever it is handed.

pub mod capability;
pub mod defaults;

pub use capability::Capabilities;
pub use defaults::{ABSENT_PANELS, layout_for, layout_for_build};

use egui_shell::dock::{Column, DockLayout, DockSide, DockState, PanelCatalog, PanelId, Stack};
use egui_shell::layout::{ResetScope, Unseen};
use egui_shell::manifest::Shell;

use crate::app::persistence::LayoutStore;
use crate::panels::Panel;

/// The prefix every mode-owned workspace name carries.
///
/// A workspace name is free text an operator chooses, so a mode's own
/// workspace has to be distinguishable from one the operator made and
/// happened to call "Read". The prefix does that, and it does two more
/// things worth having:
///
/// - it is the **mode id**, not the label, so renaming or translating
///   "Review" does not orphan the arrangement behind it;
/// - it makes the machine-owned entries filterable, so a future "load
///   workspace" menu can list the operator's own and leave these out — see
///   [`mode_of_workspace`].
pub const MODE_WORKSPACE_PREFIX: &str = "mode:"; // ui-text-exempt: a key prefix inside a file, never displayed

/// The workspace name that holds `mode_id`'s remembered arrangement.
#[must_use]
pub fn workspace_name(mode_id: &str) -> String {
    format!("{MODE_WORKSPACE_PREFIX}{mode_id}")
}

/// The mode a workspace belongs to, if it is a mode's rather than an
/// operator's.
#[must_use]
pub fn mode_of_workspace(name: &str) -> Option<&str> {
    name.strip_prefix(MODE_WORKSPACE_PREFIX)
}

/// The modes this application has, and which one is in force.
///
/// The ids come from the **manifest**, in the order it declares them, and
/// this type has no opinion about how many there are or what they are
/// called — see the module header. What it owns is the binding between a
/// mode and its remembered arrangement.
#[derive(Debug, Default, Clone)]
pub struct Modes {
    /// Every mode id the manifest declares, in declaration order.
    ids: Vec<String>,
    /// The mode in force, or `None` before the first adoption.
    ///
    /// `None` is meaningful rather than a placeholder: it is what makes the
    /// first [`Self::on_mode_changed`] an *adoption* — there is no outgoing
    /// arrangement to remember, because nothing has been arranged yet.
    active: Option<String>,
}

impl Modes {
    /// Take the mode list from a shell manifest.
    ///
    /// `None` — a manifest that failed to validate — yields no modes, and
    /// every method below then declines rather than inventing one. A build
    /// whose ribbon could not be assembled must not silently acquire a
    /// three-position mode model from somewhere else.
    #[must_use]
    pub fn from_shell(shell: Option<&Shell>) -> Self {
        Self {
            ids: shell
                .map(|s| s.modes().iter().map(|m| m.id.clone()).collect())
                .unwrap_or_default(),
            active: None,
        }
    }

    /// Every mode id, in the manifest's order.
    #[must_use]
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    /// The first mode the manifest declares — what the application opens
    /// in until the default-mode setting exists (Part 1 rule 5).
    #[must_use]
    pub fn first(&self) -> Option<&str> {
        self.ids.first().map(String::as_str)
    }

    /// The mode in force, if one has been adopted.
    #[must_use]
    pub fn active(&self) -> Option<&str> {
        self.active.as_deref()
    }

    /// Whether the manifest declares this mode.
    #[must_use]
    pub fn is_known(&self, mode_id: &str) -> bool {
        self.ids.iter().any(|id| id == mode_id)
    }

    /// **Adopt `mode_id`: remember the outgoing mode's arrangement, and
    /// restore the incoming one's.**
    ///
    /// The whole feature, in five steps:
    ///
    /// 1. A mode the manifest does not declare is declined — an unknown id
    ///    must not acquire a workspace, or a typo in a customized manifest
    ///    would quietly accumulate arrangements nothing can ever restore.
    /// 2. Re-adopting the mode already in force does nothing, so a caller
    ///    may drive this straight from `RibbonState::mode()` every frame
    ///    without checking first.
    /// 3. The **outgoing** mode's workspace is written from what is on
    ///    screen right now. This is what makes "each mode remembers your
    ///    arrangement of it" true without an explicit save.
    /// 4. The **incoming** mode's workspace is restored if it has one, and
    ///    otherwise its built-in default is used — filtered through
    ///    `catalog`, so a saved-but-stale panel and a compiled-out one are
    ///    handled identically.
    /// 5. The result is recorded, which arms the debounced write. A crash
    ///    after a mode change therefore costs nothing.
    ///
    /// Returns whether the arrangement was changed.
    ///
    /// **It cannot touch a document.** See the module header: the argument
    /// list is the proof, and a test asserts the consequence anyway.
    pub fn on_mode_changed(
        &mut self,
        mode_id: &str,
        dock: &mut DockState,
        store: &mut LayoutStore,
        catalog: &dyn PanelCatalog,
    ) -> bool {
        if !self.is_known(mode_id) {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "mode-unknown id={mode_id}"
                )
            });
            return false;
        }
        if self.active.as_deref() == Some(mode_id) {
            return false;
        }

        if let Some(from) = self.active.clone() {
            store
                .document_mut()
                .save_workspace(workspace_name(&from), dock.layout().clone());
        }

        let ws = workspace_name(mode_id);
        let restored = store.document().workspace(&ws).cloned();
        let remembered = restored.is_some();
        let default = layout_for_build(mode_id, catalog);

        // ★ A PANEL ADDED IN A NEW RELEASE MUST NOT BE BORN INVISIBLE.
        //
        // A mode is a remembered workspace, so an operator who upgrades
        // restores an arrangement saved before the new panel existed and never
        // sees it — and the portable build's own `BUILD-INFO.txt` tells them
        // to keep `userdata/`, so the more carefully they upgrade the more
        // reliably they miss it. Measured before this landed: the Pages panel
        // gave `remembered=true panels=1` against `remembered=false panels=2`
        // on a fresh profile.
        //
        // Neither obvious fix works. Forcing the default over a remembered
        // layout discards the operator's own arrangement, which is the whole
        // feature. Leaving it means every panel from now on ships hidden.
        //
        // So `egui-shell` records which panels EXISTED when a workspace was
        // written, and the two states a layout otherwise cannot tell apart —
        // *closed on purpose* and *did not exist yet* — become distinguishable.
        // This is the application half, and it is here rather than in the
        // shell because what to do about an unstamped file is a decision about
        // THIS product's upgrade path.
        let registered = registered_panels(catalog);
        let mut layout = restored.unwrap_or_else(|| default.clone());
        if remembered {
            let candidates = match store.document().unseen_panels(&ws, &registered) {
                // The ordinary case from the second launch onward: the file
                // says what it knew, and only genuinely new panels appear.
                Unseen::New(ids) => ids,
                // Written before anything was recorded — the one-time upgrade.
                // Every registered panel is a candidate, and `adopt` skips the
                // ones already mounted, so the net effect is exactly the panels
                // that are missing from a layout that predates the record.
                //
                // This can re-open a panel the operator closed BEFORE the
                // record existed, once. That is the cost, it is bounded to a
                // single launch per install, and the alternative is that the
                // upgrade case silently does nothing forever — which is the
                // defect.
                Unseen::Unknown => registered.clone(),
            };
            let added = adopt(&mut layout, &default, &candidates);
            if !added.is_empty() {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "mode-adopted-panels mode={mode_id} added={}",
                        added
                            .iter()
                            .map(PanelId::as_str)
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                });
            }
        }
        dock.set_layout(layout);
        // Stamped whether or not anything was added, so the next launch
        // reports `New(vec![])` instead of offering the same panels again.
        store.document_mut().mark_panels_seen(&ws, &registered);

        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "mode-changed from={:?} to={mode_id} remembered={remembered} panels={}",
                self.active,
                dock.layout().panels().count(),
            )
        });

        self.active = Some(mode_id.to_owned());
        // ★ Which mode, alongside its arrangement. The two are one answer to
        // the question *"where was I?"* and storing only the second is what
        // made the application come back in Read with an Edit layout in it.
        //
        // After the assignment above rather than before, so that this line
        // cannot be read as the thing that decides the mode — it records a
        // decision already made two statements up.
        store.record_active_mode(mode_id);
        self.record_layout(dock.layout(), store);
        true
    }

    /// Record the live arrangement as both the document's `active` and the
    /// current mode's workspace.
    ///
    /// Called when the dock reports
    /// [`egui_shell::dock::DockFrameReport::layout_changed`].
    ///
    /// **Both, and that is the point.** The document's `active` is the
    /// arrangement in force; the mode's workspace is the arrangement to
    /// come back to. Writing only the first would mean a crash mid-session
    /// cost the operator every rearrangement they had made since the last
    /// mode change, which is the "only saved at exit" failure wearing a
    /// different hat. Writing only the second would leave `active` stale in
    /// a file an operator may read.
    ///
    /// Idempotent: recording an arrangement that is already recorded arms
    /// no write, so a caller that calls it unconditionally costs nothing.
    pub fn record_layout(&self, layout: &DockLayout, store: &mut LayoutStore) {
        store.record_active(layout);
        let Some(active) = self.active.as_deref() else {
            return;
        };
        let name = workspace_name(active);
        if store.document().workspace(&name) == Some(layout) {
            return;
        }
        store.document_mut().save_workspace(name, layout.clone());
    }

    /// Restore part of the current mode's arrangement to its default.
    ///
    /// `RIBBON_IA.md`'s rule is why this has a scope at all: *"an operator
    /// who only wanted the right dock back must not lose their left one."*
    /// The scoping itself is `egui-shell`'s; what this adds is the one
    /// thing the shell cannot know — **which** default, given that the
    /// right default depends on the mode in force.
    ///
    /// Saved workspaces are untouched, including the current mode's, which
    /// is then immediately overwritten by [`Self::record_layout`] with the
    /// reset arrangement. That is the intended reading of "reset this
    /// mode": the mode goes back to its default and remembers that it did.
    ///
    /// Returns whether anything changed.
    pub fn reset(
        &self,
        scope: ResetScope,
        dock: &mut DockState,
        store: &mut LayoutStore,
        catalog: &dyn PanelCatalog,
    ) -> bool {
        let default = layout_for_build(self.active.as_deref().unwrap_or_default(), catalog);
        let mut layout = dock.layout().clone();
        if !egui_shell::layout::reset::reset(&mut layout, scope, &default) {
            return false;
        }
        dock.set_layout(layout);
        self.record_layout(dock.layout(), store);
        true
    }
}

/// Everything the application needs to start with a persisted, mode-aware
/// dock.
///
/// Returned as a struct rather than a tuple because three values whose
/// types are `Modes`, `LayoutStore` and `DockState` are easy to bind in the
/// wrong order and hard to notice having done so.
pub struct Startup {
    /// The modes, with the opening one already adopted.
    pub modes: Modes,
    /// The layout file, already loaded. Its
    /// [`LayoutStore::report`] is what a status surface should disclose.
    pub layout: LayoutStore,
    /// The dock, already holding the opening mode's arrangement.
    pub dock: DockState,
}

/// Load the layout and adopt the manifest's first mode.
///
/// The whole start-up sequence, in one call, because its order is
/// load-bearing and getting it wrong is silent:
///
/// 1. The mode list comes from the manifest.
/// 2. The **fallback** handed to the loader is the opening mode's default,
///    so a first run — or a file that could not be parsed at all — starts
///    from an arrangement that suits the mode the application opens in
///    rather than from some other mode's.
/// 3. The document is loaded, fail-soft, with `catalog` deciding which
///    saved mounts this build can honour.
/// 4. The opening mode is adopted, which restores its remembered
///    arrangement if it has one.
///
/// ## Why step 4 may discard the file's `active` arrangement
///
/// The document's `active` is *"the arrangement in force"* — in force in
/// whichever mode was showing when the application last closed, which is
/// not necessarily the one it now opens in. The mode's own workspace is the
/// better answer to "what should Read look like", so it wins. `active` is
/// still kept current by [`Modes::record_layout`], because it is what a
/// person reading the file expects to find and what the loader falls back
/// to if a workspace has to be dropped.
#[must_use]
pub fn start(shell: Option<&Shell>, catalog: &dyn PanelCatalog) -> Startup {
    let modes = Modes::from_shell(shell);
    let fallback = layout_for_build(modes.first().unwrap_or_default(), catalog);
    assemble(modes, LayoutStore::load(&fallback, catalog), catalog)
}

/// [`start`], reading from an explicit directory. For tests and a future
/// `--user-data-dir` override.
#[must_use]
pub fn start_in(
    dir: &std::path::Path,
    shell: Option<&Shell>,
    catalog: &dyn PanelCatalog,
) -> Startup {
    let modes = Modes::from_shell(shell);
    let fallback = layout_for_build(modes.first().unwrap_or_default(), catalog);
    assemble(
        modes,
        LayoutStore::load_in(dir, &fallback, catalog),
        catalog,
    )
}

/// **Every panel this build registers**, for the upgrade reconciliation
/// below.
///
/// Derived from [`Panel::ALL`] filtered through the live catalog, so it is
/// the same set the §5b capability rule uses — a build with a capability
/// compiled out reports fewer, which is exactly right: a panel that does not
/// exist here must not be recorded as one this layout has seen, or removing
/// and restoring a capability would leave it permanently invisible.
fn registered_panels(catalog: &dyn PanelCatalog) -> Vec<PanelId> {
    Panel::ALL
        .into_iter()
        .map(|p| PanelId::new(p.command_id()))
        .filter(|id| catalog.contains(id.as_str()))
        .collect()
}

/// Mount `new` into `layout` wherever `default` puts them.
///
/// Returns the ids actually added, for the trace.
///
/// # Why the default's placement rather than a fixed corner
///
/// A panel appended to the end of the first stack would land beside
/// whatever happens to be there, which for Pages means tabbed behind
/// Bookmarks on the day it appears — the operator sees a tab bar grow by one
/// and has no reason to think a feature arrived. Placing it where the mode
/// intended puts it where the documentation, the mockups and the next
/// release's default all agree it goes.
///
/// A panel already present is skipped rather than duplicated. That is not
/// defensive: `Unseen::Unknown` deliberately reports every registered panel,
/// including ones the restored layout already mounts.
fn adopt(layout: &mut DockLayout, default: &DockLayout, new: &[PanelId]) -> Vec<PanelId> {
    let mut added = Vec::new();
    for id in new {
        if layout.contains(id) {
            continue;
        }
        let Some(at) = default.find(id) else {
            // In the default for no mode — a panel that exists but that this
            // mode does not mount. Read mode has no Objects panel by design,
            // and adding one here because it is new would override a decision
            // `layout_for` made deliberately.
            continue;
        };
        let side = match at.side {
            DockSide::Left => &mut layout.left,
            DockSide::Right => &mut layout.right,
        };
        // Fall back outwards rather than giving up: a remembered layout may
        // have fewer columns or stacks than the default (the operator
        // collapsed one), and the panel still has to arrive somewhere. The
        // last stack of the last column is the closest surviving relative of
        // the position the default asked for.
        // Indices are resolved BEFORE borrowing, because `get_mut(..)
        // .or_else(|| ..last_mut())` borrows the same vector twice and the
        // compiler is right to refuse it.
        let column_ix = at.column.min(side.columns.len().saturating_sub(1));
        if let Some(column) = side.columns.get_mut(column_ix) {
            let stack_ix = at.stack.min(column.stacks.len().saturating_sub(1));
            if let Some(stack) = column.stacks.get_mut(stack_ix) {
                stack.tabs.push(id.clone());
                // ★★★ **And it is RAISED, or the whole function is a no-op the
                // trace reports as a success.**
                //
                // This line's absence was found on 2026-08-19 by the check that
                // asks what a first frame shows. `stack.tabs.push` mounts the
                // panel and leaves whatever was active still active, so on any
                // profile older than the panel it arrives **behind another
                // tab** — present in `layout.ron`, present in the tab strip,
                // and invisible.
                //
                // For a panel whose whole purpose is discoverability that is
                // identical to not shipping it. The **Tool panel** is exactly
                // that panel: built to answer *"no side bar area showing what
                // tool is active"*, adopted correctly into every upgraded
                // layout, and never once seen by the operator who reported the
                // gap — because he has run this build for two weeks and his
                // layout predates it.
                //
                // ★ This function's own doc comment had already reasoned to the
                // edge of it: *"a panel appended to the end of the first stack
                // would land beside whatever happens to be there … the operator
                // sees a tab bar grow by one and has no reason to think a
                // feature arrived."* It then solved the placement half and left
                // the visibility half, which is the harder half and the one an
                // operator can actually perceive.
                //
                // **Once, on adoption — not every frame.** A panel that raised
                // itself on every start would fight an operator who had
                // deliberately tabbed it behind something else, which is a
                // choice they are entitled to make about a panel they have
                // already met. Adoption happens exactly once per panel per
                // layout, which is precisely the moment "a feature arrived" is
                // true.
                stack.active = stack.tabs.len().saturating_sub(1);
                added.push(id.clone());
                continue;
            }
        }
        {
            // The side is empty — Read mode's right dock, for instance. Not a
            // fallback for an unexpected state: a mode with nothing on one
            // side is an ordinary arrangement, and a panel the default puts
            // there still has to arrive.
            side.columns.push(Column {
                stacks: vec![Stack::new(id.clone())],
                share: 1.0,
            });
            // ★ **And the side has to be shown, or this whole function is a
            // no-op the trace reports as a success.**
            //
            // Found by running the binary, not by a test: adopting Forms into
            // an upgraded Read layout printed `added=view.panel_forms`, the
            // saved `layout.ron` carried the panel, and the window drew no
            // right dock at all. A side that has never held anything is
            // persisted `visible: false`, and adding a column to it does not
            // change that.
            //
            // **The condition is `columns.is_empty()`, and the narrowness is
            // the point.** `SideLayout::visible`'s own doc calls hiding "a
            // view state, not a destruction" — an operator who collapses a
            // populated dock has made a choice, and a new panel arriving must
            // not overrule it; it goes into the arrangement the side keeps
            // while hidden, and appears when they open it again. A side with
            // no columns has no such choice behind it: nothing was ever there
            // to collapse, so `false` means "empty", not "not now".
            //
            // This is the same distinction the upgrade path already draws one
            // level up — a genuinely new panel appears, one closed on purpose
            // stays closed — applied to the side rather than to the panel.
            side.visible = true;
            added.push(id.clone());
        }
    }
    added
}

/// The shared tail of the two start-up paths.
///
/// ★★★ **Where the application decides which mode it opens in**, and since
/// 2026-08-27 that is *the one it was left in* rather than always the first the
/// manifest declares.
///
/// The operator, 2026-08-26, reporting the consequence rather than the cause:
/// *"I can't figure out how to click on objects to edit them."* Part of that is
/// an engine limitation and is filed as one — but the part nothing explained is
/// that the program opened in **Read** on every launch, where a click on page
/// content selects nothing at all, and no surface said so. Someone who spent an
/// afternoon in Edit came back the next morning to a program that had silently
/// forgotten.
///
/// # The three ways this can decline, all of which land on the first mode
///
/// 1. **No stored id** — a fresh profile, or a file written before this field
///    existed.
/// 2. **An id the manifest no longer declares** — a mode renamed or removed in
///    a customized manifest between two runs. `is_known` is what catches it,
///    and the fallback is what stops a rename leaving the shell with no mode.
/// 3. **No modes at all** — a manifest that failed to validate. `first()` is
///    `None` too, and nothing is adopted.
///
/// ★ Note what is *not* checked: whether the stored mode is one this build
/// considers safe or sensible. It is the operator's own last choice, made in
/// this program, and second-guessing it would be the program deciding it knows
/// better than the person using it.
fn assemble(mut modes: Modes, mut layout: LayoutStore, catalog: &dyn PanelCatalog) -> Startup {
    let mut dock = DockState::new(layout.active().clone());
    let remembered = layout
        .active_mode()
        .filter(|id| modes.is_known(id))
        .map(str::to_owned);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // Both fields, because "the stored id was not honoured" and "there
            // was no stored id" are different findings and only the pair tells
            // them apart from a trace alone.
            "mode-restore stored={:?} using={:?}",
            layout.active_mode(),
            remembered.as_deref().or_else(|| modes.first())
        )
    });
    if let Some(start) = remembered.or_else(|| modes.first().map(str::to_owned)) {
        modes.on_mode_changed(&start, &mut dock, &mut layout, catalog);
    }
    Startup {
        modes,
        layout,
        dock,
    }
}

#[cfg(test)]
mod tests {
    use super::defaults::pages;
    use super::*;
    use crate::app::PdfcerApp;
    use crate::app::state::Status;
    use egui_shell::dock::{AnyPanel, PanelInfo, PanelRegistry, SideLayout};
    use std::path::PathBuf;

    /// The panel registry a full build would have: every panel this crate
    /// actually implements.
    ///
    /// Duplicated in [`defaults`]' own test module rather than shared,
    /// because a `#[cfg(test)]` helper reachable across module boundaries has
    /// to be made visible in the non-test build too. Six lines of fixture is
    /// the cheaper of the two costs.
    fn registry() -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for panel in Panel::ALL {
            let id = panel.command_id();
            r.register(PanelInfo::new(id, id));
        }
        r
    }

    /// The manifest's real mode list.
    fn shell() -> Shell {
        crate::shell::manifest::built_in()
    }

    /// A fresh, empty directory nothing else is using.
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("pdfcer-gui-modes-{tag}-{nanos}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        dir
    }

    /// Every mode id, as string slices, for comparison against a literal
    /// array.
    fn names(modes: &Modes) -> Vec<&str> {
        modes.ids().iter().map(String::as_str).collect()
    }

    /// ★ **The mode list comes from the manifest, not from this module.**
    ///
    /// `SHELL_FRAMEWORK.md` §4 makes Read/Review/Edit a configuration
    /// rather than a built-in, and `egui-shell`'s workspace store refuses
    /// to ship three magic names for the same reason. This module is the
    /// third place that rule could have been broken — and the place where
    /// breaking it would look most reasonable, because it is the one that
    /// legitimately knows what "read" means as an arrangement.
    ///
    /// Asserted by driving a manifest with *different* modes: a fourth mode
    /// must be adoptable, and a mode the manifest does not declare must
    /// not be.
    #[test]
    fn the_mode_list_is_whatever_the_manifest_declares() {
        let real = Modes::from_shell(Some(&shell()));
        assert_eq!(names(&real), ["read", "review", "edit"]);
        assert_eq!(real.first(), Some("read"));

        let other = Shell::new()
            .with_mode(egui_shell::manifest::Mode::new(
                "proofing",
                "Proofing",
                ["file"],
            ))
            .with_mode(egui_shell::manifest::Mode::new(
                "drafting",
                "Drafting",
                ["file"],
            ));
        let modes = Modes::from_shell(Some(&other));
        assert_eq!(names(&modes), ["proofing", "drafting"]);
        assert_eq!(modes.first(), Some("proofing"));
        assert!(modes.is_known("drafting"));
        assert!(!modes.is_known("read"), "no mode is built in here");

        // A manifest that failed to validate leaves no modes at all rather
        // than a three-position model from nowhere.
        assert!(Modes::from_shell(None).ids().is_empty());
        assert!(Modes::from_shell(None).first().is_none());
    }

    /// A mode's workspace name is derived from the id, not the label, and
    /// is distinguishable from one the operator made.
    #[test]
    fn a_mode_workspace_is_named_by_id_and_is_recognisable() {
        assert_eq!(workspace_name("review"), "mode:review");
        assert_eq!(mode_of_workspace("mode:review"), Some("review"));
        assert_eq!(
            mode_of_workspace("Review"),
            None,
            "an operator's own workspace called Review is not a mode's"
        );
    }

    /// ★ **Read → Edit → Read restores YOUR Edit, not a default.**
    ///
    /// The behaviour the whole module exists for, and the one
    /// `MODES_AND_PANELS.md` Part 1 rule 3 states: *"Each mode remembers
    /// its own panel layout. Leaving Edit and coming back restores the
    /// arrangement, not a default."*
    #[test]
    fn a_mode_remembers_the_operators_own_arrangement_of_it() {
        let dir = temp_dir("remember");
        let registry = registry();
        let shell = shell();
        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);

        assert_eq!(modes.active(), Some("read"));
        let read_default = dock.layout().clone();

        // Into Edit, and rearrange it the way an operator would: a wider
        // dock and a different tab selected.
        assert!(modes.on_mode_changed("edit", &mut dock, &mut store, &registry));
        assert_ne!(dock.layout(), &read_default);
        dock.layout_mut().left.width_pts = 461.0;
        assert!(dock.activate(&PanelId::new(Panel::Fonts.command_id())));
        let my_edit = dock.layout().clone();
        modes.record_layout(&my_edit, &mut store);

        // Back to Read: Read's own arrangement, untouched by what was done
        // in Edit.
        assert!(modes.on_mode_changed("read", &mut dock, &mut store, &registry));
        assert_eq!(dock.layout(), &read_default);

        // …and back to Edit: MINE, not the default.
        assert!(modes.on_mode_changed("edit", &mut dock, &mut store, &registry));
        assert_eq!(dock.layout(), &my_edit);
        assert_ne!(
            dock.layout(),
            &layout_for_build("edit", &registry),
            "the default came back instead of the operator's arrangement"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **…and it survives a restart.**
    ///
    /// The round trip that makes the previous test worth anything: the same
    /// sequence, through a real file, across two `Startup`s. A rearrangeable
    /// layout that forgets itself each restart is worse than a fixed one.
    #[test]
    fn a_modes_arrangement_survives_a_restart() {
        let dir = temp_dir("restart");
        let registry = registry();
        let shell = shell();

        let my_edit = {
            let Startup {
                mut modes,
                layout: mut store,
                mut dock,
            } = start_in(&dir, Some(&shell), &registry);
            modes.on_mode_changed("edit", &mut dock, &mut store, &registry);
            dock.layout_mut().right.width_pts = 377.0;
            let mine = dock.layout().clone();
            modes.record_layout(&mine, &mut store);
            assert!(store.flush(), "the change was outstanding");
            mine
        };

        let Startup {
            modes,
            layout: store,
            dock,
        } = start_in(&dir, Some(&shell), &registry);
        // ★★ **The session opens in Edit, because that is where it was left.**
        //
        // This line asserted `Some("read")` until 2026-08-27 and was correct
        // about the program at the time: it opened in the manifest's first mode
        // however the operator had left it. That behaviour is the invisible
        // first failure of every session — Read cannot select page content, and
        // an operator who spent yesterday in Edit gets a canvas that ignores
        // their clicks and no surface saying why.
        //
        // ★ Note what the assertion below now proves that it could not before:
        // the arrangement is restored **without a mode change**, because the
        // right mode was already adopted at startup. The old version had to
        // call `on_mode_changed("edit")` to get there, which meant it could not
        // have distinguished "the layout was restored" from "the layout was
        // rebuilt on the way in".
        assert_eq!(
            modes.active(),
            Some("edit"),
            "the session opens in the mode it was left in"
        );
        assert_eq!(
            dock.layout(),
            &my_edit,
            "the arrangement did not survive the restart"
        );
        assert_eq!(
            store.active_mode(),
            Some("edit"),
            "the file records the mode as well as the arrangement"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★★ **A stored mode this manifest no longer declares is declined**, and
    /// the application opens in the first mode rather than in none.
    ///
    /// The case is real rather than theoretical: the mode list comes from a
    /// manifest an operator may customize, and renaming a mode between two runs
    /// leaves the previous run's id stored and unresolvable. Without the
    /// `is_known` filter the shell would adopt nothing, which is a state with no
    /// ribbon tabs and no way back.
    #[test]
    fn a_stored_mode_the_manifest_no_longer_declares_is_declined() {
        let dir = temp_dir("stale-mode");
        let registry = registry();
        let shell = shell();

        {
            let Startup {
                mut modes,
                layout: mut store,
                mut dock,
            } = start_in(&dir, Some(&shell), &registry);
            modes.on_mode_changed("edit", &mut dock, &mut store, &registry);
            // Write an id nothing declares, the way a renamed mode would leave
            // one behind.
            store.record_active_mode("a-mode-that-was-renamed-away");
            assert!(store.flush(), "the change was outstanding");
        }

        let Startup { modes, .. } = start_in(&dir, Some(&shell), &registry);
        assert_eq!(
            modes.active(),
            Some("read"),
            "an unknown stored mode falls back to the first the manifest declares"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Adopting the mode already in force does nothing, so a caller may
    /// drive this from the ribbon's state every frame.
    #[test]
    fn re_adopting_the_current_mode_is_a_no_op() {
        let dir = temp_dir("idempotent");
        let registry = registry();
        let shell = shell();
        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);

        store.flush();
        let saves = store.saves();
        let before = dock.layout().clone();
        for _ in 0..10 {
            assert!(!modes.on_mode_changed("read", &mut dock, &mut store, &registry));
        }
        assert_eq!(dock.layout(), &before);
        assert!(!store.is_dirty(), "a no-op must not arm a write");
        store.flush();
        assert_eq!(store.saves(), saves);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A mode the manifest does not declare is declined rather than given a
    /// workspace of its own.
    #[test]
    fn an_undeclared_mode_is_declined() {
        let dir = temp_dir("undeclared");
        let registry = registry();
        let shell = shell();
        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);

        let before = dock.layout().clone();
        assert!(!modes.on_mode_changed("proofing", &mut dock, &mut store, &registry));
        assert_eq!(modes.active(), Some("read"));
        assert_eq!(dock.layout(), &before);
        assert!(
            !store
                .document()
                .workspace_names()
                .contains(&"mode:proofing"),
            "an unknown mode must not accumulate a workspace"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **Switching modes touches neither the document nor the
    /// selection.**
    ///
    /// `MODES_AND_PANELS.md` Part 1 rule 1: *"Switching modes never
    /// destroys work. Read ⇄ Edit is a view stance, not a save boundary."*
    ///
    /// The argument list of [`Modes::on_mode_changed`] already makes this
    /// impossible — it can reach a `DockState` and a `LayoutStore`, and
    /// neither can reach an `EditSession` — so this test exists to make a
    /// *later* edit that widens that argument list fail here rather than
    /// pass a review.
    #[test]
    fn switching_modes_touches_neither_the_document_nor_the_selection() {
        use crate::canvas::selection::ClickHit;
        use crate::canvas::target::TargetId;

        let dir = temp_dir("no-loss");
        let mut app = PdfcerApp::new();
        app.open_path(crate::panels::objects::test_support::engine_fixture(
            "pageops/four-pages.pdf",
        ));
        let Status::Open(doc) = &mut app.status else {
            panic!("the fixture opens")
        };
        doc.view.page_index = 2;
        doc.edit_epoch = 7;
        doc.selection.click(
            2,
            ClickHit {
                object: Some(TargetId::Object(1)),
                ..ClickHit::default()
            },
            false,
            false,
        );

        let shell = shell();
        let mut modes = Modes::from_shell(Some(&shell));
        let mut store = LayoutStore::load_in(
            &dir,
            &layout_for_build("read", &app.panel_registry),
            &app.panel_registry,
        );
        for mode in ["read", "edit", "review", "read"] {
            modes.on_mode_changed(mode, &mut app.dock, &mut store, &app.panel_registry);
        }

        let Status::Open(doc) = &app.status else {
            panic!("the document is still open")
        };
        assert_eq!(doc.view.page_index, 2, "the view moved");
        assert_eq!(doc.edit_epoch, 7, "the document was touched");
        assert_eq!(doc.selection.len(), 1, "the selection was lost");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A scoped reset restores the mode's own default on one side and
    /// leaves the other alone — *"an operator who only wanted the right
    /// dock back must not lose their left one."*
    #[test]
    fn a_scoped_reset_restores_this_modes_default_on_one_side_only() {
        let dir = temp_dir("reset");
        let registry = registry();
        let shell = shell();
        let Startup {
            mut modes,
            layout: mut store,
            mut dock,
        } = start_in(&dir, Some(&shell), &registry);
        modes.on_mode_changed("edit", &mut dock, &mut store, &registry);

        dock.layout_mut().left.width_pts = 500.0;
        dock.layout_mut().right.width_pts = 500.0;
        let mangled_left = dock.layout().left.clone();

        assert!(modes.reset(ResetScope::Right, &mut dock, &mut store, &registry));
        assert_eq!(dock.layout().left, mangled_left, "the left dock moved");
        assert_eq!(
            dock.layout().right,
            layout_for_build("edit", &registry).right,
            "the right dock is not this mode's default"
        );
        // And the mode remembers that it was reset.
        assert_eq!(
            store.document().workspace("mode:edit"),
            Some(dock.layout()),
            "the reset arrangement was not recorded"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A build with no manifest still starts: no modes, the full
    /// arrangement, and a dock that works.
    #[test]
    fn a_build_with_no_manifest_still_gets_a_working_dock() {
        let dir = temp_dir("no-manifest");
        let Startup {
            modes,
            layout: store,
            dock,
        } = start_in(&dir, None, &AnyPanel);
        assert!(modes.ids().is_empty());
        assert_eq!(modes.active(), None);
        assert!(dock.layout().panels().count() > 0, "the dock is not empty");
        assert!(!store.is_noteworthy());

        let _ = std::fs::remove_dir_all(&dir);
    }

    // -----------------------------------------------------------------
    // A panel added in a new release reaches an operator who upgrades
    // -----------------------------------------------------------------

    /// ★ **The regression test for the defect this was written for.**
    ///
    /// A layout written before a panel existed — no `known_panels` at all,
    /// which is every file any existing install has — must gain the panel,
    /// and must gain it *where the mode default puts it*.
    ///
    /// Simulated the way it actually happens rather than by constructing the
    /// end state: a workspace is saved holding ONLY Bookmarks (which is what
    /// Read's remembered layout contained before Pages was registered), the
    /// store is then read back through the real `on_mode_changed`, and the
    /// result is asserted.
    #[test]
    fn a_layout_that_predates_a_panel_gains_it() {
        let dir = temp_dir("upgrade-adopts");
        let catalog = registry();

        // The old world: a remembered Read layout with one panel and no
        // record of what existed when it was written.
        {
            let mut store = LayoutStore::load_in(&dir, &DockLayout::default(), &catalog);
            store.document_mut().save_workspace(
                workspace_name("read"),
                DockLayout::new(
                    SideLayout::single(PanelId::new(Panel::Bookmarks.command_id())),
                    SideLayout::none(),
                ),
            );
            store.flush();
        }

        let start = start_in(&dir, Some(&crate::shell::manifest::built_in()), &catalog);
        let mounted: Vec<&str> = start.dock.layout().panels().map(PanelId::as_str).collect();

        assert!(
            mounted.contains(&Panel::Bookmarks.command_id()),
            "the operator's own arrangement must survive: {mounted:?}"
        );
        assert!(
            mounted.contains(&pages()),
            "a panel registered since the layout was written must appear: {mounted:?}"
        );

        // ★ **Mounted is not shown, and this test used to stop one line above
        // this comment.**
        //
        // Forms adopts into Read's RIGHT side, which a layout of this vintage
        // records as `SideLayout::none()` — no columns, `visible: false`.
        // `adopt` gave it a column and the assertions above passed, because
        // `panels()` walks the arrangement a hidden side keeps. The running
        // binary drew no right dock at all: `mode-adopted-panels
        // added=view.panel_forms` in the trace, the panel in the saved
        // `layout.ron`, and nothing on screen.
        //
        // So the assertion that matters is the side's own `visible`, and it
        // is worth stating why the weaker one is so tempting: a panel that is
        // *in the layout* is one the operator can reach by opening the dock,
        // and every in-memory check agrees it is there. Only the pixels
        // disagree.
        assert!(
            mounted.contains(&Panel::Forms.command_id()),
            "Read mounts Forms since 2026-08-14: {mounted:?}"
        );
        assert!(
            start.dock.layout().right.visible,
            "a side that was empty because nothing had ever been there must be \
             SHOWN when a panel arrives in it — otherwise adoption reports a \
             success the operator cannot see"
        );
    }

    /// ★ **…and a side the operator collapsed on purpose stays collapsed.**
    ///
    /// The other half of the rule, and the reason `adopt` keys on
    /// `columns.is_empty()` rather than on `!visible`. `SideLayout::visible`'s
    /// own documentation calls hiding *"a view state, not a destruction"* — so
    /// a populated side that is hidden carries a decision, and a new panel
    /// must join the arrangement it keeps rather than overrule it.
    ///
    /// Asserting the collapse survives is what makes the pair a rule instead
    /// of a patch: a fix that simply set `visible = true` whenever anything
    /// was adopted would pass the test above and re-open a dock the operator
    /// closed, on every release that adds a panel, forever.
    #[test]
    fn adoption_does_not_re_open_a_side_the_operator_collapsed() {
        let dir = temp_dir("upgrade-collapsed");
        let catalog = registry();

        // A remembered Edit layout whose right side holds a panel and is
        // hidden — the shape an operator produces by collapsing the dock.
        {
            let mut store = LayoutStore::load_in(&dir, &DockLayout::default(), &catalog);
            let mut right = SideLayout::single(PanelId::new(Panel::Properties.command_id()));
            right.visible = false;
            store.document_mut().save_workspace(
                workspace_name("edit"),
                DockLayout::new(
                    SideLayout::single(PanelId::new(Panel::Bookmarks.command_id())),
                    right,
                ),
            );
            store.flush();
        }

        let mut start = start_in(&dir, Some(&crate::shell::manifest::built_in()), &catalog);
        let mut store = LayoutStore::load_in(&dir, &DockLayout::default(), &catalog);
        start
            .modes
            .on_mode_changed("edit", &mut start.dock, &mut store, &catalog);

        let mounted: Vec<&str> = start.dock.layout().panels().map(PanelId::as_str).collect();
        assert!(
            mounted.contains(&Panel::Objects.command_id()),
            "the new panel still arrives — it is only its side that is unchanged: {mounted:?}"
        );
        assert!(
            !start.dock.layout().right.visible,
            "a populated side the operator collapsed must stay collapsed"
        );
    }

    /// …and it happens **once**. The second launch adopts nothing.
    ///
    /// The property that makes the `Unknown` branch acceptable. Without it,
    /// every launch would re-open every panel the operator had closed, which
    /// is a far worse bug than the one being fixed — it would undo a decision
    /// they made, repeatedly, forever.
    #[test]
    fn the_upgrade_adoption_happens_only_once() {
        let dir = temp_dir("upgrade-once");
        let catalog = registry();
        {
            let mut store = LayoutStore::load_in(&dir, &DockLayout::default(), &catalog);
            store.document_mut().save_workspace(
                workspace_name("read"),
                DockLayout::new(
                    SideLayout::single(PanelId::new(Panel::Bookmarks.command_id())),
                    SideLayout::none(),
                ),
            );
            store.flush();
        }

        // First launch: adopts, and stamps.
        {
            let mut start = start_in(&dir, Some(&crate::shell::manifest::built_in()), &catalog);
            // The operator then closes Pages deliberately and it is saved.
            let mut layout = start.dock.layout().clone();
            layout.close(&PanelId::new(pages()));
            start
                .layout
                .document_mut()
                .save_workspace(workspace_name("read"), layout);
            start.layout.flush();
        }

        // Second launch: it must stay closed.
        let start = start_in(&dir, Some(&crate::shell::manifest::built_in()), &catalog);
        let mounted: Vec<&str> = start.dock.layout().panels().map(PanelId::as_str).collect();
        assert!(
            !mounted.contains(&pages()),
            "a panel the operator closed AFTER the record existed must stay closed: {mounted:?}"
        );
    }

    /// A fresh install is untouched by any of this.
    ///
    /// There is no remembered workspace, so the default arrangement is used
    /// whole and the adoption path is never entered — asserted because a
    /// reconciliation that also fired on first run would be indistinguishable
    /// from one that worked, right up until it added a panel twice.
    #[test]
    fn a_fresh_install_gets_the_default_and_nothing_else() {
        let dir = temp_dir("upgrade-fresh");
        let catalog = registry();
        let start = start_in(&dir, Some(&crate::shell::manifest::built_in()), &catalog);
        let mounted: Vec<&str> = start.dock.layout().panels().map(PanelId::as_str).collect();
        let default = layout_for_build("read", &catalog);
        let expected: Vec<&str> = default.panels().map(PanelId::as_str).collect();
        assert_eq!(mounted, expected);
    }

    /// ★★★ **An adopted panel arrives VISIBLE, not merely mounted.**
    ///
    /// The regression test for the defect that made the (since retired) Tool
    /// panel — built to answer *"no side bar area showing what tool is
    /// active"* — invisible to the operator who reported the gap, for its
    /// entire life, on the profile he had been running for two weeks.
    ///
    /// ★ The panel in the assertion is now Layers, because the Tool panel was
    /// dissolved by `OPERATOR_REQUESTS.md` O123. The property under test is
    /// `adopt`'s and has nothing to do with which panel arrives — but naming a
    /// panel that no longer exists would have made the test read as being
    /// about a surface, which it never was.
    ///
    /// `stack.tabs.push` mounted it and left whatever was active still active,
    /// so it landed behind another tab: present in `layout.ron`, present in the
    /// tab strip, and never seen. For a panel whose whole purpose is
    /// discoverability that is identical to not shipping it.
    ///
    /// ★ Found by a driven check asking *what does a first frame show* — not by
    /// any of the tests of `adopt`, every one of which asked whether the panel
    /// was PRESENT. Presence was never in doubt.
    #[test]
    fn an_adopted_panel_is_raised_and_not_merely_mounted() {
        let mut layout = DockLayout::default();
        let occupant = PanelId::new("view.panel_pages");
        let arrival = PanelId::new("view.panel_layers");
        layout.left.columns.push(Column {
            stacks: vec![Stack::new(occupant.clone())],
            share: 1.0,
        });
        let mut default = DockLayout::default();
        default.left.columns.push(Column {
            stacks: vec![Stack::new(occupant.clone())],
            share: 1.0,
        });
        default.left.columns[0].stacks[0].tabs.push(arrival.clone());

        let added = adopt(&mut layout, &default, std::slice::from_ref(&arrival));
        assert_eq!(added, vec![arrival.clone()]);

        let stack = &layout.left.columns[0].stacks[0];
        assert!(stack.tabs.contains(&arrival), "mounted");
        assert_eq!(
            stack.tabs.get(stack.active),
            Some(&arrival),
            "the arriving panel must be the one on screen"
        );
    }
}
