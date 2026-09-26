# `wordmarkup` — the three markup kinds that carry WORDS


## Why they were left out, and why that was right at the time

`shell::commands::reach`'s register carries the reason verbatim, quoting
`canvas::markup`'s own table of kinds it deliberately does not handle:

> *Note · text box · sticky · stamp — Text-bearing, not geometric. A
> different gesture (place, then type) and a different spec type
> (`TextAnnotSpec`).*

Both halves are true and neither is small. **Nothing about the
drag-and-release machinery the seven geometric kinds share applies here**:
those author on release, from geometry alone, with a pen. These cannot —
releasing the mouse produces an *empty box*, and an empty box is not an
annotation, it is a rectangle nobody asked for.

## The gesture: place, then type, then commit

| kind | placing gesture | why |
|---|---|---|
| [`TextAnnotKind::TextBox`] | **drag a rectangle** | a `/FreeText` is painted *into* its rect and wraps to it, so the operator is choosing how wide the text is. A click would have to invent a width |
| [`TextAnnotKind::Sticky`] | **one click** | a `/Text` marker is fixed-size and `NoZoom` — its rect's width and height do not affect what is drawn, so asking the operator to drag one would be asking for a number that is discarded |
| [`TextAnnotKind::Stamp`] | **drag a rectangle** | pdfcer's stamp appearance is a framed label scaled into its rect, so the drag is choosing how big the stamp is |

Then the dialog opens, and **nothing is authored until Accept**. That is
rule 4 applied to a gesture whose output is words: a half-typed note
committed on a stray click would be content the operator did not write.

## Escape has two meanings here and they are ordered

A placing drag in flight is abandoned by Escape, exactly as a markup band
is — that rung already exists and this kind rides it. Escape with the
**dialog** open is the dialog's, and closes it without authoring.

The two cannot both be live: the dialog only opens once the drag is over.
Stating it because the ordering is the kind of thing that looks obvious
until a third claimant is added to `canvas::keys`' ladder.

## Item notes

### `fn every_trace_token_is_parseable_and_distinct`

This is the failure mode worth naming. A token that drifts does not make
the harness fail to *build*; it makes it fail to *match*, and a check
that cannot find `size=24` reports **"the operator's chosen size did not
reach the engine"** — which is a defect report about the application,
written by a defect in the token. This project has spent whole
investigations on exactly that shape: a harness with a bad input
produced six plausible failure reports and four filed defects, none of
which existed.

So four separate clauses, each of which a plausible future edit
breaks on its own:

1. the derived case is the literal word `derived` — not `auto`, not
   `fit`, not `none`;
2. it contains **no digit**, so a check can distinguish "he let the box
   decide" from "he asked for a number" without knowing the vocabulary;
3. a stated size is **just the number**, with no unit — the trace field
   is named `size=` and a parser reading `24pt` as an integer gets `0`
   or an error, neither of which says what happened;
4. every offered choice produces a **distinct** token, or two different
   operator choices become the same observation.

### `fn every_kind_authors_the_annotation_it_names`

The failure this catches is a copy-paste between arms — a sticky
authored as a `FreeText` would paint the operator's private note onto
the page, which is the opposite of what a sticky is for and is not
recoverable by anything but noticing.

### `fn a_blank_text_authors_nothing`

The one refusal this module makes, and it is worth a test rather than a
comment: an annotation with no words is an empty box on the operator's
drawing that they then have to find and delete. Whitespace counts as
empty, or a stray space bar would author one.

### `fn the_text_is_trimmed`

Leading space in a `/FreeText` is painted, so an operator who typed a
space before their sentence would get an indented callout they did not
ask for and cannot see the cause of.

### `fn the_words_a_text_box_paints_are_the_words_its_note_will_carry`

`EditError::FreeTextNoteConflictsWithText` (`pdfcer-core` `95a936e`)
rejects the whole authoring call when
`TextAnnotSpec::FreeText { text }` and `MarkupOptions::note` differ.
`crate::app::actions::textannot::commit` passes both — it must, since
`/T` and `/M` are reachable only through the note — so this is the
assertion standing between an operator's stray space bar and a gesture
that authors nothing.

# Two assertions, and the second is what makes the first a test

Asserting only `text == painted_text(typed)` passes on a
[`painted_text`] reduced to the identity function AND on a [`spec`] that
had stopped normalising — the two would agree by both doing nothing,
which is precisely the shape the engine warned about when it shipped
`Pass 258.1`. So the input carries whitespace that must not survive, and
the second assertion says out loud that something was removed.

### `fn the_two_defining_properties_hold`

The two properties that make each kind the thing it is. A `/FreeText`
that did not wrap puts the operator's second sentence outside the box
they drew; a sticky whose contents were painted would publish a private
note onto the drawing.

### `fn the_two_odd_ones_out_are_the_ones_expected`

Both are properties of the FORMAT rather than of taste — a `/Text`
marker discards its rect, and a stamp with a free-text label is a text
box with a border. A second kind acquiring either predicate would mean
one of those two facts had changed, which is worth failing over.

### `fn the_stamp_gallery_is_usable`

A default outside its own list would leave the dialog opening on a
value no control can select — the same defect the settings window's
range checks exist for.

### `fn a_stamp_authors_the_chosen_name_and_no_competing_label`

The regression test for the mistake this module made in its first
draft: inventing label strings and leaving `/Name` at the enum's
`Draft` default, so a stamp reading APPROVED would carry `/Name /Draft`
and any reader but pdfcer would show *Draft*. An annotation that
disagrees with its own appearance is the quietest way to be wrong about
a document.

`label: None` is asserted with it, because a label and a name that both
carry text is the same disagreement arrived at from the other side.

### `fn the_icon_list_covers_every_variant_the_engine_has`

The same guard `panels::properties::markup::ALL_ENDINGS` carries, and
for the same reason: the engine publishes no `ALL` for this enum, so the
list above is written by hand and would otherwise go quietly short the
day the engine gains an eighth icon — quietly, and in the direction that
**withholds** a choice that had started working.

The `match` is exhaustive **with no wildcard**, which is the whole
mechanism: a new variant is a compile error here, not a failing
assertion, so it is caught by `cargo build` before any test runs.
`StickyIcon` is not `#[non_exhaustive]`, which is what makes that
possible — `StampName` is, which is why [`STAMPS`] gets no equivalent
and is documented as an editorial subset instead.

### `fn only_a_sticky_note_is_given_an_icon_and_it_is_the_chosen_one`

The negative half is asserted **beside** it rather than alone, which
is the methodology note of 2026-09-06: *"a negative assertion is vacuous
when the thing that would produce the positive is absent."* Asserting
only that a text box carries no icon would pass on a [`spec`] that had
stopped threading the argument at all — it would pass on the code this
change replaced. The sticky arm proves the operand arrives; the other two
arms prove it stops where §12.5.6.4 stops, which is also where
`set_text_annot_style` refuses it by name
(`EditError::StylePropertyNotApplicable`).

### `enum TextAnnotKind`

# One enum carrying three kinds, not three tools

The same argument `MarkupKind` and `MeasureKind` both make, and for the
third time it is a statement about types rather than about tidiness: the
operator is placing exactly one annotation, so a type that could say
*text box* and *sticky* at once — which three booleans, or three tool
variants plus a "which is active" rule, both can — is a type whose illegal
states are prevented by discipline instead of by construction.

### `fn from_command`

Derived from [`Self::command`] rather than written out a second time,
exactly as `markup_for_command` and `measure_for_command` are — so the
two directions cannot disagree even in principle.

### `fn is_dragged`

See the module header's table. The sticky note is the exception and the
reason is that its rect is not read: a `/Text` marker is fixed-size and
`NoZoom`, so a dragged width would be a number the operator chose and
the format discards.

### `fn uses_gallery`

Only the stamp. `manifest/markup.rs` recorded the blocker as *"the
stamp control exists and needs a GALLERY … a stamp with no chooser has
no operand"*, and that is what this predicate drives: the dialog offers
[`STAMP_LABELS`] instead of an empty field.

### `const STAMPS`

# The ENGINE's names, not a list of my own


Inventing strings would have authored `/Name /Draft` — the enum's default —
on every stamp regardless of what it said, so a reader other than pdfcer
would show *Draft* under a stamp reading `APPROVED`. The annotation would
have disagreed with its own appearance, which is the quietest possible way
to be wrong about a document.

# Which of the fourteen, and why not all of them

The engine offers fourteen. These are the ones a **drawing revision**
workflow uses; the rest (`TopSecret`, `Sold`, `Departmental`,
`NotForPublicRelease`) belong to document control rather than to drafting,
and a gallery of fourteen is a list an operator scans instead of a set they
know. Adding one is a line here — the constraint is the enum, not this
list.

### `const STICKY_ICONS`

# The ENGINE's set, enumerated from the engine's own enum

`pdfcer_core::annot_author::StickyIcon` models seven standard names —
`Comment`, `Key`, `Note`, `Help`, `NewParagraph`, `Paragraph`, `Insert` —
and that is exactly §12.5.6.4 Table 172's standard set, which is exactly
the seven Acrobat's note tool offers. Its eighth variant, `Other(Vec<u8>)`,
carries a `/Name` a foreign file used and is deliberately **not** here: it
is a value this shell preserves when it meets one, never one it authors.
So unlike
[`STAMPS`], which is a **subset** this shell chose out of fourteen for a
drafting workflow, this list is the whole enum and there is no editorial
decision in it.

It is nonetheless written out here rather than taken from an `ALL`
constant, because the engine publishes none — the same position
`annot_author::LineEnding` is in, and
`panels::properties::markup::ALL_ENDINGS` is the precedent. What stops it
drifting is [`tests::the_icon_list_covers_every_variant_the_engine_has`],
which `match`es an **exhaustive** set of variants with no wildcard: an icon
the engine gains fails to compile here rather than quietly going missing
from the chooser. `StickyIcon` is not `#[non_exhaustive]`, which is what
makes that check possible at all.

# The order

[`DEFAULT_STICKY_ICON`] first, then Table 172's own order for the rest. A
list whose first entry is the one already selected is a list an operator
reads downwards from the answer they have, rather than hunting for it.

### `const DEFAULT_STICKY_ICON`

# `Comment`, not the engine's `Note` default — and it is MEASURED

`ACROBAT_DEFAULTS.md`'s non-colour table reads *"default sticky-note icon —
**`Comment`** — `cAnnot` `tnoteIcon`"*, taken from Acrobat's own registry
hive on this machine. The operator's instruction of 2026-09-06 was to *"make
sure you've used the same default colours and style look for these things as
Adobe"*, and the icon is the same class of answer as the violet `/C` that
instruction already moved.

The difference from `StickyIcon::default()` is deliberate and is the same
difference [`DEFAULT_STAMP`] carries: `Note` is the right default for a
**format** that must name something in Table 172, and `Comment` is what the
program on the operator's desk actually places.

# ⚠ The provenance, stated because `ACROBAT_DEFAULTS.md` requires it

That file's load-bearing caveat is that `cAnnots` is a **live preference
hive**, and its sibling test — *is this value repeated across keys nobody
would set together?* — is what separates a factory value from an operator's
override. **`tnoteIcon` fails that test, and it cannot pass it**: it is a
singleton, there is no second key in the tree carrying a sticky-note icon
name, and it lives in `cAnnot`, the very key whose `tauthor=Ken` proves
Acrobat has written to the hive.

⇒ So this is adopted on the *other* argument the same file records, the one
that overturned the highlighter: *"the operator asked to match the program on
his desk, and the program on his desk"* opens its note tool on Comment. A
factory default he has never seen is not what "the same as Adobe" means. The
cost of being wrong is one constant and this paragraph.

### `const DEFAULT_STAMP`

`Approved` rather than the engine's `Draft` default, and the difference is
deliberate: `Draft` is the right default for a *format* that must pick
something, and the wrong one for an *operator* who has just pressed a stamp
control on a drawing they are reviewing. The commonest first stamp in a
review is the one that says the review passed.

### `enum StampSize`

# Why this control exists, and it is not "the engine gained a field"

Before `Pass 287.0` a stamp's text size was **derived from the box**:
`(height * 0.42).clamp(8.0, 28.0)`, stored nowhere. Drag a bigger box, get
bigger text. That is the behaviour on the operator's desk today and it is
also the behaviour he complained about, for a reason that only looks
contradictory until the two halves are separated:

  * **What he liked** — the drag chooses the size. That is how every stamp
    tool he has ever used works, Acrobat's included, where a stamp is
    artwork scaled into the box.
  * **What he reported** — *"I have to draw the size before it gets
    applied"*, and widening a stamp to reveal clipped text enlarged the text
    by the same act, so it never stopped being clipped.

The engine's answer was to make the size a property with a default of a flat
12 pt. ⚠ **Taking that default silently would have shrunk every stamp he
draws**: a typical 60 pt-high stamp box derived a 25 pt label before the
bump and would have got 12 pt after it — a visible change in his documents,
arriving as a side effect of a fix he asked for, with no control anywhere to
undo it. That is the shape this project treats as a defect regardless of
which side of the crate boundary caused it.

So the default here is [`Self::FitTheBox`], which keeps the derived size,
and the stated sizes are the new capability sitting beside it.

# Every variant pairs its size with `StampFit::GrowToText`, and the REASON changed



Until `pdfcer-core` `Pass 291.0` the other two policies were *unofferable*,
not merely unoffered. Both decide something the operator did not ask for —
a label drawn at a size they did not choose, or characters dropped — and
R8b rule 4 owes a sentence for exactly that. There was nothing to build the
sentence from: `AuthoredTextAnnot::applied_autosize` was the only outcome
field, it carries the **variable-text** auto-size, and it is `None`
whenever `/DA` names an explicit size. A stamp's fitted size *is* written
as an explicit size (`Pass 287.0`), so it was `None` on every stamp, always
— including every stamp where a size had been chosen for the operator. The
number was computed, used, written to the file and dropped on the way back.
Measured against `annot_author.rs` at pin `d4a4e3b`.

☑ Filed rather than worked around, and answered: `Pass 291.0` added
`stamp_label_fit` — a **second** field, which is what the request asked for
by name in preference to widening `applied_autosize`'s meaning — to both
the authoring outcome and the restyle change.
[`crate::app::actions::textannot`] takes the reporting entry point to read
it, and [`crate::panels::properties::markup::textannot`] reads the restyle
half. **The engine limit is gone.**

## Why the placing dialog still shows no fit control anyway

Because the reason is now an interaction judgement, and the two are worth
keeping apart: an engine limit disappears the day a pin moves, a judgement
does not, and a comment that conflates them sends the next reader to the
wrong repository.

  * `GrowToText` widens the box when the label does not fit. **Visible on
    the canvas as itself** — the operator sees a wider stamp — and
    [`crate::text::textannot::stamp_size_bound`] says so in the dialog
    *before* the drag is committed. It cannot be quietly wrong.
  * `ClipToBox` is, in the engine's own words, *"the one the operator
    reported"*: widening a stamp to reveal clipped text enlarged the text
    by the same act, so it never stopped being clipped. Offering the
    reported defect back as a choice on the dialog that fixes it is not a
    feature.
  * `ShrinkToBox` is a real preference some operator will want, and the
    placing dialog is the wrong surface for it. At placing time the box is
    being dragged **this instant**, so *"what if the words do not fit"* has
    a better answer than a dropdown: draw it bigger. On a stamp already on
    the page it does not, which is precisely where the chooser lives — in
    the properties panel, under the number it qualifies, sharing one
    session preference through [`crate::canvas::stampfit`].

O171 is also on the record here: the operator's report that this dialog
was already too small to reach its own buttons. That was fixed
structurally rather than by height, so a row is no longer unaffordable —
which is why the case above is made on what the control *means* and not on
what it costs. If it were an affordability argument it would expire the
next time the window grew.

# The point sizes

A stamp is a word or two in Helvetica Bold, so the useful range is wide and
coarse. These are the sizes a word processor's font-size box offers over the
same span, which is the list an operator already knows how to read; nothing
here depends on the exact set.

### `const STAMP_SIZES`

[`StampSize::FitTheBox`] first because it is the default and because a list
whose first entry is the one already selected is a list an operator reads
downwards from the answer they have — the same ordering rule
[`STICKY_ICONS`] states.

### `fn style`

Built from `StampStyle::default()` by `with_font_size` rather than by
struct literal, and not only because `StampStyle` is
`#[non_exhaustive]`. A field the engine adds arrives here carrying the
engine's own default instead of failing to compile with a value this
shell would have to invent — and the one field this shell has an opinion
about is stated explicitly, so the opinion is visible in the diff.

### `fn trace_token`

# Why this exists rather than `{:?}`

A `Debug` rendering is a *rendering*, and this project has already been
bitten by one: a driven check once reported the opposite of the truth
while quoting the truth in its own failure message, because it was
pattern-matching on a `{:?}` tuple whose shape changed. The rule that
came out of it is absolute — **never `Debug`-format a field a machine
reads**. `Debug` belongs to the programmer at a breakpoint; a trace
field belongs to a parser, and a parser needs a contract.

So the contract is here, in one place, testable:

| choice | token |
|---|---|
| [`StampSize::FitTheBox`] | `derived` |
| [`StampSize::Points`] | the number, e.g. `24` |

**`derived`, not `auto` and not `fit`.** The word has to say what
the engine is being asked to do — work the size out from the box the
operator drew — because the failure this token exists to catch is a
build that sends the engine's flat 12 pt instead. `auto` would be true
of that build too.

⚠ **It is deliberately not a number for the derived case**, even
though this shell could compute `(height × 0.42).clamp(8, 28)` and emit
one. That formula is the *engine's*, it is not part of any contract this
shell is entitled to restate, and it changed once already. Emitting it
would make this line a claim about the engine's arithmetic instead of a
record of the operator's choice — and the operator's choice is the only
thing the shell is answerable for.

### `const STICKY_PT`

# It is not a size the operator sees


So this number decides nothing about the picture. What it must be is
**non-degenerate** — a zero-area rect is refused by the engine's geometry
validation and would turn a placed note into a silent refusal — and roughly
icon-sized, so that anything reading the rect for a hit test or a bounding
box gets an answer near the truth rather than a point.

20 pt is about the size Acrobat draws its note icon at, which makes it the
least surprising answer to a question the format says is not being asked.

### `const MAX_TEXT_CHARS`

# It bounds the FIELD, not the format

`/Contents` is a PDF string and has no length worth naming. What is bounded
is what an operator can usefully put on a drawing: a `/FreeText` is painted
into a box the operator dragged, and at some length it either shrinks below
legibility or is clipped. 512 characters is a long paragraph — comfortably
more than any callout on a drawing sheet — and short enough that the
operator meets the bound while typing rather than in the saved file.

### `const TEXT_SIZE_PT`

**Not zero.** `TextAnnotSpec::FreeText` documents `0.0` as auto-size, and
auto-size is the right default for a box whose content is unknown — but it
is the engine's heuristic rather than the operator's choice, and this shell
has no surface to override it. 11 pt is a legible caption on a drawing
sheet at the sizes this shell is for, and choosing it explicitly means the
operator gets the same size on every sheet rather than one that varies with
how big a box they happened to drag.

### `fn painted_text`

# Why this is a named function rather than a `.trim()` in two places

Because for a `/FreeText` the two places are **the same PDF key**, and as of
`pdfcer-core` `95a936e` a disagreement between them is a hard refusal rather
than a silent mess:

> `EditError::FreeTextNoteConflictsWithText` — *"a `/FreeText`'s note and
> its painted text are the same key (`/Contents`) and cannot differ"*.
> Identical strings are still fine.

[`spec`] normalises the words it bakes into the appearance;
`crate::app::actions::textannot::commit` passes the same words again as a
`MarkupOptions::note`, because `/T` and `/M` are reachable only through the
note. Before this function existed those two read the *same variable* and
produced *different strings* — the spec trimmed and the note did not — so a
text box typed with a trailing space would have been refused outright, with
nothing authored and only the generic decline sentence to show for it.

⇒ One function, called by both, so the two cannot come apart. This is the
engine's own framing of the trap this shell reported to it: *"a loaded gun
on a verb whose two arguments look independent and are not."*

Trailing and leading whitespace only. It is what an operator's stray space
bar produces, it is invisible in the box, and it must not decide whether the
annotation is authored at all.

### `fn spec`

# Pure, and separate from the action arm for the standing reason

It is the part that could be wrong in a way an operator would notice — a
stamp authored with the wrong quadding, a sticky whose words went into the
wrong field — and a `&mut EditSession` is not available to a test that only
wants to ask what was built. Every geometry rule in this crate is split
this way.

Returns `None` for an empty text, which is the one refusal this function
makes: an annotation carrying no words is not a thing the operator asked
for, and authoring one would put an empty box on their drawing that they
then have to find and delete.
