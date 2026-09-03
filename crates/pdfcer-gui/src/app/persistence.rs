//! # `app::persistence` — the dock layout, on disk
//!
//! `egui-shell` can already read and write a [`LayoutDocument`]; what it
//! deliberately does **not** do is decide *where* the file lives or *when*
//! it is written. Its own header says so in as many words:
//!
//! > It does not decide *when* to save, and it does not choose a path. An
//! > application saves on `DockFrameReport::layout_changed` and picks its
//! > own location — which for this project means a named partition of the
//! > distribution folder rather than a platform app-data directory.
//!
//! This module is pdfcer's answer to both questions, and nothing else. It
//! owns one type, [`LayoutStore`]: a path, a document, the report from the
//! load that produced it, and a small amount of write-scheduling.
//!
//! ## Why a rearrangeable layout that forgets is worse than a fixed one
//!
//! `MODES_AND_PANELS.md` Part 2's build order opens with the argument, and
//! it is the whole reason this module exists before anything else in the
//! dock is made more flexible:
//!
//! > **(f) persistence** — cheapest, highest value, unblocks everything.
//! > *A rearrangeable layout that forgets itself each restart is worse than
//! > a fixed one.*
//!
//! Worse, not merely less good. A fixed layout costs an operator nothing
//! after the first day; a rearrangeable one that forgets charges them the
//! rearrangement every single session, and teaches them not to bother.
//!
//! ## ★ Where the file lives, and why it is not this module's decision
//!
//! `<the settings directory>/layout.ron` — **beside `settings.txt`**, and
//! the directory is resolved by asking `pdfcer-core` rather than by
//! computing it here:
//!
//! ```text
//! pdfcer_core::settings::resolve_store()      →  StoreLocation { path, kind }
//!                        .directory()        →  <exe dir>/userdata      (Portable)
//!                                            or the platform config dir (PlatformFallback)
//!                                            or None                    (None)
//! ```
//!
//! Three properties follow from deferring to that call, and every one of
//! them would be lost by a second implementation here:
//!
//! 1. **One location decision, not two.** `ARCHITECTURE.md` §6's
//!    single-folder-portable posture is enforced by the *ordering* inside
//!    `resolve_store` — portable first, platform config only as a fallback
//!    — and `pdfcer-core`'s own docs end that paragraph with *"do not
//!    reverse it"*. A layout file that resolved its own directory could
//!    reverse it by accident and nothing would notice until an operator's
//!    settings and layout ended up in different folders.
//! 2. **The writability probe comes with it.** `resolve_store` does not
//!    assume the executable's folder is writable, it creates and removes a
//!    temporary file to find out — because a program that assumes it can
//!    write beside itself *"works perfectly on the developer's machine and
//!    fails the first time someone installs it under `Program Files`"*.
//!    That probe is exactly as necessary for a layout file as for a
//!    settings file, and it is not worth writing twice.
//! 3. **[`StoreKind::None`] stays a usable state.** No writable location
//!    at all is not a failure to start: the defaults load, the session
//!    runs, the dock is arrangeable, and only saving is impossible. See
//!    [`LayoutStore::can_save`].
//!
//! What is deliberately **not** shared with the settings file is the file
//! itself. `settings.txt` is a flat `key = value` grammar written by hand;
//! a layout is a tree, and RON is what `egui-shell` serializes it as.
//! Two files, one directory.
//!
//! ## When it is written
//!
//! On change — [`egui_shell::dock::DockFrameReport::layout_changed`] —
//! **debounced**, and with a ceiling on the debounce. Three requirements
//! pull in different directions and the schedule satisfies all three:
//!
//! | Requirement | What it rules out |
//! |---|---|
//! | not every frame | a splitter drag reports a change on every frame of the gesture; writing per frame is one file write per frame of a drag |
//! | not only at exit | *"a crash must not cost the arrangement"* — and the benchmarked application's file being *"rewritten on every exit"* is precisely what makes its community's copy-the-file-aside workaround race |
//! | eventually, unconditionally | a debounce that is re-armed by every change can be starved forever by a slow continuous drag |
//!
//! So: a change arms a deadline at `last change + `[`SAVE_SETTLE`], capped
//! at `first unsaved change + `[`SAVE_MAX_DEFER`]. [`LayoutStore::tick`]
//! writes when the deadline passes and otherwise reports how long is left,
//! so the caller can ask `egui` for a repaint then — without which an idle
//! window would sit on an unsaved change until the operator happened to
//! move the mouse. That is the same shape, for the same reason, as the
//! zoom debounce in [`crate::app::state`], which schedules its own wake-up
//! with `request_repaint_after`.
//!
//! ## Fail-soft, and disclosed — but not popped up
//!
//! Reading **never fails**: `LayoutDocument::from_ron` drops what it cannot
//! use, item by item, and returns a [`LoadReport`] saying what went. This
//! module carries that report rather than consuming it, because the surface
//! that should say so is the status bar and this is not it. Two rules the
//! caller inherits:
//!
//! - **A missing file is a first run and is not news.**
//!   `LoadReport::is_noteworthy` already excludes it, and
//!   [`LayoutStore::is_noteworthy`] forwards to that rather than
//!   re-deciding. An application that announced every skip would tell every
//!   operator, on the first launch of a fresh profile, that their layout
//!   could not be restored — from a profile that never had one.
//! - **Never a dialog.** A layout is not worth interrupting anybody for.
//!
//! ## ★ A dropped panel is never written back
//!
//! `SHELL_FRAMEWORK.md` §5b: *a capability's presence is expressed by
//! registering it, and by nothing else.* A build compiled without some
//! capability registers no panel for it, so a saved layout naming that
//! panel loses **that tab** on load, with a
//! [`egui_shell::layout::LayoutSkipReason::UnknownPanel`] disclosing it,
//! and keeps everything else.
//!
//! The half that belongs to *this* module is the write path: what is saved
//! is [`LayoutStore::document`], which is the **sanitized** document — the
//! one the loader already pruned — updated from a live
//! [`egui_shell::dock::DockState`] that by construction cannot contain a
//! panel the dock never drew. There is no path by which an id that was
//! dropped on load can reappear in the file, and
//! `a_dropped_panel_is_not_written_back_into_the_file` is the test that
//! says so.
//!
//! The honest consequence, stated rather than discovered: **the entry is
//! then gone for good.** Running a reduced build once and rearranging
//! anything costs the full build's tab for that capability, permanently.
//! The alternative — preserving unknown ids and re-emitting them — was
//! rejected because it makes the file accumulate entries nothing can ever
//! validate, and because a stale id that survives a *rename* would then
//! outlive every migration.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use egui_shell::dock::{DockLayout, PanelCatalog};
use egui_shell::layout::{LayoutDocument, LoadReport};
use pdfcer_core::settings::{self, StoreKind};

/// The layout file's name, inside the settings directory.
///
/// `.ron` because that is what `egui-shell` serializes a
/// [`LayoutDocument`] as, and because an operator may open it: RON keeps
/// real enum names, comments and trailing commas, which a layout file
/// benefits from for the same reason the shell manifest does.
pub const LAYOUT_FILE: &str = "layout.ron"; // ui-text-exempt: a file name, never displayed as copy

/// How long the layout must stop changing before it is written.
///
/// A splitter drag reports a change on every frame of the gesture, so this
/// is what turns one gesture into one write. It is short enough that
/// letting go of a splitter and pulling the power cord a second later
/// still keeps the arrangement, and long enough that a two-second drag
/// costs one write rather than a hundred and twenty.
pub const SAVE_SETTLE: Duration = Duration::from_millis(750);

/// The longest a change may be deferred, however continuously the operator
/// keeps changing things.
///
/// Without a ceiling, [`SAVE_SETTLE`] is re-armed by every change and a
/// slow, continuous rearrangement can starve the write indefinitely — the
/// exact failure the "not only at exit" requirement exists to prevent,
/// reintroduced by the mechanism that was supposed to prevent it.
pub const SAVE_MAX_DEFER: Duration = Duration::from_secs(5);

/// When a change is waiting to be written.
///
/// Two instants rather than one, because the deadline is the *earlier* of
/// two independent promises: quiet for [`SAVE_SETTLE`], and never later
/// than [`SAVE_MAX_DEFER`] after the first unsaved change.
#[derive(Debug, Clone, Copy)]
struct Pending {
    /// When the first still-unwritten change happened.
    first: Instant,
    /// When the most recent change happened.
    last: Instant,
}

impl Pending {
    /// The instant at which this must be written.
    fn due(self) -> Instant {
        (self.last + SAVE_SETTLE).min(self.first + SAVE_MAX_DEFER)
    }
}

/// The dock layout's home on disk: where it is, what it says, and what the
/// load could not carry across.
///
/// Held by the application for the whole session. Cheap to construct once
/// and never again — [`Self::load`] performs the writability probe and one
/// file read, and nothing after that touches the filesystem except a save.
#[derive(Debug)]
pub struct LayoutStore {
    /// Where the file is, or `None` when no writable location exists.
    ///
    /// `None` is a working state, not an error: everything loads from
    /// defaults and only saving is impossible. See [`Self::can_save`].
    path: Option<PathBuf>,
    /// Which of `pdfcer-core`'s two homes this is, carried so a diagnostic
    /// or a settings surface can say *which* — the operator's update
    /// procedure differs between them.
    kind: StoreKind,
    /// The live arrangement and every named workspace.
    document: LayoutDocument,
    /// What the load could not carry across. See the module header.
    report: LoadReport,
    /// The unwritten change, if there is one.
    pending: Option<Pending>,
    /// Why the last write failed, already rendered.
    ///
    /// A `String` rather than the error, because
    /// [`egui_shell::layout::LayoutError`] is not `Clone` and the only
    /// thing anybody does with it here is show it. Rendering it at the
    /// point of failure also captures the `io::Error`'s own account, which
    /// is the actionable half ("access is denied", "the device is full").
    save_error: Option<String>,
    /// How many times the file has been written this session.
    ///
    /// Diagnostic, and the thing a test asserts against to prove the
    /// debounce actually debounces — "it saved" is satisfied by saving
    /// sixty times.
    saves: u64,
}

impl Default for LayoutStore {
    /// A store with nowhere to write and nothing loaded.
    ///
    /// Exists for one concrete reason rather than for symmetry:
    /// [`crate::app::PdfcerApp`] derives [`Default`], and a field whose type
    /// has no `Default` would break that derive — turning "hold the layout
    /// store on the application" into a refactor of every place an app is
    /// constructed.
    ///
    /// It is deliberately the same state as [`StoreKind::None`]: an empty
    /// document, no path, and saving that quietly does nothing. That is the
    /// honest meaning of "a store nobody has loaded", and it is a state the
    /// rest of the module already handles — see
    /// `a_store_with_nowhere_to_write_still_loads_and_runs`. What it must
    /// **not** be is a store pointing at the real layout file with an empty
    /// document, which would overwrite the operator's arrangement the first
    /// time anything armed a write.
    fn default() -> Self {
        Self {
            path: None,
            kind: StoreKind::None,
            document: LayoutDocument::default(),
            report: LoadReport::default(),
            pending: None,
            save_error: None,
            saves: 0,
        }
    }
}

impl LayoutStore {
    /// Load the layout from the directory `pdfcer-core` puts settings in.
    ///
    /// **Never fails.** A missing file, an unreadable one, broken syntax
    /// or a newer schema all yield `fallback` with a reason recorded in
    /// [`Self::report`]; a panel `catalog` does not recognise loses its tab
    /// and nothing else. See the module header.
    ///
    /// `fallback` is the arrangement a fresh profile starts with — for
    /// pdfcer, [`crate::app::modes::layout_for_build`] of the mode the
    /// application opens in. `catalog` must be the **real** panel registry:
    /// passing [`egui_shell::dock::AnyPanel`] would disable the check that
    /// turns a mount for a compiled-out capability into a disclosed skip
    /// rather than an empty compartment.
    #[must_use]
    pub fn load(fallback: &DockLayout, catalog: &dyn PanelCatalog) -> Self {
        let store = settings::resolve_store();
        let path = store.directory().map(|dir| dir.join(LAYOUT_FILE));
        Self::at(path, store.kind, fallback, catalog)
    }

    /// Load from an explicit directory.
    ///
    /// The twin of `pdfcer_core::settings::store_in`, and it exists for the
    /// same two reasons: tests, and a future `--user-data-dir` override.
    /// It reports [`StoreKind::Portable`] because an explicitly named
    /// directory is portable by definition — it travels with whatever the
    /// operator pointed at.
    #[must_use]
    pub fn load_in(dir: &Path, fallback: &DockLayout, catalog: &dyn PanelCatalog) -> Self {
        Self::at(
            Some(dir.join(LAYOUT_FILE)),
            StoreKind::Portable,
            fallback,
            catalog,
        )
    }

    /// The shared body of the two constructors.
    fn at(
        path: Option<PathBuf>,
        kind: StoreKind,
        fallback: &DockLayout,
        catalog: &dyn PanelCatalog,
    ) -> Self {
        // No writable location: the defaults load and the session runs.
        // Deliberately no skip is recorded — nothing was *skipped*, there
        // was nowhere to look — and `can_save` is what a surface asks.
        let Some(path) = path else {
            return Self {
                path: None,
                kind,
                document: LayoutDocument::new(fallback.clone()),
                report: LoadReport::default(),
                pending: None,
                save_error: None,
                saves: 0,
            };
        };

        let loaded = LayoutDocument::load_from_path(&path, fallback, catalog);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "layout-load path={:?} kind={:?} workspaces={} skipped={} noteworthy={}",
                path,
                kind,
                loaded.document.workspaces.len(),
                loaded.report.len(),
                loaded.report.is_noteworthy(),
            )
        });
        for skip in loaded.report.skips() {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "layout-skip {skip}"
                )
            });
        }

        Self {
            path: Some(path),
            kind,
            document: loaded.document,
            report: loaded.report,
            pending: None,
            save_error: None,
            saves: 0,
        }
    }

    /// The path the layout would be read from and written to, if there is
    /// one.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// The path [`Self::load`] resolves, without loading anything.
    ///
    /// Exists so the location convention is assertable: it is derived from
    /// the same `pdfcer_core::settings::resolve_store()` call that decides
    /// where `settings.txt` goes, so the two cannot drift.
    #[must_use]
    pub fn default_path() -> Option<PathBuf> {
        settings::resolve_store()
            .directory()
            .map(|dir| dir.join(LAYOUT_FILE))
    }

    /// Which of `pdfcer-core`'s homes this store is using.
    #[must_use]
    pub fn kind(&self) -> StoreKind {
        self.kind
    }

    /// Whether a save can be attempted at all.
    ///
    /// `false` means no writable location was found — a state in which
    /// everything else works. A status surface may want to say so **once**,
    /// on the first change the operator makes, rather than at start-up:
    /// nobody cares that their layout cannot be saved until they have
    /// arranged something.
    #[must_use]
    pub fn can_save(&self) -> bool {
        self.path.is_some()
    }

    /// What the load could not carry across.
    ///
    /// Returned rather than rendered: the shell has no business deciding
    /// how the application words a note to its operator, and a surface that
    /// wants to offer "remove this stale entry" needs the structured skip
    /// rather than a sentence containing it. Every [`egui_shell::layout::LayoutSkip`]
    /// also implements `Display` for the diagnostic case.
    #[must_use]
    pub fn report(&self) -> &LoadReport {
        &self.report
    }

    /// Whether the load lost anything an operator would want to hear about.
    ///
    /// Forwards to `LoadReport::is_noteworthy`, which excludes "there was
    /// no file" — a first run is not a failure. Deliberately a forward
    /// rather than a re-implementation: one definition of "worth saying".
    #[must_use]
    pub fn is_noteworthy(&self) -> bool {
        self.report.is_noteworthy()
    }

    /// The whole document: the live arrangement and every named workspace.
    #[must_use]
    pub fn document(&self) -> &LayoutDocument {
        &self.document
    }

    /// The document, mutably — how a workspace is saved, renamed or
    /// deleted.
    ///
    /// **Arms a write.** Handing out `&mut` means this module cannot see
    /// what was changed, so it assumes something was; a caller that takes
    /// the borrow and changes nothing costs one file write. That is the
    /// right way round: a missed write loses an operator's arrangement, and
    /// a spurious one costs a few kilobytes.
    pub fn document_mut(&mut self) -> &mut LayoutDocument {
        self.arm(Instant::now());
        &mut self.document
    }

    /// The live arrangement.
    #[must_use]
    pub fn active(&self) -> &DockLayout {
        &self.document.active
    }

    /// The mode in force when this file was last written, if any.
    ///
    /// `None` covers three real cases and the caller must treat them alike —
    /// see [`egui_shell::layout::LayoutDocument::active_mode`]. So must an id
    /// this build's manifest no longer declares, which is why the caller checks
    /// `Modes::is_known` rather than trusting what it reads here.
    #[must_use]
    pub fn active_mode(&self) -> Option<&str> {
        self.document.active_mode.as_deref()
    }

    /// Record which mode is in force, arming a write if it actually changed.
    ///
    /// [`Self::record_active`]'s twin, and the equality check is there for the
    /// same reason: `Modes::on_mode_changed` may be driven from the ribbon's
    /// state every frame, so re-recording the mode already recorded must cost
    /// nothing. Without the check, every frame would arm the debounce and the
    /// ceiling in [`SAVE_MAX_DEFER`] would turn an idle application into one
    /// that writes its layout file every five seconds forever.
    ///
    /// Returns whether anything changed.
    pub fn record_active_mode(&mut self, mode_id: &str) -> bool {
        if self.document.active_mode.as_deref() == Some(mode_id) {
            return false;
        }
        self.document.active_mode = Some(mode_id.to_owned());
        self.arm(Instant::now());
        true
    }

    /// Record the live arrangement, arming a write if it actually moved.
    ///
    /// Called when the dock reports
    /// [`egui_shell::dock::DockFrameReport::layout_changed`]. The equality
    /// check is what keeps a caller honest: a frame that reports a change
    /// which nets out to nothing — a splitter dragged one way and back
    /// within the frame, a menu that closed without acting — does not cost
    /// a write.
    ///
    /// Returns whether anything changed.
    pub fn record_active(&mut self, layout: &DockLayout) -> bool {
        self.record_active_at(layout, Instant::now())
    }

    /// [`Self::record_active`], against a supplied clock.
    ///
    /// The debounce is a **schedule**, and a schedule is only assertable
    /// against a clock the test controls: the alternative is a suite that
    /// sleeps, which is slow, flaky, and still cannot reach
    /// [`SAVE_MAX_DEFER`] without taking five real seconds. Both halves of
    /// the schedule therefore take their instant from the caller — this and
    /// [`Self::tick`] — and a caller that mixes the wall clock into one and
    /// a synthetic instant into the other gets nonsense, which is exactly
    /// why they are spelled differently.
    ///
    /// Public rather than test-only because a harness driving the
    /// application frame by frame is a real second caller, and a
    /// `#[cfg(test)]` seam is one such harness cannot use.
    pub fn record_active_at(&mut self, layout: &DockLayout, now: Instant) -> bool {
        if self.document.active == *layout {
            return false;
        }
        self.document.active = layout.clone();
        self.arm(now);
        true
    }

    /// Arm the debounce. See [`Pending`].
    fn arm(&mut self, now: Instant) {
        self.pending = Some(match self.pending {
            Some(p) => Pending {
                first: p.first,
                last: now,
            },
            None => Pending {
                first: now,
                last: now,
            },
        });
    }

    /// Whether a change is waiting to be written.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.pending.is_some()
    }

    /// When the outstanding change will be written, if there is one.
    ///
    /// The schedule, exposed rather than inferred: a diagnostic surface can
    /// say *"unsaved, writing in 0.4 s"* and a test can assert the deadline
    /// itself instead of guessing at it from the outside.
    #[must_use]
    pub fn due_at(&self) -> Option<Instant> {
        self.pending.map(Pending::due)
    }

    /// Write if the debounce has expired; otherwise say how long is left.
    ///
    /// Call once per frame with `Instant::now()`. A `Some(remaining)`
    /// answer is a request for another frame at about that time —
    /// `ctx.request_repaint_after(remaining)` — because nothing else will
    /// wake `egui` on an idle window and the change would otherwise sit
    /// unwritten until the operator moved the mouse. `None` means there is
    /// nothing outstanding.
    ///
    /// Takes `now` rather than reading the clock so the schedule is
    /// testable without sleeping.
    pub fn tick(&mut self, now: Instant) -> Option<Duration> {
        let pending = self.pending?;
        let due = pending.due();
        if now < due {
            return Some(due - now);
        }
        self.write();
        None
    }

    /// Write immediately, if anything is outstanding.
    ///
    /// For an exit path, which must not lose the last change to a debounce
    /// that had not yet expired. Returns whether a write was attempted;
    /// [`Self::save_error`] says whether it succeeded.
    pub fn flush(&mut self) -> bool {
        if self.pending.is_none() {
            return false;
        }
        self.write();
        true
    }

    /// Perform the write, clearing the pending state either way.
    ///
    /// **A failure clears the pending state too**, deliberately. Retrying
    /// every frame against a path that cannot be written — a read-only
    /// share, a full disk — burns the write attempt sixty times a second
    /// to produce the same error, and the operator has already been told.
    /// The next actual change re-arms it, which is the same posture
    /// [`crate::app::PdfcerApp::settle_and_rasterize`] takes towards a page
    /// that would not render.
    fn write(&mut self) {
        self.pending = None;
        let Some(path) = self.path.as_ref() else {
            return;
        };
        match self.document.save_to_path(path) {
            Ok(()) => {
                self.saves += 1;
                self.save_error = None;
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "layout-save path={:?} workspaces={} n={}",
                        path,
                        self.document.workspaces.len(),
                        self.saves,
                    )
                });
            }
            Err(error) => {
                let rendered = error.to_string();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "layout-save-failed path={path:?} error={rendered}"
                    )
                });
                self.save_error = Some(rendered);
            }
        }
    }

    /// Why the last write failed, if it did.
    ///
    /// A real `Result` on the way in, because a failed save is something
    /// the operator must be told about — they are about to close an
    /// application believing their arrangement is safe. Cleared by the next
    /// successful write.
    #[must_use]
    pub fn save_error(&self) -> Option<&str> {
        self.save_error.as_deref()
    }

    /// How many times the file has been written this session.
    ///
    /// Diagnostic. A test asserting "the debounce works" needs a count,
    /// not a boolean: "it saved" is satisfied by saving on every frame.
    #[must_use]
    pub fn saves(&self) -> u64 {
        self.saves
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::dock::{
        AnyPanel, Column, DockLayout, PanelId, PanelInfo, PanelRegistry, SideLayout, Stack,
    };

    /// A registry holding exactly the panels a hypothetical build "offers".
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

    /// A fresh, empty directory nothing else is using.
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("pdfcer-gui-layout-{tag}-{nanos}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        dir
    }

    /// ★ **The layout file sits beside the settings file, in the directory
    /// `pdfcer-core` chose.**
    ///
    /// The location rule, asserted against `pdfcer-core`'s own resolution
    /// rather than against a path spelled out here — a second spelling is
    /// exactly how a layout file ends up in a different folder from the
    /// settings file it was supposed to sit next to.
    #[test]
    fn the_layout_file_lives_beside_the_settings_file() {
        let dir = temp_dir("beside");
        let store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);
        let settings = settings::store_in(&dir).path.expect("an explicit store");

        assert_eq!(
            store.path().and_then(Path::parent),
            settings.parent(),
            "the two files must share a directory"
        );
        assert_eq!(store.path(), Some(dir.join(LAYOUT_FILE).as_path()));

        // …and the no-argument constructor derives its directory from the
        // same call, rather than computing one of its own.
        assert_eq!(
            LayoutStore::default_path(),
            settings::resolve_store()
                .directory()
                .map(|d| d.join(LAYOUT_FILE)),
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **A first run is not news.**
    ///
    /// A missing file yields the fallback, records the fact, and reports
    /// nothing worth telling anybody — an application that announced this
    /// would tell every operator on every fresh profile that their layout
    /// could not be restored.
    #[test]
    fn a_first_run_loads_the_fallback_and_says_nothing() {
        let dir = temp_dir("first-run");
        let store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);

        assert_eq!(store.active(), &fallback());
        assert!(!store.is_noteworthy(), "a first run is not a failure");
        assert!(store.can_save());
        assert!(!store.is_dirty(), "loading is not a change");
        assert_eq!(store.saves(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **Broken syntax falls back, discloses, and does not interrupt.**
    ///
    /// The one genuinely wholesale case — a parser cannot say which half of
    /// a broken file was meant — and it must still be a *disclosure*, never
    /// a dialog and never a silent reset.
    #[test]
    fn broken_syntax_falls_back_and_is_reported() {
        let dir = temp_dir("broken");
        std::fs::write(dir.join(LAYOUT_FILE), "LayoutDocument( schema: ").expect("writes");

        let store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);
        assert_eq!(store.active(), &fallback());
        assert!(store.is_noteworthy(), "the operator should hear about this");
        assert!(!store.report().is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An unreadable file is a different fact from a missing one, and both
    /// keep the session running.
    #[test]
    fn a_directory_where_the_file_should_be_is_survived() {
        let dir = temp_dir("unreadable");
        // A directory named `layout.ron` cannot be read as a file, on every
        // platform, without needing permissions the test cannot set.
        std::fs::create_dir_all(dir.join(LAYOUT_FILE)).expect("a decoy directory");

        let store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);
        assert_eq!(store.active(), &fallback());
        assert!(store.is_noteworthy());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A saved arrangement comes back on the next session, through a real
    /// file — the property the whole module exists for.
    #[test]
    fn an_arrangement_survives_a_restart() {
        let dir = temp_dir("round-trip");
        {
            let mut store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);
            assert!(store.record_active(&rich()));
            assert!(store.flush(), "a change was outstanding");
            assert_eq!(store.saves(), 1);
            assert_eq!(store.save_error(), None);
        }
        let reopened = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);
        assert_eq!(reopened.active(), &rich());
        assert!(reopened.report().is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **A panel this build does not offer loses its tab — and the save
    /// does not put it back.**
    ///
    /// `SHELL_FRAMEWORK.md` §5b from both sides. The loader drops the
    /// unregistered mount and discloses it; the write path emits the
    /// sanitized document, so there is no route by which the dropped id
    /// reappears in the file. Asserted on the *text on disk*, because an
    /// assertion on the in-memory document would pass even if the writer
    /// re-emitted something it had kept aside.
    #[test]
    fn a_dropped_panel_is_not_written_back_into_the_file() {
        let dir = temp_dir("dropped");
        let saved = LayoutDocument::new(rich());
        std::fs::write(dir.join(LAYOUT_FILE), saved.to_ron_pretty().expect("ron"))
            .expect("writes the previous session's file");

        // This "build" has no `tools` panel.
        let catalog = registry(&["pages", "layers", "bookmarks", "objects"]);
        let mut store = LayoutStore::load_in(&dir, &fallback(), &catalog);
        assert!(store.is_noteworthy(), "the drop is disclosed");
        assert!(!store.active().contains(&PanelId::new("tools")));
        assert!(
            store.active().contains(&PanelId::new("layers")),
            "and only that one tab went"
        );

        // Any change at all rewrites the file.
        let mut moved = store.active().clone();
        moved.left.width_pts = 411.0;
        assert!(store.record_active(&moved));
        store.flush();

        let text = std::fs::read_to_string(dir.join(LAYOUT_FILE)).expect("reads back");
        assert!(
            !text.contains("tools"),
            "the unregistered panel was written back: {text}"
        );
        assert!(text.contains("layers"), "and everything else survived");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **One gesture is one write, not one write per frame.**
    ///
    /// A splitter drag reports a change every frame. Sixty ticks inside the
    /// settle window must produce no writes at all; the write happens once,
    /// after the gesture stops.
    #[test]
    fn a_continuous_drag_costs_one_write_rather_than_one_per_frame() {
        let dir = temp_dir("debounce");
        let mut store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);

        // One synthetic clock for both halves of the schedule — the whole
        // reason `record_active_at` exists.
        let start = Instant::now();
        let mut layout = fallback();
        let mut at = start;
        for frame in 1..=60_u32 {
            at = start + Duration::from_millis(u64::from(frame) * 16);
            layout.left.width_pts = 200.0 + f32::from(u16::try_from(frame).expect("small"));
            assert!(store.record_active_at(&layout, at));
            assert!(
                store.tick(at).is_some(),
                "frame {frame} is still within the settle window"
            );
        }
        assert_eq!(store.saves(), 0, "not one write during the gesture");

        // The gesture stops. One tick just before the deadline still writes
        // nothing; one at the deadline writes exactly once.
        let due = store.due_at().expect("a change is outstanding");
        assert_eq!(due, at + SAVE_SETTLE, "the settle window did not re-arm");
        assert!(store.tick(due - Duration::from_millis(1)).is_some());
        assert_eq!(store.saves(), 0);
        assert_eq!(store.tick(due), None);
        assert_eq!(store.saves(), 1);
        assert!(!store.is_dirty());
        // An idle tick afterwards writes nothing more.
        assert_eq!(store.tick(due + SAVE_SETTLE), None);
        assert_eq!(store.saves(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **A change cannot be deferred forever.**
    ///
    /// A continuous rearrangement re-arms the settle window on every frame,
    /// which without a ceiling would starve the write for as long as the
    /// operator keeps dragging — reintroducing the "only saved at exit"
    /// failure through the mechanism meant to prevent it.
    #[test]
    fn an_endless_gesture_still_gets_written_within_the_ceiling() {
        let dir = temp_dir("ceiling");
        let mut store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);

        const FRAME: Duration = Duration::from_millis(16);
        let start = Instant::now();
        let mut layout = fallback();
        let mut first_change = None;
        let mut wrote_at = None;
        for frame in 1..=1_000_u32 {
            let at = start + FRAME * frame;
            layout.left.width_pts = 200.0 + f32::from(u16::try_from(frame % 97).expect("small"));
            if store.record_active_at(&layout, at) && first_change.is_none() {
                first_change = Some(at);
            }
            if store.tick(at).is_none() && wrote_at.is_none() {
                wrote_at = Some(at);
            }
        }
        let waited = wrote_at.expect("a write must have happened")
            - first_change.expect("a change must have been recorded");
        assert!(
            // One frame of slack: the write happens on the first *tick* at
            // or after the deadline, which can be up to a frame late.
            waited <= SAVE_MAX_DEFER + FRAME,
            "the ceiling did not bite: the first write was {waited:?} after the first change"
        );
        assert!(store.saves() >= 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A frame reporting a change that nets out to nothing costs no write.
    #[test]
    fn a_change_that_changes_nothing_arms_nothing() {
        let dir = temp_dir("no-op");
        let mut store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);
        assert!(!store.record_active(&fallback()));
        assert!(!store.is_dirty());
        assert!(!store.flush(), "nothing to flush");
        assert_eq!(store.saves(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Named workspaces round-trip with the live arrangement, and taking
    /// the document mutably arms a write.
    #[test]
    fn a_workspace_saved_through_the_store_survives_a_restart() {
        let dir = temp_dir("workspaces");
        {
            let mut store = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);
            assert!(!store.is_dirty());
            store.document_mut().save_workspace("Marking up", rich());
            assert!(store.is_dirty(), "handing out `&mut` arms a write");
            store.flush();
        }
        let reopened = LayoutStore::load_in(&dir, &fallback(), &AnyPanel);
        assert_eq!(reopened.document().workspace_names(), vec!["Marking up"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **No writable location is a working session, not a failure.**
    ///
    /// `StoreKind::None` is what `pdfcer-core` returns when neither the
    /// portable directory nor the platform one can be written. Everything
    /// loads from defaults, the dock is arrangeable, and only saving is
    /// impossible — and a caller can find that out before promising the
    /// operator anything.
    #[test]
    fn a_store_with_nowhere_to_write_still_loads_and_runs() {
        let mut store = LayoutStore::at(None, StoreKind::None, &fallback(), &AnyPanel);
        assert!(!store.can_save());
        assert_eq!(store.path(), None);
        assert_eq!(store.active(), &fallback());
        assert!(
            !store.is_noteworthy(),
            "nothing was skipped; there was no file"
        );

        // Arranging still works; the write is simply a no-op that fails
        // quietly rather than an error the operator must dismiss.
        assert!(store.record_active(&rich()));
        assert!(store.flush());
        assert_eq!(store.saves(), 0);
        assert_eq!(store.save_error(), None);
    }

    /// ★ **A default store cannot overwrite an operator's layout.**
    ///
    /// `Default` exists so `PdfcerApp` can keep deriving it. The hazard that
    /// buys is a store that looks loaded, holds an empty document, and
    /// points at the real file — which would erase the operator's
    /// arrangement the first time anything armed a write. It points nowhere
    /// instead.
    #[test]
    fn a_default_store_points_nowhere_and_can_erase_nothing() {
        let mut store = LayoutStore::default();
        assert_eq!(store.path(), None);
        assert!(!store.can_save());
        assert!(store.document().workspaces.is_empty());
        assert!(store.record_active(&rich()));
        assert!(store.flush());
        assert_eq!(store.saves(), 0, "a default store must write nothing");
    }

    /// A write into a path that cannot exist records the reason and does
    /// not retry on every subsequent frame.
    #[test]
    fn a_failed_write_is_reported_once_and_not_retried_every_frame() {
        let dir = temp_dir("unwritable");
        // A file where the parent directory must be — `create_dir_all`
        // cannot make a directory out of a regular file, on any platform.
        let blocker = dir.join("blocked");
        std::fs::write(&blocker, "not a directory").expect("writes");

        let mut store = LayoutStore::load_in(&blocker.join("nested"), &fallback(), &AnyPanel);
        assert!(store.record_active(&rich()));
        assert!(store.flush());
        assert!(store.save_error().is_some(), "the reason is kept");
        assert_eq!(store.saves(), 0);
        assert!(
            !store.is_dirty(),
            "a permanent failure must not be retried on every frame"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
