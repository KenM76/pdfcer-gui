# `panels::properties::paint` — **the colour of the selected path(s)**

`OPERATOR_REQUESTS.md` **O89**, and the half that did not exist:

> *"I don't see where I am able to edit the color of text, vectors, etc."*

Text had a control and he could not find it. Vectors had none, because
`pdfcer-core` had no verb — every colour verb it owned worked on an
annotation or on text. `Pass 218.0`/`219.0` shipped
`EditSession::set_object_paint`, and this is the control.

## The swatch shows the object's OWN colour, or refuses to show one

`PathPaint` has three states and the difference between them is the whole
design of this section:

| state | what it means | what is drawn |
|---|---|---|
| `Default` | §8.6.8 black — **nobody chose a colour** | a swatch, black |
| `Device { rgb, … }` | somebody chose this | a swatch, that colour |
| `Other { space, … }` | a space pdfcer does not decode — a spot ink, a pattern | **the ink's NAME, and no swatch** |

⇒ The request made the argument and the engine wrote it into the verb's
docs: *"a colour control with no current value is a control that silently
discards what was there the moment it is touched."* A swatch opening on
black over a `/Separation` stroke is exactly that — one click and a named
spot ink is screen colour, permanently, and it looked right while it
happened.

`Default` and `Device`-holding-black are drawn the same and are **not**
the same fact. Only the first may be replaced without comment; the
distinction is kept because it costs nothing to keep and cannot be
recovered once collapsed.

## Why a spot ink is NAMED rather than converted

Evaluating a tint transform lives in `pdfcer-render`, which `pdfcer-core`
cannot depend on. The engine declined to duplicate it — a second colour-space
implementation is the class of defect this whole Pass removed — and it is
also the better answer for this panel: *"this stroke is spot ink PANTONE
300"* tells a drawing office more than a square of approximate blue.

## Fill and stroke are separate, and a refusal is per channel

`None` leaves a channel alone. Recolouring the fill of an object whose
stroke is a spot ink is **not** blocked by the channel nobody touched — the
engine has a test for it, and this section relies on it rather than
pre-emptively greying a control that would have worked.



> *"One object at a time, for now. The engine will recolour a whole
> selection; the control does not offer it yet, because when the objects
> disagree there is no honest colour to open on and picking the first one's
> would quietly propose flattening the rest to it."*

The danger in that sentence is real and the conclusion was wrong.
**Every editor in this product class has already solved it** — Illustrator,
Inkscape, Figma and Word all show an *indeterminate* control over a
disagreeing selection, and applying a value sets every member. This
project's standing rule is that the convergence of the product class **is**
the specification and an invented interaction is a defect even when it
works, so the mixed state is not one option among several; it is the answer.

[`super::swatch`] is that control. It opens on nothing in particular (PDF
§8.6.8's default, never applied unless the operator moves the picker), it
reads as *no single value* using the marker this shell already writes for
one, and picking a colour applies it to the whole selection.

**One undo step for the whole gesture** — and that took the swatch being
hand-built rather than `ui.color_edit_button_srgb`. `egui`'s own colour
button marks itself changed on *every frame of a drag inside the picker*, so
acting on `.changed()` authors an edit per frame; [`super::swatch`]'s header
carries the measurement and the fix (commit when the picker closes). That
defect was present in the single-object control this section shipped with —
it is fixed by the same change, for the same reason, and nothing about it is
new to the multi-object path.

## A MIXED SELECTION CONTAINING ONE SPOT INK — the decision, and why

The choice was between refusing the whole apply by name and applying to the
process-colour members while reporting the spot ones off-canvas. **This
section applies and reports**, and the argument is four things rather than a
preference:

1. **The guard is held by the engine, structurally, not by this control.**
   `EditSession::set_object_paint` tests every object's paint on every
   channel it is asked to change and returns
   `PaintOutcome { changed, refused }` — a `/Separation` member is refused
   *by the verb*, whatever this panel draws. A shell-side blanket refusal
   would add nothing to the plate's safety and would remove a capability.
   ⇒ There is **no hole**: the single-select path's guard is *"do not offer
   a swatch whose only possible effect is destruction"*, and it still holds,
   because where every member is a named ink no swatch is drawn at all.
2. **The operator ruled on this exact case**, and it is quoted in
   `crate::text::paint::recoloured_partly`'s own doc comment: *"a selection
   of twelve strokes where three are in a colour space pdfcer will not
   rewrite needs to say 'nine changed', not 'done'."* That is
   apply-and-report, in his words, about this shape.
3. **Refusing would be unusable on the documents this program is for.** One
   spot-inked line inside a marquee of two hundred would block the gesture,
   and the remedy — find it and deselect it — is being asked of an operator
   who cannot see which line it is. A safe operation would have been traded
   for an impossible one.
4. **The disclosure arrives BEFORE the gesture, not only after it.**
   `crate::text::paint::mixed_named_inks` names how many members carry an
   ink and what those inks are called, on the row, above the swatch. The
   status line's *"nine changed, three left alone"* is the confirmation, not
   the first news.

## Rule 4 — nothing here marks the canvas

The recoloured objects render exactly as the saved file will render them.
Which members were skipped, which are named inks and whether they disagreed
is disclosed **off-canvas**: on this row, and in the status bar through
`crate::app::actions::disclosure`. No badge, no tint, no outline on the page.
