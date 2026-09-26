//! # `text::panels::textobject` — the words the CLICKED-TEXT colour control uses
//!
//! `OPERATOR_REQUESTS.md` **O89**, piece 1, and the candidate O89 called
//! *"closest to what you tried"*:
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/textobject.md`.

/// The section heading.
#[must_use]
pub const fn heading() -> &'static str {
    "Text"
}

/// **What the control is about to act on**, stated before it is used.
#[must_use]
pub fn covers(runs: usize) -> String {
    format!("The text in this shape — {runs} run(s).")
}

/// The colour row's label.
#[must_use]
pub const fn colour_label() -> &'static str {
    "Colour"
}

/// Drawn **instead of** a swatch when some of the object's text is painted
/// in a colour space this shell will not round-trip.
#[must_use]
pub fn ink_present(affected: usize, total: usize) -> String {
    if affected == total {
        "Set in CMYK or a spot colour. pdfcer will not overwrite it with a screen colour, because \
         that would look right here and change what prints."
            .to_owned()
    } else {
        format!(
            "{affected} of these {total} runs are set in CMYK or a spot colour. pdfcer will not \
             overwrite those with a screen colour, because that would look right here and change \
             what prints — so it offers no swatch for the shape as a whole. Sweep the runs you \
             want with the Text tool (T) to change them one at a time."
        )
    }
}

// THE ROUTE SENTENCE IS NOT HERE, AND THERE IS NO LONGER A ROUTE TO NAME.
//
// Until 2026-09-14 this comment pointed at `super::properties::text_object_route`,
// a sentence that told the operator to *"press T for the Text tool and sweep
// across them"* in order to reach the font controls for a clicked text object.
// O198 deleted that sentence, because the thing it routed around was a defect
// rather than a design: clicking a text object now resolves its runs through
// `pdfcer_gui::app::textoperand`, so the face, size, bold and italic controls are
// live on the click and there is nowhere to send anybody.
//
// What survives here is the COLOUR refusal above, and it is a different
// kind of sentence. It is not a route around a missing capability; it is a
// disclosure that pdfcer will not repaint a CMYK or spot-colour run with a
// screen colour, which is true no matter how the operator reached the control.
// The sweep it suggests is a narrower operand, not a workaround.
//
// Its test, `the_text_route_sentence_names_the_bound_chord`, went with it.
// That test was the only reader of `view.tool_text`'s chord outside the keymap,
// so if a future string ever writes a chord into prose again, restore it from
// git rather than re-deriving the idea.

/// Drawn where the swatch would be when the object's runs **disagree**.
#[must_use]
pub const fn mixed_hint() -> &'static str {
    "These words are not all one colour. Picking one sets all of them to it."
}
