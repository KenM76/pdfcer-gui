# `panels::properties::textobject` — the colour of the text you CLICKED

`OPERATOR_REQUESTS.md` **O89**, piece 1:

> *"I don't see where I am able to edit the color of text, vectors, etc."*

## What was wrong, in one paragraph

Text colour shipped in two places — Format ▸ Font ▸ Colour, and Properties ▸
Text — and **both were gated on a swept text range**. Clicking a piece of
text with the Select tool selects the *object*, so both controls stayed
greyed, and the way to un-grey them (press `T`, sweep across the words) is
not guessable from anything on screen. The capability shipped; the way in
did not. This project's standing reading of that shape is that a request for
something already shipped is a **discoverability report**.

O89 listed three candidates and picked none. This module is the first —
*"a colour control on a selected text object that sweeps it for you"*, which
O89 itself called *"closest to what you tried"*.

**The other two were already built**, which was measured rather than
assumed and is recorded in O89 in place of the sentences that said
otherwise. The Properties panel's *"press T and sweep"* sentence existed
from 2026-08-29 until 2026-09-14, when O198 made the route it named
unnecessary and it was deleted; and every one of the five Font commands'
tooltips has ended
*"Sweeping text with the Text tool (T) chooses what it applies to"* since
the group shipped. What was NOT true was O89's third row — *"the greyed
button saying so on hover"* — for exactly **one** of the five controls: the
ribbon's Colour swatch answered a greyed hover with the CMYK-and-spot-ink
sentence, a claim about text it had not read. Fixed in `app::fontband`.

## THE OPERAND IS THE OBJECT'S OWN BYTE SPAN, NOT A GUESS AT GEOMETRY

[`super::text`]'s header states, correctly, that the object selection and
the text selection are unrelated index spaces and that an inference between
them *"restyles text the operator did not select, silently, in a file they
then send to somebody."* That paragraph was written about a **bounding-box
overlap**, and it is still right about one.

This is not one. `pdfcer_core::vector::TextObject` carries the `BT`…`ET`
**byte span** in the decoded content buffer, and every glyph's
`GlyphProvenance` carries its show operator's byte span *in the same
buffer*, with the buffer named beside it. Membership is byte-range
containment: exact, total, and the same kind of fact the pinned edit path
already stakes every restyle on. `crate::canvas::textedit::pin::object_text`
is the join and its header carries the argument.

⇒ **No new selection unit was invented.** The operand handed to the engine
is a list of extraction run ordinals — the same operand a hand sweep
produces, through the same `Action::TextStyle`, into the same
`EditSession::format_text` calls. This module chooses *which* runs; it
changes nothing about what a restyle is.

## Why the range and not the exact set — "sweeps it for you", literally

The operand is `first_run..=last_run`, which is precisely what
`TextSelection::runs` produces for a hand sweep (`(start.run..=end.run)`).
Taking the inclusive range rather than the exact membership set makes the
two routes **the same gesture with the same operand**, rather than two
things that usually agree and diverge on a document nobody tested. They can
differ only where another object's show operators interleave inside this
one's `BT`…`ET`, which §9.4's grammar forbids.

## Why COLOUR is the only control still drawn here


⇒ **The premise was true and the conclusion did not follow.** "No single
answer" is an argument about what a control may *display*, and this
project's own product class answers it with an indeterminate presentation
rather than by withholding the control. It is not an argument about what a
press may *apply*: `format_text` takes a run list, the object's run list
is exact, and applying one face to nine runs is precisely what a hand
sweep across the same words does. The operator, 2026-09-14: *"the
properties area is uneditable too. This is true even when I add a new line
of text."*

So [`super::text::section`] now draws face, size, bold and italic for a
clicked object as well as for a sweep, reading the first run for its
read-back, and the route sentence is **deleted** rather than re-aimed —
every clause of it had become false.

**Colour stays here, and only colour.** It is the one property whose
disagreement this shell must act on rather than merely render: a run
painted in a `/Separation` gets **no swatch at all**, which [`Colour`]
decides by looking at every run in the object. `super::text`'s own colour
row reads the FIRST run and would report a nine-run object's ink from one
of them — so that section draws its colour row only for a swept operand
and defers to this one for an object. Exactly one Colour control is on
screen in any frame.

## The spot-ink guard survives the object route

`pdfcer_core::text_extract::TextColor::Other` means *"set in a colour space
this extraction does not decode"* — a `/Separation`, a `/DeviceN`, an
`/ICCBased`. O89's ruling for paths applies here word for word: *"a colour
picker that opened on black over a spot ink would be one click from
destroying a plate, and it would look completely normal while it
happened."*

So an object **any** of whose runs is painted in a space
[`super::text::rgb_of`] will not round-trip gets **no swatch at all** — the
sentence [`t::ink_present`] stands where it would have been, and it names
how many of how many runs are affected. Two properties of that rule matter:

* It is **all-or-nothing for the object**, because the operand is the
  object. A partial apply would need the operator to have asked for a
  partial thing, and they asked for *this shape*.
* The refusal is decided by the **same function** the swept-text swatch uses
  to decide whether to show a colour at all, so the two surfaces cannot
  disagree about which spaces are safe. That is why `rgb_of` was widened to
  `pub(super)` rather than copied.

Note what this does NOT do: it does not name the ink.
`TextColor::Other` is a fieldless variant and carries no `/Separation` name,
unlike `PathPaint::Other`. [`t::ink_present`]'s doc comment carries that
distinction; a sentence naming a spot colour here would be invented.

## The canvas selection is NOT changed by a press, and that is on purpose

An early design had this control set `doc.text_selection` to the object's
runs after applying, so the operator would end the gesture with the words
visibly swept and the whole Font group live — teaching the route by doing
it. It is not built, for a reason found by reading the existing code rather
than by taste: **a restyle bumps `edit_epoch`, and a `TextSelection` records
the epoch it was resolved against**, so the selection would be stale on the
very next frame and the group would grey itself immediately. That is already
what happens after a swept-text restyle, and `app::conditions`' note on
`selection.text` argues it is the honest behaviour. Producing a selection
that is dead on arrival would have looked like a bug in the feature.

## The cost, and where it is paid

[`TextObjectDraft::sync`] runs one page extraction with provenance capture
on — **392 ms on the operator's benchmark sheet** — behind a
`(page, object, edit epoch)` stamp. It is therefore paid **once per object
the operator clicks**, and only while this section is actually drawn: a
docked pane behind another tab draws nothing, so a panel the operator is not
looking at costs nothing at all.

That is the same trade [`super::text::TextStyleDraft`] already makes on
every text-selection change, for the same measured reason, and it is stated
here rather than discovered: the alternative — matching the object's box
against the runs' boxes, which is free — is the geometric inference the
header above refuses.

## Rule 4

Nothing here marks the canvas. The recoloured text renders exactly as the
saved file will render it; what was skipped, what disagreed and what is a
named ink is disclosed **off-canvas**, in this panel and in the status bar
through `app::actions::disclosure`.

## Item notes

### `fn sync`

The expensive call is behind the stamp comparison and nothing else, so
the ordinary frame — the operator looking at a selection they made three
seconds ago — costs one tuple comparison.

### `fn classify`

Its own function, and every branch is a decision O89 argued:

1. **A glyphless run is not a colour.** It has no show operator, so
   `textstyle::apply` skips it; counting it would let a derived word space
   make an object "mixed".
2. **An ink pdfcer will not convert wins over everything.** It is checked
   before agreement, not after, because *"they all agree and one of them is a
   spot ink"* must not draw a swatch. The check is
   [`super::text::rgb_of`] — the same predicate the swept-text swatch uses —
   so the two surfaces cannot disagree about which spaces are safe.
3. **`DefaultBlack` is black**, not "no opinion". §8.6.8 says an absent
   colour operator paints black, so an object of one red run and one
   default-black run is genuinely **mixed** and must say so. See
   [`RunFill`]'s own docs for the flattening that collapsing this would
   cause.

### `fn one_undecodable_run_removes_the_swatch_for_the_whole_object`

The assertion this module exists for. If the ink check ran *after* the
agreement check, an object of nine black runs and one `/Separation` run
would draw a black swatch, and one click would convert the plate colour
— invisibly, permanently, looking entirely normal.

### `fn a_default_black_run_disagrees_with_a_coloured_one`

The [`RunFill`] distinction, asserted where it is consumed. Written as
its own test because the failure it guards has no symptom: the control
would open on red, and pressing nothing would change nothing, so only a
deliberate check can see it.
