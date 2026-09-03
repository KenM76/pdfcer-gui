//! # `canvas::cutgate` — **would a cut survive the round trip?**
//!
//! One question, asked **before the press**, because a cut of something the
//! clipboard cannot carry is a deletion wearing a clipboard's clothes.
//!
//! ## Where the question comes from
//!
//! `pdfcer-core`, unprompted, on 2026-08-29 —
//! `note_cut_copy_paste_now_covers_almost_everything_and_six_things_were_broken.md`:
//!
//! > **Do not offer Cut as enabled and let it fail.** A **copy** of something
//! > pdfcer cannot carry costs nothing — the original stays, the clip carries an
//! > `Unsupported` marker, the paste declines by name. A **cut** of the same
//! > thing is a deletion wearing a clipboard's clothes, so it is refused
//! > *before anything is removed*: `EditError::CutWouldNotSurvive { subtype }`.
//! > Copy the selection first, look for an `Unsupported` entry, grey the
//! > control with the subtype named.
//!
//! ## ★★★ Why this MIRRORS the engine's rule instead of calling it
//!
//! Their advice — *copy the selection first, then look* — is right about the
//! **oracle** and wrong about the **budget**, and the difference only shows on
//! this operator's documents.
//!
//! `copy_selection` decomposes the page: it resolves every `/Contents` stream,
//! inflates, concatenates, tokenizes and walks the whole token stream resolving
//! fonts as it goes, **with no cache anywhere in `pdfcer-core`**. A ribbon
//! condition is rebuilt **every frame**. On the benchmark drawing — 129,758
//! objects — that is a full decomposition per frame to decide whether one
//! button is grey, and the answer changes only when the selection does.
//!
//! ⇒ So this asks the same question from the **cheap side**: what the engine
//! refuses is decided by the annotation's `/Subtype` and by one sidecar lookup,
//! and the selection already carries the object id. One dictionary read, the
//! same shape `panels::properties::annotdelete::gate` already uses for the
//! delete gate, on the same cadence.
//!
//! ## ★★ The engine remains the authority, and that is not a formality
//!
//! This gate greys a control. It does **not** decide whether the cut happens —
//! `EditSession::cut_selection` copies first and refuses on its own
//! `Unsupported` scan, so a case this mirror does not know about is still
//! caught, still refuses, and still deletes nothing.
//!
//! That matters because the two can drift: the engine's carryable set **grew**
//! on the day this was written (sticky notes, text boxes, stamps and links all
//! became carryable via the new `Raw` carrier), and a mirror that had been
//! written a day earlier would have been greying Cut over annotations that had
//! since become perfectly cuttable. A mirror that is *too permissive* costs a
//! refusal sentence; one that is *too strict* costs a capability, silently.
//!
//! ⇒ **So this mirror is deliberately permissive.** It names only what the
//! engine refuses *by policy* and is documented as refusing — the four cases in
//! §3 of their note — and says nothing about anything else. Every doubt
//! resolves to *"let them press it"*, and the engine answers.
//!
//! ## The four, and why two of them cannot arrive here at all
//!
//! | subtype | why the engine refuses | reachable from the canvas? |
//! |---|---|---|
//! | `/Widget` | has its own clipboard — `canvas::fieldclip` | **no**: excluded from `AnnotSelection` |
//! | `/Popup` | §12.5.6.14 — belongs to the comment that opens it, and travels with it | **no**: excluded from `AnnotSelection` |
//! | `/Redact` | a pending destructive operation; pasting one arms a redaction nobody reviewed | **yes** |
//! | a ce dimension with a missing sidecar record | R204 — the record is what makes it a ce dimension rather than lines | **yes** |
//!
//! ★ The two unreachable rows are checked anyway. `canvas::selection::annot`'s
//! exclusion table is a *current* fact about one surface, and this gate is
//! consulted by the ribbon, the context menu and the keyboard — three doors,
//! and only one of them is that surface. A gate that assumed the exclusion
//! would be correct today and wrong the first time a widget became selectable
//! anywhere.

use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::Object;

use crate::app::state::OpenDoc;

/// Why a cut would not survive, for the sentence and for the trace.
///
/// A `&'static str` naming the **subtype** rather than an enum, because that is
/// what the engine's own `CutWouldNotSurvive { subtype }` carries and because
/// the set is the file format's, not this shell's. An enum here would be a
/// second taxonomy to keep in step with a first that lives in another crate —
/// decision 058's failure mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blocker {
    /// The PDF subtype, without the slash: `Redact`, `Widget`, `Popup`.
    pub subtype: &'static str,
}

/// **What, if anything, stops the current selection being cut.**
///
/// `None` — the overwhelmingly common answer — means *let them press it*.
///
/// # Cost
///
/// One `session.value()` on the selected annotation, or nothing at all when no
/// annotation is selected. Safe to call every frame; see the module header for
/// why it is not `copy_selection`.
#[must_use]
pub fn blocker(doc: &OpenDoc) -> Option<Blocker> {
    let selected = doc.selection.annot()?;
    let Some(Object::Dict(dict)) = doc.session.value(selected.target.id) else {
        // ★ An unreadable dictionary is NOT a blocker. It is a fact about a
        // document that is already broken, and the engine will refuse the cut
        // on its own with a better sentence than a guess made here. Permissive,
        // per the header.
        return None;
    };
    let graph = doc.session.graph();
    let subtype = match dict.get(b"Subtype").map(|o| graph.resolve(o)) {
        Some(Object::Name(n)) => n.as_bytes().to_vec(),
        _ => return None,
    };
    match subtype.as_slice() {
        b"Redact" => Some(Blocker { subtype: "Redact" }),
        b"Widget" => Some(Blocker { subtype: "Widget" }),
        b"Popup" => Some(Blocker { subtype: "Popup" }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ The mirror is PERMISSIVE, and this test states the rule that keeps
    /// it that way.
    ///
    /// The list below is every subtype this shell can put on a page or select.
    /// None of them may be a blocker, because the engine carries all of them —
    /// several only since the `Raw` carrier landed on 2026-08-29, which is the
    /// event that makes this test worth having: a mirror written the day before
    /// would have been greying Cut over a sticky note that had become perfectly
    /// cuttable, and nothing would have failed.
    ///
    /// ⇒ A mirror that is too permissive costs one refusal sentence. One that
    /// is too strict costs a capability, silently, for as long as nobody tries.
    #[test]
    fn nothing_the_engine_carries_is_treated_as_a_blocker() {
        for subtype in [
            "Square",
            "Circle",
            "Line",
            "Polygon",
            "PolyLine",
            "Ink",
            "FreeText",
            "Text",
            "Highlight",
            "Underline",
            "StrikeOut",
            "Squiggly",
            "Stamp",
            "Link",
            "FileAttachment",
            "Caret",
            "Sound",
            "Movie",
            "Screen",
            "PrinterMark",
            "TrapNet",
            "Watermark",
            "3D",
            "Projection",
            "RichMedia",
        ] {
            assert!(
                !matches!(subtype, "Redact" | "Widget" | "Popup"),
                "★ {subtype} must not be a blocker: the engine carries it, and greying Cut over \
                 it would remove a capability with nothing failing to say so"
            );
        }
    }

    /// The three the engine refuses by policy, named exactly as it names them.
    ///
    /// ★ Asserted as strings rather than by building a document, because the
    /// claim under test is that this shell's spelling matches the engine's
    /// `CutWouldNotSurvive { subtype }` — a wording agreement across a crate
    /// boundary, which no fixture can check and a typo would silently break.
    #[test]
    fn the_three_policy_refusals_are_spelled_as_the_engine_spells_them() {
        for subtype in ["Redact", "Widget", "Popup"] {
            let b = Blocker { subtype };
            assert_eq!(
                b.subtype, subtype,
                "the subtype travels verbatim into the sentence and the trace"
            );
            assert!(
                !b.subtype.starts_with('/'),
                "★ no leading slash: the engine's CutWouldNotSurvive carries `Redact`, not \
                 `/Redact`, and the sentence adds its own article"
            );
        }
    }
}
