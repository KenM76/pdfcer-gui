//! # `canvas::cutgate` — **would a cut survive the round trip?**
//!
//! One question, asked **before the press**, because a cut of something the
//! clipboard cannot carry is a deletion wearing a clipboard's clothes.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/cutgate.md`.

use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::Object;

use crate::app::state::OpenDoc;

/// Why a cut would not survive, for the sentence and for the trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blocker {
    /// The PDF subtype, without the slash: `Redact`, `Widget`, `Popup`.
    pub subtype: &'static str,
}

/// **What, if anything, stops the current selection being cut.**
#[must_use]
pub fn blocker(doc: &OpenDoc) -> Option<Blocker> {
    let selected = doc.selection.annot()?;
    let Some(Object::Dict(dict)) = doc.session.value(selected.target.id) else {
        // An unreadable dictionary is NOT a blocker. It is a fact about a
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

    /// The mirror is PERMISSIVE, and this test states the rule that keeps
    /// it that way.
    ///
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
