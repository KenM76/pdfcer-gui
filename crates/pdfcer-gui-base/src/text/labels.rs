//! # `text::labels` — every string Pages ▸ Number pages… shows
//!
//! Consumed by `pdfcer_gui::dialogs::labels` and the labels arms of
//! `pdfcer_gui::app::actions::pages`.

use pdfcer_core::page_labels::{LabelFormat, LabelRange, LabelStyle};

/// The window's title.
#[must_use]
pub fn window_title() -> &'static str {
    "Number pages"
}

/// What the window does, in one line.
#[must_use]
pub fn intro() -> &'static str {
    "Choose how a run of pages is numbered in the page box and the page thumbnails — for example i–iv for front matter, then 1 onwards. The pages themselves are not changed; to print numbers on them, use Bates numbering."
}

/// The page-range row's leading words.
#[must_use]
pub fn from_page() -> &'static str {
    "Pages from"
}

/// Between the two page numbers.
#[must_use]
pub fn to_page() -> &'static str {
    "to"
}

/// After the range: how many pages the document has.
#[must_use]
pub fn of_pages(total: usize) -> String {
    format!("of {total}")
}

/// The style picker's label.
#[must_use]
pub fn style_label() -> &'static str {
    "Style"
}

/// A style's name in the picker.
#[must_use]
pub fn style_name(style: LabelStyle) -> &'static str {
    match style {
        LabelStyle::Decimal => "1, 2, 3",
        LabelStyle::UpperRoman => "I, II, III",
        LabelStyle::LowerRoman => "i, ii, iii",
        LabelStyle::UpperLetters => "A, B, C",
        LabelStyle::LowerLetters => "a, b, c",
        _ => "No number (the prefix alone)",
    }
}

/// The prefix box's label.
#[must_use]
pub fn prefix_label() -> &'static str {
    "Prefix"
}

/// The prefix box's tip.
#[must_use]
pub fn prefix_tooltip() -> &'static str {
    "Text before each number, such as \"A-\" for A-1, A-2. Leave it empty for none."
}

/// The start box's label.
#[must_use]
pub fn start_label() -> &'static str {
    "Start at"
}

/// The start box's tip.
#[must_use]
pub fn start_tooltip() -> &'static str {
    "The number the first page of the run gets, from 1. With the style i, ii, iii, a start of 5 begins at v."
}

/// The preview line under the form.
#[must_use]
pub fn preview(first: usize, last: usize, labels: &[String]) -> String {
    let shown = match labels {
        [] => String::new(),
        [only] => format!("\"{only}\""),
        [a, b] => format!("\"{a}\", \"{b}\""),
        [a, .., z] => format!("\"{a}\" … \"{z}\""),
    };
    if first == last {
        format!("Page {} will be labelled {shown}.", first + 1)
    } else {
        format!("Pages {}–{} will be labelled {shown}.", first + 1, last + 1)
    }
}

/// The range is back to front or off the end.
#[must_use]
pub fn bad_range(total: usize) -> String {
    format!("The pages must run from a lower number to a higher one, within 1–{total}.")
}

/// The current labels, when there are none.
#[must_use]
pub fn none_yet() -> &'static str {
    "This document does not label its pages; they are numbered 1 onwards."
}

/// The current labels' heading.
#[must_use]
pub fn current_heading() -> &'static str {
    "Labels now:"
}

/// One stored range, for the current-labels list.
#[must_use]
pub fn range_line(range: &LabelRange, next_first: Option<usize>, total: usize) -> String {
    let end = next_first.unwrap_or(total).min(total);
    let pages = if end <= range.first_page + 1 {
        format!("Page {}", range.first_page + 1)
    } else {
        format!("Pages {}–{end}", range.first_page + 1)
    };
    format!("{pages}: {}", describe(&range.format))
}

/// A format in words: its first label and its style.
#[must_use]
pub fn describe(format: &LabelFormat) -> String {
    let first = format.label(0);
    match format.style {
        LabelStyle::PrefixOnly => format!("\"{first}\" on every page"),
        _ => format!("from \"{first}\" ({})", style_name(format.style)),
    }
}

/// The commit button.
#[must_use]
pub fn apply_button() -> &'static str {
    "Apply"
}

/// The clear button.
#[must_use]
pub fn clear_button() -> &'static str {
    "Remove all labels"
}

/// The clear button's tip.
#[must_use]
pub fn clear_tooltip() -> &'static str {
    "Number every page 1 onwards again, as a document without labels is."
}

/// The close button.
#[must_use]
pub fn close_button() -> &'static str {
    "Close"
}

/// Applied.
#[must_use]
pub fn applied(first: usize, last: usize, ranges: usize) -> String {
    let pages = if first == last {
        format!("page {}", first + 1)
    } else {
        format!("pages {}–{}", first + 1, last + 1)
    };
    let stored = if ranges == 1 {
        "1 numbering range".to_owned()
    } else {
        format!("{ranges} numbering ranges")
    };
    format!("Relabelled {pages}; the document now stores {stored}.")
}

/// Cleared.
#[must_use]
pub fn cleared() -> &'static str {
    "Removed the page labels; every page is numbered 1 onwards."
}

/// Nothing to clear.
#[must_use]
pub fn nothing_to_clear() -> &'static str {
    "This document has no page labels to remove."
}
