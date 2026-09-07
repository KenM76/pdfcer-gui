//! # `text::panels::properties::markup` — every word the **markup style**
//! section of the Properties panel says
//!
//! ## Why this is a separate file
//!
//! R2. `properties.rs` carried four subjects — the markup's style, the
//! selection's geometry, the selected text's style and the document's own
//! properties — and it passed 1,500 lines on 2026-09-07 when the geometry
//! section grew an **Angle** field. The markup section is the largest of the
//! four and the most self-contained: nothing outside it reads these strings,
//! and `pdfcer-core`'s `set_markup_style` is the single verb every one of them
//! is about.
//!
//! ## ★★ These strings age faster than any others in the catalogue
//!
//! Three of them have been corrected for being **factually wrong about the
//! engine** rather than badly worded, and two of those corrections happened
//! within hours of the sentence being written — the engine shipped the
//! capability the same afternoon the limitation was described. The comment
//! blocks recording each of those are kept verbatim, including the struck-out
//! reasoning, because the shape of the mistake is the useful part.
//!
//! ⇒ **A limitation sentence on this project has a shelf life measured in
//! hours.** Before repeating any claim here about what `set_markup_style` will
//! or will not do, re-read the verb at the current pin. Do not quote this file
//! as a source about the engine; it is a record of what was true on a date.

// ===========================================================================
// The selected markup's style — `set_markup_style`
// ===========================================================================

/// The heading over the markup restyle controls.
#[must_use]
pub const fn markup_heading() -> &'static str {
    "This markup"
}

/// The line under it: what kind of mark is selected.
///
/// ★ The file's own `/Subtype`, translated. An operator placed a *rectangle*
/// and the file calls it `Square`; they placed an *arrow* and the file calls it
/// `Line`. Showing the file's word would be correct and useless — the standing
/// rule in `text::commands` is that a label is the operator's vocabulary and an
/// id is the format's.
#[must_use]
pub fn markup_subtype(subtype: &str) -> String {
    let name = match subtype {
        "Square" => "Rectangle",
        "Circle" => "Ellipse",
        "Line" => "Arrow or line",
        "Polygon" => "Polygon or revision cloud",
        "PolyLine" => "Polyline",
        "Ink" => "Freehand",
        "Highlight" => "Highlight",
        "Underline" => "Underline",
        "StrikeOut" => "Strikeout",
        "Squiggly" => "Squiggly",
        "FreeText" => "Text box",
        "Text" => "Sticky note",
        "Stamp" => "Stamp",
        // ★ Not "Unknown". A subtype this catalogue has no word for is still a
        // real mark the operator can see and is about to restyle, and the
        // file's own spelling is the most honest thing left to show them.
        other => other,
    };
    format!("{name} on this page")
}

/// The colour control's label.
#[must_use]
pub const fn markup_colour_label() -> &'static str {
    "Colour"
}

/// The width control's label.
#[must_use]
pub const fn markup_width_label() -> &'static str {
    "Line width"
}

/// The suffix on the width control.
#[must_use]
pub const fn markup_width_suffix() -> &'static str {
    " pt"
}

/// The Line style row's label.
///
/// ★ *"Line style"* is `RIBBON_IA.md` §5.8's own name for the row and is what
/// the Format tab's command is called, so the two surfaces agree. It sits
/// directly under *Line width*, and the shared first word is doing work: the two
/// rows are one subject and read as a pair.
///
/// ★★ Not *"Dash pattern"*. The chooser's first entry is **Solid**, and under a
/// label reading *Dash pattern* that entry would read as *no dash pattern* — the
/// absence of the thing the label names rather than one of its values. The entry
/// names themselves are `crate::text::markup`'s, because three surfaces show
/// them and only one of the three is this panel.
#[must_use]
pub const fn markup_line_style_label() -> &'static str {
    "Line style"
}

/// The opacity control's label.
#[must_use]
pub const fn markup_opacity_label() -> &'static str {
    "Opacity"
}

/// The suffix on the opacity control.
///
/// A percentage, because that is the unit every application an operator has
/// used states opacity in. `/CA`'s own `0.0..=1.0` is a file-format detail they
/// should never meet.
#[must_use]
pub const fn markup_opacity_suffix() -> &'static str {
    " %"
}

/// The button that removes a property, restoring the file's own default.
///
/// ★ *"Clear"*, not *"Reset"* or *"Default"*. It removes the key from the
/// annotation dictionary, and what happens then is that the **standard's**
/// default applies — which is not necessarily what the mark looked like when
/// the operator placed it. "Reset" would promise a return to a previous state
/// that pdfcer does not remember.
#[must_use]
pub const fn markup_clear() -> &'static str {
    "Clear"
}

/// ★★ What restyling costs, said once under the whole section.
///
/// Two facts an operator cannot see and would otherwise discover from a
/// changed file:
///
/// 1. **The appearance is regenerated.** `set_markup_style` redraws the mark
///    from the geometry pdfcer models, so anything the original expressed
///    *outside* that model — a border effect pdfcer does not author, a producer's
///    own decoration — is gone from the new appearance even though its
///    dictionary key survives. The engine reports each one, and those arrive
///    verbatim on the status row; this sentence is the standing warning that
///    such a report is possible at all.
/// 2. **A wider line moves the box.** For every subtype except a rectangle and
///    an ellipse, `/Rect` is derived from the geometry plus a margin that
///    contains the stroke and any arrowheads — so widening the pen makes the
///    annotation's rectangle bigger. That is the engine's own ⚠, and it is the
///    difference between a mark that looks the same and a mark that occupies
///    the same space.
#[must_use]
pub const fn markup_note() -> &'static str {
    "Changing any of these redraws the mark from the shape pdfcer has recorded for it. A wider \
     line also makes the mark's own box bigger, except on rectangles and ellipses."
}

/// Why the controls are greyed on a locked annotation.
///
/// Names the standard, because an operator who meets this wants to know whether
/// pdfcer is refusing or the document is — and it is the document. It also names
/// the one thing that is still possible, which is the rule a refusal follows
/// everywhere in this shell.
#[must_use]
pub const fn markup_locked() -> &'static str {
    "This mark is locked by the document, so its appearance cannot be changed here. You can \
     still delete it."
}

/// ★★★ **What is possible on a mark this shell cannot restyle** — the sentence
/// that replaced three live controls that could not commit.
///
/// The defect, the reachability test that fixes it, and why R9 wants a sentence
/// here rather than an empty space, are all in
/// `crate::panels::properties::markup`'s header. This doc records the part that
/// belongs to the **words**: that each of the four claims was checked against
/// the engine on 2026-09-06 before it was written, because a limitation
/// sentence has an hours-long shelf life on this project and a false one is
/// worse than none.
///
/// # ⚠⚠⚠ AND IT BECAME PARTLY FALSE THAT AFTERNOON — the shelf life was hours
///
/// **What this returned until 2026-09-06 (afternoon)**, kept verbatim because
/// it was reasonable when written and the correction is the useful part:
///
/// > *"pdfcer does not redraw this kind of mark, so its colour, line width and
/// > opacity cannot be changed here. You can still move it, resize it, delete
/// > it, and edit the note it carries."*
///
/// True of the only style verb that existed; `set_text_annot_style` shipped
/// that afternoon and restyles two of the three subtypes it was shown for.
/// [`crate::text::panels::textannotstyle`]'s header has the whole account.
///
/// - **move** — `move_annotation` refuses a ce dimension and a form widget by
///   name, then works from `/Rect` and whatever geometry keys are present.
/// - **resize** — `resize_annotation` refuses the same two and otherwise
///   **carries** a foreign appearance rather than rebuilding it; a uniform
///   scale is exact. (A non-uniform scale of a foreign appearance is refused
///   unless distortion is allowed — that refusal arrives from the engine with
///   its own message and is not this sentence's subject.)
/// - **delete** — [`markup_locked`] already promises it, and
///   `crate::panels::properties::annotdelete` speaks for any annotation: the
///   verb is document-wide rather than per-subtype.
/// - **the note** — `set_markup_note` refuses a ce dimension and a widget, and
///   nothing else.
#[must_use]
pub const fn markup_not_restylable() -> &'static str {
    "pdfcer cannot read this mark's shape back, so its appearance cannot be changed here. You \
     can still move it, resize it, delete it, and edit the note it carries."
}

/// The fill control's label — `/IC`, the interior colour.
///
/// *"Fill"* rather than *"Interior"*: `/IC` is the format's word and every
/// drawing application an operator has used calls it fill. The standing rule in
/// `text::commands` is that a label is the operator's vocabulary and an id is
/// the format's.
#[must_use]
pub const fn markup_fill_label() -> &'static str {
    "Fill"
}

/// What sits beside the fill swatch when the mark has no `/IC` at all.
///
/// ★ It exists because a swatch cannot show *absence*. With no `/IC` the swatch
/// falls back to black, and a black square beside the word "Fill" says "this
/// shape is filled black" — which is the opposite of the truth. Acrobat draws a
/// red diagonal through its no-colour swatch for exactly this reason; this
/// shell says the word instead, which survives a theme change and a screen
/// reader where a drawn diagonal does not.
#[must_use]
pub const fn markup_fill_none() -> &'static str {
    "None"
}

/// The label over the chooser for the ending drawn at a line's **start** —
/// `/LE`'s first element (§12.5.6.7, Table 176).
#[must_use]
pub const fn markup_line_start_label() -> &'static str {
    "Line start"
}

/// The label over the chooser for the ending drawn at a line's **end** —
/// `/LE`'s second element.
#[must_use]
pub const fn markup_line_end_label() -> &'static str {
    "Line end"
}

/// One line-ending style, in the operator's words.
///
/// ★ Table 176 names ten endings and pdfcer authors three of them —
/// `annot_author::LineEnding` has exactly `None`, `OpenArrow` and `ClosedArrow`,
/// and its own doc comment calls the rest "a documented not-yet-authored
/// remainder". The chooser offers what the engine can draw, because a
/// fourth entry that produced a `/Butt` the appearance did not show would be
/// the inert control this project forbids, one level down.
///
/// The words are the operator's rather than the file's: *"Open arrow"*, not
/// `/OpenArrow`.
#[must_use]
pub const fn markup_line_ending_name(
    ending: pdfcer_core::annot_author::LineEnding,
) -> &'static str {
    use pdfcer_core::annot_author::LineEnding as L;
    // Exhaustive on purpose, with no wildcard: `LineEnding` is NOT
    // `#[non_exhaustive]`, so an ending the engine learns to draw breaks this
    // match at compile time rather than silently reaching a fallback word. That
    // is the whole reason the list is not written out in the panel module.
    match ending {
        L::None => "No end",
        L::OpenArrow => "Open arrow",
        L::ClosedArrow => "Closed arrow",
    }
}

/// The disclosure under the two line-ending choosers.
///
/// ★ It is owed because the readback is **lossy in one direction and silent
/// about it**: `annot_author::read_line_endings` degrades any Table 176 ending
/// pdfcer does not author down to `None`, so a `/Line` a foreign producer gave
/// a `/Butt` or a `/Diamond` end reads here as *No end* — and the mark on the
/// page plainly has one. Without this sentence the operator's conclusion is
/// that the chooser is broken.
#[must_use]
pub const fn markup_line_ending_note() -> &'static str {
    "pdfcer draws three of the standard's line ends. A mark that carries any other one shows as \
     No end here, and redrawing it does not put that end back."
}

/// **The fifth state of the arrowhead controls — take the setting OUT of the
/// file rather than write "none" into it.**
///
/// # ★★★ Two states, one picture, and why the operator is offered both
///
/// `MarkupStyle::endings` became a `StyleEdit` on 2026-09-06, in answer to this
/// shell's own request (`pdfcer-core` `edit.rs:4390`, and the field's own doc
/// comment at `edit.rs:4373`): `Set` writes `/LE`, and `Clear` **removes the
/// key**. Until that day *"draw no arrowheads"* was expressible and *"have no
/// `/LE` at all"* was not, so an operator who turned an arrow's heads off got a
/// document that no longer matched the one they opened, differing in a key
/// neither this panel nor the Format tab shows.
///
/// Table 176 makes `/None` the default for both ends, so `/LE [/None /None]`
/// and an absent `/LE` draw the same line. The difference is **bytes**, and
/// this project does not treat different bytes as the same document. The
/// engine's reply names the argument that decided it over a cheaper
/// doc-comment fix: a **signed** drawing, and the question *"is this
/// byte-identical to what my client sent me"* — a question undo does not
/// answer, because undo covers the session and not the round trip.
///
/// # ★★ Why these words
///
/// *"Clear"* is the verb this panel already uses for `/C`, `/IC` and `/CA`, and
/// [`markup_clear`] argues it: the word is honest about the **act** — the key
/// goes, and what applies afterwards is the standard's default rather than
/// anything pdfcer remembers — where *"Reset"* or *"Default"* would promise a
/// return to a previous state pdfcer never recorded.
///
/// ★ It carries a noun where the other three do not. A bare *"Clear"* in a list
/// whose first entry is already *"No arrowheads"* reads as a second name for
/// that entry, which is the one misreading this control cannot afford: those
/// two produce identical pictures and different files, so an operator who
/// confuses them cannot discover the mistake by looking.
///
/// ⚠ It deliberately does **not** say `/LE`, *key*, or *dictionary*. The
/// operator is a drafter; the fact they need is *the file goes back out the way
/// it came in*, and [`markup_endings_clear_hint`] says exactly that.
#[must_use]
pub const fn markup_endings_clear() -> &'static str {
    "Clear the setting"
}

/// Why an operator would press [`markup_endings_clear`] when the line looks
/// identical either way.
///
/// ★ The hover carries the whole distinction, because the control cannot: the
/// two states are indistinguishable on the page and the difference only shows
/// up in a byte comparison of the saved file. `REVIEW_TRIAGE.md`'s rule — a
/// caveat below the thing it qualifies arrives after the operator has drawn
/// their conclusion — is why it is a hover on the control rather than a note
/// underneath it.
///
/// ★ It names the **consequence** an operator has met (a drawing that comes
/// back different from the one that went out) rather than the mechanism (a
/// dictionary key). Both surfaces read this one string, so the Format tab and
/// this panel cannot come to explain the same act two different ways.
#[must_use]
pub const fn markup_endings_clear_hint() -> &'static str {
    "Takes the arrowhead setting out of the file instead of writing \"no arrowheads\" into it. \
     The line looks the same either way. Use it when a mark arrived without arrowheads and you \
     want it to go back out the way it came in — on a signed or issued drawing, a setting that \
     was not there before is a difference someone will find."
}

/// ★★★ The narrowing the colour swatches perform, said where the operator can
/// see it — **before** the click rather than after.
///
/// §12.5.2 lets `/C` and `/IC` be a 0-, 1-, 3- or **4**-component array, and the
/// four-component case is CMYK, which is not rare on a CAD sheet where the
/// producer is plotter-bound. The swatches convert one for display and a change
/// made through them writes RGB in its place, which is a real narrowing of the
/// colour space and is disclosed rather than performed quietly — the engine's
/// own posture on every conversion it makes.
///
/// The full argument, including the refuse-to-show behaviour this replaced and
/// why that was worse than an approximation, is on
/// `crate::panels::properties::markup`'s `swatch_of`.
#[must_use]
pub const fn markup_colour_narrowed() -> &'static str {
    "This mark's colour is recorded in CMYK and the swatches above are an approximation of it. \
     Picking a new colour here records an RGB one in its place."
}

/// ★★ What regenerating an appearance LOST, in the operator's terms.
///
/// `set_markup_style` redraws a mark from the geometry pdfcer models, so
/// anything the original expressed *outside* that model is gone from the new
/// appearance even though its dictionary key survives. The engine names each
/// one; this is the sentence that reaches the operator, and it is owed under
/// rule 4's surviving half — **an inference the operator cannot see still owes
/// an off-canvas report.**
///
/// ★ Every sentence says **what they will see**, not what a key is called. An
/// operator who is told *"the `/BE` border effect was dropped"* has been told
/// nothing; one who is told *"its cloudy edge is now a plain outline"* can look
/// at the page and decide whether they mind.
///
/// # ★★★ TWO OF THESE NARROWED ON 2026-09-06, AND ONE OF THEM WAS LEFT SAYING
/// # SOMETHING FALSE
///
/// `DroppedProperty::BorderStyle` and `::DashPattern` *"now fire much less
/// often"* — the engine's words — because a dashed border is read back and
/// re-authored rather than solidified
/// (`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:4220-4241`, and the variants'
/// own docs at `edit.rs:4884-4904`). That is the same narrowing `BorderEffect`
/// took after `Pass 98.0`, and it is disclosed here because **a narrowed
/// disclosure whose wording was written for the wide case is a false
/// disclosure**, which rule 4 forbids in exactly the direction it forbids
/// silence.
///
/// What each one now means, read out of the emission site rather than assumed:
///
/// | variant | fires when |
/// |---|---|
/// | `BorderStyle` | `/BS` `/S` names a style pdfcer does not redraw — `/B`, `/I`, `/U` — **or `/S /D` whose dash was not carried** |
/// | `DashPattern` | `/BS` `/D` is present and the dash was not carried: an array §8.4.3.6 does not admit, or a caller that **cleared** it |
///
/// ⚠ **The `BorderStyle` string used to name only a bevel, an inset and an
/// underline, and after the narrowing that became wrong.** The `/S /D` row is
/// new to it: an operator who presses **Solid** on a dashed mark clears the
/// dash, the original dictionary still says `/S /D`, and both variants fire — so
/// the old wording would have told them their mark had a *bevel*. It now names
/// the dash case too, and both sentences are phrased as facts about the new
/// outline rather than about a cause, so each is true whether the change was
/// asked for or merely disclosed.
///
/// ★ The redundancy on a requested clear is accepted rather than engineered
/// away. Suppressing a disclosure when the shell believes the operator asked for
/// it would put the decision *"was this loss requested?"* into
/// `app::actions::apply`'s routing arm, which that arm's own note forbids — it
/// routes and does not compute — and a suppression rule that got it wrong would
/// hide a real loss. A true sentence twice beats a missing one once.
#[must_use]
pub const fn markup_dropped(dropped: pdfcer_core::edit::DroppedProperty) -> &'static str {
    use pdfcer_core::edit::DroppedProperty as D;
    match dropped {
        D::BorderEffect => {
            "This mark had a cloudy or hand-drawn edge that pdfcer does not redraw. It is now a plain outline."
        }
        D::BorderStyle => {
            "This mark's border was declared in a style that is not in the new outline — a bevel, an inset, an underline, or a dash. It is drawn as a plain line now."
        }
        D::DashPattern => {
            "This mark carried a dash pattern that is not in the new outline. Its border is drawn solid now."
        }
        D::RectDifferences => {
            "This mark's own box was inset from the area it covered, and that inset is gone. The mark is drawn to the box now."
        }
        D::LineEnding => {
            "This mark had arrowheads or line ends pdfcer does not redraw, and they are gone."
        }
        // `DroppedProperty` is `#[non_exhaustive]`, so a wildcard is required
        // rather than optional. It answers with the general form of the same
        // fact, which is true of every member: something the file expressed is
        // not in the picture pdfcer just drew, and saying so imprecisely is far
        // better than saying nothing.
        _ => {
            "This mark carried something pdfcer does not redraw, and it is not in the new appearance."
        }
    }
}
