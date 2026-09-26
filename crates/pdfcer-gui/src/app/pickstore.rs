//! # `app::pickstore` — the selection filter, on disk
//!
//! One question, answered the way [`crate::app::persistence`] answers it for
//! the dock layout: *where does the operator's selection filter live, and when
//! is it written?*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/pickstore.md`.

use std::path::{Path, PathBuf};

use pdfcer_core::settings;

use crate::canvas::pick::PickFilter;

/// The file the filter is written to, beside `settings.txt` and `layout.ron`.
///
/// A plain text file rather than RON, because the whole content is a single
/// line of space-separated words. RON would add a schema wrapper, a parser
/// dependency and a version field to serialise eleven booleans that already
/// have a stable textual form — and would make the file unreadable by the one
/// tool most likely to be pointed at it, which is a person with a text editor
/// trying to work out why their canvas stopped selecting things.
pub const FILTER_FILE: &str = "select-filter.txt"; // ui-text-exempt: a file name, never displayed as copy

/// Where the filter file would live, or `None` if this install has nowhere to
/// put one.
///
/// `None` is not an error: `pdfcer-core` reports it for a build with no
/// writable profile location at all, and the correct behaviour then is to run
/// with defaults and save nothing.
#[must_use]
pub fn path() -> Option<PathBuf> {
    settings::resolve_store()
        .directory()
        .map(|dir| dir.join(FILTER_FILE))
}

/// Read the operator's filter, or the default if they have never set one.
///
/// **Never fails** — see the module header for the three on-disk states and
/// what each means.
#[must_use]
pub fn load() -> PickFilter {
    match path() {
        Some(path) => load_from(&path),
        None => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "pick-filter-load nowhere-to-look default=1".to_owned()
            });
            PickFilter::default()
        }
    }
}

/// Read from an explicit path. The twin of [`load`], for tests and for a
/// future `--user-data-dir` override.
///
/// The `Err` arm and the `Ok` arm are deliberately **not** merged. They mean
/// different things — "you have never set this" against "these are your
/// settings" — and only the former may be answered with the default. See the
/// module header's table.
#[must_use]
pub fn load_from(path: &Path) -> PickFilter {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let filter = PickFilter::from_tokens(&text);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "pick-filter-load path={path:?} classes={} of {} empty={}",
                    filter.count(),
                    crate::canvas::pick::PickClass::COUNT,
                    filter.is_none(),
                )
            });
            filter
        }
        Err(err) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "pick-filter-load path={path:?} unreadable={} default=1",
                    err.kind(),
                )
            });
            PickFilter::default()
        }
    }
}

/// Write the operator's filter.
///
/// Returns `Ok(false)` when this install has nowhere to write — which is a
/// successful no-op rather than a failure, and is distinguished from
/// `Ok(true)` so a caller can tell "saved" from "there is no profile".
///
/// Creates the directory if it is missing, because on a fresh portable install
/// the first thing that wants to persist anything is whatever the operator
/// touches first, and that may well be this.
pub fn save(filter: PickFilter) -> std::io::Result<bool> {
    let Some(path) = path() else {
        return Ok(false);
    };
    save_to(&path, filter).map(|()| true)
}

/// Write to an explicit path. The twin of [`save`].
pub fn save_to(path: &Path, filter: PickFilter) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    // A trailing newline, so the file is a well-formed text line rather than a
    // bare fragment — `from_tokens` splits on any whitespace, so it costs
    // nothing to read and makes the file behave in an editor and in `cat`.
    let mut text = filter.to_tokens();
    text.push('\n');
    std::fs::write(path, text)?;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed.
            "pick-filter-save path={path:?} classes={}",
            filter.count(),
        )
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::pick::PickClass;

    /// A scratch directory that cleans itself up.
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("pdfcer-pickstore-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// The header's table, row 1: no file means "never touched", which is the
    /// default and *not* "nothing selectable".
    #[test]
    fn a_missing_file_yields_the_default_rather_than_an_empty_filter() {
        let dir = scratch("missing");
        let filter = load_from(&dir.join(FILTER_FILE));
        assert_eq!(filter, PickFilter::default());
        assert!(!filter.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The header's table, row 2.
    #[test]
    fn a_saved_filter_comes_back_exactly() {
        let dir = scratch("roundtrip");
        let path = dir.join(FILTER_FILE);
        let saved = PickFilter::default()
            .with(PickClass::Path, false)
            .with(PickClass::FormXObject, false);
        save_to(&path, saved).expect("save");
        assert_eq!(load_from(&path), saved);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The header's table, row 3 — the one that is easy to get wrong.
    ///
    /// An operator who switched every class off and quit must get that back,
    /// not a helpfully-restored default. If this ever fails, the shell has
    /// started overruling a deliberate choice once per restart, and the
    /// operator's report will be "my filter keeps resetting" with no way for
    /// them to see why.
    #[test]
    fn an_empty_file_means_nothing_selectable_not_never_configured() {
        let dir = scratch("empty");
        let path = dir.join(FILTER_FILE);
        save_to(&path, PickFilter::none()).expect("save");
        let loaded = load_from(&path);
        assert!(
            loaded.is_none(),
            "an explicit 'everything off' was resurrected"
        );
        assert_ne!(loaded, PickFilter::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Garbage in the file must not stop the shell starting.
    #[test]
    fn an_unparseable_file_still_yields_a_working_filter() {
        let dir = scratch("garbage");
        let path = dir.join(FILTER_FILE);
        std::fs::write(&path, "\u{0}\u{1}not tokens at all \u{feff}").expect("write");
        let loaded = load_from(&path);
        // Nothing recognisable, so nothing is on — but it loaded, and the
        // status bar's "nothing selectable" line is what the operator sees.
        assert!(loaded.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Saving into a directory that does not exist yet must create it, because
    /// on a fresh portable install this may be the first thing to persist.
    #[test]
    fn saving_creates_the_profile_directory_if_it_is_missing() {
        let dir = scratch("mkdir");
        let nested = dir.join("userdata").join(FILTER_FILE);
        save_to(&nested, PickFilter::default()).expect("save");
        assert!(nested.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The file is one readable line, because the person most likely to open
    /// it is trying to work out why their canvas stopped selecting things.
    #[test]
    fn the_file_is_one_line_of_words() {
        let dir = scratch("shape");
        let path = dir.join(FILTER_FILE);
        save_to(&path, PickFilter::default()).expect("save");
        let text = std::fs::read_to_string(&path).expect("read");
        assert_eq!(text.lines().count(), 1);
        assert!(text.ends_with('\n'));
        assert!(text.contains(PickClass::Text.token()));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
