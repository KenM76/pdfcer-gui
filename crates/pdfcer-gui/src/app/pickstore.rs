//! # `app::pickstore` — the selection filter, on disk
//!
//! One question, answered the way [`crate::app::persistence`] answers it for
//! the dock layout: *where does the operator's selection filter live, and when
//! is it written?*
//!
//! ## Why a second module rather than a second type in `persistence`
//!
//! `app::persistence`'s header opens by saying it *"owns one type,
//! [`crate::app::persistence::LayoutStore`]"*, and the file is shaped around
//! that claim — its whole write-scheduling apparatus exists for one specific
//! problem which this file does not have. Bolting a second, differently-shaped
//! store into it would falsify the first line of its documentation and would
//! put two unrelated schedules in one place.
//!
//! ## ★ Why there is no debounce here, when the layout needs one
//!
//! This is the one interesting difference between the two stores and it is
//! worth being explicit, because the absence looks like an oversight.
//!
//! `LayoutStore` debounces because a splitter drag reports a change **on every
//! frame of the gesture** — a two-second drag is a hundred and twenty change
//! notifications describing one operator decision, and writing each one would
//! be a hundred and twenty file writes for one intent.
//!
//! A selection filter cannot do that. Its only inputs are discrete clicks on
//! eleven checkboxes and two buttons; there is no continuous gesture that can
//! produce one. So a change here is already exactly one operator decision, and
//! the correct number of writes for one decision is one, immediately. Adding a
//! delay would buy nothing and would introduce a window in which pulling the
//! power cord loses a choice the operator has already seen take effect on the
//! canvas.
//!
//! ## Where the file lives
//!
//! `<the settings directory>/select-filter.txt` — beside `settings.txt` and
//! `layout.ron`, with the directory resolved by asking `pdfcer-core` rather
//! than computing it here:
//!
//! ```text
//! pdfcer_core::settings::resolve_store()      →  StoreLocation { path, kind }
//!                        .directory()        →  <exe dir>/userdata      (Portable)
//!                                            or the platform config dir (PlatformFallback)
//!                                            or None                    (None)
//! ```
//!
//! Deferring to that call is what keeps a portable install portable: an
//! operator who copies the folder to a memory stick takes their filter with
//! them, because it never was anywhere else. A second implementation here
//! would be a second answer to *"where is the profile?"*, and the two would
//! disagree the first time either moved.
//!
//! ## ★★ Failure is silent, in one direction only, and that is deliberate
//!
//! **Loading never fails.** A missing file, an unreadable one, a directory
//! that does not exist, or a line full of tokens from a newer build all yield a
//! working filter. A shell that refused to start — or that opened with
//! everything unselectable — because a preferences file was unreadable would be
//! trading a total failure for a cosmetic one.
//!
//! The distinction that makes this safe is the one
//! [`crate::canvas::pick::PickFilter::from_tokens`] cannot make for itself:
//!
//! | on disk | means | yields |
//! |---|---|---|
//! | no file at all | this operator has never touched the filter | [`PickFilter::default`] — everything the shell can pick |
//! | a file, with tokens | these are the classes they chose | exactly those |
//! | a file, **empty** | they switched everything off | [`PickFilter::none`] — and the status bar says so |
//!
//! Row three is the one worth guarding. Collapsing "an empty file" into "no
//! file" would silently overrule a deliberate choice every restart, and the
//! operator would never find out why their filter kept resetting.
//!
//! **Saving may fail, and says so.** A read-only install directory is a real
//! condition, and [`save`] returns the error rather than swallowing it, so a
//! caller can decide whether it is worth telling anyone. Today's caller does
//! not tell anyone, which is a judgement about *this* preference rather than
//! about errors: losing a selection filter across a restart is an
//! inconvenience, and a modal about a preferences file at the moment the
//! operator clicked a checkbox would be worse than the thing it reports.

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
/// ★ The `Err` arm and the `Ok` arm are deliberately **not** merged. They mean
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
        let dir = std::env::temp_dir().join(format!("pdfcer-pickstore-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// The header's table, row 1: no file means "never touched", which is the
    /// default and NOT "nothing selectable".
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

    /// ★ The header's table, row 3 — the one that is easy to get wrong.
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
