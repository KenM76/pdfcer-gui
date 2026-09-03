//! # `viewer::remembered` — the page-display choice, per document, on disk
//!
//! The operator's requirement, 2026-08-12, and `GUI_ROADMAP.md` 4.5:
//!
//! > *"Mode persists **per document**, not globally — opening a drawing set
//! > must not inherit a report's setting."*
//!
//! One file, `page-display.txt`, holding one line per document. Nothing else
//! is stored and nothing else should be.
//!
//! ## ★ Why this is a third file rather than a field in one of the two
//!
//! `PROJECT_PLAN.md`'s brief for this work named the two existing stores and
//! asked which fits. Neither does, and both say so themselves:
//!
//! | store | what it is | lifetime | verdict |
//! |---|---|---|---|
//! | [`crate::app::persistence`] `layout.ron` | the dock arrangement and named workspaces | **per operator**, outlives every document | wrong axis entirely — one layout serves all documents, and a per-document key in it would make the arrangement a document property |
//! | [`crate::app::recent`] `recent.txt` | the ten documents most recently opened | per operator, keyed by document | *nearly* right, and explicitly refused |
//!
//! `recent.rs`'s own header settles the second, in a section headed *"What is
//! deliberately NOT here"*:
//!
//! > **No per-document view state** (last page, last zoom). That is a
//! > different feature with a different lifetime and a different file; a
//! > recent list that quietly became a session store would be the thing
//! > nobody could later separate.
//!
//! That paragraph was written before there was any per-document view state to
//! store. There is now, and the conclusion it reached is the one taken here:
//! **a different file.** Three concrete consequences follow, and each is a
//! reason rather than a restatement:
//!
//! 1. **The lifetimes genuinely differ.** The recent list is capped at ten
//!    because it is *drawn* — a menu taller than ten rows is a scroll view.
//!    This list is never drawn, so its cap is about disk and nothing else, and
//!    [`CAP`] is twenty times larger. Sharing a file would force one cap to
//!    serve two purposes, and the drawn one would win.
//! 2. **Forgetting means different things.** A "clear recent files" command
//!    (in `PLANNED`) must not silently reset every document's display mode,
//!    and a document whose remembered mode is evicted must not vanish from the
//!    recent menu. Two files make that true by construction rather than by a
//!    rule somebody has to honour.
//! 3. **The format cannot serve both.** `recent.txt` is *"one path per line,
//!    newest first … nothing else is legal, so nothing else has to be
//!    parsed"*. Adding a field to it makes every line ambiguous with the
//!    format it replaced, and a half-upgraded file would read old paths as
//!    mode ids.
//!
//! ## ★ Why it lives in `viewer/` rather than beside the other two in `app/`
//!
//! Because what it persists is [`PageDisplay`], and the on-disk spelling of
//! that enum is [`PageDisplay::id`]. Keeping the reader and the writer beside
//! the type means a variant added to the enum without a spelling is a
//! compile error and a variant added with a colliding spelling is a test
//! failure in the same module — see
//! `viewer::display::tests::every_mode_round_trips_through_its_on_disk_spelling`.
//! Put the store in `app/` and the enum's spelling has two homes, which is the
//! shape of every drift this project writes headers about.
//!
//! The other two stores are in `app/` because what they persist —
//! `egui_shell::layout::LayoutDocument` and a list of paths — has no module of
//! its own to sit in.
//!
//! ## The format
//!
//! ```text
//! continuous\tD:\Drawings\job-4471\sheet-set.pdf
//! single\tC:\Users\ken\Documents\report.pdf
//! ```
//!
//! One line per document, **most recently written first**, UTF-8, no header
//! and no comments. The separator is a **tab**, chosen because it is the one
//! ASCII character a Windows path cannot contain (`< > : " / \ | ? *` and the
//! control range are all reserved) and because it needs no escaping. A line
//! with no tab, an unknown mode id, or an empty path is **dropped** — a
//! corrupt file degrades into a shorter list, exactly as `recent.txt` does,
//! rather than into an error the operator has to dismiss about a preference.
//!
//! Like `recent.txt`, this is a flat text file rather than RON because
//! **this crate cannot serialize**: `serde` and `ron` are dependencies of
//! `egui-shell`, not of `pdfcer-gui`, and `Cargo.toml` is not this work's to
//! edit.
//!
//! ## ★ Why there is no in-memory store held on the application
//!
//! Because there is nothing to hold. The file is read **once per document
//! open** and written **once per mode change** — two of the rarest events in
//! the application, against a file of at most [`CAP`] short lines. A cached
//! copy would buy a few microseconds on events that happen seconds apart, and
//! would cost a field on `PdfcerApp`, a load at start-up, and a staleness
//! question ("what if another pdfcer window wrote it?") that not caching
//! answers for free: **two windows open on two documents each write their own
//! line and read the other's**, because every write is a read-modify-write of
//! the whole file.
//!
//! The honest limit of that: two windows changing the mode at the same instant
//! can lose one of the two writes. The loss is one document's display
//! preference, the window that lost it still shows what the operator chose,
//! and the next change writes it again. A lock file would be a larger
//! mechanism than the thing it protects.

use std::path::{Path, PathBuf};

use pdfcer_core::settings;

use super::display::PageDisplay;

/// The file's name, inside the settings directory.
///
/// `.txt` because the format is one line per document and an operator who
/// opens it should find exactly what they expect — the same argument
/// `recent.txt` makes. `.ron` would promise a structure that is not there and
/// a parser this crate does not have.
pub const REMEMBERED_FILE: &str = "page-display.txt"; // ui-text-exempt: a file name, never displayed as copy

/// The separator between a mode id and a path.
///
/// A tab, because it is the one ASCII character a Windows path cannot contain
/// and therefore the one that needs no escaping. See the module header.
const SEPARATOR: char = '\t';

/// How many documents are remembered.
///
/// Two hundred, and the number is about **disk** rather than about a menu:
/// nothing draws this list, so the cap that governs [`crate::app::recent::CAP`]
/// — "what fits in a File menu without becoming a scroll view" — does not
/// apply. Two hundred lines is a few kilobytes, comfortably more documents
/// than an operator revisits, and small enough that a read-modify-write costs
/// nothing measurable.
///
/// A cap exists at all because the file is written on every mode change and an
/// uncapped one would grow forever across the life of an installation.
pub const CAP: usize = 200;

/// The path this store reads and writes, or `None` when `pdfcer-core` found no
/// writable location.
///
/// Derived from the same `pdfcer_core::settings::resolve_store()` call that
/// decides where `settings.txt`, `layout.ron` and `recent.txt` go — never a
/// directory computed here. `persistence.rs`'s header carries the three
/// reasons in full; the short one is that a second resolution is how two of an
/// application's files end up in different folders.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    settings::resolve_store()
        .directory()
        .map(|dir| dir.join(REMEMBERED_FILE))
}

/// **The display mode remembered for `document`, if any.**
///
/// `None` means *"this document has no remembered choice"*, and the caller
/// answers it with [`PageDisplay::default_for_mode`] — the per-mode default,
/// which is where "Read opens continuous" lives. It deliberately does **not**
/// mean "single page": collapsing the two would make a fresh document in Read
/// mode open paged, which is the operator decision of 2026-08-13 inverted.
///
/// Never fails. A missing file, an unreadable one and a corrupt one all answer
/// `None`, because every one of them means the same thing to the caller —
/// there is no remembered choice to honour — and a preference is not worth an
/// error path.
#[must_use]
pub fn recall(document: &Path) -> Option<PageDisplay> {
    recall_at(default_path().as_deref(), document)
}

/// **Remember that `document` is being shown in `display`.**
///
/// Read-modify-write of the whole file: the entry moves to the front, any
/// previous entry for the same document is replaced rather than duplicated,
/// and the list is truncated to [`CAP`]. Writes immediately rather than
/// debouncing, for the same reason `recent.rs` does — a mode change is a rare
/// discrete click, not a drag reporting sixty changes a second, so there is no
/// gesture to settle and nothing to gain by deferring past a crash.
///
/// Writing the mode a document *already* has costs nothing: the entry is
/// already at the front with the same value, and the function returns without
/// touching the disk. That matters because the caller cannot easily know
/// whether a click changed anything, and re-writing the file on every click of
/// an already-selected radio button would be a file write per click for no
/// change.
///
/// Failures are traced and otherwise ignored. There is no operator-facing
/// consequence worth a dialog: the mode is applied to the open document either
/// way, and the only loss is that it will not be there on the next open.
pub fn remember(document: &Path, display: PageDisplay) {
    remember_at(default_path().as_deref(), document, display);
}

/// [`recall`], against an explicit file — the seam tests use.
///
/// The twin of `pdfcer_core::settings::store_in` and of
/// `LayoutStore::load_in`, and it exists for the same two reasons: tests, and
/// a future `--user-data-dir` override.
#[must_use]
pub fn recall_at(file: Option<&Path>, document: &Path) -> Option<PageDisplay> {
    let wanted = absolute(document);
    let text = std::fs::read_to_string(file?).ok()?;
    parse(&text)
        .into_iter()
        .find(|(_, path)| *path == wanted)
        .map(|(display, _)| display)
}

/// [`remember`], against an explicit file.
pub fn remember_at(file: Option<&Path>, document: &Path, display: PageDisplay) {
    let Some(file) = file else {
        // No writable location is a working session in which only saving is
        // impossible — `persistence.rs`'s `StoreKind::None` posture, inherited
        // rather than re-decided. Silent rather than traced: this is reached
        // on every mode change for the whole session, and a line per click
        // would bury the ones that matter.
        return;
    };
    let path = absolute(document);
    let mut entries = std::fs::read_to_string(file)
        .ok()
        .map(|text| parse(&text))
        .unwrap_or_default();

    // Already recorded, with this value, at the front: nothing changes, so
    // nothing is written. See the doc comment on why this is not an
    // optimisation but a correctness-of-cost property.
    if entries
        .first()
        .is_some_and(|(d, p)| *d == display && *p == path)
    {
        return;
    }
    entries.retain(|(_, existing)| *existing != path);
    entries.insert(0, (display, path.clone()));
    entries.truncate(CAP);

    let mut text = String::new();
    for (display, path) in &entries {
        // A path that is not valid Unicode cannot be spelled in this format.
        // Dropped at *save* time, which is the one place dropping is right —
        // it is not a judgement about the document, it is the format saying it
        // cannot write the name. `recent.rs` makes the identical call for the
        // identical reason.
        if let Some(spelled) = path.to_str() {
            text.push_str(display.id());
            text.push(SEPARATOR);
            text.push_str(spelled);
            text.push('\n');
        }
    }

    // Create the directory rather than assume it: on a first run nothing has
    // written to the settings folder yet, and `resolve_store` proved the
    // location writable without necessarily creating it.
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::write(file, text) {
        Ok(()) => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-display-remembered mode={} n={} path={path:?}",
                display.id(),
                entries.len()
            )
        }),
        Err(error) => crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "page-display-remember-failed file={file:?} error={error}"
            )
        }),
    }
}

/// Parse the file into `(mode, path)` pairs, dropping anything unreadable.
///
/// Every rejection is silent and local: a line with no separator, an unknown
/// mode id, or an empty path is skipped and the rest of the file is kept. That
/// is the whole error strategy, and it is the right one for a preferences
/// cache — the alternative is discarding two hundred good entries because one
/// line was edited badly by hand.
fn parse(text: &str) -> Vec<(PageDisplay, PathBuf)> {
    text.lines()
        .filter_map(|line| {
            let (id, path) = line.split_once(SEPARATOR)?;
            let display = PageDisplay::from_id(id.trim())?;
            let path = path.trim_end_matches(['\r', '\n']);
            if path.is_empty() {
                return None;
            }
            Some((display, PathBuf::from(path)))
        })
        .collect()
}

/// Resolve `path` against the current directory **without touching the
/// filesystem**.
///
/// The identical treatment `recent.rs` gives a path it is about to persist,
/// and for the identical reason: `pdfcer-gui drawing.pdf` gives a relative
/// `argv[1]`, and a relative path in a persisted file means something
/// different — or nothing — the next time the application starts from another
/// directory. [`std::fs::canonicalize`] would also resolve symlinks and, on
/// Windows, return a `\\?\` extended-length path; it also requires the file to
/// exist, and a document may be remembered and then moved.
///
/// The consequence, stated rather than discovered: two spellings of one file
/// (a mapped drive and its UNC path) are two entries with two remembered
/// modes. That is a strictly better failure than de-duplicating with
/// `canonicalize` and thereby refusing to remember anything on an unreachable
/// share.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty directory nothing else is using.
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("pdfcer-gui-display-{tag}-{nanos}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        dir
    }

    /// ★ **A sheet set does not inherit a report's setting.**
    ///
    /// The operator's requirement of 2026-08-12, stated as the scenario they
    /// described rather than as an API exercise: two documents, two different
    /// modes, and each remembers its own across a "restart" (a fresh read of
    /// the same file).
    #[test]
    fn a_sheet_set_does_not_inherit_a_reports_setting() {
        let dir = temp_dir("per-document");
        let file = dir.join(REMEMBERED_FILE);
        let report = dir.join("quarterly-report.pdf");
        let sheets = dir.join("job-4471-sheet-set.pdf");

        remember_at(Some(&file), &report, PageDisplay::Continuous);
        remember_at(Some(&file), &sheets, PageDisplay::Single);

        assert_eq!(
            recall_at(Some(&file), &report),
            Some(PageDisplay::Continuous)
        );
        assert_eq!(recall_at(Some(&file), &sheets), Some(PageDisplay::Single));
        // A document nobody has chosen for has no remembered choice — which is
        // NOT the same as a remembered `Single`, because the caller answers
        // `None` with the per-mode default and Read's default is continuous.
        assert_eq!(recall_at(Some(&file), &dir.join("never-opened.pdf")), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Changing a document's mode replaces its entry rather than adding a
    /// second one that the next read might find first.
    #[test]
    fn changing_a_documents_mode_replaces_its_entry() {
        let dir = temp_dir("replace");
        let file = dir.join(REMEMBERED_FILE);
        let doc = dir.join("a.pdf");

        remember_at(Some(&file), &doc, PageDisplay::Single);
        remember_at(Some(&file), &doc, PageDisplay::FacingContinuous);
        remember_at(Some(&file), &doc, PageDisplay::Facing);

        let text = std::fs::read_to_string(&file).expect("written");
        assert_eq!(text.lines().count(), 1, "one document, one line: {text}");
        assert_eq!(recall_at(Some(&file), &doc), Some(PageDisplay::Facing));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Re-writing the mode a document already has costs no file write.
    ///
    /// The radio raises its command on every click, including a click on the
    /// position that is already active, so without this a operator resting a
    /// finger on the mouse would write the file repeatedly.
    #[test]
    fn re_recording_the_same_choice_writes_nothing() {
        let dir = temp_dir("idempotent");
        let file = dir.join(REMEMBERED_FILE);
        let doc = dir.join("a.pdf");

        remember_at(Some(&file), &doc, PageDisplay::Continuous);
        let first = std::fs::metadata(&file).expect("written").modified().ok();
        remember_at(Some(&file), &doc, PageDisplay::Continuous);
        let second = std::fs::metadata(&file)
            .expect("still there")
            .modified()
            .ok();
        assert_eq!(first, second, "the file was rewritten for no change");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The newest entry is first, and the list is capped.
    #[test]
    fn the_list_is_newest_first_and_capped() {
        let dir = temp_dir("cap");
        let file = dir.join(REMEMBERED_FILE);
        for n in 0..(CAP + 20) {
            remember_at(
                Some(&file),
                &dir.join(format!("doc-{n}.pdf")),
                PageDisplay::Continuous,
            );
        }
        let text = std::fs::read_to_string(&file).expect("written");
        assert_eq!(text.lines().count(), CAP);
        // The newest survives; the oldest has been evicted.
        assert_eq!(
            recall_at(Some(&file), &dir.join(format!("doc-{}.pdf", CAP + 19))),
            Some(PageDisplay::Continuous)
        );
        assert_eq!(recall_at(Some(&file), &dir.join("doc-0.pdf")), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ★ **A corrupt file degrades into a shorter list, never into an error.**
    ///
    /// Every rejection is local: a line with no separator, an unknown mode id
    /// and an empty path are each skipped and the rest is kept. The
    /// alternative — refusing the whole file — would discard two hundred good
    /// entries because one line was edited badly by hand.
    #[test]
    fn a_corrupt_line_is_dropped_and_the_rest_survives() {
        let dir = temp_dir("corrupt");
        let file = dir.join(REMEMBERED_FILE);
        let good = dir.join("good.pdf");
        std::fs::write(
            &file,
            format!(
                "this line has no separator\n\
                 spiral\t{unknown}\n\
                 single\t\n\
                 \n\
                 facing\t{good}\n",
                unknown = dir.join("unknown-mode.pdf").display(),
                good = good.display(),
            ),
        )
        .expect("writes");

        assert_eq!(recall_at(Some(&file), &good), Some(PageDisplay::Facing));
        assert_eq!(recall_at(Some(&file), &dir.join("unknown-mode.pdf")), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A missing file, a missing directory and no writable location at all are
    /// each a working session in which only remembering is impossible.
    #[test]
    fn no_file_and_nowhere_to_write_are_both_survivable() {
        let dir = temp_dir("absent");
        let missing = dir.join("not-written-yet").join(REMEMBERED_FILE);
        assert_eq!(recall_at(Some(&missing), &dir.join("a.pdf")), None);

        // `None` — `StoreKind::None`, no writable location. Nothing panics and
        // nothing is written.
        assert_eq!(recall_at(None, &dir.join("a.pdf")), None);
        remember_at(None, &dir.join("a.pdf"), PageDisplay::Continuous);

        // Writing into a directory that does not exist yet creates it, because
        // a first run has never touched the settings folder.
        remember_at(Some(&missing), &dir.join("a.pdf"), PageDisplay::Continuous);
        assert_eq!(
            recall_at(Some(&missing), &dir.join("a.pdf")),
            Some(PageDisplay::Continuous)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A relative path is resolved before it is stored, so the entry still
    /// names the same document from another working directory.
    #[test]
    fn a_relative_path_is_absolutized_before_it_is_stored() {
        let dir = temp_dir("relative");
        let file = dir.join(REMEMBERED_FILE);
        remember_at(Some(&file), Path::new("drawing.pdf"), PageDisplay::Facing);

        let text = std::fs::read_to_string(&file).expect("written");
        let stored = text
            .lines()
            .next()
            .and_then(|l| l.split_once(SEPARATOR))
            .map(|(_, p)| PathBuf::from(p))
            .expect("one entry");
        assert!(stored.is_absolute(), "stored as {stored:?}");
        // …and it is still found by the same relative spelling, because the
        // read absolutizes the same way.
        assert_eq!(
            recall_at(Some(&file), Path::new("drawing.pdf")),
            Some(PageDisplay::Facing)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The file sits in the directory `pdfcer-core` resolves, beside the other
    /// three — asserted against that call rather than against a path spelled
    /// out twice.
    #[test]
    fn the_file_lives_beside_the_other_stores() {
        assert_eq!(
            default_path(),
            settings::resolve_store()
                .directory()
                .map(|d| d.join(REMEMBERED_FILE))
        );
        assert_eq!(
            default_path().as_deref().and_then(Path::parent),
            crate::app::recent::RecentFiles::default_path()
                .as_deref()
                .and_then(Path::parent),
            "the per-document store must share a directory with the recent list"
        );
    }
}
