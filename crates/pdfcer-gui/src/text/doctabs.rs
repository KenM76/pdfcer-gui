//! # `text::doctabs` — what a document tab says, and what a page drag says it
//! is about to do
//!
//! Two families of string, and they are here together because they are read in
//! the same gesture: the operator drags a page out of one document, reads the
//! tab strip to find the other, and reads the caption to check where it will
//! land.
//!
//! ## The unsaved marker is a PREFIX, and that is not a style choice
//!
//! A tab is truncated from the right with an ellipsis when the strip is
//! crowded — which is exactly when several documents are open, which is
//! exactly when knowing which of them has unsaved work matters most. A
//! trailing marker is the first thing the ellipsis eats. Word, Bluebeam and
//! Notepad++ all put theirs after the name and all three are showing a name
//! that has room; a strip of nine drawings is not.
//!
//! So it goes in front, where truncation cannot reach it.
//!
//! ## And the tooltip is the whole path, always
//!
//! `SW41177.pdf` and `SW41177.pdf` are two different drawings when they are in
//! two different job folders, and a CAD office has that situation constantly.
//! The label is the file name because that is what fits; the tooltip is the
//! location because that is what disambiguates. Neither on its own is enough.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/doctabs.md`.

use std::path::Path;

/// The **unsaved marker**, in front of the name.
const UNSAVED_MARKER: char = '*';

/// The label on a document's tab.
#[must_use]
pub fn tab_label(path: &Path, unsaved: bool) -> String {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into(),
    );
    if unsaved {
        format!("{UNSAVED_MARKER}{name}")
    } else {
        name
    }
}

/// The hover text on an **open** document's tab: where it is, and whether it
/// has unsaved work.
///
/// Two sentences rather than one, because they answer two different questions
/// and an operator scanning a strip of tabs is usually asking only one of them.
#[must_use]
pub fn tab_tooltip_open(path: &Path, unsaved: bool) -> String {
    let where_it_is = path.display();
    if unsaved {
        format!("{where_it_is}\nThis document has edits that have not been saved.")
    } else {
        where_it_is.to_string()
    }
}

/// The hover text on a **created** document's tab.
#[must_use]
pub fn tab_tooltip_created(name: &Path) -> String {
    format!(
        "{} — made in this session and never saved to a file.",
        name.display()
    )
}

/// The hover text on a tab whose file would not open, whichever of the three
/// ways it failed.
#[must_use]
pub fn tab_tooltip_unopened(path: &Path, reason: &str) -> String {
    format!("{}\n{reason}", path.display())
}

/// The reason line for a tab waiting on a password.
#[must_use]
pub const fn tab_reason_needs_password() -> &'static str {
    "This document is encrypted and pdfcer has not been given the password."
}

/// **The window title**, from what is open.
#[must_use]
pub fn window_title(active: Option<&Path>, count: usize, read_mode: Option<&str>) -> String {
    let base = crate::text::window_title();
    let stamp = build_day();
    // One join, in one place. The alternative — four `format!`s each with the
    // prefix threaded in — is four chances for one of them to drop it, and the
    // one that dropped it would be the no-document form, which is exactly the
    // state an operator reaches by closing a file *while in read mode*.
    let lead = |rest: String| match read_mode {
        Some(exit) => format!("{exit} — {rest}"),
        None => rest,
    };
    let Some(active) = active else {
        return lead(format!("{base} — {stamp}"));
    };
    let name = tab_label(active, false);
    if count > 1 {
        lead(format!(
            "{name} — {count} documents open — {base} — {stamp}"
        ))
    } else {
        lead(format!("{name} — {base} — {stamp}"))
    }
}

/// **The day this build was made**, for the window title.
fn build_day() -> &'static str {
    stamp_for_title(env!("PDFCER_BUILD_TIME"))
}

/// [`build_day`]'s rule, over a stamp passed in so it can be tested.
fn stamp_for_title(stamp: &'static str) -> &'static str {
    // `YYYY-MM-DD HH:MM` is sixteen characters. Anything shorter is not a shape
    // this function knows, so it is shown whole.
    let Some((minute_end, _)) = stamp.char_indices().nth(16) else {
        return stamp;
    };
    // The zone is whatever follows, and only a NON-local one is kept. `UTC`
    // is the fallback's label; a numeric offset means the packager set it from
    // the machine's own clock and the operator is already standing in it.
    let zone = stamp[minute_end..].trim();
    if zone.starts_with('+') || zone.starts_with('-') || zone.is_empty() {
        &stamp[..minute_end]
    } else {
        stamp
    }
}

// ===========================================================================
// The page drag
// ===========================================================================

/// **Where a page drag would land, when it lands in the document it came
/// from.** A reorder.
#[must_use]
pub fn drag_landing_here(moving: usize, gap: usize, page_count: usize) -> String {
    crate::text::pages::drag_landing(moving, gap, page_count)
}

/// **Where a page drag would land, when it lands in a DIFFERENT document.**
#[must_use]
pub fn drag_landing_other(moving: usize, gap: usize, source: &str, page_count: usize) -> String {
    let sheets = if moving == 1 { "sheet" } else { "sheets" };
    if gap >= page_count {
        format!("Copy {moving} {sheets} from {source} to the end.")
    } else {
        format!(
            "Copy {moving} {sheets} from {source} to before page {}.",
            gap + 1
        )
    }
}

/// **Where a page drag would land, when Shift is held and it therefore MOVES.**
#[must_use]
pub fn drag_landing_move(moving: usize, gap: usize, source: &str, page_count: usize) -> String {
    let sheets = if moving == 1 { "sheet" } else { "sheets" };
    let where_to = if gap >= page_count {
        "to the end".to_owned()
    } else {
        format!("to before page {}", gap + 1)
    };
    format!("Move {moving} {sheets} {where_to} — they will be REMOVED from {source}.")
}

/// **The copy caption with the hint that the other half of the gesture
/// exists.**
#[must_use]
pub fn drag_landing_copy_with_hint(
    moving: usize,
    gap: usize,
    source: &str,
    page_count: usize,
) -> String {
    format!(
        "{} Hold Shift to move them instead.",
        drag_landing_other(moving, gap, source, page_count)
    )
}

/// **What a move actually did**, on the status row afterwards.
#[must_use]
pub fn moved_out_of(moving: usize, source: &str) -> String {
    let sheets = if moving == 1 {
        "sheet was"
    } else {
        "sheets were"
    };
    format!(
        "{moving} {sheets} removed from {source}. Undo works one document at a time, so \
         undoing this here does not put them back there."
    )
}

/// A move inserted its pages and could not remove them from the source.
#[must_use]
pub fn move_left_the_source_alone(source: &str) -> String {
    format!(
        "The sheets were placed here and could NOT be removed from {source}, so they are in \
         both documents now. Switch to {source} and delete them there if you meant to move them."
    )
}

/// The drag is over something that is not a drop target.
#[must_use]
pub const fn drag_over_nothing() -> &'static str {
    "Drop this on a page list or on the page view to place it."
}

/// The whole document is being dragged and the target is the document it came
/// from — a copy of a document into itself.
#[must_use]
pub const fn drag_refused_self_copy() -> &'static str {
    // "the Pages tab" rather than the ribbon-path spelling with a U+25B8
    // in it. `icons::glyphs` refuses that codepoint in operator-visible
    // strings and is right to: the font stack cannot draw it, so it renders as
    // a substitution box — and this sentence's whole job is to give
    // directions. `text::dropped` carries the same note for the same reason.
    "Dragging every page of a document into itself would double it. Pick the sheets you want, \
     or use Insert from file on the Pages tab."
}

/// The document a drag would land in cannot take pages.
#[must_use]
pub fn drag_target_refused(reason: &str) -> String {
    format!("Those pages could not be placed here. {reason}")
}

#[cfg(test)]
mod title_stamp_tests {
    use super::stamp_for_title;

    /// **A packaged build shows the time and drops the offset.**
    #[test]
    fn a_packaged_stamp_shows_the_local_time_without_its_offset() {
        assert_eq!(
            stamp_for_title("2026-09-02 06:25 +0100"),
            "2026-09-02 06:25"
        );
        assert_eq!(
            stamp_for_title("2026-09-02 06:25 -0400"),
            "2026-09-02 06:25"
        );
    }

    /// **A dev build KEEPS its `UTC`, and that is the point of the rule.**
    #[test]
    fn an_unlocalised_stamp_keeps_its_zone() {
        assert_eq!(
            stamp_for_title("2026-09-02 06:25 UTC"),
            "2026-09-02 06:25 UTC"
        );
    }

    /// Anything unrecognised is shown whole rather than replaced.
    #[test]
    fn an_unexpected_shape_is_shown_whole() {
        assert_eq!(stamp_for_title("unknown"), "unknown");
        assert_eq!(stamp_for_title(""), "");
        assert_eq!(stamp_for_title("2026-09-02"), "2026-09-02");
    }

    /// The date still leads, so the title is sortable by eye.
    #[test]
    fn the_date_still_comes_first() {
        let out = stamp_for_title("2026-09-02 06:25 +0100");
        assert!(out.starts_with("2026-09-02"));
        assert!(out.contains("06:25"), "the time is the whole point: {out}");
    }
}

/// The read-mode prefix on the window title — `OPERATOR_REQUESTS.md` O115.
#[cfg(test)]
mod title_read_mode_tests {
    use super::window_title;
    use std::path::Path;

    /// **The ordinary title says nothing about read mode**, and this is the
    /// assertion the obvious wrong implementation fails.
    #[test]
    fn an_ordinary_title_carries_no_hint() {
        let title = window_title(Some(Path::new("C:/jobs/SW41177.pdf")), 1, None);
        assert!(!title.contains("Read mode"), "{title}");
        assert!(title.starts_with("SW41177.pdf"), "{title}");
    }

    /// **In read mode the way out leads the title.**
    #[test]
    fn read_mode_puts_the_way_out_first() {
        let exit = crate::text::window::title_read_mode("Ctrl+H");
        let title = window_title(Some(Path::new("C:/jobs/SW41177.pdf")), 3, Some(&exit));
        assert!(
            title.starts_with(&exit),
            "the exit must survive a truncated taskbar button: {title}"
        );
        assert!(title.contains("SW41177.pdf"), "{title}");
        assert!(title.contains("3 documents open"), "{title}");
    }

    /// **The build stamp is still last**, in every form.
    #[test]
    fn the_build_stamp_is_still_the_last_field() {
        let exit = crate::text::window::title_read_mode("Ctrl+H");
        for title in [
            window_title(None, 0, Some(&exit)),
            window_title(Some(Path::new("a.pdf")), 1, Some(&exit)),
            window_title(Some(Path::new("a.pdf")), 4, Some(&exit)),
        ] {
            let tail = title.rsplit('—').next().unwrap_or_default().trim();
            assert!(
                tail.starts_with(|c: char| c.is_ascii_digit()) || tail == super::build_day(),
                "the trailing field must still be the build stamp: {title}"
            );
            assert!(title.starts_with(&exit), "{title}");
        }
    }

    /// **With no document open the hint is still there.**
    #[test]
    fn closing_the_last_document_does_not_lose_the_hint() {
        let exit = crate::text::window::title_read_mode("Ctrl+H");
        let title = window_title(None, 0, Some(&exit));
        assert!(title.starts_with(&exit), "{title}");
        assert!(title.contains(crate::text::window_title()), "{title}");
    }
}
