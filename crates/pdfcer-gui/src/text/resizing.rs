//! # `text::resizing` — every sentence the resize grips show
//!
//! Six refusals and one disclosure, for [`crate::canvas::resizing`].
//!
//! ## Why a refusal here is worth more than the feature it refuses
//!
//! The eight grips have been drawn, cursored and drag-consuming since S4 and
//! have **committed nothing** for the whole life of this shell. An operator
//! aiming at one got a resize cursor, a drag that felt like it was doing
//! something, and no change — which is the exact shape of `DEFECTS.md` D4a,
//! the defect that began this project: *the old shell's answer to a caret it
//! could not place was a boolean and a keyboard that stopped responding.*
//!
//! So the sentences below are not decoration on a new feature. **Three of the
//! six describe cases the new feature still cannot do**, and saying so is the
//! difference between a limit and a bug. A drag on a text run will still change
//! nothing; what changes today is that the operator is told why in one
//! sentence, and told what would work instead.
//!
//! ## The rule every sentence follows
//!
//! **Name the thing the operator can see, never the thing pdfcer models.** They
//! can see a line of text, a picture, and a shape made of corners; they cannot
//! see a "show operator", a "path object" or a "node". `canvas::textedit`'s
//! catalogue makes the same choice in the same words, and for the same reason:
//! a refusal phrased in the file format's vocabulary is a refusal that reads as
//! an internal error.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/resizing.md`.

use crate::canvas::resizing::Refusal;

/// The sentence for a refusal to resize.
///
#[must_use]
pub const fn refusal(reason: Refusal) -> &'static str {
    match reason {
        Refusal::NothingSelected => {
            "Select something first. Click a shape on the page, then drag one of the squares \
             around it to resize it."
        }
        //
        // They said:
        //
        //   ManyObjects — *"pdfcer resizes one shape at a time. Select just the
        //   one you want and drag again."*
        //   NotAPath    — *"pdfcer cannot resize text or pictures — only shapes
        //   drawn out of lines and curves."*
        //
        // Both were honest and both are now false. `Pass 113.0`'s
        // `transform_objects` wraps each object's operator run in `q <cm> … Q`,
        // which never looks at an operand — so it works on text, pictures,
        // forms and inline images, and it takes a **slice**, so a multi-object
        // resize is one command and one undo entry rather than N of each.
        //
        // Kept as a comment rather than deleted with the strings, because the
        // shape of the episode is the durable part: **a refusal is a claim with
        // a date on it.** The `NotAPath` sentence carried *"is not built yet"*
        // and was right for one day; the `ManyObjects` one carried an
        // architectural reason (`move_nodes` is per object) that stopped being
        // true the moment a different verb existed. A refusal nobody re-reads
        // is a limit that outlives the thing that caused it — decision 058's
        // failure mode wearing a friendlier face.
        Refusal::NoObjectModel => {
            "pdfcer could not read this page's shapes, so it will not guess at what a resize \
             would do to them."
        }
        // Refused rather than clamped, and the sentence says which direction to
        // go so the operator does not simply try the same drag again.
        Refusal::Degenerate => {
            "That would flatten the shape to nothing or turn it inside out. Drag back the other \
             way to make it smaller without collapsing it."
        }
    }
}

/// The disclosure a completed resize owes — **line weight does not scale**.
#[must_use]
pub const fn line_weight_disclosure() -> &'static str {
    "layout: the shape changed size and its line thickness did not, which is how a drawing \
     standard works. If you wanted the outline heavier too, that is a separate change pdfcer \
     cannot make yet."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every refusal has a real sentence, and none of them is empty.**
    ///
    /// The whole point of the module. The grips' answer to every case they
    /// could not handle was silence, for the entire life of the shell.
    #[test]
    fn every_refusal_says_something() {
        for r in [
            Refusal::NothingSelected,
            Refusal::NoObjectModel,
            Refusal::Degenerate,
        ] {
            let s = refusal(r);
            assert!(s.len() > 40, "{r:?} needs a real sentence, got {s:?}");
            assert!(
                s.ends_with('.'),
                "{r:?} is prose and prose is punctuated: {s:?}"
            );
        }
    }

    /// **No sentence uses a word from the file format.**
    #[test]
    fn nothing_is_phrased_in_the_file_formats_vocabulary() {
        for r in [
            Refusal::NothingSelected,
            Refusal::NoObjectModel,
            Refusal::Degenerate,
        ] {
            let s = refusal(r).to_lowercase();
            for word in [
                "node",
                "operator",
                "path object",
                "subpath",
                "content stream",
            ] {
                assert!(
                    !s.contains(word),
                    "{r:?} says {word:?}, which names nothing an operator can see: {s:?}"
                );
            }
        }
    }

    /// **A refusal that has a working alternative names it.**
    #[test]
    fn the_refusal_with_an_alternative_offers_it() {
        assert!(refusal(Refusal::NothingSelected).contains("Click"));
    }

    /// **Every refusal still standing describes a state the shell can
    /// actually reach.**
    #[test]
    fn every_refusal_is_still_raised_somewhere() {
        let src = include_str!("../canvas/resizing.rs");
        for r in [
            Refusal::NothingSelected,
            Refusal::NoObjectModel,
            Refusal::Degenerate,
        ] {
            let name = format!("{r:?}");
            assert!(
                src.contains(&format!("Refusal::{name}")),
                "`{name}` has words and no call site — either it is dead and should go with its \
                 sentence, or the code that raised it was deleted and an operator now gets \
                 silence where they used to get an explanation"
            );
        }
    }

    /// The line-weight disclosure names both what happened and what it means.
    ///
    /// Not "the stroke width was preserved", which states a fact about the file
    /// and leaves the operator to work out whether that was on purpose.
    #[test]
    fn the_line_weight_disclosure_explains_itself() {
        let s = line_weight_disclosure();
        assert!(s.contains("line thickness"));
        assert!(
            s.contains("drawing standard"),
            "it must say WHY this is the right default, or it reads as pdfcer failing to scale \
             something: {s:?}"
        );
    }
}
