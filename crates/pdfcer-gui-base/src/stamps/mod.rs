//! # Acrobat-compatible **stamp collections** — the shell's model
//!
//! Engine `Pass 288.0` (`pdfcer_core::stamp_file`) reads and writes the file
//! format. This module is the part that cannot live in the engine: turning a
//! document into a *plan* an operator can look at and correct, and turning
//! what the engine read into sentences a panel can show.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/stamps/mod.md`.

use pdfcer_core::stamp_file::{StampCollection, StampEntry};

/// Where Acrobat looks for the operator's own stamps, so the picker can
/// suggest it. Read-only discovery; it creates nothing.
pub mod folder;
/// **The operator's own stamps, found and offered** — O172's half, and the
/// answer to the "What is NOT here" note below, which was true until engine
/// `Pass 293.0` shipped `place_page_artwork`.
pub mod library;

/// **Which stamp he reached for last** -- O172's *"and it remembers the
/// last one used"* clause. A NAME re-resolved against a fresh scan, never a
/// stored `CustomStamp`, because editing a collection in Acrobat renumbers its
/// pages and a remembered index would silently name a different stamp.
pub mod lastused;
/// Turning a [`Plan`] into the bytes of a file Acrobat will load.
pub mod write;

#[cfg(test)]
mod tests;

/// The longest an internal name may be before this module truncates it.
pub const MAX_INTERNAL_LEN: usize = 64;

/// What happened to a display name on its way to becoming an internal name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Adjustment {
    /// Characters that cannot live in a name-tree key were dropped.
    ///
    /// Carries the count rather than the characters: the operator can see what
    /// he typed, and a sentence listing `\t` back at him explains nothing he
    /// did not already know.
    CharactersRemoved(usize),
    /// A leading `#` was removed because it marks a **dynamic** stamp.
    ///
    /// The one adjustment that is about correctness rather than tidiness.
    /// Keeping it would write a stamp Acrobat believes recomputes its own text
    /// and which never will.
    DynamicMarkerRemoved,
    /// A number was appended because another page already claimed this name.
    ///
    /// Carries the name that was actually written, because *"we renamed it"*
    /// without saying to what is a disclosure that costs the operator a
    /// round trip to check.
    MadeUnique(String),
    /// The name was longer than [`MAX_INTERNAL_LEN`] and was cut.
    Truncated,
}

/// One page's entry in a collection about to be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedStamp {
    /// 0-based page index. **The identity of the row.** Never reordered —
    /// see the module header on why tree order is not page order.
    pub page_index: usize,
    /// What the operator typed, verbatim, and what a picker will show.
    ///
    /// Stored unedited on purpose: it is written to the file's display half
    /// unchanged, and it is what the dialog echoes back. Only the *internal*
    /// half is sanitised.
    pub display: String,
    /// The sanitised, unique key. Derived, never typed.
    pub internal: String,
    /// What [`derive_internal`] had to do to get there.
    pub adjustments: Vec<Adjustment>,
    /// Whether this page is included in the collection at all.
    ///
    /// A page can be excluded. The engine's `name_stamp_pages` names
    /// `stamps[i]` to page `i` positionally, so an excluded page is not a
    /// gap in that list — [`Plan::for_engine`] rebuilds the list so the
    /// positional contract still holds. Getting this wrong would name the
    /// wrong artwork, silently, which is why it has its own test.
    pub include: bool,
}

/// A name a collection already carries, reduced to the two fields a plan
/// needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingName {
    /// The page this name points at, when it points at one of this document's.
    ///
    /// ⚠ `None` still is **not** "this stamp is broken" — it means the name
    /// cannot be attached to a row, which is all a [`Plan`] needs to know.
    /// Since engine `Pass 290.1` the *reason* is answerable
    /// ([`page_tree_unreadable`]), but it is deliberately not carried here:
    /// the reason changes what an operator should be **told**, and nothing
    /// about which row a name attaches to.
    pub page_index: Option<usize>,
    /// The display half of the stored `internal=display` string.
    ///
    /// The internal half is deliberately dropped: re-opening a collection
    /// re-derives every identifier from the display names, so carrying the old
    /// one would let a name Acrobat wrote survive an edit that should have
    /// changed it.
    pub display: String,
}

/// Reduce what [`pdfcer_core::stamp_file::read`] returned to what a [`Plan`]
/// needs.
///
/// The one place the engine's type is converted, so the narrowing argued for
/// on [`ExistingName`] has exactly one implementation to keep honest.
#[must_use]
pub fn existing_names(collection: &StampCollection) -> Vec<ExistingName> {
    collection
        .stamps
        .iter()
        .map(|s: &StampEntry| ExistingName {
            page_index: s.page_index,
            display: s.display.clone(),
        })
        .collect()
}

/// A whole collection, as it will be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The category — the file's `/Info` `/Title`.
    pub category: String,
    /// One row per page in the source document, in page order.
    pub stamps: Vec<PlannedStamp>,
}

impl Plan {
    /// Build a plan for a document with `pages` pages.
    /// `default_name` words a page number for a page nothing else names.
    #[must_use]
    pub fn new(
        pages: usize,
        category: &str,
        existing: &[ExistingName],
        default_name: fn(usize) -> String,
    ) -> Self {
        let by_page = |index: usize| -> Option<&ExistingName> {
            existing.iter().find(|s| s.page_index == Some(index))
        };

        let mut taken: Vec<String> = Vec::new();
        let mut stamps = Vec::with_capacity(pages);
        for page_index in 0..pages {
            let found = by_page(page_index);
            let display = found
                .map(|s| s.display.clone())
                .filter(|d| !d.is_empty())
                .unwrap_or_else(|| default_name(page_index.saturating_add(1)));
            let (internal, adjustments) = derive_internal(&display, &taken);
            taken.push(internal.clone());
            stamps.push(PlannedStamp {
                page_index,
                display,
                internal,
                adjustments,
                include: true,
            });
        }

        Self {
            category: category.to_owned(),
            stamps,
        }
    }

    /// Recompute every internal name from the current display names.
    pub fn rederive(&mut self) {
        let mut taken: Vec<String> = Vec::new();
        for stamp in &mut self.stamps {
            if !stamp.include {
                // Excluded rows claim no name. Including them in `taken` would
                // push a number onto a row the operator can see and cannot
                // explain, because the row that took the name is not in the
                // file.
                stamp.internal = String::new();
                stamp.adjustments.clear();
                continue;
            }
            let (internal, adjustments) = derive_internal(&stamp.display, &taken);
            taken.push(internal.clone());
            stamp.internal = internal;
            stamp.adjustments = adjustments;
        }
    }

    /// The `(internal, display)` list, positionally aligned to the pages that
    /// will exist in the written document.
    #[must_use]
    pub fn for_engine(&self) -> Vec<(String, String)> {
        self.stamps
            .iter()
            .filter(|s| s.include)
            .map(|s| (s.internal.clone(), s.display.clone()))
            .collect()
    }

    /// The 0-based page indices to carry into the collection, in page order.
    #[must_use]
    pub fn pages_to_extract(&self) -> Vec<usize> {
        self.stamps
            .iter()
            .filter(|s| s.include)
            .map(|s| s.page_index)
            .collect()
    }

    /// Every adjustment in the plan, with the page it happened on.
    #[must_use]
    pub fn adjustments(&self) -> Vec<(usize, &Adjustment)> {
        self.stamps
            .iter()
            .filter(|s| s.include)
            .flat_map(|s| s.adjustments.iter().map(move |a| (s.page_index, a)))
            .collect()
    }

    /// How many stamps this plan will write.
    #[must_use]
    pub fn included(&self) -> usize {
        self.stamps.iter().filter(|s| s.include).count()
    }

    /// Why this plan cannot be written, if it cannot.
    #[must_use]
    pub fn blocker(&self) -> Option<Blocker> {
        if self.included() == 0 {
            return Some(Blocker::NoStamps);
        }
        if self.category.trim().is_empty() {
            return Some(Blocker::NoCategory);
        }
        None
    }
}

/// Why a plan cannot be written yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blocker {
    /// Every page was excluded.
    NoStamps,
    /// The category name is empty. It becomes `/Info` `/Title`, which is what
    /// Acrobat's stamp menu uses as the submenu heading.
    NoCategory,
}

/// Turn a display name into a legal, unique internal name.
#[must_use]
pub fn derive_internal(display: &str, taken: &[String]) -> (String, Vec<Adjustment>) {
    let mut adjustments = Vec::new();

    // 1 — keep what survives a round trip.
    let kept: String = display
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '#')
        .collect();
    let removed = display.chars().count().saturating_sub(kept.chars().count());
    if removed > 0 {
        adjustments.push(Adjustment::CharactersRemoved(removed));
    }

    // 2 — the dynamic marker, and every `#` after it. A `#` anywhere in a key
    // is legal, but one that could migrate to the front through a later edit
    // is a trap, and no Adobe key contains a non-leading `#`.
    let had_marker = kept.starts_with('#');
    let mut name: String = kept.chars().filter(|c| *c != '#').collect();
    if had_marker {
        adjustments.push(Adjustment::DynamicMarkerRemoved);
    }

    // 3 — something must survive.
    if name.is_empty() {
        name = FALLBACK_INTERNAL.to_owned();
    }

    // 4 — length.
    if name.chars().count() > MAX_INTERNAL_LEN {
        name = name.chars().take(MAX_INTERNAL_LEN).collect();
        adjustments.push(Adjustment::Truncated);
    }

    // 5 — uniqueness (§7.9.6 keys are unique).
    if taken.iter().any(|t| t == &name) {
        let base = name.clone();
        let mut n = 2_u32;
        loop {
            let candidate = format!("{base}{n}");
            if !taken.contains(&candidate) {
                name = candidate;
                break;
            }
            n = n.saturating_add(1);
        }
        adjustments.push(Adjustment::MadeUnique(name.clone()));
    }

    (name, adjustments)
}

/// The internal name used when a display name sanitises to nothing.
///
/// `// ui-text-exempt:` — this is a **file-format identifier**, not a label.
/// It is written into a PDF name tree, compared byte-for-byte by readers, and
/// never shown to an operator; the *display* half beside it is whatever he
/// typed. Translating it would change the bytes in the file.
const FALLBACK_INTERNAL: &str = "Stamp"; // ui-text-exempt: a name-tree key written into the file, never displayed.

/// Whether an open document is a stamp collection at all.
#[must_use]
pub fn is_collection(collection: &StampCollection) -> bool {
    collection.is_stamp_file()
}

/// How many of a collection's stamps are dynamic.
#[must_use]
pub fn dynamic_count(collection: &StampCollection) -> usize {
    collection.stamps.iter().filter(|s| s.dynamic).count()
}

/// Why this document's page tree could not be read, when it could not be.
#[must_use]
pub fn page_tree_unreadable(collection: &StampCollection) -> Option<&str> {
    collection.page_tree_error.as_deref()
}

/// Stamps whose named page is not one of the document's own — or `None` when
/// that question is unanswerable.
#[must_use]
pub fn unresolved_count(collection: &StampCollection) -> Option<usize> {
    if collection.page_tree_error.is_some() {
        return None;
    }
    Some(
        collection
            .stamps
            .iter()
            .filter(|s| s.page_index.is_none())
            .count(),
    )
}
