//! # `stamps::lastused` — which stamp he reached for last, and why that is
//! a NAME rather than a stamp
//!
//! `OPERATOR_REQUESTS.md` **O172**, and this module is the clause that was
//! left over when the placing half landed:
//!
//! > *"in Acrobat a custom stamp is a menu entry beside the standard ones,
//! > grouped by the category it was authored under, chosen the same way,
//! > placed the same way, **and it remembers the last one used**."*
//!
//! That sentence is not a nicety. Stamping a drawing set is a repetitive act —
//! the same `ISSUED FOR CONSTRUCTION` on thirty sheets — and a gallery that
//! opens on `Approved` every single time makes the operator re-find his own
//! stamp thirty times. Acrobat's Stamp menu re-opens on the last stamp placed;
//! under this project's standing *use the conventional interaction, never
//! invent one* rule, the convergence of the product class **is** the spec, so
//! this is not a design question and there was nothing to decide.
//!
//! ## ★★★ The whole design is in the choice of what to store
//!
//! The obvious implementation stores the [`CustomStamp`] that was placed. It
//! is wrong, and wrong in a way that produces a silent, operator-invisible
//! defect rather than an error:
//!
//! A [`CustomStamp`] carries `file` and `page_index`. Between one dialog
//! opening and the next, the operator can go into Acrobat and edit his own
//! stamp collection — that is the *point* of it being his folder, and O169
//! built the authoring half precisely so he would. Adding a stamp to a
//! collection re-sorts the name tree (§7.9.6 requires lexicographic order),
//! which **renumbers pages**. A remembered `page_index` of 2 then names a
//! different stamp, with a straight face: the gallery would show the right
//! label selected and place the wrong artwork, and nothing in the program
//! would be in a position to notice.
//!
//! ⇒ **So the memory is a name, and it is re-resolved against a freshly
//! scanned library every time the dialog opens.** A name that no longer
//! resolves is not an error and not a disclosure — it is the same state as
//! never having placed a stamp, and it opens on the default. See
//! [`LastStamp::resolve`].
//!
//! ## Why `(category, label)` and not just `label`
//!
//! Two collections may each carry a stamp called `Approved` — Adobe's own
//! shipped `Standard` and `StandardBusiness` both do, which is one of the
//! reasons [`crate::stamps::library`] excludes them. A label alone would pick
//! whichever came first in the scan, and the scan order is the file system's
//! opinion, not the operator's. The category is the collection's `/Info`
//! `/Title`, which is what the gallery draws as the section heading — so the
//! pair is exactly what the operator sees when he makes the choice, which is
//! the right thing to key a memory on.
//!
//! ## Scope: the session, deliberately not the disk
//!
//! This lives on `dialogs::DialogsState` in the application-scoped half — it
//! survives closing a document, and it does **not** survive closing the
//! program. That is a deliberate stopping point rather than an oversight:
//!
//! - Surviving the document is required by the use case. Stamping a drawing
//!   set means opening thirty files.
//! - Surviving the *program* would mean writing it into the settings store,
//!   and a preference has to be reachable — something the operator can see and
//!   clear. A hidden persisted selection that reaches back across a reboot to
//!   pre-select a stamp is the kind of state that produces "why did it put THAT
//!   on my drawing" reports, and the answer would be invisible to him.
//!
//! ⚠ If Acrobat is later measured to persist this across restarts, the
//! convention rule above says to follow it — but that would then arrive with a
//! visible control, not as a silent file. This paragraph exists so that is a
//! decision rather than a discovery.

use crate::stamps::library::{CustomStamp, Library};
use pdfcer_core::annot_author::StampName;

/// **The stamp the operator placed most recently**, in a form that survives
/// his stamp folder changing underneath it.
///
/// One of the two things a stamp gallery can be showing: a standard ISO 32000-2
/// Table 181 face, which pdfcer writes as a real named `/Stamp` annotation, or
/// one of his own, which is artwork imported out of a collection file.
///
/// Deliberately NOT `Copy` and deliberately holding owned `String`s: the
/// alternative is borrowing from a [`Library`] that is re-scanned on every
/// dialog opening, and the whole point of this type is to outlive one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LastStamp {
    /// A standard face, by name. These cannot go missing — the vocabulary is
    /// closed and compiled in — so this variant always resolves.
    Standard(StampName),
    /// One of his own, identified the way he identified it: by the section
    /// heading and the entry he clicked.
    ///
    /// See the module header for why this is a name pair rather than a
    /// [`CustomStamp`]: a remembered `page_index` silently names a different
    /// stamp the moment he adds one to that collection in Acrobat.
    Custom {
        /// The collection's `/Info` `/Title` — the gallery's section heading.
        category: String,
        /// The stamp's display label — the text on the radio button.
        label: String,
    },
}

impl LastStamp {
    /// Build the memory from what a dialog actually committed.
    ///
    /// Takes the same pair the dialog holds — the standard selection, which is
    /// always populated, and the custom selection, which shadows it when
    /// `Some`. That mirrors `TextAnnotDialog`'s own rule (`custom.is_none()`
    /// is what makes a standard radio read as selected), so the memory cannot
    /// disagree with the window it was taken from.
    #[must_use]
    pub fn taken(standard: StampName, custom: Option<&CustomStamp>) -> Self {
        custom.map_or(Self::Standard(standard), |c| Self::Custom {
            category: c.category.clone(),
            label: c.label.clone(),
        })
    }

    /// **Resolve the memory against a library scanned just now.**
    ///
    /// Returns the selection a freshly opened gallery should show:
    ///
    /// - `(standard, None)` for [`Self::Standard`], or for a custom stamp that
    ///   is no longer in the folder.
    /// - `(standard, Some(stamp))` when the remembered pair is still there,
    ///   with the *current* `file` and `page_index` — which is the entire
    ///   reason this is a resolve and not a clone.
    ///
    /// The `fallback` argument is the gallery's own default
    /// ([`crate::canvas::textannot::DEFAULT_STAMP`]); it is passed rather than
    /// read here so this module does not acquire an opinion about what the
    /// default should be.
    ///
    /// ★ A custom memory that fails to resolve deliberately produces **no
    /// disclosure**. It is indistinguishable, to the operator, from the first
    /// stamp of a session — and a sentence explaining that a stamp he deleted
    /// is not being pre-selected would be noise attached to a window he opened
    /// to do something else. The trace records it so the harness can tell the
    /// two apart; the operator has nothing to be told.
    #[must_use]
    pub fn resolve(
        &self,
        library: &Library,
        fallback: StampName,
    ) -> (StampName, Option<CustomStamp>) {
        match self {
            Self::Standard(name) => (*name, None),
            Self::Custom { category, label } => {
                let found = library
                    .categories
                    .iter()
                    .find(|c| &c.name == category)
                    .and_then(|c| c.stamps.iter().find(|s| &s.label == label).cloned());
                (fallback, found)
            }
        }
    }

    /// A short, stable token for the diagnostic trace. Never displayed.
    ///
    /// ⚠ Deliberately hand-built rather than a `Debug` rendering. `Debug` is a
    /// shape the compiler is free to change and that a `#[derive]` in another
    /// crate owns; a driven check greping for it reports the opposite of the
    /// truth the day a field is added. This project has been bitten by exactly
    /// that, in a check that quoted the true line in the message it used to
    /// state its false conclusion.
    ///
    /// It is also not the gallery's operator copy: that is allowed to be
    /// reworded on a Tuesday and is translatable in principle. Same rule and
    /// same reason as `StampSize::trace_token` and
    /// [`crate::canvas::stampfit::trace_token`].
    #[must_use]
    pub fn token(&self) -> String {
        match self {
            Self::Standard(name) => format!("standard:{}", standard_token(*name)),
            Self::Custom { category, label } => format!("custom:{category}/{label}"),
        }
    }
}

/// The stable trace spelling of a standard stamp face.
///
/// ★★ **Deliberately an exhaustive match with no wildcard arm.** `StampName`
/// is not `#[non_exhaustive]`, so the day the engine adds a fifteenth face this
/// function stops compiling and names itself, which is the whole point. A
/// `_ => "other"` arm would keep building and quietly collapse two stamps into
/// one token, and the check that told them apart would go on passing.
const fn standard_token(name: StampName) -> &'static str {
    match name {
        StampName::Approved => "Approved",
        StampName::Experimental => "Experimental",
        StampName::NotApproved => "NotApproved",
        StampName::AsIs => "AsIs",
        StampName::Expired => "Expired",
        StampName::NotForPublicRelease => "NotForPublicRelease",
        StampName::Confidential => "Confidential",
        StampName::Final => "Final",
        StampName::Sold => "Sold",
        StampName::Departmental => "Departmental",
        StampName::ForComment => "ForComment",
        StampName::TopSecret => "TopSecret",
        StampName::Draft => "Draft",
        StampName::ForPublicRelease => "ForPublicRelease",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamps::library::Category;
    use std::path::PathBuf;

    fn stamp(category: &str, label: &str, page_index: usize) -> CustomStamp {
        CustomStamp {
            label: label.to_owned(),
            category: category.to_owned(),
            file: PathBuf::from("C:/stamps/site.pdf"),
            page_index,
            dynamic: false,
        }
    }

    fn library(entries: &[(&str, &[(&str, usize)])]) -> Library {
        let mut lib = Library::default();
        for (title, stamps) in entries {
            lib.categories.push(Category {
                name: (*title).to_owned(),
                named_from_file: false,
                stamps: stamps
                    .iter()
                    .map(|(label, page)| stamp(title, label, *page))
                    .collect(),
            });
        }
        lib
    }

    #[test]
    fn a_standard_stamp_is_remembered_by_name() {
        let last = LastStamp::taken(StampName::Draft, None);
        assert_eq!(last, LastStamp::Standard(StampName::Draft));
        let (name, custom) = last.resolve(&Library::default(), StampName::Approved);
        assert_eq!(name, StampName::Draft);
        assert!(custom.is_none());
    }

    #[test]
    fn his_own_stamp_is_remembered_by_category_and_label() {
        let chosen = stamp("Site Review", "Issued", 2);
        let last = LastStamp::taken(StampName::Approved, Some(&chosen));
        assert_eq!(
            last,
            LastStamp::Custom {
                category: "Site Review".to_owned(),
                label: "Issued".to_owned(),
            }
        );
    }

    /// ★★★ The test this module exists for. A collection edited between two
    /// openings renumbers its pages; the memory must survive that by naming,
    /// and must come back carrying the NEW page index, not the old one.
    #[test]
    fn a_renumbered_collection_still_resolves_to_the_right_artwork() {
        let before = library(&[("Site Review", &[("Issued", 2), ("Approved", 0)])]);
        let chosen = before.categories[0].stamps[0].clone();
        assert_eq!(chosen.page_index, 2);
        let last = LastStamp::taken(StampName::Approved, Some(&chosen));

        // He adds a stamp in Acrobat; the name tree re-sorts and `Issued`
        // moves from page 2 to page 3.
        let after = library(&[(
            "Site Review",
            &[("Issued", 3), ("Approved", 1), ("Cancelled", 0)],
        )]);
        let (_, resolved) = last.resolve(&after, StampName::Approved);
        let resolved = resolved.expect("the stamp is still in the folder");
        assert_eq!(resolved.label, "Issued");
        assert_eq!(
            resolved.page_index, 3,
            "the memory must carry the CURRENT page, or it places the wrong artwork \
             while showing the right label"
        );
    }

    /// The negative control for the test above: without the re-resolve, a
    /// stored `CustomStamp` would still have said page 2 — which is now a
    /// different stamp. This asserts the two libraries genuinely differ at
    /// that index, so the test above cannot pass by coincidence.
    #[test]
    fn the_renumbering_fixture_really_does_move_the_stamp() {
        let after = library(&[(
            "Site Review",
            &[("Issued", 3), ("Approved", 1), ("Cancelled", 0)],
        )]);
        let at_old_page = after.categories[0]
            .stamps
            .iter()
            .find(|s| s.page_index == 2);
        assert!(
            at_old_page.is_none(),
            "if page 2 still existed the renumbering test would prove nothing"
        );
    }

    #[test]
    fn a_deleted_stamp_falls_back_to_the_default_and_says_nothing() {
        let last = LastStamp::Custom {
            category: "Site Review".to_owned(),
            label: "Issued".to_owned(),
        };
        let (name, custom) = last.resolve(&Library::default(), StampName::Approved);
        assert_eq!(name, StampName::Approved);
        assert!(custom.is_none());
    }

    /// Two collections with the same label. The category is what tells them
    /// apart, and a label-only memory would pick by scan order.
    #[test]
    fn the_category_disambiguates_two_stamps_with_the_same_label() {
        let lib = library(&[
            ("Site Review", &[("Approved", 0)]),
            ("Shop Drawings", &[("Approved", 0)]),
        ]);
        let last = LastStamp::Custom {
            category: "Shop Drawings".to_owned(),
            label: "Approved".to_owned(),
        };
        let (_, resolved) = last.resolve(&lib, StampName::Draft);
        assert_eq!(resolved.expect("present").category, "Shop Drawings");
    }

    #[test]
    fn a_category_that_went_missing_does_not_match_a_label_elsewhere() {
        let lib = library(&[("Shop Drawings", &[("Issued", 0)])]);
        let last = LastStamp::Custom {
            category: "Site Review".to_owned(),
            label: "Issued".to_owned(),
        };
        let (_, resolved) = last.resolve(&lib, StampName::Approved);
        assert!(
            resolved.is_none(),
            "a stamp in a different collection is a different stamp"
        );
    }

    #[test]
    fn the_trace_token_is_hand_built_and_distinguishes_the_two_kinds() {
        assert_eq!(
            LastStamp::Standard(StampName::Draft).token(),
            "standard:Draft"
        );
        assert_eq!(
            LastStamp::Custom {
                category: "Site Review".to_owned(),
                label: "Issued".to_owned(),
            }
            .token(),
            "custom:Site Review/Issued"
        );
    }
}
