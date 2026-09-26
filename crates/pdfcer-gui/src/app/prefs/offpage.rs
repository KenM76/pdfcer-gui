//! **Whether the canvas grows to show what sits off the sheet — remembered
//! separately for each ribbon mode.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/prefs/offpage.md`.

use std::collections::BTreeMap;

use super::printing::KeyOutcome;
use crate::app::actions::ViewChrome;
use crate::app::state::Status;

/// The prefix every key in this family shares: `off_page.<mode>`.
///
/// It is a **prefix**, not three fixed keys, and that is deliberate. Ribbon
/// modes come from the shell manifest, which an operator may customize (see
/// `crate::shell::manifest`), so this family cannot be closed over the three
/// modes that ship. A build whose manifest grows a fourth mode remembers that
/// mode's answer with no change here and no new key list to keep in step.
// ui-text-exempt: a file KEY prefix, written into preferences.txt and parsed
// back out of it. Never displayed.
const PREFIX: &str = "off_page.";

/// The ribbon mode whose default is **off** — the only one.
///
/// Spelled as a constant rather than inline in [`default_for_mode`] because it
/// is the one place this module knows a mode's *name*, and a reader looking
/// for "where does Read get special-cased" should find exactly one answer.
// ui-text-exempt: a MODE id from the shell manifest, compared against, never
// displayed. The label an operator sees comes from `crate::text`.
const READING_MODE: &str = "read";

/// Off-page display, remembered per ribbon mode.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OffPagePrefs {
    /// Mode id → the operator's answer. Absent means *never answered*.
    answers: BTreeMap<String, bool>,
}

impl OffPagePrefs {
    /// What this build ships for `mode`, before any operator answer.
    #[must_use]
    pub fn default_for_mode(mode: &str) -> bool {
        mode != READING_MODE
    }

    /// The answer to use for `mode` — the operator's, or this build's.
    #[must_use]
    pub fn for_mode(&self, mode: &str) -> bool {
        self.answers
            .get(mode)
            .copied()
            .unwrap_or_else(|| Self::default_for_mode(mode))
    }

    /// Remember `on` as the answer for `mode`.
    pub fn set(&mut self, mode: &str, on: bool) {
        self.answers.insert(mode.to_owned(), on);
    }
}

// ---------------------------------------------------------------------------
// The file format for this group — the parser and the writer, together
// ---------------------------------------------------------------------------

/// Read one `key = value` line into [`OffPagePrefs`], if it belongs here.
pub(super) fn parse_key(prefs: &mut OffPagePrefs, key: &str, value: &str) -> KeyOutcome {
    let Some(mode) = key.strip_prefix(PREFIX) else {
        return KeyOutcome::NotMine;
    };
    if mode.is_empty() {
        return KeyOutcome::BadValue;
    }
    match super::opening::bool_from_key(value) {
        Some(on) => {
            prefs.set(mode, on);
            KeyOutcome::Accepted
        }
        None => KeyOutcome::BadValue,
    }
}

/// Write this group's commented block into the file.
pub(super) fn write_block(prefs: &OffPagePrefs, out: &mut String) {
    out.push_str(
        "\n\
         # Off-page display, one answer per ribbon mode:\n\
         #   off_page.read = true | false\n\
         #   off_page.review = true | false\n\
         #   off_page.edit = true | false\n\
         #\n\
         # Some drawings carry marks outside the sheet itself -- a title block\n\
         # dragged off the page, a detail parked in the margin. When this is\n\
         # on, pdfcer draws them and lets you click them, and the canvas grows\n\
         # a band around the sheet wide enough to hold them. When it is off,\n\
         # the page is shown on its own and there is no band and no gap\n\
         # between pages.\n\
         #\n\
         # Unset means: off in Read, on in Review and Edit. A line appears\n\
         # here only for a mode where you have used View > Off-Page Content\n\
         # yourself, and that answer then wins for that mode from then on.\n",
    );
    for (mode, on) in &prefs.answers {
        // ui-text-exempt: a file KEY, written into preferences.txt and parsed
        // back out of it. Never displayed.
        out.push_str(PREFIX);
        out.push_str(mode);
        // ui-text-exempt: the file's key/value separator. Never displayed.
        out.push_str(" = ");
        out.push_str(super::opening::bool_key(*on));
        out.push('\n');
    }
}

// ---------------------------------------------------------------------------
// The write side — what happens when the operator uses the toggle
// ---------------------------------------------------------------------------

/// **Remember the operator's answer to `View ▸ Off-Page Content`**, for the
/// ribbon mode they are in, and put it on disk.
pub fn remember(chrome: ViewChrome, on: bool, prefs: &mut super::Prefs, mode: Option<&str>) {
    if chrome != ViewChrome::OffPage {
        return;
    }
    let Some(mode) = mode else {
        return;
    };
    prefs.off_page.set(mode, on);
    let _ = prefs.save();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("off-page-remembered mode={mode} on={on}")
    });
}

/// **Put the mode's answer into every open document's view state.**
pub fn apply_mode(prefs: &super::Prefs, mode: &str, status: &mut Status, parked: &mut [Status]) {
    let on = prefs.off_page.for_mode(mode);
    for status in std::iter::once(status).chain(parked.iter_mut()) {
        if let Status::Open(doc) = status {
            doc.view.off_page = on;
        }
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("off-page-mode mode={mode} on={on}")
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The operator's request, restated as an assertion.
    #[test]
    fn read_ships_off_and_the_working_modes_ship_on() {
        assert!(!OffPagePrefs::default_for_mode("read"));
        assert!(OffPagePrefs::default_for_mode("review"));
        assert!(OffPagePrefs::default_for_mode("edit"));
    }

    /// A mode this build has never heard of behaves like Edit, not like Read.
    ///
    /// See [`OffPagePrefs::default_for_mode`] for the argument: a cosmetic
    /// cost beats a silent omission.
    #[test]
    fn an_unknown_mode_shows_off_page_content() {
        assert!(OffPagePrefs::default_for_mode("redline-2027"));
    }

    /// The three modes are three independent memories.
    ///
    /// The failure this catches is the one a single global flag would have:
    /// answering in one mode changing the answer in another.
    #[test]
    fn an_answer_in_one_mode_does_not_move_another() {
        let mut prefs = OffPagePrefs::default();
        prefs.set("edit", false);
        assert!(!prefs.for_mode("edit"));
        assert!(prefs.for_mode("review"), "review followed edit");
        assert!(!prefs.for_mode("read"), "read followed edit");
    }

    /// An answer equal to the shipped default is still stored.
    #[test]
    fn choosing_the_default_is_still_an_answer() {
        let mut prefs = OffPagePrefs::default();
        prefs.set("read", false);
        assert!(
            prefs
                .write_to_string_fragment()
                .contains("off_page.read = false"),
            "an explicit answer was not written"
        );
    }

    /// A fresh profile writes no key lines at all — only the comment.
    #[test]
    fn an_unanswered_profile_writes_no_keys() {
        let text = OffPagePrefs::default().write_to_string_fragment();
        assert!(
            !text.contains(&format!("\n{PREFIX}")),
            "a fresh profile wrote a key line: {text}"
        );
    }

    /// Every answer survives the file, and nothing else is invented.
    #[test]
    fn answers_round_trip_through_the_block() {
        let mut original = OffPagePrefs::default();
        original.set("read", true);
        original.set("edit", false);

        let mut read_back = OffPagePrefs::default();
        for line in original.write_to_string_fragment().lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if line.starts_with('#') {
                continue;
            }
            assert_eq!(
                parse_key(&mut read_back, key.trim(), value.trim()),
                KeyOutcome::Accepted
            );
        }
        assert_eq!(read_back, original);
    }

    /// A prefixed key with an unreadable value is reported against ITS OWN key.
    ///
    /// `BadValue`, never `NotMine` — the distinction the caller turns into
    /// *"this value is wrong"* versus *"this key does not exist"*.
    #[test]
    fn a_bad_value_and_an_empty_mode_are_both_ours() {
        let mut prefs = OffPagePrefs::default();
        assert_eq!(
            parse_key(&mut prefs, "off_page.edit", "yes"),
            KeyOutcome::BadValue
        );
        assert_eq!(
            parse_key(&mut prefs, "off_page.", "true"),
            KeyOutcome::BadValue
        );
        assert_eq!(
            parse_key(&mut prefs, "show_rulers", "true"),
            KeyOutcome::NotMine
        );
        assert_eq!(
            prefs,
            OffPagePrefs::default(),
            "a refused line stored something"
        );
    }

    impl OffPagePrefs {
        /// This group's block on its own — a test helper, so the assertions
        /// above read the writer's real output rather than a re-implementation
        /// of it.
        fn write_to_string_fragment(&self) -> String {
            let mut out = String::new();
            write_block(self, &mut out);
            out
        }
    }
}
