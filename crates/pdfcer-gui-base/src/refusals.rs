//! # `refusals` — why a canvas verb declined
//!
//! The refusal vocabularies the text catalog words, kept apart from the
//! canvas modules that raise them so the catalog can name them. Each
//! `pdfcer_gui::canvas` module re-exports its own.

/// Why a resize could not be committed.
pub mod resize {
    /// Why a resize could not be committed.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Refusal {
        /// Nothing is selected, or the selection names no object on this page.
        NothingSelected,
        /// The object model could not be read, so nothing can be verified and
        /// therefore nothing may be promised.
        NoObjectModel,
        /// The drag would collapse the selection to nothing on an axis, or invert
        /// it.
        ///
        /// Refused rather than clamped: a zero or negative factor is a shape the
        /// operator cannot have meant, and clamping would silently substitute a
        /// different edit for the one they made.
        Degenerate,
    }
}

/// Why a Delete removes nothing.
pub mod delete {
    /// Why a Delete removes nothing.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Refusal {
        /// The page has no readable object model, so nothing can be verified and
        /// nothing may be promised. Reachable when the page failed to decompose.
        ///
        /// Only the deeper rungs need the model at all; the Object rung is
        /// answered from the selection alone, so a page that will not decompose
        /// can still have its objects deleted. That asymmetry is deliberate — see
        /// `subject`.
        NoObjectModel,
        /// Nothing is selected on this page.
        NothingSelected,
        /// A rung above Object with no entry to be inside of. `normalise` makes
        /// this unrepresentable; it is carried so the recovery is named rather
        /// than silent.
        NoPartEntered,
        /// The Node rung with no anchor picked — *"inside this part, nothing
        /// picked yet"*, which is a real state the ladder can be in.
        NoNodeEntered,
        /// The entered object is not addressable by a page paint-order index and
        /// is not a leaf either — unreachable through `TargetId`'s two variants,
        /// carried so a third variant is a compile error rather than a silence.
        UnaddressableObject,
        /// The entered object has no parts at all — an image, or a form treated as
        /// one object. There is nothing below it to delete.
        NoPartsInObject,
        /// The **Node** rung on a text object. A run's glyphs are not anchors and
        /// `pdfcer-core` has no verb that removes one character from a show
        /// operator; editing the string is `format_text`'s job and a different
        /// gesture entirely.
        NoNodeVerbForText,
        /// **Several anchors are selected and `delete_node` is singular.**
        ///
        /// Refused rather than looped, and this is the one judgement in this
        /// module that is worth arguing with. `move_nodes` exists and takes a
        /// slice, so a multi-anchor drag is one command; there is no `delete_nodes`,
        /// so a multi-anchor delete would be N commands and N undo entries for one
        /// press — and worse, each `delete_node` excises a byte span and therefore
        /// **renumbers**, so the second index would be planned against offsets the
        /// first invalidated. Acting on only the entered one would be the
        /// `selected_nodes_on` defect exactly: four anchors highlighted, one
        /// removed, nothing said. Carries the count, because a refusal that cannot
        /// say how many were selected is one the operator cannot act on.
        ManyNodes(usize),
        /// **Several chunks are selected and `delete_text_run` is singular.**
        ///
        /// The Part-rung twin of `Self::ManyNodes`, and it refuses for the
        /// stronger of that variant's two reasons. A plural MOVE is safe and is
        /// built — `move_text_run` rewrites an operand in place, adds no operator a
        /// run index counts, so a loop over N lines renumbers nothing. Deleting
        /// **excises** a show operator, so every later run index shifts down by one
        /// and the second call in a loop would address a line the first moved.
        ///
        /// Carries the count, for the same reason `ManyNodes` does: a refusal that
        /// cannot say how many were selected is one the operator cannot act on.
        ManyLines(usize),
        /// **§9.4.2 — removing this label would slide the next one.** R83, asked
        /// before the press. Carries the run index the operator picked; the remedy
        /// is to delete the later one first. See the module header for why this one
        /// refusal is pre-empted and the rest are left to the engine.
        RunWouldMoveNext(usize),
    }
}

/// Why a form field could not be copied, cut or pasted.
pub mod fieldclip {
    /// Why a field could not be copied, cut or pasted.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Refusal {
        /// No form field is selected.
        NothingSelected,
        /// The selection names a field the document no longer has.
        Vanished,
        /// The widget has no `/Rect`, so there is no box to land the paste against.
        NoGeometry,
        /// The clipboard holds no form field.
        NothingCopied,
        /// **The engine declined, in its own words.**
        ///
        /// A `String` rather than a mirror of `EditError`'s taxonomy, for the
        /// reason `canvas::clipboard::Refusal::EngineRefused`'s doc gives about the
        /// same choice: a shell that modelled the engine's internals a second time
        /// is decision 058's failure mode. The cases that actually arrive here —
        /// `SignedFieldNotCopyable`, `FieldNameTaken`, `FieldNotFound`,
        /// `RadioExportValueTaken`, and the encryption and certification guards —
        /// each already carry a sentence written by the party that knows why.
        EngineRefused(String),
    }
}

/// Whether the selected annotation can be deleted.
pub mod annotdelete {
    use crate::text::markup as t;
    use crate::text::markup::AnnotDeleteRefusal;

    /// What [`gate`] found, when it found something.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Refusal {
        /// §12.5.3 Table 165 bit 8 is set on this annotation.
        Locked,
        /// The document itself refuses, for the reason carried.
        Document(AnnotDeleteRefusal),
    }

    impl Refusal {
        /// The sentence.
        #[must_use]
        pub const fn line(self) -> &'static str {
            match self {
                Self::Locked => t::annot_delete_locked(),
                Self::Document(why) => why.line(),
            }
        }
    }
}

/// Why a copy or a cut could not happen.
pub mod clipboard {
    /// Why a copy or a cut could not happen.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Refusal {
        /// **The cut's DELETE half would be refused, so its copy half did not
        /// run either.**
        ///
        /// Only `cut` can answer this, and only `cut` returns it: a plain copy
        /// changes nothing and is correct on a document that forbids every change.
        ///
        /// It exists as its own variant rather than reusing [`Self::Unreadable`]
        /// because the operator's next move differs — a clip the engine could not
        /// assemble is a fact about *the selection*, and this is a fact about *the
        /// document*, true of every annotation in it until the signature or the
        /// encryption goes. The sentence is on the status row already, put there by
        /// `app::status::decline`, which is the same surface the three other doors
        /// onto this verb use.
        DeleteRefused(super::annotdelete::Refusal),
        /// **The clipboard could not carry it, so the cut was refused before
        /// anything was removed.**
        ///
        /// `pdfcer-core`'s `CutWouldNotSurvive { subtype }`, and the subtype travels
        /// so the sentence can name it — a greyed button has one static tooltip and
        /// the operator may have several things selected.
        ///
        /// # Why a cut is refused where a copy is not
        ///
        /// The engine's own words: *"a copy of something pdfcer cannot carry costs
        /// nothing — the original stays, the clip carries an `Unsupported` marker,
        /// the paste declines by name. A cut of the same thing is a deletion
        /// wearing a clipboard's clothes."*
        ///
        /// `&'static str` rather than an enum, matching `canvas::cutgate::Blocker`
        /// and for its reason: the set of subtypes is the file format's, and a
        /// second taxonomy here would be one more thing to keep in step with
        /// another crate.
        CutWouldNotSurvive(&'static str),
        /// Nothing is selected.
        NothingSelected,
        /// The engine refused to copy the selection: a clip it could not
        /// assemble. One variant rather than a mirror of the engine's taxonomy.
        EngineRefused,
        /// **The engine refuses to put that annotation on a clipboard at
        /// all**, and the `/Subtype`s it named travel with the refusal.
        ///
        /// `/Widget`, `/Popup` and `/Redact` — `EditSession::raw_copy_refusal`,
        /// and each for a stated reason rather than because it is hard: a widget
        /// would need a field name in the destination's `/AcroForm` that pdfcer
        /// cannot guess, a popup is not an independent annotation (§12.5.6.14) and
        /// belongs to the comment that opens it, and a redaction is a **pending
        /// destructive operation** — pasting one arms a redaction in a document
        /// nobody reviewed.
        ///
        /// The list is read off the clip, never mirrored here. `canvas::cutgate`
        /// does keep a mirror of the same three, and its own header explains why
        /// that one has to exist — it greys a control *before* the gesture, where
        /// nothing but a compile-time string will do. This is after the gesture,
        /// and the engine has already answered.
        CannotCarry(Vec<String>),
        /// **The selected annotation is no longer on the page it names** — a
        /// stale selection after an undo, an external reload, or a truncated
        /// `/Annots` walk. Clicking it again is the operator's next move.
        Unreadable,
        /// The clipboard is empty.
        NothingCopied,
    }
}
