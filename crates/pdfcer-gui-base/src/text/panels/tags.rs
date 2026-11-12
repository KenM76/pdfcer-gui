//! # `text::panels::tags` — every string the Tags panel shows
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/tags.md`.

use crate::tagtree::NoBox;

/// Shown in place of a tree for a file with no structure tree.
#[must_use]
pub fn untagged() -> &'static str {
    "This document is untagged: it has no structure tree, so there are no tags to show."
}

/// Shown when the tree could not be read; `why` is the engine's message.
#[must_use]
pub fn failed(why: &str) -> String {
    format!("The structure tree could not be read: {why}")
}

/// Shown for a structure tree with no elements in it.
#[must_use]
pub fn empty() -> &'static str {
    "The document has a structure tree, but it holds no elements."
}

/// The summary line above the tree.
#[must_use]
pub fn count(elements: usize) -> String {
    if elements == 1 {
        "1 structure element.".to_owned()
    } else {
        format!("{elements} structure elements.")
    }
}

/// Elements reached a second time, which the reader skipped.
#[must_use]
pub fn revisited(n: usize) -> String {
    format!(
        "{n} element(s) are reached more than once (a cycle or a shared kid); each is shown once."
    )
}

/// Content whose page could not be resolved.
#[must_use]
pub fn unresolved(n: usize) -> String {
    format!("{n} content item(s) name a page that could not be found.")
}

/// MCIDs the tree names that no page declares.
#[must_use]
pub fn not_declared(n: usize) -> String {
    format!("{n} marked-content reference(s) name content no page declares.")
}

/// The hint under the tree before anything is picked.
#[must_use]
pub fn pick_hint() -> &'static str {
    "Click an element to outline its content on the page."
}

/// The tree's nesting limit was reached.
#[must_use]
pub fn too_deep() -> &'static str {
    "Deeper levels are not shown."
}

/// The facts grid's row labels.
pub mod field {
    /// The resolved type.
    #[must_use]
    pub fn kind() -> &'static str {
        "Type"
    }
    /// The type as written, before the role map.
    #[must_use]
    pub fn written() -> &'static str {
        "Written as"
    }
    /// `/T`.
    #[must_use]
    pub fn title() -> &'static str {
        "Title"
    }
    /// `/Alt`.
    #[must_use]
    pub fn alt() -> &'static str {
        "Alternate text"
    }
    /// `/ActualText`.
    #[must_use]
    pub fn actual() -> &'static str {
        "Actual text"
    }
    /// `/Lang`, or the inherited language.
    #[must_use]
    pub fn lang() -> &'static str {
        "Language"
    }
    /// A header cell's `Scope`.
    #[must_use]
    pub fn scope() -> &'static str {
        "Scope"
    }
    /// A cell's `Headers`.
    #[must_use]
    pub fn headers() -> &'static str {
        "Headers"
    }
    /// `RowSpan`.
    #[must_use]
    pub fn row_span() -> &'static str {
        "Row span"
    }
    /// `ColSpan`.
    #[must_use]
    pub fn col_span() -> &'static str {
        "Column span"
    }
    /// The pages the outline is drawn on.
    #[must_use]
    pub fn pages() -> &'static str {
        "Outlined on"
    }
}

/// A language the element inherits rather than states.
#[must_use]
pub fn inherited(lang: &str) -> String {
    format!("{lang} (inherited)")
}

/// The pages an outline is drawn on, 1-based.
#[must_use]
pub fn page_list(pages: &[usize]) -> String {
    let list: Vec<String> = pages.iter().map(|p| (p + 1).to_string()).collect();
    if pages.len() == 1 {
        format!("page {}", list[0])
    } else {
        format!("pages {}", list.join(", "))
    }
}

/// A cell's `Headers`, the IDs of the header cells it names.
#[must_use]
pub fn header_list(ids: &[String]) -> String {
    ids.join(", ")
}

/// Why the picked element has no outline.
#[must_use]
pub fn no_box(why: NoBox) -> &'static str {
    match why {
        NoBox::NoContent => "No outline: neither this element nor anything inside it owns content.",
        NoBox::PageUnresolved => "No outline: its content names a page that could not be found.",
        NoBox::McidNotFound => {
            "No outline: its marked content is not declared on the page it names."
        }
        NoBox::NothingTextual => {
            "No outline: its content paints no text. This build outlines text and annotations \
             only, so a rule, a fill or an image is not outlined yet."
        }
    }
}

/// A row's open/close button, the Objects tree's glyphs.
#[must_use]
pub fn toggle(open: bool) -> &'static str {
    if open { "v" } else { ">" }
}

/// A row's label: the element's type, and its `/T` when it has one.
#[must_use]
pub fn row(kind: &str, title: Option<&str>) -> String {
    match title {
        Some(title) if !title.is_empty() => format!("{kind} \u{201C}{title}\u{201D}"),
        _ => kind.to_owned(),
    }
}
