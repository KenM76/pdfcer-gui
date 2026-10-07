//! Document properties ▸ Security notes, and the open-time wrapper warning.
//!
//! Every sentence says what the file CAN do and that pdfcer does none of it;
//! `text::reachout`'s header carries why both halves are needed.

/// The section heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Security notes"
}

/// Over the action rows.
#[must_use]
pub const fn actions_heading() -> &'static str {
    "What this file would run in Acrobat or Reader"
}

/// Nothing found by a walk that finished.
#[must_use]
pub const fn nothing_runs() -> &'static str {
    "Nothing: no scripts, links, buttons or triggers that act when the file is opened or clicked."
}

/// The walk stopped at its ceiling.
#[must_use]
pub fn stopped_looking(scanned: usize) -> String {
    format!(
        "pdfcer stopped looking after {scanned} actions, so there may be more than this. Treat \
         the file as unchecked, not clean."
    )
}

/// Under the rows.
#[must_use]
pub const fn none_run_here() -> &'static str {
    "pdfcer reads these and runs none of them; a viewer that runs them would. Nothing is \
     executed to find them."
}

/// The answer to *does it reach outside the file?*
#[must_use]
pub const fn reaches_outside(yes: bool) -> &'static str {
    if yes {
        "It reaches outside the file: it can send data somewhere or start a program."
    } else {
        "It does not reach outside the file: no submit, web link or program launch."
    }
}

/// Runs a script the moment it opens.
#[must_use]
pub const fn script_on_open() -> &'static str {
    "Runs a script as soon as it is opened"
}

/// One row label per counted carrier, in the order the rows are drawn.
#[must_use]
pub const fn row_labels() -> [&'static str; 9] {
    [
        "Document-level scripts, run when the file opens",
        "Actions when a page is opened or closed",
        "Bookmarks that do something other than go to a page",
        "Click actions on links, buttons and comments",
        "Field scripts (calculate, format, validate, keystroke)",
        "Scripts in all, wherever they sit",
        "Actions hidden behind another action",
        "Can send data somewhere (submit, web link, import)",
        "Can start another program",
    ]
}

/// One census row: what, then how many.
#[must_use]
pub fn row(label: &str, count: usize) -> String {
    format!("{label}: {count}")
}

/// The status-line warning for a §7.6.7 wrapper: the engine's sentence, then
/// where the rest is written.
#[must_use]
pub fn wrapper_status(engine: &str) -> String {
    format!("{engine} File > Document properties > Security notes says the same.")
}

/// One field holding a stored password value, and the saved version it is
/// in (1-based, of `of`). Never the value.
#[must_use]
pub fn stored_password_at(field: &str, version: usize, of: usize) -> String {
    format!("{field} (version {version} of {of})")
}

/// Password values the file holds, each named by
/// [`stored_password_at`]; `earlier` and `current` count them by whether a
/// reader still shows the version they are in. `remedy` is the command that
/// removes them.
#[must_use]
pub fn stored_passwords(found: &[String], earlier: usize, current: usize, remedy: &str) -> String {
    let place = match (earlier, current) {
        (_, 0) => "in earlier versions a reader no longer shows",
        (0, _) => "in the version a reader opens",
        _ => "in earlier versions and in the version a reader opens",
    };
    format!(
        "This file still holds what was typed into password fields, {place}: {}. Hidden or not, \
         the values are in the file's bytes. File > Security > {remedy} writes a copy without \
         them.",
        found.join(", ")
    )
}

/// Saved versions of a form that could not be opened on their own, so were
/// not checked for stored passwords.
#[must_use]
pub fn stored_passwords_unchecked(count: usize) -> String {
    let (word, verb) = if count == 1 {
        ("version", "was")
    } else {
        ("versions", "were")
    };
    format!(
        "{count} saved {word} of this form could not be opened on its own and {verb} not checked \
         for stored passwords."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stopped_walk_never_reads_as_clean() {
        let s = stopped_looking(4096);
        assert!(
            s.contains("4096") && s.contains("unchecked, not clean"),
            "{s}"
        );
    }

    #[test]
    fn stored_passwords_name_field_version_and_remedy_and_say_where() {
        let found = [stored_password_at("PIN", 1, 2)];
        let s = stored_passwords(&found, 1, 0, "Remove old passwords…");
        assert!(
            s.contains("PIN (version 1 of 2)") && s.contains("Remove old passwords…"),
            "{s}"
        );
        assert!(s.contains("no longer shows"), "{s}");
        assert_ne!(s, stored_passwords(&found, 0, 1, "x"));
    }

    #[test]
    fn the_two_answers_about_reaching_outside_differ() {
        assert_ne!(reaches_outside(true), reaches_outside(false));
    }
}
