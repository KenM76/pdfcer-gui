//! # `recentfiles` — the documents this operator had open, on disk
//!
//! `GUI_ROADMAP.md` Phase 3 asks for a recent-files list. Nothing in the
//! salvaged shell wrote one — `grep -i recent` over the old crate returns
//! nothing at all — so this is new rather than carried across, and every
//! decision below is therefore made here for the first time and written
//! down.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/recentfiles.md`.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use pdfcer_core::settings;

/// The list's file name, inside the settings directory.
pub const RECENT_FILE: &str = "recent.txt"; // ui-text-exempt: a file name, never displayed as copy

/// How many documents are remembered.
pub const CAP: usize = 10;

/// How long a presence check is trusted before it is taken again.
pub const PRESENCE_TTL: Duration = Duration::from_secs(2);

/// **The documents this operator had open, newest first.**
#[derive(Debug, Default)]
pub struct RecentFiles {
    /// Where the file is, or `None` when no writable location exists.
    ///
    /// `None` is a working state, not an error: the list simply does not
    /// survive the session. See [`Self::can_save`].
    path: Option<PathBuf>,
    /// The paths, newest first, absolute, de-duplicated, capped at [`CAP`].
    ///
    /// **Never filtered for existence.** See the module header.
    entries: Vec<PathBuf>,
    /// Whether each entry existed at the last sweep, parallel to
    /// [`Self::entries`], empty before the first sweep.
    present: Vec<bool>,
    /// When that sweep ran, or `None` if it has not.
    checked_at: Option<Instant>,
    /// Why the last write failed, already rendered.
    ///
    /// A `String` rather than the `io::Error` because the only thing anybody
    /// does with it is show it or trace it, and rendering it at the point of
    /// failure captures the operating system's own account ("access is
    /// denied", "the device is full"), which is the actionable half.
    save_error: Option<String>,
    /// How many times the file has been written this session.
    ///
    /// Diagnostic, and what a test asserts against to prove a re-open of the
    /// same document does not cost a write.
    saves: u64,
}

impl RecentFiles {
    /// Load the list from the directory `pdfcer-core` puts settings in.
    #[must_use]
    pub fn load() -> Self {
        Self::at(
            settings::resolve_store()
                .directory()
                .map(|dir| dir.join(RECENT_FILE)),
        )
    }

    /// Load from an explicit directory.
    #[must_use]
    pub fn load_in(dir: &Path) -> Self {
        Self::at(Some(dir.join(RECENT_FILE)))
    }

    /// The path [`Self::load`] resolves, without loading anything.
    #[must_use]
    pub fn default_path() -> Option<PathBuf> {
        settings::resolve_store()
            .directory()
            .map(|dir| dir.join(RECENT_FILE))
    }

    /// The shared body of both constructors.
    fn at(path: Option<PathBuf>) -> Self {
        let entries = path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|text| Self::parse(&text))
            .unwrap_or_default();
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "recent-load path={path:?} n={}",
                entries.len()
            )
        });
        Self {
            path,
            entries,
            present: Vec::new(),
            checked_at: None,
            save_error: None,
            saves: 0,
        }
    }

    /// Parse the file: one path per line, newest first.
    fn parse(text: &str) -> Vec<PathBuf> {
        let mut out: Vec<PathBuf> = Vec::new();
        for line in text.split('\n') {
            let line = line.trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                continue;
            }
            let path = PathBuf::from(line);
            if !out.contains(&path) {
                out.push(path);
            }
            if out.len() == CAP {
                break;
            }
        }
        out
    }

    /// Render the file. See [`Self::parse`].
    fn render(&self) -> String {
        let mut out = String::new();
        for path in &self.entries {
            if let Some(text) = path.to_str() {
                out.push_str(text);
                out.push('\n');
            }
        }
        out
    }

    /// The whole list, newest first — **including entries whose file is not
    /// there right now**.
    #[must_use]
    pub fn entries(&self) -> &[PathBuf] {
        &self.entries
    }

    /// Whether anything has ever been opened.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether a save can be attempted at all.
    ///
    /// `false` means no writable location was found — a state in which
    /// everything else works and only persistence is impossible.
    #[must_use]
    pub fn can_save(&self) -> bool {
        self.path.is_some()
    }

    /// Where the list is read from and written to, if anywhere.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Why the last write failed, if it did. Cleared by the next success.
    #[must_use]
    pub fn save_error(&self) -> Option<&str> {
        self.save_error.as_deref()
    }

    /// How many times the file has been written this session.
    #[must_use]
    pub fn saves(&self) -> u64 {
        self.saves
    }

    /// **Record that `path` was opened, and write the list.**
    ///
    /// Newest first: an entry already in the list **moves to the front**
    /// rather than being added twice, which is what makes the list a
    /// most-recently-used order rather than a log.
    ///
    /// # The path is absolutized, and that is load-bearing
    ///
    /// `pdfcer-gui drawing.pdf` gives `argv[1] = "drawing.pdf"`, and a
    /// relative path in a persisted list is a path that means something
    /// different — or nothing — the next time the application starts from a
    /// different directory. [`std::path::absolute`] resolves it against the
    /// current directory **without touching the filesystem**, which is
    /// exactly the right amount of work:
    /// [`std::fs::canonicalize`] would also resolve symlinks and, on Windows,
    /// return a `\\?\` extended-length path that no operator recognises and
    /// that reads badly in a menu.
    ///
    /// The consequence, stated rather than discovered: two spellings of one
    /// file (a mapped drive and its UNC path, a symlink and its target) are
    /// two entries. De-duplicating those needs `canonicalize`, which needs
    /// the file to *exist* — and a list that could only de-duplicate
    /// reachable files would drop the network entries this module goes out of
    /// its way to keep.
    ///
    /// Writes immediately rather than debouncing, unlike
    /// `pdfcer_gui::app::persistence::LayoutStore`: opening a document is a rare
    /// discrete event, not a drag that reports sixty changes a second, so
    /// there is no gesture to settle and nothing to be gained by deferring
    /// past a crash.
    pub fn remember(&mut self, path: &Path) {
        let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        // Already at the front: nothing about the list changes, so nothing is
        // written. Re-opening the same document repeatedly must not cost a
        // file write each time.
        if self.entries.first() == Some(&path) {
            return;
        }
        self.entries.retain(|existing| existing != &path);
        self.entries.insert(0, path);
        self.entries.truncate(CAP);
        // The presence cache is indexed positionally, so any change to the
        // list invalidates it wholesale. Cheap: the next sweep is at most
        // `CAP` stats, and it only happens if a menu is actually opened.
        self.invalidate();
        self.write();
    }

    /// Forget the cached presence answers.
    fn invalidate(&mut self) {
        self.present.clear();
        self.checked_at = None;
    }

    /// Perform the write.
    fn write(&mut self) {
        let Some(path) = self.path.as_ref() else {
            return;
        };
        let text = self.render();
        // The parent may not exist on a first run — `resolve_store` probes
        // for writability, not for existence.
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(path, text) {
            Ok(()) => {
                self.saves += 1;
                self.save_error = None;
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "recent-save path={path:?} n={} saves={}",
                        self.entries.len(),
                        self.saves
                    )
                });
            }
            Err(error) => {
                let rendered = error.to_string();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "recent-save-failed path={path:?} error={rendered}"
                    )
                });
                self.save_error = Some(rendered);
            }
        }
    }

    /// **The entries whose file can be seen right now, newest first.**
    pub fn present_at(&mut self, now: Instant) -> Vec<PathBuf> {
        let stale = self
            .checked_at
            .is_none_or(|at| now.saturating_duration_since(at) >= PRESENCE_TTL);
        if stale {
            self.present = self.entries.iter().map(|p| p.is_file()).collect();
            self.checked_at = Some(now);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "recent-presence n={} present={}",
                    self.entries.len(),
                    self.present.iter().filter(|p| **p).count()
                )
            });
        }
        self.entries
            .iter()
            .zip(&self.present)
            .filter(|(_, present)| **present)
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// The newest entry whose file can be seen right now.
    pub fn newest_present(&mut self, now: Instant) -> Option<PathBuf> {
        self.present_at(now).into_iter().next()
    }
}

// ===========================================================================
// The control
// ===========================================================================

/// **Draw the Recent control, and report the document the operator chose.**
pub fn menu(ui: &mut egui::Ui, recent: &mut RecentFiles, now: Instant) -> Option<PathBuf> {
    let text = crate::text::commands::file_recent();
    let mut chosen: Option<PathBuf> = None;
    ui.add_enabled_ui(!recent.is_empty(), |ui| {
        // `icons::image` takes its tint from THIS `Ui`'s `text_color()`, so
        // inside `add_enabled_ui(false, …)` the glyph fades in lockstep with
        // the word beside it and no disabled-state branch is needed here.
        let glyph = crate::icons::image(ui, crate::icons::Icon::Recent);
        let button = ui.menu_image_text_button(glyph, text.label, |ui| {
            // Inside the popup body, which `egui` runs only while the menu is
            // OPEN — the first half of what keeps the presence sweep off the
            // per-frame path. See the module header.
            let entries = recent.present_at(now);
            if entries.is_empty() {
                ui.add_enabled(false, egui::Button::new(crate::text::files::recent_empty()));
                return;
            }
            for path in entries {
                let row = ui
                    .button(crate::text::files::recent_entry_label(&path))
                    .on_hover_text(crate::text::files::recent_entry_tooltip(&path));
                if row.clicked() {
                    chosen = Some(path);
                    ui.close();
                }
            }
        });
        button
            .response
            .on_hover_text(text.tooltip)
            .on_disabled_hover_text(text.tooltip);
    });
    chosen
}
