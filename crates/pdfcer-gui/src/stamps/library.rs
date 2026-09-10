//! # `stamps::library` — the operator's OWN stamps, found and offered
//!
//! `OPERATOR_REQUESTS.md` **O172**: *"We also need to make it easy to add our
//! own custom stamps and use them, preferrably exactly the same way acrobat
//! does."* — and the load-bearing half of that sentence is **use them**.
//!
//! [`super`] already authors a collection and [`super::folder`] already knows
//! where Acrobat keeps one. What neither could do was put a stamp the operator
//! made onto a drawing, because the engine had no verb that carried one page's
//! artwork onto another. `pdfcer-core` `Pass 293.0` shipped
//! `EditSession::place_page_artwork`, and this module is the half that has to
//! exist above it: **which stamps are there, what are they called, and which
//! page of which file is each one.**
//!
//! ## What a "library" is here, concretely
//!
//! A flat scan of one directory, turned into a list a gallery can draw:
//!
//! ```text
//! %APPDATA%\Adobe\Acrobat\DC\Stamps\*.pdf
//!     |- each file  = one CATEGORY   (its /Info /Title)
//!         |- each named page = one STAMP  (/Names -> /Pages, `internal=display`)
//! ```
//!
//! Nothing is cached across a scan and nothing is watched: [`scan`] reads the
//! folder each time it is called, and its callers call it when a stamp gallery
//! opens. That is deliberate — the alternative is a cache that is stale
//! exactly when it matters, on the run after the operator has just made a new
//! stamp in Acrobat and gone looking for it here.
//!
//! ## ★★ Why the SHIPPED collections are excluded, and it is argued
//!
//! Acrobat's install carries four more collections at
//! `…/Acrobat DC/Acrobat/plug_ins/Annotations/Stamps/ENU/` — `Standard`,
//! `StandardBusiness`, `Dynamic`, `SignHere` — and this module does not scan
//! them. Three reasons, in decreasing order of how much they matter:
//!
//! 1. **`Standard` would DOUBLE the gallery and demote the better entry.**
//!    pdfcer already offers Table 181's stamp faces as real `/Name`
//!    annotations ([`crate::canvas::textannot::STAMPS`]) — which is what
//!    Acrobat itself writes for those, and what any other reader understands.
//!    Importing Adobe's artwork instead would author a *nameless* form XObject
//!    where a named annotation belongs, and the gallery would show `Approved`
//!    twice with no way to tell which was which.
//! 2. **`Dynamic` is a promise pdfcer cannot keep.** Its text comes from
//!    AcroForm calculation JavaScript that Acrobat runs at placement time.
//!    `place_page_artwork` imports the artwork and *not* the machinery, so a
//!    dynamic stamp arrives frozen at its design-time text — a stamp reading
//!    `Received 10 Sep 2026` forever. See [`CustomStamp::dynamic`], which is
//!    how a dynamic stamp in the operator's OWN folder is disclosed rather
//!    than hidden.
//! 3. **They are not "our own custom stamps"**, which is what the request
//!    says.
//!
//! ⚠ The one that is genuinely arguable is `SignHere`: five faces pdfcer does
//! not offer, in a category a drafting workflow uses. It is excluded here only
//! because it arrives bundled with the other three, and the exclusion is one
//! constant away from being reversed. This paragraph exists so that reversal
//! is a decision rather than a discovery.
//!
//! ## R8b rule 4 — what this module discloses, and where
//!
//! Everything here is **off-canvas by construction**: it produces names for a
//! gallery and counts for a status line, and touches no painter. Two
//! inferences are made and both are reported rather than silently applied:
//!
//! | inference | reported as |
//! |---|---|
//! | a collection with no `/Info` `/Title` is labelled from its FILENAME | [`Category::named_from_file`] |
//! | a stamp file that will not open, or names no page, is left out | [`Library::unreadable`] / [`Library::unplaceable`] |
//!
//! The second is the one that would otherwise be invisible: a folder of six
//! stamps that shows five is indistinguishable from a folder of five.

use std::path::{Path, PathBuf};

use pdfcer_core::document::Document;

/// One stamp the operator can place — everything the placing verb needs, and
/// everything a label needs, in one clonable value.
///
/// # ★ Why the whole thing travels rather than an index
///
/// This is carried on `Action::CommitTextAnnot` and therefore has to survive
/// the dialog that produced it. An index into a [`Library`] would be a handle
/// into a list that is rescanned on the next gallery, which is the exact shape
/// of bug where the operator places *Ken* and gets *Savy* because a file
/// landed in the folder in between. Three `String`s and a `PathBuf` per
/// placement is not a cost worth that risk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomStamp {
    /// What the gallery shows: the display half of `internal=display`, or the
    /// internal name when the file carries no display half.
    pub label: String,
    /// The category this stamp belongs to — the collection's `/Info`
    /// `/Title`. Carried per stamp rather than looked up, for the reason in
    /// the type's own header.
    pub category: String,
    /// The collection file. Reopened at placement time; not held open.
    pub file: PathBuf,
    /// The 0-based page of that file whose artwork this stamp is.
    ///
    /// ★ Keyed by page, never by position in the list. §7.9.6 requires the
    /// name tree be sorted lexicographically, so **page order is not tree
    /// order** — `StandardBusiness.pdf` proves it, with `SBApproved` on page 1
    /// and `SBCompleted` on page 5. A gallery that placed "the nth page"
    /// would mislabel every stamp and look entirely correct doing it.
    pub page_index: usize,
    /// Whether the collection marks this stamp **dynamic** (a leading `#` on
    /// the internal name).
    ///
    /// ⚠ pdfcer places the artwork, never the AcroForm machinery that makes a
    /// dynamic stamp's text current — so one placed here shows the words it
    /// was *drawn* with, forever. Correct as a picture, wrong as a promise.
    /// The gallery discloses this before the operator commits; nothing marks
    /// the placed stamp on the canvas afterwards, per R8b rule 4.
    pub dynamic: bool,
}

/// One collection file, as a gallery section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    /// The heading the gallery draws.
    pub name: String,
    /// True when [`Self::name`] came from the **filename** because the file
    /// carried no `/Info` `/Title`.
    ///
    /// An inference — pdfcer named something the operator did not name — so it
    /// is reported. Acrobat in the same position shows nothing useful at all,
    /// which is worse but is not a licence to be silently wrong.
    pub named_from_file: bool,
    /// Its stamps, in the collection's own (name-tree) order.
    pub stamps: Vec<CustomStamp>,
}

/// Everything found in one scan, plus what was NOT found and why.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Library {
    /// The categories, sorted by name so the gallery is stable between runs.
    pub categories: Vec<Category>,
    /// The folder that was scanned, when one could be identified at all.
    ///
    /// `None` on a machine with no `%APPDATA%`, or where no Acrobat
    /// generation folder could be picked. **R9**: the gallery then renders
    /// nothing rather than an empty section explaining itself.
    pub folder: Option<PathBuf>,
    /// How many `.pdf` files in the folder would not open, or opened and
    /// carried no stamp name tree at all.
    ///
    /// Not an error — an ordinary PDF someone dropped in that folder is
    /// exactly this — but a number worth being able to say out loud when the
    /// operator asks why he can see four of his five stamps.
    pub unreadable: usize,
    /// How many name-tree entries named a page that could not be resolved.
    ///
    /// These are stamps the file claims to have and pdfcer cannot place. The
    /// engine reports the same condition as `StampEntry::page_index == None`,
    /// usually because the collection's page tree will not walk.
    pub unplaceable: usize,
}

impl Library {
    /// Every stamp in every category, flattened — for a check or a count.
    pub fn stamps(&self) -> impl Iterator<Item = &CustomStamp> {
        self.categories.iter().flat_map(|c| c.stamps.iter())
    }

    /// How many stamps are offered in total.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stamps().count()
    }

    /// Whether the gallery has nothing of the operator's to show.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.categories.iter().all(|c| c.stamps.is_empty())
    }
}

/// **Scan the operator's Acrobat stamps folder.**
///
/// Never fails: a missing folder, an unreadable file and a PDF that is not a
/// stamp collection are all ordinary outcomes, and each is *counted* rather
/// than raised. The caller draws whatever came back.
///
/// # Cost, because this runs when a dialog opens
///
/// One `read_dir` plus one [`Document::load`] per `.pdf` in the folder. A
/// stamp collection is a handful of kilobytes; the operator's own is one file
/// with two stamps. If that folder ever holds something large this is the
/// place that will show it, and the answer then is a cache with an mtime, not
/// a background thread.
#[must_use]
pub fn scan() -> Library {
    let Some(folder) = super::folder::user_stamps_dir() else {
        return Library::default();
    };
    let mut library = Library {
        folder: Some(folder.clone()),
        ..Library::default()
    };

    let Ok(entries) = std::fs::read_dir(&folder) else {
        // The folder is identified but absent — the ordinary state of a
        // machine whose operator has never made a stamp in Acrobat. Not a
        // failure, and specifically not `unreadable`, which counts *files*.
        return library;
    };

    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf")))
        .collect();
    // Deterministic before anything else: two runs over an unchanged folder
    // must produce the same gallery, and `read_dir` promises no order.
    files.sort();

    for path in files {
        match read_collection(&path) {
            Some((category, unplaceable)) => {
                library.unplaceable += unplaceable;
                if category.stamps.is_empty() {
                    // A PDF in the stamps folder with a name tree that named
                    // nothing placeable is, to the operator, a stamp file that
                    // did not show up. Counted for the same reason.
                    library.unreadable += 1;
                } else {
                    library.categories.push(category);
                }
            }
            None => library.unreadable += 1,
        }
    }

    library.categories.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name))
    });
    library
}

/// Read one collection file into a [`Category`], plus how many of its stamps
/// named no resolvable page.
///
/// `None` when the file will not open at all, or opens and has no stamp name
/// tree — the two cases a caller counts identically because to the operator
/// they are the same thing: a file in the folder that is not a stamp file.
fn read_collection(path: &Path) -> Option<(Category, usize)> {
    let doc = Document::load(path).ok()?;
    let collection = pdfcer_core::stamp_file::read(&doc);
    if collection.stamps.is_empty() {
        return None;
    }

    let (name, named_from_file) = match collection.category.as_deref().map(str::trim) {
        Some(title) if !title.is_empty() => (title.to_owned(), false),
        // ★ The fallback, disclosed by the flag rather than silently pretty.
        // A collection with no `/Info` `/Title` is a file Acrobat itself
        // labels uselessly; the filename is the only other thing that was ever
        // the operator's choice.
        _ => (
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_owned(),
            true,
        ),
    };

    let mut unplaceable = 0;
    let mut stamps = Vec::new();
    for entry in collection.stamps {
        let Some(page_index) = entry.page_index else {
            unplaceable += 1;
            continue;
        };
        stamps.push(CustomStamp {
            label: label_for(&entry.display, &entry.internal),
            category: name.clone(),
            file: path.to_owned(),
            page_index,
            dynamic: entry.dynamic,
        });
    }

    Some((
        Category {
            name,
            named_from_file,
            stamps,
        },
        unplaceable,
    ))
}

/// What the gallery calls a stamp.
///
/// The display half when there is one, the internal name when there is not —
/// with the `#` that marks a dynamic stamp stripped, because that character is
/// a *marker in the format*, not part of what the stamp is called. Acrobat
/// shows `Received`, not `#Received`.
///
/// ⚠ Stripping it here is presentation only. [`CustomStamp::dynamic`] carries
/// the fact, so nothing downstream has to re-derive it from a string.
fn label_for(display: &str, internal: &str) -> String {
    let chosen = if display.trim().is_empty() {
        internal
    } else {
        display
    };
    chosen.trim_start_matches('#').trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_display_half_wins_over_the_internal_name() {
        assert_eq!(label_for("Sign here", "SignHere"), "Sign here");
    }

    #[test]
    fn an_absent_display_half_falls_back_to_the_internal_name() {
        assert_eq!(label_for("", "SBApproved"), "SBApproved");
        assert_eq!(label_for("   ", "SBApproved"), "SBApproved");
    }

    /// The `#` is a marker in the FORMAT, not part of the name — and the fact
    /// it carried lives on [`CustomStamp::dynamic`], never in the label.
    #[test]
    fn the_dynamic_marker_is_not_shown_to_the_operator() {
        assert_eq!(label_for("", "#Received"), "Received");
        assert_eq!(label_for("#Received", "#Received"), "Received");
    }

    /// A library with categories that hold no stamps is empty to the gallery,
    /// which is the only sense of "empty" the caller cares about.
    #[test]
    fn a_category_with_no_stamps_leaves_the_library_empty() {
        let library = Library {
            categories: vec![Category {
                name: "Signatures".to_owned(),
                named_from_file: false,
                stamps: Vec::new(),
            }],
            ..Library::default()
        };
        assert!(library.is_empty());
        assert_eq!(library.len(), 0);
    }

    #[test]
    fn stamps_flattens_every_category_in_order() {
        let stamp = |label: &str, category: &str| CustomStamp {
            label: label.to_owned(),
            category: category.to_owned(),
            file: PathBuf::from("x.pdf"),
            page_index: 0,
            dynamic: false,
        };
        let library = Library {
            categories: vec![
                Category {
                    name: "A".to_owned(),
                    named_from_file: false,
                    stamps: vec![stamp("one", "A"), stamp("two", "A")],
                },
                Category {
                    name: "B".to_owned(),
                    named_from_file: true,
                    stamps: vec![stamp("three", "B")],
                },
            ],
            ..Library::default()
        };
        let labels: Vec<_> = library.stamps().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, ["one", "two", "three"]);
        assert_eq!(library.len(), 3);
        assert!(!library.is_empty());
    }

    /// ⚠ A scan on a machine with no stamps folder must be a quiet nothing,
    /// not an error and not an empty section that explains itself. **R9.**
    #[test]
    fn a_scan_never_panics_and_reports_a_folder_or_nothing() {
        let library = scan();
        // Whatever this machine has, the invariants hold: every counted
        // category has a name, and a library with no folder has no categories.
        if library.folder.is_none() {
            assert!(library.categories.is_empty());
        }
        assert!(library.categories.iter().all(|c| !c.name.is_empty()));
    }
}
