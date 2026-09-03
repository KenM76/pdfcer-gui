//! # `text::settings::bytes` — what changing it makes pdfcer WRITE
//!
//! One of three copy modules under [`crate::text::settings`], split on
//! 2026-08-17 at rule R2's 1,500-line ceiling.
//!
//! ## ★ The split is by BLAST RADIUS, which is the window's own taxonomy
//!
//! Not by dialog group, and not alphabetically. Every setting in this window
//! carries a `*_radius` line stating *which way costs what*, and that line is
//! one of exactly three things:
//!
//! | module | radius | settings |
//! |---|---|---|
//! | [`super::look`] | changes what you SEE; the file is untouched | theme, CMYK intent, CMYK JPEG polarity, mask resampling, minification |
//! | [`super::extract`] | changes what you GET OUT — copy, search, redaction-by-pattern, new dimensions | word gap, unmappable codes, replacement text, parallel tolerance |
//! | [`super::bytes`] | changes what pdfcer WRITES | separations, missing appearance state, index line endings, trailing newline |
//!
//! That taxonomy is load-bearing rather than a filing convenience: it is the
//! distinction the window exists to make legible, and a test in
//! [`super`] asserts that exactly the byte-changing settings say they change
//! the file — in both directions, so a preview setting cannot quietly claim a
//! consequence it does not have.
//!
//! One setting is filed by its radius rather than by its group and it is worth
//! naming: **CMYK JPEG polarity** appears above under *look*, and its radius
//! line also says *"and the saved file if pdfcer re-compresses the image"*. It
//! is the only setting whose radius spans two of the three. It sits with the
//! others in its dialog group, where an operator looks for it.

// ===========================================================================
// Pages and printing — separations
// ===========================================================================

/// Separations: what it is.
#[must_use]
pub const fn separations_title() -> &'static str {
    "When pages are printing separations"
}

/// Separations: what the standard leaves open.
///
/// ★ Careful wording, and the care is the point. This is **not** a spec
/// ambiguity: §14.11.4 is perfectly clear about the invariant. What it does
/// not say is what an *editor* should do when an edit breaks it, and all three
/// answers are defensible for different workflows. Blurring the two shapes of
/// silence would make the window's whole framing dishonest.
#[must_use]
pub const fn separations_silence() -> &'static str {
    "Some print-ready files split one page into several — one per printing plate \
     (cyan, magenta, yellow, black). The standard says those pages must list each \
     other, but says nothing about what an editor should do when you delete or \
     extract only some of them."
}

/// Separations: what changing it costs.
#[must_use]
pub const fn separations_radius() -> &'static str {
    "Affects the file you save."
}

/// The default.
#[must_use]
pub const fn separations_repair_label() -> &'static str {
    "Keep them and fix the list (pdfcer's default)"
}

/// What repair means.
#[must_use]
pub const fn separations_repair_note() -> &'static str {
    "The pages you keep still know which plate they are, and their list is \
     updated to name only the plates that are still there. Nothing is left \
     pointing at a page that has gone."
}

/// The simplifying option.
#[must_use]
pub const fn separations_discard_label() -> &'static str {
    "Turn them into ordinary pages"
}

/// What it loses.
#[must_use]
pub const fn separations_discard_note() -> &'static str {
    "The pages are kept but forget they were printing plates. Simpler, and loses \
     the record of which ink each page was."
}

/// The strict option.
#[must_use]
pub const fn separations_refuse_label() -> &'static str {
    "Refuse the operation"
}

/// Who wants it.
#[must_use]
pub const fn separations_refuse_note() -> &'static str {
    "pdfcer declines rather than splitting a set of plates apart, and tells you \
     why. Use this if such files should never be edited a page at a time."
}

// ===========================================================================
// Pages and printing — missing appearance state
// ===========================================================================

/// Missing `/AS`: what it is.
#[must_use]
pub const fn missing_as_title() -> &'static str {
    "An annotation that does not say which of its looks to use"
}

/// Missing `/AS`: what the standard leaves open.
#[must_use]
pub const fn missing_as_silence() -> &'static str {
    "A checkbox or stamp can carry several appearances — on, off, and so on — and \
     is supposed to say which one applies. The standard does not say what a \
     reader should do when that is missing."
}

/// Missing `/AS`: what changing it costs.
///
/// The only setting whose radius separately names **printing**, and it needs
/// to: an appearance chosen for the screen is the appearance that goes on
/// paper, and an operator checking a form before printing it is exactly the
/// person this setting is for.
#[must_use]
pub const fn missing_as_radius() -> &'static str {
    "Affects what you see and what prints. Does not change the file."
}

/// The default.
#[must_use]
pub const fn missing_as_nothing_label() -> &'static str {
    "Draw nothing (pdfcer's default)"
}

/// Why refusing to guess is the shipped answer.
///
/// ★ The guess disclosure here is inverted from every other setting's, and
/// deliberately: what is disclosed is that *the other two are the guesses*.
/// Making either of them the default would be the "sneaky" failure the
/// disclosure rule forbids, because the operator would see a plausible
/// appearance with no indication pdfcer had chosen it.
#[must_use]
pub const fn missing_as_nothing_note() -> &'static str {
    "Refuses to guess. Nothing appears, and pdfcer counts it so you can see how \
     many were affected — better than inventing a state the document never chose. \
     The two options below are guesses, which is why neither is the default."
}

/// Show something.
#[must_use]
pub const fn missing_as_first_label() -> &'static str {
    "Draw the first one"
}

/// What "first" means, and what it risks.
#[must_use]
pub const fn missing_as_first_note() -> &'static str {
    "Shows something rather than nothing. \"First\" is the order the document \
     itself happens to list them in, which nothing guarantees is meaningful, so \
     this can show a ticked box that should be empty."
}

/// Assume off.
#[must_use]
pub const fn missing_as_off_label() -> &'static str {
    "Draw the \"off\" one if there is one"
}

/// Where it works and where it does not.
#[must_use]
pub const fn missing_as_off_note() -> &'static str {
    "Assumes an unset control is off, which is usually right for checkboxes and \
     meaningless for stamps."
}

// ===========================================================================
// Saving files — cross-reference entry line endings
// ===========================================================================

/// Xref EOL: what it is.
#[must_use]
pub const fn xref_eol_title() -> &'static str {
    "Line endings inside the file's index"
}

/// Xref EOL: what the standard leaves open.
#[must_use]
pub const fn xref_eol_silence() -> &'static str {
    "The standard fixes the length of each index entry but allows more than one \
     way to end the line. This is a genuine, recorded ambiguity."
}

/// Xref EOL: what changing it costs.
///
/// Bytes and nothing else. Every value here is conforming, so unlike the
/// preview settings there is nothing for the operator to *see* and therefore
/// nothing to disclose beyond the fact itself.
#[must_use]
pub const fn xref_eol_radius() -> &'static str {
    "Changes the bytes pdfcer writes. Nothing visible."
}

/// The default.
#[must_use]
pub const fn xref_eol_match_label() -> &'static str {
    "Keep whatever the file already uses (pdfcer's default)"
}

/// Why matching is the default, and what a fixed form would cost.
///
/// This default was changed on an operator ruling after the register pointed
/// out the shipped one was *"arguably wrong on pdfcer's own invariant"* — and
/// it was: objects pdfcer did not logically touch are re-emitted byte-identical,
/// and a full rewrite of a `CR LF` file under a fixed `SP LF` changes two bytes
/// in every entry. On a 5,000-object file that is a 10,000-byte diff in a
/// document nobody edited.
///
/// The note's *"below"* is only correct because the panel renders this option
/// **first**, which is not the order the functions are declared in. The
/// rendering order is the contract; see [`crate::dialogs::settings`].
#[must_use]
pub const fn xref_eol_match_note() -> &'static str {
    "Saving a document pdfcer did not otherwise change leaves its index untouched. \
     Picking a fixed form below would rewrite two bytes on every line of the index \
     of every file that used a different one — a large change to a file you did \
     not edit. Files that have no index of this kind get a space then a newline."
}

/// The former default.
#[must_use]
pub const fn xref_eol_space_lf_label() -> &'static str {
    "Always space then newline"
}

/// Who would want a fixed form.
#[must_use]
pub const fn xref_eol_space_lf_note() -> &'static str {
    "What pdfcer wrote for every file before it learned to match. Choose a fixed \
     form only if a specific tool in your workflow demands one."
}

/// The second fixed form.
///
/// No note: *"Space then carriage return"* describes itself completely, and
/// padding it would be noise. The `Option<&str>` in the option helper exists
/// for exactly these two entries.
#[must_use]
pub const fn xref_eol_space_cr_label() -> &'static str {
    "Space then carriage return"
}

/// The third fixed form. No note, as above.
#[must_use]
pub const fn xref_eol_cr_lf_label() -> &'static str {
    "Carriage return then newline"
}

// ===========================================================================
// Saving files — trailing end-of-line
// ===========================================================================

/// Trailing EOL: what it is.
#[must_use]
pub const fn trailing_eol_title() -> &'static str {
    "A final line ending at the end of the file"
}

/// Trailing EOL: what the standard leaves open.
#[must_use]
pub const fn trailing_eol_silence() -> &'static str {
    "The standard does not say whether anything may follow the end-of-file marker."
}

/// Trailing EOL: what changing it costs. One byte.
#[must_use]
pub const fn trailing_eol_radius() -> &'static str {
    "Changes the bytes pdfcer writes. Nothing visible."
}

/// The default.
#[must_use]
pub const fn trailing_eol_lf_label() -> &'static str {
    "End with a newline (pdfcer's default)"
}

/// ★ The guess disclosure the old note omitted.
///
/// Both readings of the standard are self-consistent and it does not choose.
/// The note read as a plain recommendation; it now says which of the two
/// pdfcer picked and that it picked.
#[must_use]
pub const fn trailing_eol_lf_note() -> &'static str {
    "Conventional, and what most tools produce. Both readings of the standard are \
     defensible and it does not choose between them, so this is pdfcer taking the \
     safer one — a trailing newline has never broken a reader."
}

/// The strict option.
#[must_use]
pub const fn trailing_eol_none_label() -> &'static str {
    "End immediately after the marker"
}

/// Who wants it.
#[must_use]
pub const fn trailing_eol_none_note() -> &'static str {
    "For a strict checker that objects to trailing bytes."
}

// ===========================================================================
// Saving files — /QuadPoints corner order
//
// ★ The register's own WORST CASE, and the one setting in this window whose
// effect nobody can ever see in pdfcer.
//
// The other two settings in this group say "changes the bytes pdfcer writes,
// nothing visible" and mean it about a viewer's rendering. This one is
// stronger than that: pdfcer bakes a full appearance stream for every markup
// annotation (R44), so pdfcer's OWN rendering never consults /QuadPoints at
// all. The order matters only to a third-party consumer that re-derives
// geometry from it — and a wrong order there draws a bow-tie rather than a
// rectangle.
//
// Which makes the disclosure the whole point of the control. An operator can
// mark up a document, look at it, save it, reopen it, and be perfectly happy
// while the file is producing bow-ties in a colleague's checker. There is no
// symptom on this side of the handover.
// ===========================================================================

/// Quad-point order: what it is.
#[must_use]
pub const fn quad_order_title() -> &'static str {
    "The corner order pdfcer writes for highlights and other text markup"
}

/// Quad-point order: what the standard leaves open.
///
/// ★ It does NOT leave it open, and that is the honest and unusual thing to
/// have to say in a window whose every other silence line means *the standard
/// declines to choose*. Section 12.5.6.10 states an order and essentially no
/// producer follows it, so pdfcer is choosing between the clause and the world.
/// Saying "the standard is silent" here would be a comfortable sentence and a
/// false one.
#[must_use]
pub const fn quad_order_silence() -> &'static str {
    "The standard states one order and almost no program follows it. Acrobat, \
     PDFBox and pdf.js all write and expect a different one, so this is a choice \
     between the wording and what the tools around you actually do."
}

/// Quad-point order: what changing it costs, and where the cost lands.
#[must_use]
pub const fn quad_order_radius() -> &'static str {
    "Changes the bytes pdfcer writes for new text markup. Nothing changes in \
     pdfcer, which draws these marks from their own stored appearance and never \
     reads these numbers back."
}

/// The default.
#[must_use]
pub const fn quad_order_reading_label() -> &'static str {
    "The order other programs use (pdfcer's default)"
}

/// Why the default is the one that departs from the wording.
#[must_use]
pub const fn quad_order_reading_note() -> &'static str {
    "Upper-left, upper-right, lower-left, lower-right. A markup annotation is \
     read by whatever the person you send it to already has, and that is \
     overwhelmingly one of the programs that expect this order."
}

/// The strict option.
#[must_use]
pub const fn quad_order_ccw_label() -> &'static str {
    "The order the standard describes"
}

/// What choosing it will cost, said plainly.
#[must_use]
pub const fn quad_order_ccw_note() -> &'static str {
    "Upper-left, upper-right, lower-right, lower-left — a counterclockwise walk. \
     For output going to a conformance checker. Expect Acrobat to draw the \
     marked area wrongly if it works the shape out from these numbers."
}

// ===========================================================================
// Fonts — faking bold and italic
// ===========================================================================

/// Faking bold/italic: what it is.
///
/// ★ Named for the ACT, not for the engine's type. `StylePolicy` means nothing
/// to an operator; *"faking bold and italic"* is what they will have seen
/// happen and the phrase they would search for.
#[must_use]
pub const fn style_policy_title() -> &'static str {
    "Faking bold and italic"
}

/// Faking bold/italic: what is left open.
///
/// ★★ This is the one `*_silence` line in the window that is **not** about the
/// standard being silent. Every other setting here exists because ISO 32000-1
/// permits two readings; this one exists because the *page* may not carry what
/// the operator asked for, and there is no answer in any standard to what a
/// program should do then.
///
/// It says so outright rather than borrowing the shape of the others. A
/// sentence implying the standard is undecided about synthesised weights would
/// send an operator looking for a clause that does not exist.
#[must_use]
pub const fn style_policy_silence() -> &'static str {
    "Nothing in the PDF standard says what a program should do when you ask for bold and the page carries no bold face. It describes how to thicken letters artificially and leaves the choice of whether to entirely to the program."
}

/// Faking bold/italic: what it costs.
///
/// ★★★ It changes **the bytes pdfcer writes**, and that is not obvious.
///
/// A faked weight is not a display trick: it is text rendering mode 2 plus a
/// stroke width written into the page's content stream, and a faked slant is a
/// shear term written into the text matrix. Both survive Save and both are what
/// every other viewer will show. An operator who read this as a preview setting
/// would hand on a drawing carrying artificial letterforms they thought were
/// only on their screen.
#[must_use]
pub const fn style_policy_radius() -> &'static str {
    "Changes the bytes pdfcer writes: a faked weight or slant is drawn into the page itself and is what every other viewer will show."
}

/// Faking bold/italic: the default.
#[must_use]
pub const fn style_policy_auto_label() -> &'static str {
    "Fake it quietly"
}

/// Faking bold/italic: the default's note.
///
/// ★ It states what pdfcer does FIRST, because the thing operators get wrong
/// about this setting is assuming it decides whether a real face is used. It
/// does not — a real face is always preferred, under all three choices.
#[must_use]
pub const fn style_policy_auto_note() -> &'static str {
    "pdfcer always uses a real bold or italic face when the page carries one. This is only about what happens when it does not: the letters are thickened or slanted artificially, and pdfcer reports that it did. The shipped default."
}

/// Faking bold/italic: warn.
#[must_use]
pub const fn style_policy_warn_label() -> &'static str {
    "Fake it, and say so plainly"
}

/// Faking bold/italic: warn's note.
#[must_use]
pub const fn style_policy_warn_note() -> &'static str {
    "The same, but faking gets a sentence of its own on the status bar rather than sitting among the edit's other notes. For work where an artificial weight in the finished file is worth noticing the moment it is made."
}

/// Faking bold/italic: refuse.
#[must_use]
pub const fn style_policy_refuse_label() -> &'static str {
    "Never fake it"
}

/// Faking bold/italic: refuse's note.
///
/// ★★ It says the button will appear not to work, in advance. This is the only
/// setting in this window that can make a control do nothing, and an operator
/// who chose it months earlier will otherwise read the silence as a defect.
#[must_use]
pub const fn style_policy_refuse_note() -> &'static str {
    "Bold and italic then change nothing on a page that carries no real face for them, and pdfcer says which face it looked for. Choose this if an artificial weight would be worse than no change at all — and expect the buttons to decline on some pages."
}

/// Faking bold/italic: the bound, disclosed under the whole group.
///
/// ★★★ The fact that makes this setting narrower than it looks, and it is a
/// fact rather than a direction — so it is drawn under the group rather than
/// attached to one option, exactly as `actual_text_bound` is.
///
/// A real face is preferred under **every** choice here. pdfcer asks the engine
/// which real face is on offer before it asks for anything to be faked, and
/// takes the offer when there is one. None of the three options can turn that
/// off, and an operator who read *"Never fake it"* as *"never change my font"*
/// has misread it in the direction that matters.
#[must_use]
pub const fn style_policy_bound() -> &'static str {
    "Whichever you choose, pdfcer looks for a real bold or italic face on the page first and uses it if there is one — including a face from another family when nothing in the text's own family will do. These options only decide what happens after that search comes up empty."
}
