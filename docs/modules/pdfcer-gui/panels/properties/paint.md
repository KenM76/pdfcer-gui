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

## Item notes

### `const REGION_SUBJECT`

Its own region since the multi-object state shipped, because *"the section
drew"* and *"the section told the operator how many things it is about to
change"* are two different claims and a driven check has to be able to
assert the second.

### `struct Recolour`

A named pair rather than a tuple of two options, because the two positions
are not interchangeable and a tuple invites reading them the wrong way round
exactly once — after which the fill control recolours the line. Clippy asked
for the type; the naming is why it was worth asking.

### `fn path_indices`

Re-derived rather than collected in the walk above, because the walk's
output is *paints* and pairing them with indices would make one `Vec` whose
two halves have to be kept in step by hand. The provider read is cheap (a
slice index per entry) and the alternative is the class of bug where a
filter and its operand drift.

### `fn channel`

The ink check comes **before** agreement, not after, and that ordering
is the guard. *"They all agree and one of them is a spot ink"* must never
draw a swatch: agreement between two members of a named-ink selection is not
permission to overwrite them.

A member whose paint cannot be shown is excluded from the agreement
question entirely rather than counted as a disagreement. It is not a colour
this control can compare, and folding it in would report *"mixed"* for a
selection of one red line and one PANTONE line — implying a value would
unify them, which is exactly what will not happen.

### `fn row`

Returns the newly chosen colour, or `None` when nothing was committed this
frame. *Committed*, not *changed*: [`super::swatch::show`] answers only on
the frame the picker closes, so one drag through a colour wheel is one
action and one undo entry.

### `fn to_bytes`

Rounded rather than truncated. Truncation makes 1.0 into 255 correctly and
0.5 into 127 — half a step dark on every mid-tone, which over a round trip
through the swatch would walk a colour steadily darker every time it was
opened and closed without being changed.

### `fn ink_name`

Raw bytes, decoded loosely. A colour-space resource name is a PDF name
object and carries no declared encoding; showing it as it is beats showing
nothing, and beats a repaired version that no longer matches what the
operator would find in the file.

### `fn a_spot_ink_offers_no_colour_to_edit`

The one assertion this module exists for. If `rgb()` ever answered
`Some` for `Other`, this section would draw a swatch over a spot ink and
the first click would convert it — invisibly, permanently, and looking
entirely normal.

### `fn two_different_colours_read_as_mixed`

The whole of O89 piece 2. The fixture genuinely disagrees — red and
green — because a fixture whose members all share one colour would pass
against an implementation that simply showed the first one's.

### `fn one_spot_ink_among_process_colours_keeps_the_swatch_and_names_the_ink`

The decision the module header argues, asserted rather than left to the
prose: the swatch survives (so the nine reachable strokes can be
recoloured), and the ink is listed (so the operator knows before
pressing that one of them will be left alone).

### `fn a_selection_of_named_inks_offers_no_swatch`

The single-object guard, unchanged by the selection size. This is the
case where a swatch's only possible effect is destruction, and it is the
reason the partial case above is safe to allow: the two are different
states and this test is what keeps them different.

### `fn a_spot_ink_does_not_make_the_process_colours_look_mixed`

If it were, one red line plus one PANTONE line would read as "mixed" —
which tells the operator that picking a colour will unify them, and it
will not. The honest reading is "red, and one ink I will leave alone".

### `fn section`

Returns `false` for a selection this section has nothing to say about — an
annotation, a form field, a selection with no path in it — rather than
drawing an empty heading. `geometry::section` states the same rule and for
the same reason: a heading with nothing under it reads as a control that
failed to load.

## Parts of a placed drawing

With no page object selected, the section reads the selected leaves of one
placed drawing (`strokestyle::selected`) and the action carries `leaves`, so
`EditSession::set_object_paint_in_form` rewrites the drawing's own stream.
The engine's reach disclosures (other placements of the same drawing) reach
the status line.

## Region and trace

`properties.paint` (`REGION_PAINT`); the swatches are `properties.paint.fill`
and `.stroke`, their pickers `<swatch>.picker`.

`paint-shown leaves= paths= fill= stroke=` is written on change. A channel
prints `r,g,b`, `mixed`, or `ink` when no swatch is offered.
