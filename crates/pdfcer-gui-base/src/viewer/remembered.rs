//! # `viewer::remembered` — the page-display choice, per document, on disk
//!
//!
//! > *"Mode persists **per document**, not globally — opening a drawing set
//! > must not inherit a report's setting."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/viewer/remembered.md`.

use std::path::{Path, PathBuf};

use pdfcer_core::settings;

use super::display::PageDisplay;

/// The file's name, inside the settings directory.
pub const REMEMBERED_FILE: &str = "page-display.txt"; // ui-text-exempt: a file name, never displayed as copy

/// The separator between a mode id and a path.
///
/// A tab, because it is the one ASCII character a Windows path cannot contain
/// and therefore the one that needs no escaping. See the module header.
const SEPARATOR: char = '\t';

/// How many documents are remembered.
pub const CAP: usize = 200;

/// The path this store reads and writes, or `None` when `pdfcer-core` found no
/// writable location.
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    settings::resolve_store()
        .directory()
        .map(|dir| dir.join(REMEMBERED_FILE))
}

/// **The display mode remembered for `document`, if any.**
#[must_use]
pub fn recall(document: &Path) -> Option<PageDisplay> {
    recall_at(default_path().as_deref(), document)
}

/// **Remember that `document` is being shown in `display`.**
pub fn remember(document: &Path, display: PageDisplay) {
    remember_at(default_path().as_deref(), document, display);
}

/// [`recall`], against an explicit file — the seam tests use.
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
        // The pid is what `tools/gates/check-test-temp-paths.py` requires;
        // `nanos` stays because it also separates repeated runs inside one
        // process, which the pid does not.
        let dir = std::env::temp_dir().join(format!(
            "pdfcer-gui-display-{tag}-{nanos}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir");
        dir
    }

    /// **A sheet set does not inherit a report's setting.**
    ///
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

    /// **A corrupt file degrades into a shorter list, never into an error.**
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
    }
}
