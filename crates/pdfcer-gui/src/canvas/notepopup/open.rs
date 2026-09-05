//! # `canvas::notepopup::open` — which pop-ups are showing, and who decided
//!
//! One subject: **the open/closed state of every note pop-up**, and the rule
//! that the file gets the first word and the operator gets the last.
//!
//! ## ★★★ The model: overrides, not a set of open windows
//!
//! The obvious implementation holds a `BTreeSet<ObjId>` of open pop-ups and
//! seeds it from the document. **Do not**, and the reason is the requirement
//! this whole feature was commissioned under:
//!
//! > *"`/Popup`'s `/Open` state is in the file. A note authored open should
//! > open. Read it; do not default it."*
//!
//! A seeded set has to be seeded *somewhere*, on some frame, from some
//! document — and every candidate for that moment is wrong in a way that is
//! invisible until it bites. Seed on first frame and a document opened into a
//! second tab never gets seeded. Seed per edit epoch and every keystroke in a
//! note re-opens every pop-up the file authored open, throwing away what the
//! operator closed. Seed on document change and you need a document-change
//! event this canvas does not have.
//!
//! So this store holds **only what the operator has explicitly done**, as
//! `ObjId → bool`, and the effective state is:
//!
//! ```text
//! open(note) = override(note.id)  ?? note.authored_open
//! ```
//!
//! Three properties fall out, and all three are what was wanted:
//!
//! 1. **The file's `/Open` is honoured with no seeding at all** — an untouched
//!    note reads its state straight out of the document, on every frame,
//!    including the first.
//! 2. **An edit cannot reset a pop-up**, because the override does not depend
//!    on the epoch. Saving a note leaves its window exactly where it was,
//!    which is what Acrobat does and what an operator mid-review expects.
//! 3. **Closing a pop-up the file authored open is expressible.** A bare set
//!    seeded from the document cannot represent that without a second set of
//!    "explicitly closed", which is this map wearing a worse shape.
//!
//! ## ★★ Rule 4: nothing here reaches the document
//!
//! An override is **interface state and only interface state**. It is never
//! written back, never saved, and never included in any comparison of what the
//! document says. `pdfcer-core` v0.38.0 has no verb that could write `/Open`
//! on an existing annotation anyway — audited 2026-09-05, `b"Open"` appears
//! exactly twice in the crate and both are authoring sites — and that gap is
//! filed as `request_a_notes_open_state_cannot_be_changed.md`.
//!
//! ⇒ Which means the honest sentence today is: **closing a pop-up is a thing
//! you do to your screen, not to the file.** The operator is not told that on
//! every click, because it is the behaviour of every reader they have used;
//! the day the engine can persist it, this module gains a verb and nothing
//! else changes.
//!
//! ## Where it lives, and why not on `OpenDoc`
//!
//! `egui::Memory`'s temporary data, keyed by the document's path — the same
//! store `crate::canvas::interact`'s gesture machine and
//! `crate::canvas::textedit`'s draft already use, and for the same reason: it
//! is per-frame interface state with no place in the document model, and
//! `crate::app::state::OpenDoc` belongs to no track this session.
//!
//! ★ **Keyed by path, so two open documents do not share a state.** Without
//! that, opening a second drawing in a new tab would show its notes with the
//! first document's pop-ups open — object ids collide across files freely, so
//! the collision is not a rare case, it is the normal one.
//!
//! ⚠ Temporary memory is dropped on restart, which is correct: an override is
//! a statement about this sitting, and a pop-up the operator closed last
//! Tuesday should not stay closed against a file that says it is open.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use pdfcer_core::object::ObjId;

/// The operator's explicit open/closed decisions for one document.
///
/// `Arc`-wrapped inside egui's store, so a read is a pointer clone rather than
/// a map clone — this is read once per frame per visible note and written only
/// on a click.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overrides(BTreeMap<ObjId, bool>);

impl Overrides {
    /// **Is this note's pop-up showing?**
    ///
    /// `authored` is what the file says — [`super::model::NoteView::authored_open`]
    /// — and it is the answer whenever the operator has not spoken. See the
    /// module header for why that is an argument rather than a fallback.
    #[must_use]
    pub fn is_open(&self, id: ObjId, authored: bool) -> bool {
        self.0.get(&id).copied().unwrap_or(authored)
    }

    /// Record a decision.
    ///
    /// Stores the value even when it equals `authored`: *"the operator closed
    /// this and the file also says closed"* and *"the operator has not
    /// touched it"* are the same on screen today and differ the moment
    /// anything reads the file again, and a store that collapsed them would be
    /// deciding that on the caller's behalf.
    pub fn set(&mut self, id: ObjId, open: bool) {
        self.0.insert(id, open);
    }

    /// **Has the operator spoken about this note?**
    ///
    /// The question the trace needs and nothing on screen can answer: a pop-up
    /// that is open because the *file* said `/Open` and one that is open
    /// because the operator *clicked* look identical, and the whole of this
    /// module's contract is the difference between them. An implementation
    /// that quietly defaulted `/Open` to `false` would look perfect right up
    /// until somebody opened a document another product had authored, and
    /// `note-popup from_file=` is the only oracle for it from outside the
    /// process.
    #[must_use]
    pub fn touched(&self, id: ObjId) -> bool {
        self.0.contains_key(&id)
    }

    /// How many decisions have been recorded — for the trace, and for the test
    /// that asserts a click is recorded rather than merely appearing to work.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing has been decided. Present because clippy requires it
    /// beside [`Self::len`], and it is the honest name for "the document's own
    /// state is the whole answer right now".
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The egui id this document's overrides are stored under.
///
/// The **path**, not the `OpenDoc` address or an index: see the module header.
/// A path is stable across a re-render, unique between open tabs, and already
/// the thing this shell uses to say which document it means everywhere else.
fn key(path: &Path) -> egui::Id {
    egui::Id::new(("pdfcer-note-popup-open", path)) // ui-text-exempt: internal widget id, never displayed
}

/// Read this document's overrides.
///
/// Returns an owned `Arc` rather than borrowing out of the memory lock,
/// because egui's `data` is behind a `RwLock` that must not be held while the
/// caller draws — the same shape `crate::canvas::interact::load_gesture` takes.
#[must_use]
pub fn load(ctx: &egui::Context, path: &Path) -> Arc<Overrides> {
    ctx.data_mut(|d| d.get_temp::<Arc<Overrides>>(key(path)).unwrap_or_default())
}

/// Record one decision, in place.
///
/// Read-modify-write under one lock. The map is small — one entry per pop-up
/// the operator has personally opened or closed in this sitting — so the clone
/// is a handful of `(ObjId, bool)` pairs and the alternative (interior
/// mutability inside the `Arc`) would buy nothing and cost a `Mutex`.
pub fn set(ctx: &egui::Context, path: &Path, id: ObjId, open: bool) {
    ctx.data_mut(|d| {
        let key = key(path);
        let mut overrides = (*d.get_temp::<Arc<Overrides>>(key).unwrap_or_default()).clone();
        overrides.set(id, open);
        d.insert_temp(key, Arc::new(overrides));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(num: u32) -> ObjId {
        ObjId::new(num, 0)
    }

    /// ★★★ **An untouched note reads its state out of the file.**
    ///
    /// The assertion this module's whole shape exists for. An implementation
    /// that started every pop-up closed — the obvious one — passes every other
    /// test here and fails this one, and would have shipped the operator's
    /// complaint back to him in a new form: a note he authored open that
    /// stays shut.
    #[test]
    fn an_untouched_note_takes_the_files_word() {
        let overrides = Overrides::default();
        assert!(overrides.is_open(id(7), true));
        assert!(!overrides.is_open(id(7), false));
    }

    /// The operator outranks the file, in **both** directions.
    ///
    /// Both, because asserting only "opening a closed note works" would pass
    /// on an implementation whose override could only ever turn a pop-up on —
    /// and then the close button would be dead on exactly the notes that need
    /// it most, the ones the file authored open.
    #[test]
    fn an_override_wins_either_way() {
        let mut overrides = Overrides::default();
        overrides.set(id(7), false);
        assert!(!overrides.is_open(id(7), true));
        overrides.set(id(7), true);
        assert!(overrides.is_open(id(7), false));
    }

    /// One note's decision says nothing about another's. Trivial of a map and
    /// asserted anyway, because the failure — one click opening every pop-up
    /// on the sheet — is the exact shape of a store keyed by something coarser
    /// than the annotation.
    #[test]
    fn a_decision_is_per_note() {
        let mut overrides = Overrides::default();
        overrides.set(id(7), true);
        assert!(overrides.is_open(id(7), false));
        assert!(!overrides.is_open(id(8), false));
    }
}
