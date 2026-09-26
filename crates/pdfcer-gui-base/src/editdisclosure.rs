//! The sentences one edit owed, kept until they are said.
//!
//! The seam against [`super`] is the one **R2** asks for — *do these two change
//! for different reasons?*:
//!
//! | file | subject | changes when |
//! |---|---|---|
//! | [`super`] | the **vocabulary**: one variant per operator intent | the set of things an operator can ask for changes |
//! | this one | what an edit **reported about itself**, and how long it stays true | the disclosure contract changes |
//!
//! The thing to read before touching it is why a disclosure is keyed on the
//! **epoch** rather than merely stored — a sentence about an edit that has
//! since been undone is worse than no sentence, and the key is what makes that
//! unrepresentable rather than merely avoided.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/editdisclosure.md`.

use std::cell::RefCell;

/// The rule-4 sentences one vector edit owed, and the revision they describe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditDisclosure {
    /// The revision this describes — [`OpenDoc::edit_epoch`] **after** the
    /// edit. A disclosure whose epoch is not the document's current one
    /// describes an edit that has since been undone or superseded, and must
    /// not be shown.
    pub epoch: u64,
    /// The sentences, in the order the planner pushed them, **verbatim** from
    /// `pdfcer-core`. They are finished English prose written where the fact is
    /// known; this shell frames them (see
    /// [`crate::text::status::edit_disclosure_line`]) and rewrites nothing.
    pub notes: Vec<String>,
}

thread_local! {
    /// The most recent vector edit's disclosures, waiting to be read by the
    /// status bar.
    static LAST_EDIT: RefCell<Option<EditDisclosure>> = const { RefCell::new(None) };
}

/// What the last vector edit disclosed, if it still describes the open
/// document.
#[must_use]
pub fn last_edit_disclosure(epoch: u64) -> Option<EditDisclosure> {
    LAST_EDIT.with_borrow(|slot| {
        slot.as_ref()
            .filter(|d| d.epoch == epoch && !d.notes.is_empty())
            .cloned()
    })
}

/// Record what an edit disclosed — or, with `None`, that it disclosed nothing.
pub fn record_edit_disclosure(disclosure: Option<EditDisclosure>) {
    LAST_EDIT.with_borrow_mut(|slot| *slot = disclosure);
}

/// **Put one sentence on the status bar's disclosure row**, stamped with the
/// revision currently on screen.
pub fn record_note(epoch: u64, note: String) {
    record_notes(epoch, vec![note]);
}

/// **Put several sentences on the status bar's disclosure row**, stamped with
/// the revision currently on screen.
pub fn record_notes(epoch: u64, notes: Vec<String>) {
    record_edit_disclosure(Some(EditDisclosure { epoch, notes }));
}
