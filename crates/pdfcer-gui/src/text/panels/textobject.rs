//! # `text::panels::textobject` — the words the CLICKED-TEXT colour control uses
//!
//! `OPERATOR_REQUESTS.md` **O89**, piece 1, and the candidate O89 called
//! *"closest to what you tried"*:
//!
//! > *"I don't see where I am able to edit the color of text, vectors, etc."*
//!
//! He clicked a piece of text. That selects the **object**, and every text
//! colour control in the program was gated on a **swept range**, so the swatch
//! he was looking for was greyed and the way to un-grey it — arm the Text tool,
//! sweep the words — is unguessable. `crate::panels::properties::textobject`
//! is the control that acts on the object he actually clicked; these are its
//! words.
//!
//! ## ★★★ Why a separate strings module rather than `super::properties`
//!
//! **R2, measured rather than assumed.** `text::panels::properties` was at
//! **1,446 lines of its 1,500** on the day this was written — 54 lines of
//! headroom for a subject that needs six strings and the argument behind each.
//! Adding them there would have spent the whole remaining budget of the file
//! that holds every other Properties string, and the next person to reword a
//! form-field caption would have met the gate instead.
//!
//! ★ The seam is real and not merely a size cut: every string in
//! `super::properties` describes **what is selected**; every string here
//! describes **a route that was not obvious** and what the control on that
//! route will and will not touch. They change for different reasons —
//! `super::properties` changes when a property is added, this changes when the
//! route changes.
//!
//! ## ★★★ The refusal is the load-bearing string, exactly as it is for vectors
//!
//! `crate::text::paint`'s header states it for a path and it is if anything
//! sharper here, because **a text run's unmodelled colour cannot even be
//! named**: `pdfcer_core::text_extract::TextColor::Other` is a fieldless
//! variant — the extraction records *"set in a space this pass does not
//! decode"* and carries no `/Separation` name with it, where
//! `pdfcer_core::vector::PathPaint::Other` carries `space`.
//!
//! ⇒ So [`ink_present`] must **not** promise a name. Writing *"PANTONE 300 — a
//! named ink"* here would be a sentence this shell cannot source, which is the
//! claim-bearing-copy failure in miniature. It says what is true: the colour is
//! set in a space pdfcer will not overwrite with a screen colour, and it says
//! which surface *can* name it (the Objects panel's own reading), rather than
//! inventing one.
//!
//! ## Rule 4 — none of these words reaches the page
//!
//! Every string here is drawn in the Properties panel. Nothing in this module
//! marks the canvas, tints a run, or renders a recoloured object differently
//! from the way the saved file will render it. What was skipped and why is
//! disclosed **off-canvas**, here and in the status bar, which is where Rule 4
//! puts it.

/// The section heading.
///
/// ★ The same word the swept-text section uses (`super::properties`'s
/// `text_heading`) and deliberately so: an operator who reaches the colour by
/// clicking and an operator who reaches it by sweeping must not think they
/// found two different features. The two sections are mutually exclusive, so
/// the heading never appears twice.
#[must_use]
pub const fn heading() -> &'static str {
    "Text"
}

/// **What the control is about to act on**, stated before it is used.
///
/// ★★ The count is the disclosure that makes this control safe to press, and
/// it is not decoration. A `BT`…`ET` on a CAD export is free to hold every
/// label on the sheet: `pdfcer_core::vector::TextObject::runs`' own docs record
/// a measured SolidWorks export where **one** text object's bounds ran
/// `23,14 → 1564,1216` — the whole drawing. Recolouring that is a legitimate
/// thing to ask for and a terrible thing to do by accident, so the number of
/// runs is on screen **before** the swatch, not in a report afterwards.
///
/// ★ "runs" is the program's own word for the unit, and it is the unit the
/// operator will meet again in the status line after the press
/// (`super::properties`'s `text_covers`). Two different nouns for one thing
/// across two adjacent surfaces is how a disclosure stops being read.
#[must_use]
pub fn covers(runs: usize) -> String {
    format!("The text in this shape — {runs} run(s).")
}

/// The colour row's label.
#[must_use]
pub const fn colour_label() -> &'static str {
    "Colour"
}

/// ★★★ Drawn **instead of** a swatch when some of the object's text is painted
/// in a colour space this shell will not round-trip.
///
/// The same guard `crate::text::paint::undecoded` states for a path, with the
/// one difference the module header argues: the ink **cannot be named** here,
/// so this sentence does not pretend to name it.
///
/// ★ It says *"some of"* whenever more than one run is involved, because a
/// single object can be part CMYK and part RGB and a sentence claiming all of
/// it would be false half the time. The absent swatch is per **object**, not
/// per run, and that is deliberate: this control's operand is the whole object,
/// so a partial refusal is a refusal of the gesture the operator would make.
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

// ★★★ THE ROUTE SENTENCE IS NOT HERE, AND ITS ABSENCE IS DELIBERATE.
//
// *"press T for the Text tool and sweep across them"* is
// `super::properties::text_object_route`, where it has lived since 2026-08-29.
// It was **re-aimed in place** when this section shipped rather than re-written
// here, because a second sentence saying nearly the same thing on the same
// panel state is exactly the drift `SALVAGE.md` forbids — and because that one
// carries a test (`the_text_route_sentence_names_the_bound_chord`) asserting it
// names whatever chord the shipped manifest binds to `view.tool_text`. A copy
// here would have had the wording and none of the guard.

/// ★★ Drawn where the swatch would be when the object's runs **disagree**.
///
/// Not an error and not a refusal: the control still applies. This is the
/// indeterminate state every editor in the class shows — Illustrator,
/// Inkscape, Figma, Word — and its meaning is *"there is no one colour to open
/// on; pick one and they all become it."*
///
/// ★ The marker itself is `super::properties`' `text_value_absent` — the em
/// dash — because that string's own doc comment already made this exact
/// argument for the size field: *"every property grid in this class shows a
/// blank or a dash for no value and for mixed values, which are the same state
/// as far as a single field is concerned."* One spelling of *no single value*
/// across the whole program.
#[must_use]
pub const fn mixed_hint() -> &'static str {
    "These words are not all one colour. Picking one sets all of them to it."
}
