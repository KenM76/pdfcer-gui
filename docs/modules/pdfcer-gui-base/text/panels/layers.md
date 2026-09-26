# `text::panels::layers`

`text::panels::layers` — **every sentence pdfcer says about which layer a
selection is on**, in one module.

# Why one module, when the sentences appear on two surfaces

The answer is shown in two places and they are deliberately not the same
width:

| surface | form | why it exists |
|---|---|---|
| the **Layers panel**, under the count | the long form, a whole sentence | the panel is where the operator went to ask |
| the **status bar**, appended to the selection line | a short clause | **the canvas is the primary surface, never a panel** — clicking the object must reach the answer with no panel open |

Two surfaces saying one thing is `DEFECTS.md` D5's shape exactly: *the same
concept implemented in two places, and the copies drift.* The drift here
would be invisible and expensive — a bar reading *"not on a layer"* beside
a panel reading *"pdfcer could not tell"* is a contradiction the operator
can only resolve by deciding the program is broken.

⇒ So both forms are generated **here**, from the same
[`Membership`](crate::panels::layers::highlight::Membership), by two
functions whose matches are exhaustive. A new state cannot be added to that
enum without both of these failing to compile, which is the mechanism this
project keeps re-learning it needs: **a hand-written list inside a
completeness sweep is not a sweep.** `check-ui-strings.sh` would have said
nothing about a missing arm; the compiler says everything.

# The register: a measurement, never a hedge

*"This may be part of a larger object"* is a disclaimer, and a disclaimer
printed on every selection teaches the operator to skip the line. *"This
object holds 1,194 parts"* is a number he can act on, and it is silent on
the object that holds one. Every sentence below is written to that rule —
see [`layer_selection_granularity`], which is the one that carries his own
finding back to him.

# Rule 4 lives in what these sentences are FOR

None of this is drawn on the drawing. An inference the operator cannot see
— an unresolvable `/OC` section, a group the document never listed, an
object nested deeper than the leaf list can see through — still owes a
report, and this is where that report is worded. **Render normally; report
separately. Both.**

## Item notes

### `fn unresolved_long`

Separate from [`layer_selection_report`] so the reasons can be read as a
set — they are meant to be *different* from one another, and a reader
checking that has them in one place rather than spread through a match with
four other arms.

### `fn unresolved_short`

Every one of these is a **cause**, not an apology. *"layer not known"*
alone would be the hedge this catalog's header forbids; the clause after
the dash is what tells the operator whether to look at their file, their
selection, or pdfcer.

### `fn every_state`

The `match` beneath it is the mechanism: adding a variant to
`Membership` makes this file fail to compile, which is what a
hand-written array in a completeness test cannot do. This project has
shipped four defects into exactly that gap (`RESUME.md`, three
separate recurrences), and the fix each time was to make the compiler
hold the list.

### `fn a_group_whose_row_is_not_on_screen_is_reported_in_words`

The failure this forbids is silent and reads as a broken feature: the
operator types in the search, the matching layer is narrowed out, the
plate goes with it, and the panel looks exactly as it would if
selecting an object highlighted nothing at all.

### `fn an_unnamed_group_gets_its_own_words`

`on layer ""` is the shape of a placeholder, and R9 forbids one. The
unnamed case is a different sentence, not the same sentence with a hole
in it.

### `enum RowOfAnswer`

# Why this is not simply `Option<&str>`

Because a highlight that lands on a row nobody can see is
indistinguishable from no highlight at all, and this panel has **two**
independent ways for that to happen:

| | what the operator sees | what they conclude |
|---|---|---|
| the row is on screen and plated | the answer | correct |
| the row exists but the **search** has filtered it out | nothing | *"selecting an object does not highlight the layer"* |
| there is **no row** — an OCMD (§8.11.2.2), or an OCG the default configuration omits | nothing | the same, and wrongly |

The second and third were both silent before this type existed. They are
different facts with different remedies — clear the search; or accept that
the document's own configuration does not list this group — so they get
different sentences rather than one apology covering both.

### `fn layer_selection_unlayered`

Shown for `Membership::None` and for nothing else.

## Why this sentence exists, when silence would be simpler

It is the disambiguating half of a pair. A selection either highlights a
row or does not, and "does not" has several causes the operator cannot
otherwise tell apart — genuinely unlayered, unresolvable, nested too deep,
spanning two layers. This line is said only in the first case, so its
*presence* is the answer and its absence is not a claim.

## "Not on a layer", not "on no layer"

The operator's mental model is that things are *put on* layers. "Not on a
layer" describes the mark; "on no layer" describes a set, and reads like
the beginning of a fault report. The sentence is a statement about the
document, as ordinary as a layer's name, so it is phrased as one.

## It says "selected", not "this object"

Because it covers an annotation — a stamp, a cloud, a note, a dimension —
as well as a content object, and the two need one sentence rather than two
nearly identical ones.

### `fn layer_selection_report`

# The two states that owe nothing, and they owe nothing for opposite
reasons

* `NothingSelected` — there is no question. A status bar or a panel that
  narrates the absence of a thing spends a permanent line on the most
  common state in the program.
* `Group` **with its row on screen** — the plate has already said it.
  Repeating it in prose would make the panel narrate its own highlight, and
  the operator would read the sentence as a *second* fact.

# Every other state owes one, including the unknowns

That is a reversal, and it is deliberate. `Unknown` used to be silent, on
the argument that a line reading *"pdfcer cannot tell you which layer this
is on"* would be a **permanent apology** — true, while `pdfcer-core` could
not answer for any content object at all. `Pass 250.0` retired that
premise: the unknowns below are now rare, specific and individually
actionable, and withholding them would be hiding an inference the operator
cannot see. See `Unresolved`'s own doc comment for the full argument.

### `fn layer_clause`

`None` when nothing is owed, on exactly the one state that owes nothing
there: nothing selected. Unlike the panel, the bar **does** speak when the
row is on screen, because the bar is the surface reached with no panel open
and it cannot lean on a plate the operator may not be looking at.

# `name` and the `Membership` are not independent

`Some(name)` is meaningful only alongside `Group`, and a caller with a
group it could not name passes `None` — which is the OCMD and
unregistered-OCG case, and gets its own words rather than an empty pair of
quotes.

### `fn selection_with_layer`

Appended rather than given a line of its own, because it is a fact about
**the same selection** — the same reasoning `status::selected` applies to
its depth clause. A second label would read as a second subject.

### `fn layer_selection_granularity`

He measured it on his own drawing — *one PDF path object holds 6,681
anchors across half his sheet* — and the largest object on `SW41177.pdf`
p1 holds **1,194 subpaths** over 550 × 500 pt. He clicks a circle; pdfcer
selects the object the circle is one subpath of.

# Why the layer answer is still exact, and why that is not enough

`/OC` membership belongs to a marked-content section, which wraps *paint
operators*. A `BDC /OC` cannot begin in the middle of a subpath, so every
part of one object shares one membership by construction. The relation has
**no finer form to be exact at** — descending to the Part or Point rung
cannot refine it.

So the sentence *"this is on layer Grid"* is true. Said about something the
operator believes is a single circle, it is a claim he will apply to the
circle, and on his files it is a claim about a thousand other curves too.

⇒ This line states the granularity as a **measurement** rather than
implying a precision that does not exist. It is off-canvas, per Rule 4, and
it is silent on the overwhelmingly common object that holds one part.
