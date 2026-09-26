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
        /// The entered target is painted **inside a form XObject**, and this shell
        /// declines rather than deleting it.
        ///
        /// THE SENTENCE IS THIS SHELL'S, NOT THE ENGINE'S. The pinned
        /// `pdfcer-core` declares `delete_text_run_in_form`,
        /// `delete_subpath_in_form` and `delete_node_in_form` alongside the six
        /// form-interior move verbs. Nothing upstream forbids this. What is missing
        /// is entirely local: a leaf carries no page paint-order index, so
        /// `part_hits_of` matches nothing for it and the Part rung cannot be
        /// entered inside a form in the first place. Opening that seam is the work,
        /// and it is the same seam the in-form move verbs already went through.
        ///
        /// Until it is opened the refusal is still the right behaviour, because a
        /// key that silently does nothing is worse than one that says why — but it
        /// must not be worded, here or on screen, as a limit of the engine. D57
        /// carries the repair.
        InsideForm,
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
