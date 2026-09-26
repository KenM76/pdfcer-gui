# `text::panels::properties` — the Properties panel

`RIBBON_IA.md` §5.8 commissions two surfaces for a selection's
properties, and is explicit about which is built first:

> The division of labour: the **tab** carries what a user changes *while
> working* — colour, width, style, align, delete. The **panel** carries
> everything, including the read-only facts (winding rule, node count,
> embedded-font status, exact geometry) that belong beside the Objects
> panel's inventory rather than in a ribbon band.
>
> Build order: **panel first, tab second.** The panel is the harder half
> and the tab's contents are a subset of it, so building the tab first
> would mean writing the property editors twice.

This is that panel's copy — **the read-only half of it**, which is all of
it at stage S3.

## What is deliberately absent, and why it is absent rather than greyed

§5.8 also says the panel is *"where the **editable geometry** lives — X,
Y, W, H as typed values"*, and calls that the surface through which
`/Rect` move-and-resize becomes reachable without a drag. **None of that
is here.**

Not because typed geometry is hard, but because there is nothing to edit:
[`crate::app::actions::Action`] carries zoom and page navigation and
nothing else, and the panel that would host the editors has no selection
to host them for. Four spinners bound to nothing would render, accept
typing, and discard it — which is not a placeholder in the harmless sense
but a control that silently loses an operator's work.

`RIBBON_IA.md` P3 states the rule this follows: *"An unavailable
capability renders nothing, not a disabled stub. Greying is reserved for
**temporarily** unavailable — no document open, document encrypted, undo
stack empty — and is always explained on hover."* "The selection model
does not exist" is not temporary unavailability; it is absence.

So the geometry is stated as **facts**, in the same field list as
everything else, and becomes editable when there is something to edit.

## The panel is the disclosure surface

Every `ObjectNote` an object carries is spelled out here in full, at the
foot of the field list. That placement is the disclosure rule's, not a
layout preference: inference reporting belongs **off-canvas** — *"a
status line, a results panel, a report after the command, a properties
field"* — and the page view must carry no badge, tint, dashed outline or
"provisional" layer at all.

The one-line test the rule offers: *would a screenshot of the editing
canvas differ from a screenshot of the same document saved and reopened?*
Nothing in this panel can make it differ, because nothing in this panel
draws on the page.

## Field wording lives next door

The *values* — kind names, paint dispositions, winding rules, colours,
font labels, note sentences — are all [`super::objects`]'s, and are
reached from here rather than re-worded. That is the same
single-description discipline
[`crate::panels::objects::summary`] exists to enforce, applied one layer
up: a path's fill colour must not be described one way in an Objects row
and another way in a Properties field.

This module owns only the **labels** — the left-hand column — and the
panel's own chrome.

## Item notes

### `fn no_field_label_carries_its_own_punctuation`

The colon is layout. Baking it into the string means a future
two-column or grid layout has to strip it back out of every entry,
and the one that gets missed renders as `Type::`.

### `fn every_field_label_is_distinct`

Two rows reading "Size" — one for the bounding box and one for the
image's samples — is exactly the confusion [`value_pixels`]'s "px vs
pt" comment is about, arriving through the label column instead of
the value column.

### `const ALL_FIELD_LABELS`

Hand-written, like every enumeration of things Rust cannot enumerate
for us. It is only used by tests, so an entry missed here weakens a
check rather than shipping a defect — but it is listed in the same
order as the panel draws them so a reader can diff the two.

### `fn geometry_values_keep_one_decimal_and_state_their_unit`

The decimal is not decoration: a horizontal rule is 0.0 pt tall and a
hairline is 0.5 pt tall, and rounding to whole points makes those the
same object.

### `fn image_samples_are_never_labelled_in_points`

The Size field a few rows above is in points and describes a
different thing. Two numbers of the same shape with the same unit
would read as one measurement stated twice.

### `fn the_embedded_font_answers_include_an_honest_dont_know`

The ambiguous one is the load-bearing case: a confidently wrong "Yes"
is indistinguishable from a right one, so the panel has to be able to
decline. It must not read like either of the definite answers, and it
must point at the surface that can be definite.

### `fn an_absent_value_says_so`

A blank field is indistinguishable from one pdfcer forgot to fill in,
and this panel's whole value is that its silences are as legible as
its numbers.

### `fn the_read_only_note_states_the_boundary_without_promising_a_control`

`RIBBON_IA.md` §5.8 specifies editable X/Y/W/H here, and it is not
built: there is no selection model and no mutating action to carry
the edit. The read-only note is the one string that says so, and a
well-meaning copy edit that turns it into "editing coming soon" would
make it a promise — which P3 forbids in prose exactly as it forbids
in a widget.

### `fn properties_object_heading`

Says **object**, and it matters — for a reason that has now changed twice
and is worth carrying rather than re-deriving.

It was written because `file.properties` had *two* scopes under one command
— `RIBBON_IA.md` §5.1 gave it the tooltip *"The document's own title,
author, subject and keywords, and the properties of whatever is selected on
the page"* — so an unheaded field list invited the reading "these are the
document's properties", which is exactly wrong for a fill colour and exactly
wrong in the other direction for `/Title`.

### `fn properties_nothing_focused`

The panel is never blanked: a blank region is indistinguishable from a
broken one, so the honest answer is a sentence naming the precondition —
and naming the surface that satisfies it, because the Objects panel is
the only route to this one at S3 and an operator has no way to guess
that.

### `fn properties_read_only_note`

**Shown once, at the top, and never repeated per field.** An operator
looking at a list of exact numbers with no input boxes will reasonably
wonder whether the boxes failed to draw; saying so costs one line and
removes the question.

It states the boundary without naming a future control (P3 again — a
promise is a placeholder made of prose).

### `fn properties_notes_heading`

A heading rather than an unlabelled run of paragraphs, because the
sentences are long and an operator scanning for a number needs to know
where the numbers stop. "Worth knowing" rather than "Warnings": every one
of these is a fact about the document, and warning styling would make a
property of the file read as a pdfcer failure.

### `fn field_colour`

"Colour", not "Fill" or "Stroke", because which of the two is showing
depends on the paint disposition — a stroke-only path never shows its
fill colour, so a field labelled "Fill" would name a colour that appears
nowhere on the page. The Paint field directly above says which it is.

### `fn value_position`

**PDF user space, y-UP, origin at the page's lower left** — the same
frame `pdfcer` prints and the same frame the object model stores. Not
the screen's y-down frame, and not adjusted for `/CropBox` or `/Rotate`.
An operator comparing this number against one from the CLI must get the
same number, and that is worth more than matching the direction their
mouse moves.

One decimal: enough to tell a 0.0-pt-tall rule from a 0.5-pt one, which
is precisely the distinction that makes a hairline look like nothing at
all.

### `fn value_index`

The `#` is not decoration: it is the form the Objects panel's row label
uses and the form `pdfcer object-list` prints, so an operator can
match a properties field against a row and against a command line without
translating. Formatting a number is a catalog decision for exactly this
reason — one place decides, and every surface inherits it.

### `fn value_line_width`

Two decimals, unlike the one [`value_size`] uses, and the difference is
deliberate: a line width is routinely 0.25 or 0.75 pt, and rounding to
one decimal makes a quarter-point hairline and a half-point one the same
number. A bounding box is never that fine.

### `fn value_size`

`×` rather than `x`, and one decimal for the same reason
[`value_position`] uses one. A zero on either axis is a real answer, not
a missing measurement — the note list below the fields says which shape
it is.

### `fn value_pixels`

"px" and never "pt": these are SAMPLES (§8.9.5, Table 89), and the Size
field a few rows above is in points. An image occupies the unit square
under the CTM, so the two numbers describe genuinely different things —
where it is, and what it is made of — and the pair is what lets an
operator judge effective resolution. They must not look alike.

### `fn value_not_stated`

One sentence fragment for every such field rather than a per-field
wording, because the answer is the same in every case and the *reason*
belongs in the note list rather than duplicated across four rows.

It is not a blank. A blank field is indistinguishable from a field pdfcer
forgot to fill in, and this panel's entire value is that its silences are
as legible as its numbers.

### `fn value_font_embedded_no`

States the consequence, not just the fact: a font the reader has to
supply is the difference between a file that prints as designed anywhere
and one that does so only on the machine it was made on. That is the
question an operator is actually asking when they look at this field.

### `fn value_font_embedded_ambiguous`

**The honest answer to a name-matching problem, and it is disclosed
rather than resolved.**

A text object records the `/BaseFont` in effect; the document's font
inventory records a program per font *dictionary*. Joining the two by
name is the only join available — the object model does not carry the
font dictionary's object id — and a name is not a key: one document can
declare two font dictionaries with the same `/BaseFont` (two independent
subsets of one face, which the survey behind the Fonts panel found in
87 % of embedding files), and they can differ in whether they embed.

So when the name matches more than one record, or none, pdfcer says it
could not tell rather than picking one. Picking would be an inference
presented as a fact, which is precisely what rule 4 exists to stop — and
unlike most inferences this one is invisible: a confidently wrong "Yes"
looks exactly like a right one.

The Fonts panel is where the per-dictionary truth lives, so this points
at it.

### `fn geometry_heading`

*"Position and size"* rather than *"Geometry"*: the second is the word a
draughtsman uses for the shape of the thing, and this section changes where
it is and how big it is. The standing rule in `text::commands` is that a
label is the operator's vocabulary.

### `fn geometry_units_note`

It names the corner as well as the unit, and that is the load-bearing
half. PDF's Y axis points **up**, so a panel showing `Y` without saying
which edge it measures is ambiguous in the one direction that matters — an
operator who reads it as a top edge and types a smaller number to move the
object up will watch it go down.

### `fn geometry_y`

*"Bottom"* rather than *"Y"*, for the reason [`geometry_units_note`] gives:
naming the edge makes the axis direction unmistakable at the point of use,
not just in a note the operator may have scrolled past.

### `fn geometry_angle`

*"Angle"* rather than *"Rotation"*, on the standing tie-breaker: the
operator's reference applications label the number on a shape's properties
panel *Angle*, and the convergence of the product class is the spec. It is
also the word he used — *"the angle should be editable from the
properties"*.

⚠ **Not written as a bare "dimension" anywhere near this**, per Rule 15.
This number is a property of a **markup annotation**, not of a ce dimension;
a ce dimension's orientation is part of its measurement and is turned by a
different verb entirely.

### `fn geometry_angle_note`

The direction is stated, and it has to be. PDF user space measures
anticlockwise from the positive x axis (§8.3.3), which is the mathematical
convention and the **opposite** of what a CAD operator reading a compass
bearing expects. A field labelled only *"Angle"* showing `30` is ambiguous
between two readings 60° apart, and the operator finds out which by typing a
number and watching the mark go the wrong way.

### `fn geometry_apply`

One button for up to two commands, and it does not say how many — *"Apply"*
is what the operator is doing; *"raise a move and a scale"* is what the
program is doing, and `RIBBON_IA.md` §2's rule is that a control is named
for the first.

### `fn geometry_nothing_typed`

R9 reserves greying for *temporarily* unavailable and requires the reason on
hover. This is the ordinary case — the section has just drawn, the fields
hold the object's current numbers, and there is nothing to do until one of
them changes.

### `fn geometry_too_small`

It says what the floor IS rather than only that one was hit, because
*"too small"* leaves the operator guessing at a threshold, and the whole
point of a typed field is that they can hit an exact number.

### `fn text_heading`

*"This text"* rather than *"Font"*, matching [`markup_heading`]'s *"This
markup"*. The panel can show several sections at once and the operator has
to be able to tell which selection each is about; a section headed with the
name of a *property* would read as a category, not as a subject.

### `fn text_covers`

It says *"pieces of text"* rather than *"runs"*. A run is a show operator,
which is a fact about the file's structure that no operator asked to learn;
what they need to know is that their one press will change more than one
thing, and how many.

### `fn text_unreadable`

It draws the heading and this sentence rather than drawing nothing,
deliberately. An operator with text selected who saw the section vanish
would conclude the feature is missing; an operator who sees it say why is
told the truth about one selection.

### `fn text_value_absent`

The Font group's size field is an `egui::DragValue` over the shared read-back
draft. With nothing swept the draft holds its `Default` — zero — and the
widget's own `range(1.0..=1440.0)` clamps that up, so the greyed control
rendered **`1.0 pt`**. The driven check saw a region at the right place and
passed, correctly: it was asserting that the control is drawn, and it was.

A greyed control showing a **false value** is worse than one showing
none. Greyed says *"not right now"*; `1.0 pt` says *"this text is one point
tall"*, which is a claim about the operator's document and it is wrong. The
same argument the Properties panel's `text_colour_not_plain` makes about a
converted swatch: a control that shows an approximation invites a press that
writes it back.

An em dash, and the convention is the reason. Word leaves its font-size
box **blank** with nothing selected; every property grid in this class —
Acrobat, SolidWorks, Figma — shows a blank or a dash for *no value* and for
*mixed values*, which are the same state as far as a single field is
concerned. A dash is chosen over a blank because an empty framed control on
a ribbon reads as a rendering fault, and because it is what the operator's
own tools do.

### `fn text_bold_hint`

It promises the *outcome* and names the fallback, because the fallback is
the thing the operator would otherwise discover as a surprise. Both routes
are honest: a page carrying a real Bold gets the real face, and one that does
not gets a thickened version of what is there. Neither is greyed, because
between pdfcer's two verbs every page is covered.

### `fn text_colour_not_plain`

The sentence protects the operator's ink. A swatch showing DeviceCMYK as
its nearest RGB would write that RGB back on the next press, moving the run
out of its original space for ever on a document heading for a printer that
cares. pdfcer deliberately stores the space it was given rather than
force-converting the way Acrobat does, and this control must not undo that.

### `fn text_bold_hint_already`

`StyleRung::AlreadyStyled` — the run's own face already claims the weight,
so the press is a no-op rather than a change. Said plainly and without a
warning tone: pressing it costs nothing and does nothing, which is what a
toggle showing a state it is already in should say.

### `fn text_bold_hint_sibling_face`

Rung 1 with `StyleLadder::same_family == Some(true)`. The best outcome the
ladder has: nothing is added to the file, nothing is embedded, and the
letterforms are the ones the document already uses.

It names the face *and* the relationship. *"pdfcer will use Arial-Bold"*
is checkable; *"the bold form of this text's own typeface"* is the part that
tells the operator the result will look like the rest of their drawing. The
shell does not work that relationship out — `same_family` is the engine's
verdict, and engine invariant R74 forbids re-deriving it here.

### `fn text_italic_hint_sibling_face`

*"Slant"*, not *"thicken"* — the two synthetic operations are different
and an operator who has read one sentence should not have to guess that the
other means something else.

### `fn text_bold_hint_other_family`

Rung 1 with `StyleLadder::same_family == Some(false)`. Still a real face,
still nothing embedded — but the letterforms will not match the rest of the
run, and that is a visible change the operator should be able to expect
rather than discover.

It says *"the letters will be shaped differently"*, which is the thing
that distinguishes this from the sibling case. Both sentences would
otherwise read *"pdfcer will use a real bold face"* and the operator would
have no way to tell from the hover which of two quite different results is
coming.

### `fn text_bold_hint_standard_sibling`

Rung 2: the standard-14 sibling of the run's own family, bound as a new
`/Font` resource with **no font file embedded** (ISO 32000-1 §9.6.2.2 —
every conforming reader is required to have these fourteen). This is the
rung the old `preview_style_resolution` hover could not see at all, and it
is the one that fires on the commonest CAD page there is: a title block set
in `Helvetica` carrying no bold resource.

*"the file does not grow"* is in the sentence deliberately. The
operator's standing worry about font work is what it does to a drawing they
have to email, and a rung that adds a resource but not a font program is
exactly the reassurance that worry wants — and it is true, which is the
only reason it is here.

### `fn text_bold_hint_synthetic`

Rung 4, the last rung: no real face anywhere on the ladder could show this
run, so pdfcer strokes the regular face. R90 makes that declinable rather
than a preference, which is why the sentence says what *will* happen rather
than merely offering to do it.


From 2026-08-29 to 2026-09-11 this read *"… pdfcer will use a real bold
typeface if it can find one and thicken the letters if it cannot — and it
will tell you which it did."* That hedge was correct and unavoidable: the
shell was previewing the **R90 gate**, which cannot see rung 2, so it knew
the gate had found nothing and did not know what the ladder would do next.
The doc comment of the day argued at length that predicting the rung needed
an instrument that did not exist.

⇒ It exists now. `preview_style_ladder` returns the rung the commit will
land on, so this sentence is only ever shown when the answer is **rung 4**,
and a hedge that offers a possibility the preview has already ruled out is
worse than the conditional it replaced. Both halves of that argument are
recorded because the hedge was right when it was written; the fix was a new
measurement, not better wording.

### `fn text_bold_hint_declined`

`FormatError::SynthesisRefusedByPosture` — the ladder reached rung 4, and
`StylePolicy::Refuse` is set. The refusal is not a defect and not a limit:
it is the setting working, and the sentence says which setting so the
operator can change it in one move if this is the run they want it for.

It names the setting rather than describing it, because a hover that says
*"your settings prevent this"* sends the operator hunting through a
preferences dialog for a phrase that may not be there. Matching the words on
the control is the difference between a disclosure and a riddle.

### `fn text_hint_faces_tried`

`StyleLadder::passed_over` — each entry a face that claimed the style and
could not show this run's characters. Written as an addendum rather than
folded into the seven sentences because it is orthogonal to all of them: a
ladder can pass over faces on its way to any rung, including the one that
ends in a refusal.

# The character, and why it earns its own parenthesis

The engine's own `reason` string is accurate and technical —
*"R-INV-1: character U+006F 'o' has no code in font 'Times-Bold'"*. On a
hover the operator wants **no 'o'**, which is the same fact in the form that
answers *why not*. `Refusal::character` is the engine handing that over as a
field, so this is a reformatting of an engine answer rather than a parse of
its prose — the distinction decision 058 turns on, and the reason
`PassedOver` was asked for as a struct instead of a `Vec<String>`.

A face with no character named gets no parenthesis rather than an empty
one. `Refusal::character` is `Option`, and a refusal about the whole run
rather than one glyph is a real case; *"Times-Bold ()"* would be this shell
rendering an absence as a presence.

Leading space, and it is not an oversight. The clause is pushed onto a
sentence that already ends in a full stop, and owning the separator here is
what keeps the call sites from each getting it right independently.
