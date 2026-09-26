# `canvas::textannot` — the three markup kinds that carry WORDS


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
