# `text::tool` — the words the tools say, wherever they are said

## This file OUTLIVED the panel it was written for


| what | who says it now |
|---|---|
| the per-tool instructions and live stages | [`crate::app::toolstatus`] — the right dock's permanent one-line strip |
| the second sentence of the stages that had one | the same strip, in its hover |
| the text pen's labels, the measure pick list, the resize switches | [`crate::panels::properties::tool`] |
| the disclosure heading | [`crate::panels::properties::disclose`] |

⚠ **What was DELETED, and it is the only deletion**: the fifteen strings the
panel's tool LIST used — `tools_heading`, `tools_hint`, `row_home` and the
nine `row_*` sentences, plus `pointer_heading`, `armed_heading` and
`no_document`. Every one of them labelled a button that duplicated a ribbon
control, and the operator's instruction was *"its buttons duplicate the
ribbon and go."* They are gone rather than left orphaned, because an unused
catalog entry is a sentence nobody can find and nobody can retire.

Worth naming what that cost: those rows were the answer to a
discoverability defect — *"The feature works. He could not find it."* The
strip that replaced them cannot list what is NOT armed. That is a real
subtraction and it is the operator's own call; it is recorded in
`crate::app::toolstatus`'s header rather than argued here.

## The three rules the whole file follows, unchanged

**1. No label is written here that the command registry already owns.**
The armed tool's name comes from `CommandRegistry` through
`crate::shell::menus::MenuHost::label`, and the chord comes from the
operator's own keymap. A second copy of a label compiles, reads identically
the day it is written, and drifts the first time either is reworded —
invisibly, because nothing renders both at once. `NO_SURFACE.md` §1 records
that exact failure with a colour.

**2. Every sentence states a fact about the program, never a tip.** The
operator's own report about the shell this replaces: *"the nagging and red
flagging in the original GUI made for a lot of extra bugs in the visibility
when editing."* *"Drag marquees objects on this page"* is a statement.
*"Try dragging to select several objects!"* is a tip, and there are none
here.

**3. An instruction says how the gesture ENDS.** Half the gestures in this
application do not end by themselves — a run of clicks does not, a text
caret does not — and *"click each corner"* is not a complete instruction
because nothing in it says when to stop. Every instruction below that
describes an open-ended gesture names its ending.

## Item notes

### `fn every_markup_instruction_says_how_the_gesture_ends`

The second half is the assertion worth having. *"Click each corner"*
is not a complete instruction — nothing in it says when to stop — and
the failure it produces is an operator clicking forever, which is
exactly what the two endings exist to prevent. Asserted as a property
(the sentence names a release, a double-click or a stop) rather than
against the literals, which would pass just as well if every kind
returned the same string.

### `fn pointer_edit`

**This sentence exists nowhere else in the application**, which is the
whole reason the unarmed panel is not a placeholder. The identical drag
means *marquee objects* here and *sweep text* in Read, decided by
`canvas::textsel::takes_the_press` reading the mode — and no surface has
ever said so. An operator who wonders why dragging behaves differently in
two modes has had no way to find out but to guess.

### `fn put_down_button`

Named for what it does to the **tool**, never "Close" — the dock tab's ✕
closes the panel and this does not, and two controls a click apart that both
read as closing something is how an operator loses a surface they wanted.

### `fn node_instruction`

Written as the two gestures in the order an operator performs them, and
naming the *thing* rather than the rung. "Anchor" is what a draughtsman
calls the point; `SelectionLevel::Node` is what this program calls the
state, and the panel speaks the first vocabulary — `text::commands`' rule
that a label is the operator's word and an id is the format's.


This tool has **two** subjects, not one — see
`canvas::tool::retire_forbidden`'s Node arm for the table. The first is an
anchor of a path on the page and needs `edit_content`; the second is a
corner of a measurement pdfcer authored, needs `author_measure`, and is
therefore the tool's only subject in Review.

The sentence names both in every mode rather than being swapped by
capability, and that is a decision with a cost attached. The precedent for
swapping is [`text_select_takes_the_press`], which is rendered only where it
is true — but the fact *that* sentence states is a **change** ("arming this
takes the press away from…"), which is either true or false. This one states
**where to aim**, and an operator in Edit who is shown only the anchor
sentence would never learn that his measurements have corners at all. R9
governs drawing a control that does nothing; naming a second subject that
exists in the mode the operator is in is the opposite of that.

⇒ The wording therefore says *a measurement you have drawn*, which is true
in both modes and locates the subject without a mode word in it.



⇒ The primary answer to O188(A) is a **right-click row**
(`format.select_text_line`), because it needs no tool armed in advance
and no prior knowledge at all. This clause is the second half, and it is
owed on the same argument the paragraph above makes for measurements: an
operator shown only two of a tool's three subjects never learns the
third exists.

*picks out just the line you clicked* rather than *selects the run*.
**Run** is the format's word and appears in the Objects panel, where a
reader is already looking at a tree of format objects; this strip speaks
the operator's, and his word for it — verbatim — is *line*.

### `fn node_shift`

## This is where the two chords are spelled, and it is the ONLY place

`canvas::dimdrag` gates adding and removing a corner on this tool being
armed — deliberately, so that a stray Ctrl during an ordinary corner drag
cannot destroy a corner. The cost of that safety is that the gesture is
invisible to anyone who has not armed the tool, and the payment is this
line: arm the tool, and the sentence under it tells you what the modifiers
do.

It is a **stopgap and is recorded as one.** The discoverable form of
these two verbs is a right-click menu on the shape — *"add a point here"*,
*"remove this point"* — which is the shape `pdfcer-core`'s own doc comments
on `insert_dimension_vertex` and `remove_dimension_vertex` describe them
in. That surface is
`canvas::menus`, and the session that built these two verbs did not own that
file. Reported rather than half-built; see `OPERATOR_REQUESTS.md` O132.

### `fn text_select_takes_the_press`

Rendered only in a mode that can select page content. Everywhere else the
select tool already swept text, so there is nothing to disclose and the line
is **absent** rather than reworded — R9's rule applied to a sentence.

### `fn markup_instruction`

These came from `MarkupKind`'s own variant doc comments, where they had
been written — correctly, and in the operator's words — since the day each
kind landed, with no surface able to render them. Moving them here is what
`check-ui-strings.sh` requires and is also what makes them reachable.

### `fn vertices_placed`

**The number is why this panel exists rather than a canvas readout.** A
rubber band and a snap indicator are the cursor and are welcome; a *number*
floated near the pointer would be pdfcer putting a surface over the drawing
on its own initiative, which `MODES_AND_PANELS.md` sets to **never**. So the
count has exactly one legal home, and it is a real need: a polygon and a
revision cloud both refuse at two corners, and an operator who double-clicks
one click early gets silence.

### `fn text_annot_release`

The distinction `CanvasTool` was split for: *"A markup band authors on
release, from geometry alone. These cannot: releasing produces an empty box,
and an empty box is not an annotation."* An operator who does not know that
reads a release-that-authors-nothing as a broken tool — which is the same
failure shape as the text-editing complaint that produced this panel.

### `fn refusal_heading`

The refusal sentences themselves are `crate::text::textedit::refusal`'s and
are **not** duplicated here. They were written well, are tested, and have
never had a surface wide enough to show them: their own module records that
they were aimed at the status bar, and that *"it shares the status row with
everything else and R128 forbids that row growing."* A dock panel's width is
the dock's, decided before the body draws, so that constraint does not apply
here at all.

This is very likely the actual cause of *"no text editing or adding text on
the canvas"*: on a dense CAD sheet the first click lands where the operator
wants text rather than where text is, the tool declines with an explanation
nobody could read, and they conclude the feature does not exist.

### `fn measure_perimeter_live`

# Why the count is in it as well as the length

Because the two answer different worries. The length says *"this is what I
have measured"*; the count says *"this is how much of the shape I have
traced"*, which is the one an operator loses track of on a footprint with
twenty corners - and the one that tells them whether a click registered at
all. A tool with no fixed arity has nothing else on screen that says so.

The length is formatted by the caller through the engine's own
`format_measurement`, so this function never sees a number it could round
differently from the committed label.

A verb rather than a bare pair of numbers: this replaces the instruction
once tracing starts, and a line reading only "4 - 12.40 m" gives an operator
who has looked away nothing to reattach to.

### `fn measure_circular_live`

# Why the SIZE is in it, and why that is the whole ask

`OPERATOR_REQUESTS.md` O105: *"selecting more points around a hole doesn't
always get it to narrow down to the size of the hole."* An operator adding
points to a fit is watching a number converge, and until 2026-09-03 there
was no number to watch — the fitted circle was drawn on the canvas and its
value appeared only once the dimension had been placed. So the tool could
not be steered: every correction was a commit-and-undo.

The count is in it for the reason it is in the perimeter's sentence — it is
the only thing on screen that says *whether the last click registered at
all*, which for a tool with no fixed arity nothing else answers.

The measurement is formatted by the caller through the engine's own
`format_measurement`, so this function never sees a number it could round
differently from the committed label.

### `fn measure_circular_needs_more`

It states the count AND what is missing, because "nothing yet" is exactly
the report that sent the operator looking for a broken tool. Two points on
an arc is not a failure, it is halfway.

### `fn measure_point_row`

# Why the ORIGIN is on the row

Because a point snapped to the drawing's own geometry and a point the
operator placed by eye on a scanned image produce the same numbers and are
**not** the same evidence. `OPERATOR_REQUESTS.md` O106 asks for the second
deliberately — it is what makes a bitmap measurable — and the honest
consequence is that the operator can see which of their points are which.

That disclosure is here and **not on the canvas**, and the placement is
the rule rather than a preference. Rule 4: applied content renders exactly
as saved content will, and a tint or a dashed marker saying *"this one is a
guess"* would be pdfcer marking its own uncertainty into the page view. The
list is off-canvas, non-blocking, and positioned relative to nothing in the
document — which is where a disclosure belongs.

The coordinates are page units to one decimal. Not the group's scale: these
are *positions*, not a measurement, and running them through
`format_measurement` would print a length unit beside something that is not
a length.

### `fn measure_point_remove_hint`

A row that removes on a single click needs to say so before it is pressed,
because the gesture is not recoverable through undo — a pick set is
pre-commit state and never enters the document's history. One click to put
it back is the whole cost, and the tooltip says which click.

### `fn measure_points_empty`

Not a placeholder row and not a greyed one: R9 reserves greying for a
*temporarily* unavailable control, and an empty set is not that. This is a
sentence, and it is the only thing this section renders until there is
something to list.

### `fn measure_point_origin`

# Why this is a separate vocabulary from the trace's

`canvas::measure::circular::origin_tag` produces short machine tags that a
driven check matches on. Those are a contract with `tools/ui-verify` and must
not move when a word is reworded here; these are what the operator reads and
must be free to. One function serving both would tie a harness assertion to
a translatable string.

**Free position** is worded as a statement of fact rather than as a warning.
It is a legitimate and often the only available pick — see
`OPERATOR_REQUESTS.md` O106 — and language like *"unsnapped"* or
*"approximate"* would be pdfcer editorialising about a choice the operator
made deliberately.

### `fn draw_into_label`

**Read-only here, and the button beside it is a route rather than a
picker.** A second group picker would be two copies of the one control that
decides where every ce dimension goes, which is precisely the duplication
this project has already been bitten by. The panel that owns it is one click
away and is the only place it can be changed.

### `fn scale_heading`

*"When you resize something"*, not *"Scaling"* or *"Transform options"*.
It names the **gesture** these modify, because that is how the operator will
arrive: they have just dragged a grip and something did or did not come with
it. A noun heading would be correct and would not connect to anything they
did.

### `fn scale_stroke_label`

**His own vocabulary.** `OPERATOR_REQUESTS.md` O51 says *"scaling line
weight, etc with resize"* — *line weight*, which is the drafting term and
the one on every CAD program's layer table. The PDF calls it a border width
and Inkscape calls it a stroke width; neither is what he said.

### `fn scale_insets_label`

**Phrased as KEEPING, matching the field it sets.** `/RD` scales by
default, so the switch is an opt-out, and `canvas::scaling` spells it that
way deliberately so `Default::default()` is correct in every field.

⇒ A label reading *"Scale the inner margins"* would read better and would
put an inversion between the words and the value. That is the single easiest
way to ship a control that does the opposite of what it says, and no test
catches it — both states are legal and both produce a plausible picture.

*"inner margins"* rather than *"rect differences"*: `/RD` is the gap
between an annotation's rectangle and the drawing inside it, which is a
margin, and nobody outside the specification says *rect difference*.

### `fn scale_distort_label`

**It says the result will be uneven, in the label itself.** This is the
one switch whose ON state makes the output worse, and O51's ruling on it is
explicit: proceed and **state** the residual distortion, *"never silently
pick a fudge factor, which is the one thing the parity reference does."*
A label reading *"Allow non-uniform resize"* would be true, neutral, and
would hide the cost inside a word the operator does not have to decode.

### `fn scale_note`

It states the default in the operator's terms and names **why** the line
weight stays put — *a drafting standard* — because on his documents that is
not a preference, it is a convention his drawings are read against. Without
the reason, "off by default" reads as an arbitrary choice somebody made.

It also says the switches apply to the **next** resize, which is the fact
that makes them a per-drag modifier rather than a setting, and the fact an
operator needs in order to use them at all: tick, then drag.

### `fn text_pen_heading`

Says **new text**, not "text", and the distinction is the whole reason
these controls are in the Tool panel rather than on the Format tab: they
decide what the *next* thing typed looks like, not what a run already on the
page looks like. An operator who reads "Text" here and expects it to restyle
the word they clicked has been misled by one word.

### `fn text_pen_font_name`

*"Helvetica Bold"*, not `HelveticaBold` — the engine's identifier is a
Rust variant and this is a font menu. The four Courier faces say
*"Courier Oblique"* rather than *"Courier Italic"*, because oblique is what
the Standard-14 set actually contains and a menu that renamed it would
promise a true italic pdfcer cannot write.

### `fn text_pen_note`

It says what they DO NOT do, because that is the thing an operator will
otherwise assume: these set the next run's appearance and change nothing
already on the page. `Edit text` beside them replaces the words in a run and
keeps its existing face — pdfcer cannot restyle a placed run at all yet, and
a control group that stayed silent about that would be read as offering it.

### `fn disclosures_heading`

Rendered only when there is something under it. R9: an unavailable
capability renders **nothing**, and a heading over an empty region is the
placeholder that rule exists to forbid.

### `fn form_instruction`

It names BOTH gestures, and that is the point of the line. The whole
feature is *"click to place, or drag for the exact size"*, and an operator
who clicks once, gets a standard-sized box and is never told about dragging
will conclude that sizing is not offered. A panel that teaches only the
gesture just performed teaches half the tool.

### `fn form_kind_hint`

Radio buttons get their own sentence because they are the only kind whose
behaviour depends on ANOTHER field — two sharing a group name are one
control. An operator who does not know that places two buttons that both
stay on and reasonably reports it as a bug.

### `fn node_tool_needs_edit_mode`

`OPERATOR_REQUESTS.md` row **O69**, the operator:

> *"I'm still not entirely clear how to reliably get to a point where I can
> edit nodes."*

One of the two reasons it felt unreliable. The Points tool needs
`Capabilities::edit_content`, which only Edit has, and its dispatch arm
declined into the trace and said nothing on screen. The ribbon item is now
withheld outside Edit (`shell::manifest::view`), so the only way left to
reach the decline is the bare `A` chord — a chord is filtered by TAB
visibility, not by item visibility, and View is in every mode.

**So this sentence exists for a route the ribbon can no longer produce**,
and that is deliberate rather than belt-and-braces. R83's rule is *a
refusal must be a sentence, never a silence*, and a chord that does nothing
is the worst kind of silence: there is no control to look at, so there is
nowhere for the operator to discover why.

# Why it names the remedy rather than the rule

*"Editing points needs Edit mode"* would be a true statement of the rule
and useless at the moment it is read — the operator pressed a key and
nothing happened, and what they need is the next act, not a diagnosis.
This names the control that fixes it, in the words printed on it, which is
the same choice `resize_not_rebuildable` makes and for the same reason.

# Why it is here and not in `crate::text::status`

`text::status` is at 1,482 lines against R2's 1,500, and its own header
records the seam being noticed rather than trimmed. This module already
owns the tool panel's sentences, and this sentence is about a tool.


The gate above it changed. `retire_forbidden`'s Node arm and
`app::dispatch::navigate`'s `view.tool_node` arm both read
`edit_content || author_measure` now, because the tool's second subject is a
ce dimension's corners and reshaping one is a **measure** edit. So this
sentence is reachable in **Read alone**, where nothing is editable, and
naming Edit is the correct next act.

⚠ It was **actively wrong in Review**, which is the state that made the
correction urgent rather than tidy. Review is the mode a measurement is
drawn in; an operator there who pressed `A` was told to switch to Edit —
and switching would not have helped him, because what he was reaching for
was either a corner he already had (his own measurement, draggable in
Review all along) or a markup shape's points, which **Edit cannot edit
either**: `pdfcer-core` models no `/Vertices` or `/InkList`, filed as
`request_a_markup_shapes_vertices_cannot_be_read_or_edited.md`. A refusal
that names a remedy which does not work is worse than one that names none,
because it spends the operator's time before it fails.
