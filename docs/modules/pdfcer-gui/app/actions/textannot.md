# `pdfcer-gui/app/actions/textannot`

## Item notes

### `fn disclosures`

# What is said, what is deliberately not, and why the silences are argued

Rule 4 has two clauses that pull in opposite directions and only one of
them is about drawing. The forbidden half — no badge, no tint, no dashed
outline, nothing that would make a screenshot of the editing canvas differ
from a screenshot of the same file saved and reopened — is honoured here by
construction: this function returns `String`s and touches no painter. The
owed half is *"an inference the operator cannot see still gets a sentence,
off-canvas"*, and that is what this decides, outcome by outcome.

# The disclosures the outcome carries, and what each is worth saying

| field | said? | why |
|---|---|---|
| `unencodable_chars` | **yes**, when non-zero | the `?` is visible; *that pdfcer put it there* is not |
| `stamp_label_fit` = `LabelShrunk` / `LabelClipped` | **yes** | the operator cannot tell a label drawn small from one they asked for small |
| `stamp_label_fit` = `BoxGrown` | **no** | a wider stamp is visible as itself, and the dialog said so before the drag |
| `stamp_label_fit` = `AsRequested` | **no** | nothing was decided for anyone |
| `applied_autosize` | **no** | always `None` here; see below |

**`BoxGrown` is the interesting silence, and it is argued rather than
overlooked.** It is an inference — pdfcer chose a rectangle the operator
did not drag — so the reflex is to disclose it. Two things say not to. The
stamp is *visibly* wider than the box that was dragged, which is the test
rule 4 states: a screenshot of this canvas matches a screenshot of the
saved file, and the difference from the drag is on the screen in front of
them. And `text::textannot::stamp_size_bound` already tells them, in the
dialog, **before** they commit — so a sentence afterwards would be pdfcer
telling the operator a thing it had just told them, which is the exact
failure mode that teaches somebody to stop reading the status line: a shell
that repeats itself is a shell whose sentences stop being read, and those
sentences are the only channel a rule 4 disclosure has.

**The restyle route rules the opposite way on the same variant, and both
are right.** [`crate::app::actions::annots::set_text_annot_style`] does
disclose `BoxGrown`, because there the rectangle is **existing content
pdfcer changed** rather than a request in progress — a stamp that has sat
on the page for months, at a size the operator chose, whose right edge
moves because they typed a number into a properties field. Nothing
forewarned them, and nothing about the act said "and the box will grow".
⇒ The question is never *"can they see it?"* — they can see it on both
routes — but *"did they author it, in the act they just performed?"* That
module's own comment carries the long form.

⚠ **`LabelShrunk` and `LabelClipped` are unreachable from this route**,
because the placing dialog offers no fit policy and every `StampSize`
variant carries the engine's default `GrowToText`. They are
handled anyway, and that is deliberate: the day a fit control appears on
this dialog — or the day the engine changes which policy `StampStyle`
defaults to — the disclosure must already be here, or the feature ships
silent. A branch that is currently dead is cheaper than a rule 4 breach
that is currently invisible.

`StampLabelFit` is `#[non_exhaustive]`. A variant this build has no words
for still answers `is_inference()` and lands on the `_` arm, which
says something true and vague rather than nothing at all. Silence there
would be the worst of the three options: an inference that happened,
reported as though it had not.

`applied_autosize` is not read, and the engine's own doc on the field says
why: it is the **variable-text** auto-size, `None` whenever `/DA` names an
explicit size, and a stamp's fitted size is always written as an explicit
size. It is therefore `None` on every stamp. A `/FreeText` this shell
authors never asks for `0 Tf`, so it is `None` there too. Reading it would
be a field that can only ever say nothing.

# Why it takes two facts rather than the outcome that carries them

So that the rules below can be asserted at all.

`TextAnnotOutcome` is `#[non_exhaustive]`, so no code outside `pdfcer-core`
can build one — which would make every assertion below reachable only by
opening a document, authoring a real annotation and hoping the engine
produced the outcome the case needs. That is a test of the engine wearing a
test of this function's rules, and the rules are where the operator-visible
decisions live. `StampLabelFit`'s variants **are** constructible, so passing
the field lets each silence be asserted directly, including the ones that
are unreachable from today's placing dialog.

The caller therefore does the unwrapping, in one place, in sight of the
engine call. That is the seam this project already uses for the same reason
elsewhere: the impure read stays where the session is, the decision stays
pure.

### `fn a_stamp_on_a_rotated_page_is_authored_upright`

An appearance authored with no rotation reads sideways on every sheet
of a drawing set exported portrait and displayed landscape, which is
invisible to any test that only asserts the annotation exists.

### `fn a_stamp_resizes_with_the_default_modifiers`

The default modifiers are the case that matters: a resize that only
works once a Tool-panel switch is set is a resize the operator will
report as broken, and the engine's refusal of a carried appearance
names it "foreign" rather than saying which switch is missing.

### `fn a_stamp_already_on_the_page_takes_a_new_label_size_and_reads_it_back`

This drives **both** engine verbs against a real document and asserts
they agree with each other, which is the property a panel depends on
and neither verb can guarantee alone. `set_text_annot_style` writing
`/DA` is worth nothing if `stamp_label_parameters` reads a different
number back — the properties spinner would then be seeded on the very
next frame with a value the operator did not type, and would look like
the edit had failed.

⚠ Deliberately not a test of the widget. It asserts the round trip the
widget sits on top of; whether a `DragValue` commits on `drag_stopped`
is a driven-check question, and this project's founding rule says a
passing unit test is not a report of working software. That check is
owed and is recorded as owed.

### `mod placing_disclosures`

Every one of these is a **silence** or a **sentence**, and the silences
are the ones worth asserting: a sentence that goes missing is noticed
the first time somebody uses the feature, while a sentence that appears
where the header argued for silence is nagging — the failure mode the
old GUI is on the record for, and the one that costs real visibility
bugs rather than a shrug.

### `fn a_grown_box_is_silent_because_it_is_visible_and_was_forewarned`

`BoxGrown` answers `is_inference() == true`, so the reflex reading of
R8b rule 4 says disclose it. The header argues the opposite on two
grounds — it is visible on the canvas as itself, and the dialog said
it would happen before the drag was committed — and an argument in a
comment is one an editor can delete by agreeing with the reflex. This
is the argument in a form that goes red.
