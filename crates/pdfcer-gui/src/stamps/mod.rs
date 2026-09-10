//! # Acrobat-compatible **stamp collections** — the shell's model
//!
//! Engine `Pass 288.0` (`pdfcer_core::stamp_file`) reads and writes the file
//! format. This module is the part that cannot live in the engine: turning a
//! document into a *plan* an operator can look at and correct, and turning
//! what the engine read into sentences a panel can show.
//!
//! ## What a custom stamp actually is, because the answer changes the UI
//!
//! **There is no stamp format and no import/export.** A stamp collection is an
//! ordinary PDF:
//!
//! * **one file per category, one page per stamp**;
//! * the **category name** is the file's `/Info` `/Title`;
//! * the stamp names live in the catalog's `/Names` → `/Pages` name tree
//!   (§7.7.4 Table 31), one string per stamp in the form `internal=display`;
//! * an internal name beginning `#` marks a **dynamic** stamp, whose text
//!   Acrobat recomputes from AcroForm scripts when it is placed.
//!
//! The operator asked for *"the same import/export"* Acrobat has (`O169`), and
//! the honest answer is that Acrobat has none — handing someone the PDF **is**
//! the export. That is why this module has no serialiser: there is nothing to
//! serialise to. It has a *plan* and a *save path*, and the save path is an
//! ordinary PDF write through the ordinary save route.
//!
//! ## ★★ Every claim above is measured, in Adobe's own files, on this machine
//!
//! Not sourced from the internet. The engine measured them from
//! `…/Acrobat DC/Acrobat/plug_ins/Annotations/Stamps/ENU/*.pdf`, and this
//! module's author re-ran that measurement through the release CLI before
//! writing a line of it:
//!
//! ```text
//! Standard.pdf          category=Standard           14 stamps
//! StandardBusiness.pdf  category=Standard Business  12 stamps
//! Dynamic.pdf           category=Dynamic             5 stamps, all `#`-prefixed
//! SignHere.pdf          category=Sign Here           5 stamps
//! ```
//!
//! ★ **`StandardBusiness.pdf` is the one that disproves the obvious design.**
//! Its `SBApproved` names page 1 and `SBCompleted` names page 5. **Page order
//! is not name-tree order** — §7.9.6 requires the tree be sorted
//! lexicographically by name, and a conforming reader may binary-search it. A
//! surface that lists pages in file order and labels them from the tree in
//! tree order will mislabel every stamp and look completely correct while
//! doing it. Everything here is keyed by page index and the sorting is left to
//! [`pdfcer_core::stamp_file::name_stamp_pages`], which does it in the engine
//! where the requirement is documented.
//!
//! ## What pdfcer authors, and the one thing it deliberately does not
//!
//! **Dynamic stamps are read and reported, never authored.** Their text comes
//! from AcroForm calculation JavaScript; a dynamic stamp pdfcer wrote would
//! carry its design-time text, which the engine's own header calls *"correct
//! as a picture, wrong as a promise"* — a stamp reading `Received 10 Sep 2026`
//! forever, on every drawing, is worse than no stamp.
//!
//! That has a consequence this module enforces rather than hopes for: a
//! display name the operator types must never *become* a dynamic marker.
//! See [`derive_internal`] and its `#`-stripping clause.
//!
//! ## R8b rule 4 — where the disclosures in here come from
//!
//! Three things happen to an operator's typing on the way to a name tree, and
//! each is an inference he did not ask for:
//!
//! | inference | why it happens | disclosed as |
//! |---|---|---|
//! | characters removed | a name-tree key is a PDF string; a tab or a newline in one is a file nobody can read back reliably | [`Adjustment::CharactersRemoved`] |
//! | a leading `#` removed | `#` means *dynamic*, and pdfcer does not author dynamic stamps — writing one would promise recomputation that never happens | [`Adjustment::DynamicMarkerRemoved`] |
//! | a number appended | §7.9.6 keys are unique; two pages both called *Approved* cannot both be `Approved` | [`Adjustment::MadeUnique`] |
//!
//! All three are reported **off-canvas**, in the dialog that is about to write
//! the file, before it is written. None of them marks anything on a page.
//!
//! ## What is NOT here, and where it went instead
//!
//! **Placing a custom stamp on a drawing.** A custom stamp's artwork is a
//! *page*, and the engine has no verb that draws one page's artwork onto
//! another. Filed as
//! `request_a_custom_stamp_can_be_read_and_authored_but_never_placed_on_a_page.md`.
//! ★ A workaround exists — rasterise the stamp page and place it as an image —
//! and is **declined**: a bitmap stamp on a vector CAD drawing does not
//! survive zooming, inflates the file, and is not what Acrobat writes, which
//! fails the operator's actual requirement. `ENGINE_BACKLOG.md`'s `Pass 288.0`
//! row carries the full argument.

use pdfcer_core::stamp_file::{StampCollection, StampEntry};

/// Where Acrobat looks for the operator's own stamps, so the picker can
/// suggest it. Read-only discovery; it creates nothing.
pub mod folder;
/// Turning a [`Plan`] into the bytes of a file Acrobat will load.
pub mod write;

#[cfg(test)]
mod tests;

/// The longest an internal name may be before this module truncates it.
///
/// Not a format limit — §7.3.4.2 puts no ceiling on a PDF string and the name
/// tree inherits none. It is a *legibility* limit: Adobe's longest shipped
/// internal name is `SBConfidential` at fourteen characters, and a key long
/// enough to wrap in a debugger is a key nobody will ever read. Truncation is
/// disclosed like every other adjustment, so an operator who wants his
/// forty-character name knows it did not survive.
pub const MAX_INTERNAL_LEN: usize = 64;

/// What happened to a display name on its way to becoming an internal name.
///
/// Each variant is an **inference** — something pdfcer decided that the
/// operator did not type — and each is disclosed before the file is written.
/// A collection whose plan has no adjustments needs no disclosure at all,
/// which is the common case and should stay silent.
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
    /// ★ The one adjustment that is about correctness rather than tidiness.
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
    /// ★ A page can be excluded. The engine's `name_stamp_pages` names
    /// `stamps[i]` to page `i` positionally, so an excluded page is not a
    /// gap in that list — [`Plan::for_engine`] rebuilds the list so the
    /// positional contract still holds. Getting this wrong would name the
    /// wrong artwork, silently, which is why it has its own test.
    pub include: bool,
}

/// A name a collection already carries, reduced to the two fields a plan
/// needs.
///
/// ★ **Why this exists rather than passing `&StampCollection` around.** Two
/// reasons, and the second is the one that matters:
///
/// 1. `StampEntry` and `StampCollection` are both `#[non_exhaustive]`, so no
///    code outside `pdfcer-core` can build one. A model that took the engine
///    type could only ever be tested by authoring a real PDF and reading it
///    back — which is a fine integration test and a terrible unit test, and
///    the pressure would be to skip the unit test.
/// 2. **The plan does not want the engine's type.** It needs a page and a
///    name; it has no business knowing what a `dynamic` flag is, because it
///    never authors one. Narrowing at the boundary is what keeps
///    [`derive_internal`]'s `#`-stripping clause honest — there is no way for
///    a dynamic marker to arrive here disguised as something else.
///
/// [`existing_names`] does the reduction, in one place.
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
    ///
    /// `existing` is what [`pdfcer_core::stamp_file::read`] found, when the
    /// document already **is** a collection. Re-opening one must show what it
    /// already says rather than a fresh set of defaults — an operator fixing
    /// one typo in a twelve-stamp set should not have to retype eleven names.
    ///
    /// ★ `existing` is matched **by page index, not by position in the tree**.
    /// The tree is sorted lexicographically (§7.9.6) and the pages are not, so
    /// zipping the two lists would attach `SBCompleted`'s name to page 1.
    #[must_use]
    pub fn new(pages: usize, category: &str, existing: &[ExistingName]) -> Self {
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
                .unwrap_or_else(|| default_display(page_index));
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
    ///
    /// Called after any edit. It re-derives the **whole** list rather than the
    /// row that changed, because uniqueness is a property of the set: renaming
    /// row 3 from `Approved` to `Issued` frees `Approved` for row 7, and a
    /// per-row update would leave row 7 as `Approved2` forever with a stale
    /// [`Adjustment::MadeUnique`] disclosure attached to it.
    ///
    /// ★ That stale-disclosure case is the reason this is not an optimisation
    /// target. A disclosure has a subject; when the subject goes, the sentence
    /// must go with it.
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
    ///
    /// ★★ **This is the contract that is easy to get wrong.**
    /// [`pdfcer_core::stamp_file::name_stamp_pages`] names `stamps[i]` to page
    /// `i` — **it counts, it does not look up.** So the returned list must be
    /// dense from page 0 of *the document being written*, which is not the
    /// document the operator has open whenever he has excluded a page.
    ///
    /// [`Self::pages_to_extract`] is the other half, and the two are
    /// documented together because using one without the other silently names
    /// the wrong artwork: exclude page 0, hand the full name list to a
    /// document whose pages start at the old page 1, and **every stamp is
    /// attached to the page below the one it describes** — a file that opens,
    /// has the right stamp count, appears in Acrobat's menu, and stamps the
    /// wrong picture every time.
    ///
    /// The two are filtered by the same predicate in the same order, which is
    /// the cheapest way to make that invariant true by construction rather
    /// than by review. `plan_and_extraction_agree` in the tests is the guard.
    #[must_use]
    pub fn for_engine(&self) -> Vec<(String, String)> {
        self.stamps
            .iter()
            .filter(|s| s.include)
            .map(|s| (s.internal.clone(), s.display.clone()))
            .collect()
    }

    /// The 0-based page indices to carry into the collection, in page order.
    ///
    /// Fed straight to `pdfcer_core::pageops::extract`, which is how exclusion
    /// is implemented: rather than deleting pages out of a session, the writer
    /// **extracts** the wanted ones into a new document and names those. That
    /// keeps the operator's open document untouched — the same argument
    /// `app::actions::extract` makes for the page verb it owns — and it makes
    /// the density [`Self::for_engine`] requires automatic.
    #[must_use]
    pub fn pages_to_extract(&self) -> Vec<usize> {
        self.stamps
            .iter()
            .filter(|s| s.include)
            .map(|s| s.page_index)
            .collect()
    }

    /// Every adjustment in the plan, with the page it happened on.
    ///
    /// Empty when nothing was inferred, which is what lets the dialog stay
    /// silent in the common case instead of showing an always-present box
    /// that says nothing.
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
    ///
    /// Returns `None` when it can. The two refusals are real states rather
    /// than defensive checks: a collection with no stamps is not a collection,
    /// and an untitled one shows up in Acrobat's menu with no heading.
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
///
/// ⚠ R9: the Save control is **greyed** for these, not hidden, because both
/// are *temporarily* unavailable — the operator fixes them by typing — and
/// both are explained on hover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blocker {
    /// Every page was excluded.
    NoStamps,
    /// The category name is empty. It becomes `/Info` `/Title`, which is what
    /// Acrobat's stamp menu uses as the submenu heading.
    NoCategory,
}

/// The display name a page gets when nothing else names it.
///
/// One-based, because it is shown to a person beside a page number that is
/// also one-based, and a set of stamps called `Stamp 0 … Stamp 11` is a set
/// somebody has to renumber by hand.
#[must_use]
pub fn default_display(page_index: usize) -> String {
    crate::text::stamps::default_stamp_name(page_index.saturating_add(1))
}

/// Turn a display name into a legal, unique internal name.
///
/// Returns the name and everything that had to be done to it. See
/// [`Adjustment`] for why each clause exists; the order below is deliberate
/// and the reasons are not interchangeable.
///
/// # The clauses, in the order they must run
///
/// 1. **Keep only characters that survive a round trip.** Alphanumerics,
///    `_` and `-`. Spaces go rather than becoming underscores: Adobe's own
///    keys (`SBApproved`, `SHInitialHere`) are unspaced camel case, and
///    matching the neighbours is worth more than preserving word boundaries in
///    a string no operator reads.
/// 2. **Strip a leading `#` — after step 1, not before.** `# Approved` and
///    `#Approved` must both lose the marker, and only step 1 makes those the
///    same string. Doing this first would let `# Approved` through as
///    `#Approved`. ★ This is the clause with a correctness consequence rather
///    than a cosmetic one; see the module header.
/// 3. **Fall back when nothing survives.** A name of `★★★` sanitises to
///    nothing, and a name tree key of `""` is a file Acrobat shows an empty
///    menu row for.
/// 4. **Truncate**, then
/// 5. **De-duplicate**, in that order — de-duplicating first and truncating
///    after would cut the number back off and reintroduce the collision.
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
///
/// The test is the **name tree**, not the title: a PDF with a `/Title` and no
/// named pages is just a PDF. Mirrors
/// [`pdfcer_core::stamp_file::StampCollection::is_stamp_file`] and exists so
/// callers need not import the engine type to ask.
#[must_use]
pub fn is_collection(collection: &StampCollection) -> bool {
    collection.is_stamp_file()
}

/// How many of a collection's stamps are dynamic.
///
/// Worth its own function because the answer drives a sentence rather than a
/// number: a collection that is *entirely* dynamic — which the operator's own
/// signature file is — deserves to be described differently from one with a
/// single dynamic entry among twelve.
#[must_use]
pub fn dynamic_count(collection: &StampCollection) -> usize {
    collection.stamps.iter().filter(|s| s.dynamic).count()
}

/// Why this document's page tree could not be read, when it could not be.
///
/// # ★★★ The field that makes every count below mean one thing again
///
/// Until engine `Pass 290.1` (2026-09-10, `bce4703`) `stamp_file::read` built
/// its page list with `page_tree::pages(doc).map(…).unwrap_or_default()`, so a
/// page tree that refused produced an **empty** list, so every
/// `StampEntry::page_index` came back `None` — and `None` already meant
/// something else and something specific: *"this name points at a page the
/// document does not have."* Two opposite facts arrived in one value, and on
/// the operator's own Acrobat-written signature file pdfcer reported both of
/// his real signatures as pointing at nothing when neither did.
///
/// The engine now carries the failure separately, which is what lets this
/// module's wording become a claim again instead of an observation carefully
/// phrased to be true either way:
///
/// | this returns | what `page_index: None` means |
/// |---|---|
/// | `Some(why)` | **nothing about the stamp.** The page tree is unreadable; no `page_index` in the collection carries information |
/// | `None` | the name genuinely points outside this document |
///
/// The names, display titles and dynamic flags are read from the `/Names` →
/// `/Pages` **name** tree and are unaffected by whatever is wrong with the
/// **page** tree, which is why `read` still returns a collection worth showing
/// and this is an `Option` rather than the whole call becoming a `Result`.
#[must_use]
pub fn page_tree_unreadable(collection: &StampCollection) -> Option<&str> {
    collection.page_tree_error.as_deref()
}

/// Stamps whose named page is not one of the document's own — or `None` when
/// that question is unanswerable.
///
/// ★ **`None` is not zero and must never be rendered as zero.** It means the
/// page tree could not be read, so no `page_index` in the collection carries
/// information; see [`page_tree_unreadable`] for why the two used to be
/// indistinguishable and what it cost. A caller that unwrapped this to `0`
/// would print *"every stamp resolves"* about a document where pdfcer resolved
/// none of them, which is the same class of defect as the one the engine just
/// removed, moved one crate along.
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
