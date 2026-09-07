//! # `text::status::waiting` — **sentences about the program being busy**
//!
//! One function today. It is its own module rather than a paragraph in
//! [`super`] because it is a different **species** of sentence from everything
//! that catalog holds, and the distinction is the one `app::status`'s own header
//! spends forty lines making.
//!
//! Every other line in that half of the bar describes **an event**: what a fill
//! inferred, what a move had to change, what a command declined to do. Each is
//! keyed on the document's edit epoch and retires when the document moves past
//! it.
//!
//! This describes **a state** — *the picture you are looking at is not the
//! answer yet* — which is live for as long as its condition holds, needs no
//! retirement rule, and can appear for one edit and not the next depending only
//! on how hard the page was to draw.
//!
//! ★ Filed apart so that the second sentence of this species, when it comes, is
//! written beside the first and under the same rules rather than filed by
//! whichever group looked closest.
//!
//! ## ★★★ The second sentence arrived on 2026-09-05, and the prediction held
//!
//! [`line_weights_off`] — *the canvas is deliberately not showing what will
//! print* — is a state exactly as [`page_catching_up`] is: live while a toggle
//! is on, retiring by itself when it goes off, with no epoch and nothing to
//! clear. It was written here without argument because the paragraph above had
//! already made it.
//!
//! ⚠ **They are not the same KIND of state, and the difference decides the
//! wording.** The first is *temporary and self-correcting* — wait, and the
//! picture catches up. The second is *chosen and permanent* — it will not
//! resolve, because the operator asked for it, and it ends only when he ends
//! it. So the first reassures and the second must NOT: it is rule 4's
//! disclosure obligation, and a soothing sentence there would be the sneaky
//! half of that rule wearing a friendly face.

/// **The page is still being redrawn** — `OPERATOR_REQUESTS.md` O63.
///
/// # ★★★ The two ways this sentence could be wrong, and they are opposite
///
/// **Too alarming** and it reads as a fault. Nothing is wrong: the edit
/// happened, the document is correct, and a picture is being made of it. A
/// sentence that sounded like a warning would teach the operator to distrust an
/// edit that worked.
///
/// **Too quiet** and it fails at its one job. This exists because on a dense
/// drawing the picture is a second or two behind the document, and an operator
/// who cannot tell *"the program is drawing"* from *"the program ignored me"*
/// presses the button again — which is a second edit, on top of the first,
/// neither of which they wanted.
///
/// ⇒ So it states the fact in the operator's own terms and puts the reassurance
/// first: **the change is already made**. The redraw is described as something
/// happening to the *picture*, never to the document.
///
/// # ★★ Why it does not name a duration or show a progress bar
///
/// Because nothing here knows one. A page's render time is a property of its
/// content — 8.97 ms for a text page against 877 ms for a CAD sheet — and a
/// number invented for the sake of having one would be wrong on most documents.
/// A bar that filled at a rate nobody could predict would be worse than a
/// sentence, because it promises a shape of wait it cannot keep.
#[must_use]
pub const fn page_catching_up() -> &'static str {
    "Your change is made \u{2014} the picture of it is still being drawn."
}

/// **Line weights are off, so this is not what will print** —
/// `OPERATOR_REQUESTS.md` **O137**.
///
/// # ★★★ Why a reading aid the operator asked for still owes a disclosure
///
/// This is not Rule 4's usual case. Rule 4 covers **pdfcer marking its own
/// uncertainty** — an inference the operator cannot see and did not request.
/// Here the operator pressed a button and got exactly what the button said.
///
/// The obligation survives anyway, and for a reason that is about the *canvas*
/// rather than about the inference: the whole of this shell's contract with the
/// operator is that **applied content renders exactly as saved content will
/// render**. The one-line test is *would a screenshot of the canvas differ from
/// a screenshot of the same document saved and reopened?* — and while this is
/// on, the answer is **yes, deliberately**. That divergence is the only one in
/// the program, it is invisible on a drawing whose strokes are already thin,
/// and it is precisely the thing an operator will forget by the time they are
/// three sheets deeper. So the canvas's own claim is suspended, and a suspended
/// claim is stated.
///
/// ★★ It is **off-canvas**, in the status bar, never a badge on the page. A
/// mark on the page would break the same rule it exists to honour, and would
/// also be the nagging `DEFECTS.md` §5 records.
///
/// # ★★★ Every clause, and what each one is against
///
/// **"Line weights are off"** names the control, in the label's own words, so
/// the sentence and the button that caused it are recognisably the same thing.
/// An operator who reads this line without remembering what they pressed can
/// find the switch from the words alone.
///
/// **"every line is drawn one pixel wide"** is the fact, and it is checkable.
/// Not "thin" — thin is relative and is the other convention's word (Acrobat's
/// *enhance thin lines* makes thin things THICKER; this makes thick things
/// thinner, and the two are opposites).
///
/// **"Printing and exporting still use the real widths"** is the reassurance,
/// and it is the half the operator actually needs. Without it the honest
/// reading of the first clause is *"my drawing has been changed"*, which would
/// send him looking for an undo. This is the guarantee the feature was built
/// around, so it is stated where he is looking.
///
/// ★ It does **not** say how to turn it back off. The button is on the View tab
/// rendered pressed, which is where a toggle's own state belongs; a status line
/// that carried instructions would be twice as long, and length in this bar is
/// paid for by the sentences beside it.
///
/// ★ It does not apologise and does not warn. He chose this.
#[must_use]
pub const fn line_weights_off() -> &'static str {
    "Line weights are off \u{2014} every line is drawn one pixel wide. Printing and exporting \
     still use the real widths."
}

/// ★★★ **The line-weights mode is on and it changed NOTHING in view** —
/// `OPERATOR_REQUESTS.md` **O137**, and the half this shell asked the engine
/// for by name.
///
/// # The two states this sentence exists to tell apart
///
/// | what the operator sees | what is true | without this line |
/// |---|---|---|
/// | flipped the switch, nothing changed | **this view has no strokes thick enough to thin** | *"the setting is broken"* |
/// | flipped the switch, nothing changed | the setting really is not reaching the renderer | *"the setting is broken"* — and right |
///
/// Those are the same screenshot. `pdfcer-core`'s reply put it exactly:
/// *"nothing reported whether the mode had done anything, so an operator could
/// switch it on, see no change, and have no way to tell 'this drawing has no
/// strokes thin enough to matter' from 'the setting is broken'."*
/// `Diagnostics::strokes_hairlined` (`Pass 254.1`) is that count, and it counts
/// strokes **thinned**, not strokes drawn.
///
/// # ★★★ IT SAYS "IN VIEW", AND THAT WORD IS LOAD-BEARING
///
/// The count is a property of **the region that was rasterised**, not of the
/// document or even of the page. Scroll to a corner with no linework and it
/// falls to zero — truthfully. A sentence saying *"this drawing has no thin
/// lines"* would then be a claim about the drawing, made from a measurement of
/// a patch of white paper, and it would flip back the moment he scrolled.
///
/// ⇒ `blend_space_disclosure` two screens up records the same hazard from the
/// other side and chose the opposite field for it: it keys on
/// `cmyk_buffer_refused` — true of the whole page — rather than on the count of
/// blends that then went wrong, precisely because the count *"is zero on a page
/// whose transparency is outside the rendered region, so keying on it would go
/// quiet exactly where the operator scrolled away from the affected patch and
/// back."*
///
/// Here the region-scoped reading is the **correct** one rather than a
/// limitation, because the question the operator is asking is about what he is
/// looking at: *why does this look the same?* So the fix is not a different
/// field, it is honest scoping in the words.
#[must_use]
pub const fn line_weights_no_effect() -> &'static str {
    "Line weights are off and nothing in view was thick enough to thin — this part of the \
     drawing looks the same either way."
}
