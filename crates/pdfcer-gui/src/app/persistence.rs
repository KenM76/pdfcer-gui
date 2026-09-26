//! # `app::persistence` — the dock layout, on disk
//!
//! `egui-shell` can already read and write a [`LayoutDocument`]; what it
//! deliberately does **not** do is decide *where* the file lives or *when*
//! it is written. Its own header says so in as many words:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/persistence.md`.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use egui_shell::dock::{DockLayout, PanelCatalog};
use egui_shell::layout::{LayoutDocument, LoadReport};
use pdfcer_core::settings::{self, StoreKind};

/// The layout file's name, inside the settings directory.
pub const LAYOUT_FILE: &str = "layout.ron"; // ui-text-exempt: a file name, never displayed as copy

/// How long the layout must stop changing before it is written.
pub const SAVE_SETTLE: Duration = Duration::from_millis(750);

/// The longest a change may be deferred, however continuously the operator
/// keeps changing things.
pub const SAVE_MAX_DEFER: Duration = Duration::from_secs(5);

/// When a change is waiting to be written.
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
    #[must_use]
    pub fn load(fallback: &DockLayout, catalog: &dyn PanelCatalog) -> Self {
        let store = settings::resolve_store();
        let path = store.directory().map(|dir| dir.join(LAYOUT_FILE));
        Self::at(path, store.kind, fallback, catalog)
    }

    /// Load from an explicit directory.
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
    #[must_use]
    pub fn can_save(&self) -> bool {
        self.path.is_some()
    }

    /// What the load could not carry across.
    #[must_use]
    pub fn report(&self) -> &LoadReport {
        &self.report
    }

    /// Whether the load lost anything an operator would want to hear about.
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
    #[must_use]
    pub fn active_mode(&self) -> Option<&str> {
        self.document.active_mode.as_deref()
    }

    /// Record which mode is in force, arming a write if it actually changed.
    pub fn record_active_mode(&mut self, mode_id: &str) -> bool {
        if self.document.active_mode.as_deref() == Some(mode_id) {
            return false;
        }
        self.document.active_mode = Some(mode_id.to_owned());
        self.arm(Instant::now());
        true
    }

    /// Record the live arrangement, arming a write if it actually moved.
    pub fn record_active(&mut self, layout: &DockLayout) -> bool {
        self.record_active_at(layout, Instant::now())
    }

    /// [`Self::record_active`], against a supplied clock.
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
    #[must_use]
    pub fn due_at(&self) -> Option<Instant> {
        self.pending.map(Pending::due)
    }

    /// Write if the debounce has expired; otherwise say how long is left.
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
    pub fn flush(&mut self) -> bool {
        if self.pending.is_none() {
            return false;
        }
        self.write();
        true
    }

    /// Perform the write, clearing the pending state either way.
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
        // The pid is what `tools/gates/check-test-temp-paths.py` requires;
        // `nanos` stays because it also separates repeated runs inside one
        // process, which the pid does not.
        let dir = std::env::temp_dir().join(format!(
            "pdfcer-gui-layout-{tag}-{nanos}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        dir
    }

    /// **The layout file sits beside the settings file, in the directory
    /// `pdfcer-core` chose.**
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

    /// **A first run is not news.**
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

    /// **Broken syntax falls back, discloses, and does not interrupt.**
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

    /// **A panel this build does not offer loses its tab — and the save
    /// does not put it back.**
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

    /// **One gesture is one write, not one write per frame.**
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

    /// **A change cannot be deferred forever.**
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

    /// **No writable location is a working session, not a failure.**
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

    /// **A default store cannot overwrite an operator's layout.**
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
