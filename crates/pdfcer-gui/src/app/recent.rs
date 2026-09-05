//! # `app::recent` — the documents this operator had open, on disk
//!
//! `GUI_ROADMAP.md` Phase 3 asks for a recent-files list. Nothing in the
//! salvaged shell wrote one — `grep -i recent` over the old crate returns
//! nothing at all — so this is new rather than carried across, and every
//! decision below is therefore made here for the first time and written
//! down.
//!
//! One type, [`RecentFiles`]: a capped list of absolute paths, newest
//! first, persisted beside the layout and the settings, plus the small
//! amount of caching that keeps *"is this file still there?"* from costing
//! a blocking stat per frame.
//!
//! ## ★ Where the file lives, and why that is not this module's decision
//!
//! `<the settings directory>/recent.txt` — **beside `settings.txt` and
//! `layout.ron`**, and the directory is resolved by asking `pdfcer-core`,
//! exactly as [`crate::app::persistence::LayoutStore`] resolves it:
//!
//! ```text
//! pdfcer_core::settings::resolve_store()      →  StoreLocation { path, kind }
//!                        .directory()        →  <exe dir>/userdata      (Portable)
//!                                            or the platform config dir (PlatformFallback)
//!                                            or None                    (None)
//! ```
//!
//! There is deliberately no second resolution here, and
//! [`tests::the_recent_file_lives_beside_the_layout_file`] asserts it against
//! `LayoutStore`'s own answer rather than against a path spelled out twice.
//! The three properties that come with deferring are the ones
//! `persistence`'s header argues in full: one location decision rather than
//! two, the writability probe comes with it, and *no writable location at
//! all* stays a working session in which only saving is impossible (see
//! [`RecentFiles::can_save`]).
//!
//! ## ★ Why a flat text file rather than RON
//!
//! Because this crate **cannot serialize**. `serde` and `ron` are workspace
//! dependencies of `egui-shell`, not of `pdfcer-gui`, and `Cargo.toml` is not
//! this work's to edit. `LayoutStore` gets RON for free because
//! `egui_shell::layout::LayoutDocument` does its own serialization behind
//! the crate boundary; a list of paths has no such type and inventing one in
//! `egui-shell` would teach the reusable shell about a pdfcer feature, which
//! `tools/gates/check-shell-purity.sh` exists to prevent.
//!
//! So the format is the one `settings.txt` already uses in spirit: **one
//! path per line, newest first, UTF-8, no header and no comments.** Every
//! non-empty line is a path; nothing else is legal, so nothing else has to
//! be parsed. It is readable, hand-editable, diffable, and impossible to
//! half-parse — a corrupt file degrades into a shorter list rather than into
//! an error the operator has to dismiss.
//!
//! The one thing the format cannot carry is a path that is not valid
//! Unicode. Such a path is **dropped at save time**, and that is the single
//! place in this module where dropping at save is right: it is not a
//! judgement about whether the file still exists, it is the format saying it
//! cannot spell the name. See [`RecentFiles::render`].
//!
//! ## ★ Missing files are dropped at DISPLAY time, never at save time
//!
//! This is the rule the whole presence cache exists to serve, and it is a
//! statement about how drafting offices actually work:
//!
//! > A network drive that is temporarily absent is not a file the operator
//! > wants forgotten.
//!
//! A laptop off the VPN, a NAS still spinning up, a share that reconnects
//! when the operator logs in — every one of those makes `Path::exists`
//! return `false` about a file that is perfectly real and will be back in a
//! minute. Pruning the list on that answer destroys information the operator
//! cannot get back, permanently, in exchange for a tidier file nobody reads.
//!
//! So [`RecentFiles::entries`] is the whole list, [`RecentFiles::render`]
//! writes the whole list, and only [`RecentFiles::present_at`] — what the
//! menu draws — filters. Reconnect the drive and the entry comes back by
//! itself.
//!
//! ## ★ …and why the presence check is cached
//!
//! `Path::exists` on a **dead** network path is not fast. It is a blocking
//! call that can take seconds while SMB waits out its own timeout, and the
//! UI thread is the thread that would be waiting. Ten entries checked on
//! every frame of a 60 Hz repaint is not a performance question, it is a
//! frozen application.
//!
//! Two defences, and both are needed:
//!
//! 1. **The check only runs while the menu is open.** The caller is
//!    [`crate::app::PdfcerApp::ribbon_band`]'s custom-item renderer, which
//!    calls this from inside the popup body — a closure `egui` runs only
//!    while the popup is showing.
//! 2. **And then at most once per [`PRESENCE_TTL`].** A popup held open for
//!    three seconds is two sweeps, not a hundred and eighty.
//!
//! The residual cost is honest and stated rather than hidden: opening the
//! menu with a dead network entry in the list can block for as long as the
//! filesystem takes to answer, once every two seconds. Fixing that properly
//! means checking on a worker thread, which is a real answer and a larger
//! one than this feature is worth today.
//!
//! ## What is deliberately NOT here
//!
//! - **No "clear recent" command.** It would be one more registered command,
//!   one more manifest entry and one more surface, and the file is one line
//!   per entry in a folder the operator already owns. It is listed in
//!   `crate::shell::manifest::PLANNED` when somebody wants it.
//! - **No pinning.** Same argument, and pinning needs a second field in the
//!   file format, which is the point at which "one path per line" stops
//!   being the right format.
//! - **No per-document view state** (last page, last zoom). That is a
//!   different feature with a different lifetime and a different file; a
//!   recent list that quietly became a session store would be the thing
//!   nobody could later separate.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use pdfcer_core::settings;

/// The list's file name, inside the settings directory.
///
/// `.txt` because the format is one path per line and an operator who opens
/// it should find exactly what they expect. `.ron` would promise a structure
/// that is not there and a parser this crate does not have.
pub const RECENT_FILE: &str = "recent.txt"; // ui-text-exempt: a file name, never displayed as copy

/// How many documents are remembered.
///
/// Ten is what a File menu can show without becoming a scroll view, and it
/// is comfortably more than the two or three an operator actually reaches
/// for. A cap exists at all because the list is drawn: an uncapped list
/// eventually costs a stat sweep per open and a menu taller than the window.
pub const CAP: usize = 10;

/// How long a presence check is trusted before it is taken again.
///
/// Short enough that a drive that comes back appears in the menu on the next
/// look, long enough that holding the menu open does not re-stat a dead
/// network path on every frame. See the module header.
pub const PRESENCE_TTL: Duration = Duration::from_secs(2);

/// **The documents this operator had open, newest first.**
///
/// Held by [`crate::app::PdfcerApp`] for the whole session. Cheap to
/// construct once — [`Self::load`] performs `pdfcer-core`'s writability probe
/// and one file read — and nothing after that touches the filesystem except
/// a save on [`Self::remember`] and the throttled presence sweep in
/// [`Self::present_at`].
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
    ///
    /// **Never fails.** A missing file is a first run, an unreadable one is
    /// an empty list, and a line that is not a usable path is skipped. There
    /// is nothing in a recent-files list worth interrupting an operator for,
    /// and there is nothing in it that cannot be rebuilt by opening a
    /// document.
    #[must_use]
    pub fn load() -> Self {
        Self::at(
            settings::resolve_store()
                .directory()
                .map(|dir| dir.join(RECENT_FILE)),
        )
    }

    /// Load from an explicit directory.
    ///
    /// The twin of `pdfcer_core::settings::store_in` and of
    /// [`crate::app::persistence::LayoutStore::load_in`], and it exists for
    /// the same two reasons: tests, and a future `--user-data-dir` override.
    #[must_use]
    pub fn load_in(dir: &Path) -> Self {
        Self::at(Some(dir.join(RECENT_FILE)))
    }

    /// The path [`Self::load`] resolves, without loading anything.
    ///
    /// Exists so the location convention is assertable against
    /// `LayoutStore`'s, which is what stops the two files from drifting into
    /// different folders.
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
    ///
    /// Blank lines are skipped and the result is capped and de-duplicated,
    /// so a hand-edited file cannot produce a list this module would not
    /// itself have written. Only the line ending is trimmed — a leading or
    /// trailing space is a legal part of a file name on some platforms, and
    /// trimming it would silently rename the operator's file.
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
    ///
    /// A path that is not valid Unicode is **dropped here**, which is the one
    /// place in this module where dropping at save time is correct: it is not
    /// a judgement about whether the file still exists, it is the format
    /// admitting it cannot spell the name. The alternative — a lossy
    /// conversion — would write a path that names a *different* file, or no
    /// file, and would look exactly like a real entry when it came back.
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
    ///
    /// This is what a save writes and what a count reports. What a menu draws
    /// is [`Self::present_at`]; the difference between the two is the whole
    /// of the module header's display-time rule.
    #[must_use]
    pub fn entries(&self) -> &[PathBuf] {
        &self.entries
    }

    /// Whether anything has ever been opened.
    ///
    /// Answered without touching the filesystem, deliberately: it is what
    /// decides whether the Recent control is enabled, and that decision is
    /// made on every frame the File tab is drawn. A list whose every entry is
    /// currently unreachable still enables the control, and the menu then
    /// says so — which is a better answer than a control that greys out
    /// because a drive is slow.
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
    /// [`crate::app::persistence::LayoutStore`]: opening a document is a rare
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
    ///
    /// **A failure clears nothing and retries nothing.** The next `remember`
    /// tries again, which is the same posture
    /// [`crate::app::persistence::LayoutStore`] takes towards a path
    /// that cannot be written: retrying a read-only share on a schedule
    /// produces the same error at a cost, and the operator has already lost
    /// nothing they can see.
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
    ///
    /// What a menu draws, and the only method that touches the filesystem
    /// after load. Throttled to one sweep per [`PRESENCE_TTL`]; the caller
    /// supplies `now` so the throttle is assertable without sleeping, exactly
    /// as [`crate::app::persistence::LayoutStore::tick`]'s deadline is.
    ///
    /// Returns owned paths rather than a borrow because the caller is a
    /// drawing closure that also needs `&mut self` for the sweep, and because
    /// the vector is at most [`CAP`] entries long.
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
    ///
    /// What `file.recent` opens when it is invoked without the menu having
    /// chosen an entry — from a rebound chord, or from a quick-access
    /// toolbar an operator has customized. See
    /// [`crate::app::PdfcerApp::dispatch_command`].
    pub fn newest_present(&mut self, now: Instant) -> Option<PathBuf> {
        self.present_at(now).into_iter().next()
    }
}

// ===========================================================================
// The control
// ===========================================================================

/// **Draw the Recent control, and report the document the operator chose.**
///
/// Called from [`crate::app::PdfcerApp`]'s custom-item renderer for the
/// `recent_files` item in File ▸ File — the extension point
/// `egui_shell::manifest::Item::Custom` exists for, whose own doc names *"a
/// split button with a gallery"* as the case. The shell reserves the space
/// and hands back a `Ui`; what goes in it is the application's business,
/// which is exactly the seam that keeps a reusable shell from growing a
/// `RecentFiles` variant.
///
/// # It reports, and changes nothing
///
/// Returning the path rather than opening it is the actions-not-mutations
/// invariant reaching one control further out: this is a widget, it is being
/// drawn mid-frame, and `crate::app::actions`' rule is that **no code path
/// runs from a widget to a document**. The caller parks the answer, returns
/// the `file.recent` handler token, and the command goes through
/// `PdfcerApp::dispatch_command` — the same choke point a ribbon click, a
/// chord and a context-menu row all reach. The menu is to `file.recent` what
/// the file dialog is to `file.open`: the operand picker, not the verb.
///
/// # Where the words come from
///
/// The button's label and hover text are **the command's**, read from
/// [`crate::text::commands::file_recent`], for the reason
/// `crate::shell::menus`' header gives about menu rows: a second copy of a
/// command's words is a second copy that can drift. Only the things the
/// command's text cannot express — the rows, which are file names, and the
/// empty state — come from [`crate::text::files`].
///
/// # Greying, and what it is allowed to mean
///
/// `RIBBON_IA.md` P3 reserves greying for *temporarily* unavailable, always
/// explained on hover. An operator who has never opened a document is exactly
/// that, so the button is disabled and both hover texts are attached — plain
/// and disabled — because `egui` shows a different one in each state and
/// attaching only the first is how a greyed control becomes unexplainable.
///
/// A list that is non-empty but every entry of which is currently unreachable
/// is **not** greyed: the answer would depend on how a network share happens
/// to be feeling, and a control that greys itself out because a drive is slow
/// teaches the operator it is broken. It opens, and says so in a row.
///
/// # ★★★ The glyph, 2026-09-05 — and the sentence that predicted this edit
///
/// `catalog::file`'s registration for `file.recent` carries `.with_icon("recent")`
/// and, beside it, a paragraph saying in as many words what was true until
/// today:
///
/// > *"this command's ribbon control is not a `Button`, it is the
/// > `recent_files` custom item, and `app::recent::menu` draws it with
/// > `ui.menu_button(text.label, …)` — application code that reads the
/// > command's LABEL and never consults its icon key. … what the operator sees
/// > in File ▸ File does not change until that custom item is taught to paint
/// > it — which is a change in `app::recent`, not here."*
///
/// This is that change. It was found by driving rather than by reading: the
/// approved mockup draws the control as `['Recent','recent',{menu:1}]` — an
/// icon **and** a word — and `tools/compare-mockup-ribbon.py` could not see the
/// difference, because a `Custom` item carries no command id for it to resolve
/// an icon key from. What made it visible was resolving both sides to the
/// **asset** each one draws and giving the custom kinds a declared glyph.
///
/// ⇒ ★★ **A comment that names the file where the rest of the work lives is
/// the best available substitute for a mechanism, and it is still not one.**
/// That sentence was correct, precise, and sat unactioned; what moved it was an
/// instrument that compares the two pictures.
///
/// [`egui::Ui::menu_image_text_button`] rather than a hand-built
/// `Button::image_and_text`: the menu behaviour — the popup, the close
/// semantics, the submenu arrow when nested — is `egui`'s and must not be
/// re-implemented for the sake of an icon.
pub fn menu(ui: &mut egui::Ui, recent: &mut RecentFiles, now: Instant) -> Option<PathBuf> {
    let text = crate::text::commands::file_recent();
    let mut chosen: Option<PathBuf> = None;
    ui.add_enabled_ui(!recent.is_empty(), |ui| {
        // ★ `icons::image` takes its tint from THIS `Ui`'s `text_color()`, so
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty directory nothing else is using.
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("pdfcer-gui-recent-{tag}-{nanos}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        dir
    }

    /// A real file inside `dir`, so `is_file` says yes about it.
    fn touch(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, b"%PDF-1.7\n").expect("writes");
        path
    }

    /// ★ **The recent list sits beside the layout file and the settings
    /// file.**
    ///
    /// Asserted against `LayoutStore`'s own resolution rather than against a
    /// path spelled out here — a second spelling is exactly how two files
    /// that were supposed to share a directory end up in different ones, and
    /// `persistence`'s header explains why the directory decision belongs to
    /// `pdfcer-core` and to nothing else.
    #[test]
    fn the_recent_file_lives_beside_the_layout_file() {
        let dir = temp_dir("beside");
        let recent = RecentFiles::load_in(&dir);
        assert_eq!(recent.path(), Some(dir.join(RECENT_FILE).as_path()));
        assert_eq!(
            RecentFiles::default_path()
                .as_deref()
                .and_then(Path::parent),
            crate::app::persistence::LayoutStore::default_path()
                .as_deref()
                .and_then(Path::parent),
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A first run is an empty list, and it is not a failure.
    #[test]
    fn a_first_run_is_an_empty_list() {
        let dir = temp_dir("first-run");
        let recent = RecentFiles::load_in(&dir);
        assert!(recent.is_empty());
        assert!(recent.can_save());
        assert_eq!(recent.saves(), 0);
        assert_eq!(recent.save_error(), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **Newest first, and re-opening a document moves it rather than
    /// duplicating it.**
    ///
    /// The property that makes this a most-recently-used list rather than a
    /// log of opens. Without the move-to-front, opening the same drawing
    /// twice fills the menu with one file.
    #[test]
    fn the_newest_document_is_first_and_a_repeat_open_moves_it() {
        let dir = temp_dir("order");
        let mut recent = RecentFiles::load_in(&dir);
        let a = touch(&dir, "a.pdf");
        let b = touch(&dir, "b.pdf");
        let c = touch(&dir, "c.pdf");

        recent.remember(&a);
        recent.remember(&b);
        recent.remember(&c);
        assert_eq!(recent.entries(), [c.clone(), b.clone(), a.clone()]);

        recent.remember(&a);
        assert_eq!(
            recent.entries(),
            [a.clone(), c.clone(), b.clone()],
            "an entry already present moves to the front rather than appearing twice"
        );

        // …and re-opening what is already at the front costs no write at all.
        let before = recent.saves();
        recent.remember(&a);
        assert_eq!(recent.saves(), before);
        assert_eq!(recent.entries().len(), 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The list is capped, and the cap drops the OLDEST entry.
    #[test]
    fn the_list_is_capped_and_the_oldest_entry_is_what_goes() {
        let dir = temp_dir("cap");
        let mut recent = RecentFiles::load_in(&dir);
        let first = touch(&dir, "0.pdf");
        recent.remember(&first);
        for n in 1..=CAP {
            let path = touch(&dir, &format!("{n}.pdf"));
            recent.remember(&path);
        }
        assert_eq!(recent.entries().len(), CAP);
        assert!(
            !recent.entries().contains(&first),
            "the oldest entry is the one the cap drops"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **A missing file is dropped at DISPLAY time and kept in the file.**
    ///
    /// The rule the module header argues: a network drive that is temporarily
    /// absent is not a file the operator wants forgotten. Asserted from both
    /// ends — the menu does not offer it, and the *file on disk* still holds
    /// it — because an assertion on the in-memory list alone would pass even
    /// if the save path had quietly pruned it.
    #[test]
    fn a_missing_file_leaves_the_menu_and_stays_in_the_file() {
        let dir = temp_dir("missing");
        let mut recent = RecentFiles::load_in(&dir);
        let here = touch(&dir, "here.pdf");
        let gone = touch(&dir, "gone.pdf");
        recent.remember(&here);
        recent.remember(&gone);

        let now = Instant::now();
        assert_eq!(recent.present_at(now).len(), 2);

        std::fs::remove_file(&gone).expect("removes");
        // Far enough past the throttle that the answer is re-taken.
        let later = now + PRESENCE_TTL;
        assert_eq!(
            recent.present_at(later),
            vec![here.clone()],
            "the menu must not offer a document that is not there"
        );
        assert!(
            recent.entries().contains(&gone),
            "…and the list must not forget it: the drive may come back"
        );

        let text = std::fs::read_to_string(dir.join(RECENT_FILE)).expect("reads back");
        assert!(
            text.contains("gone.pdf"),
            "the absent entry must survive in the file: {text}"
        );

        // And when it comes back, so does the entry — with no re-open needed.
        std::fs::write(&gone, b"%PDF-1.7\n").expect("writes");
        let later_still = later + PRESENCE_TTL;
        assert_eq!(recent.present_at(later_still).len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **The presence check is throttled.**
    ///
    /// `Path::is_file` on a dead network path blocks for as long as the
    /// filesystem takes to give up, on the UI thread. Without the throttle
    /// the menu would take that answer on every frame it is open, which is
    /// not slowness but a frozen application.
    ///
    /// Asserted by removing a file and watching the answer *not* change until
    /// the window has passed — the only way to prove a cache is a cache.
    #[test]
    fn the_presence_answer_is_reused_until_the_window_passes() {
        let dir = temp_dir("throttle");
        let mut recent = RecentFiles::load_in(&dir);
        let path = touch(&dir, "sheet.pdf");
        recent.remember(&path);

        let now = Instant::now();
        assert_eq!(recent.present_at(now).len(), 1);
        std::fs::remove_file(&path).expect("removes");
        assert_eq!(
            recent.present_at(now + PRESENCE_TTL / 2).len(),
            1,
            "inside the window the cached answer stands, whatever the disk now says"
        );
        assert_eq!(
            recent.present_at(now + PRESENCE_TTL).len(),
            0,
            "…and at the window the answer is taken again"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The list survives a restart, through a real file.
    #[test]
    fn the_list_survives_a_restart() {
        let dir = temp_dir("round-trip");
        let a = touch(&dir, "a.pdf");
        let b = touch(&dir, "b.pdf");
        {
            let mut recent = RecentFiles::load_in(&dir);
            recent.remember(&a);
            recent.remember(&b);
            assert_eq!(recent.saves(), 2);
            assert_eq!(recent.save_error(), None);
        }
        let reopened = RecentFiles::load_in(&dir);
        assert_eq!(reopened.entries(), [b, a]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A relative path is stored absolute, or it means something else next
    /// time.
    #[test]
    fn a_relative_path_is_stored_absolute() {
        let dir = temp_dir("relative");
        let mut recent = RecentFiles::load_in(&dir);
        recent.remember(Path::new("drawing.pdf"));
        let stored = &recent.entries()[0];
        assert!(
            stored.is_absolute(),
            "a relative entry names a different file from a different working \
             directory: {stored:?}"
        );
        assert_eq!(
            stored.file_name().and_then(|n| n.to_str()),
            Some("drawing.pdf")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A hand-edited file cannot produce a list this module would not itself
    /// have written: blank lines go, duplicates go, the cap holds.
    #[test]
    fn a_hand_edited_file_is_normalised_on_load() {
        let dir = temp_dir("hand-edited");
        let mut text = String::from("\n\n");
        text.push_str(&dir.join("one.pdf").to_string_lossy());
        text.push('\n');
        text.push_str(&dir.join("one.pdf").to_string_lossy());
        text.push('\n');
        for n in 0..CAP * 2 {
            text.push_str(&dir.join(format!("{n}.pdf")).to_string_lossy());
            text.push('\n');
        }
        std::fs::write(dir.join(RECENT_FILE), text).expect("writes");

        let recent = RecentFiles::load_in(&dir);
        assert_eq!(recent.entries().len(), CAP);
        assert_eq!(recent.entries()[0], dir.join("one.pdf"));
        assert_eq!(
            recent.entries()[1],
            dir.join("0.pdf"),
            "the duplicate second line is dropped rather than shifting everything"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **A default store points nowhere and can erase nothing.**
    ///
    /// `Default` exists so `PdfcerApp` keeps deriving it, and because
    /// `PdfcerApp::new` deliberately uses it under `cfg(test)` so a unit test
    /// that opens a fixture cannot scribble fixture paths into the operator's
    /// own list. The hazard it must not have is a store that looks loaded and
    /// points at the real file.
    #[test]
    fn a_default_store_points_nowhere_and_writes_nothing() {
        let mut recent = RecentFiles::default();
        assert!(!recent.can_save());
        assert_eq!(recent.path(), None);
        recent.remember(Path::new("anything.pdf"));
        assert_eq!(recent.entries().len(), 1, "the session still remembers");
        assert_eq!(recent.saves(), 0, "…and nothing was written anywhere");
        assert_eq!(recent.save_error(), None);
    }

    // =======================================================================
    // The application path: what records an entry, and what opens one
    // =======================================================================

    /// ★ **Opening a document records it; failing to open one does not.**
    ///
    /// The recording lives in [`crate::app::PdfcerApp::open_path`] — the one
    /// function that opens documents, and the one `argv` reaches without an
    /// action, so the first document of a session is recorded too.
    ///
    /// The negative half is the decision worth pinning: a file that would not
    /// open is not a *document* the operator had, and offering it from a menu
    /// whose whole promise is "this worked before" invites the same failure
    /// from the one surface that should be reliable.
    #[test]
    fn opening_a_document_records_it_and_a_failed_open_does_not() {
        use crate::panels::objects::test_support::engine_fixture;

        let mut app = crate::app::PdfcerApp::new();
        assert!(
            app.recent.is_empty(),
            "a test build starts with a store that points nowhere — see `PdfcerApp::new`"
        );

        let fixture = engine_fixture("pageops/four-pages.pdf");
        app.open_path(fixture.clone());
        assert_eq!(
            app.recent.entries(),
            [std::path::absolute(&fixture).expect("absolute")]
        );

        app.open_path(engine_fixture("not-a-pdf.bin"));
        assert!(
            matches!(app.status, crate::app::state::Status::Failed { .. }),
            "this fixture must fail to open, or the test proves nothing"
        );
        assert_eq!(
            app.recent.entries().len(),
            1,
            "a file that would not open must not be offered as a recent DOCUMENT"
        );

        // …and closing does not forget it. Closing a document is the single
        // most likely moment to reach for the one before it.
        app.close_document();
        assert_eq!(app.recent.entries().len(), 1);
    }

    /// ★ **`file.recent` opens the entry the menu parked, and falls back to
    /// the newest reachable one when there is none.**
    ///
    /// Two routes into one command, which is the whole reason the menu is a
    /// custom *item* rather than a command of its own: the item asks which,
    /// the command acts. The fallback is not a guess — it is the defined
    /// answer for an invocation that carries no operand, which an operator
    /// reaches by binding a chord or adding the command to their quick-access
    /// toolbar, neither of which draws a menu.
    #[test]
    fn the_recent_command_opens_the_parked_choice_or_the_newest_reachable() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        use crate::app::actions::Action;
        use crate::panels::objects::test_support::engine_fixture;

        let mut app = crate::app::PdfcerApp::new();
        let four = engine_fixture("pageops/four-pages.pdf");
        let layers = engine_fixture("layers/painted-layers.pdf");
        app.recent.remember(&four);
        app.recent.remember(&layers);

        // With nothing parked: the newest entry that can be seen.
        let mut actions = Vec::new();
        app.dispatch_command(&ctx, crate::shell::commands::FILE_RECENT, &mut actions);
        assert_eq!(
            actions,
            vec![Action::Open(
                std::path::absolute(&layers).expect("absolute")
            )]
        );

        // With a choice parked by the menu: that one, and the slot is emptied
        // so the next invocation cannot re-open it by accident.
        app.recent_choice = Some(four.clone());
        let mut actions = Vec::new();
        app.dispatch_command(&ctx, crate::shell::commands::FILE_RECENT, &mut actions);
        assert_eq!(actions, vec![Action::Open(four)]);
        assert!(app.recent_choice.is_none(), "the operand is consumed");
    }

    /// An empty list raises nothing rather than an action naming nothing.
    #[test]
    fn the_recent_command_with_nothing_to_open_raises_nothing() {
        // A bare context: these tests exercise the dispatcher, not a
        // frame. `dispatch_command` needs one because three navigation arms
        // write the armed tool and the zoom anchor into egui memory, which
        // is where per-frame UI state lives.
        let ctx = egui::Context::default();
        let mut app = crate::app::PdfcerApp::new();
        let mut actions = Vec::new();
        app.dispatch_command(&ctx, crate::shell::commands::FILE_RECENT, &mut actions);
        assert!(actions.is_empty());
    }

    /// An unreadable file is an empty list and a working session.
    #[test]
    fn a_directory_where_the_file_should_be_is_survived() {
        let dir = temp_dir("unreadable");
        // A directory named `recent.txt` cannot be read as a file, on every
        // platform, without needing permissions a test cannot set.
        std::fs::create_dir_all(dir.join(RECENT_FILE)).expect("a decoy directory");
        let recent = RecentFiles::load_in(&dir);
        assert!(recent.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
