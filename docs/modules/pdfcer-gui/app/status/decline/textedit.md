# `app::status::decline::textedit` — the two declines the TEXT CARET raises

`OPERATOR_REQUESTS.md` **O127**, defects 2 and 3. Two recording functions,
and the argument they share.

## Why it is its own file

**R2.** [`super`] crossed 1,500 lines the day these two arrived — the second
split from it, after `decline/floor.rs` — and the seam is a real one rather
than a size-driven cut. Everything else in that file answers *"what is a
decline, and how long does it owe its sentence?"*; this answers *"what does
the text caret decline, and who says so?"*, which is a subject with two
call-site families and an argument about **channels** that nothing else on
that surface shares.

## The argument, once, for both: a sentence in the wrong slot is silence

Every cause below was **already being reported** before O127. Four of the
reflow refusals went through `crate::app::actions::record_note`, which the
bar draws under **`⚑ About your last edit:`**; four more collapsed into
[`super::Declined::EditRefused`]'s nine cause-free words; and Enter in an
existing run said nothing at all, because it quietly committed instead.

The operator's verdict on the first family was *"I haven't seen the reflow
option actually work with anything when I press it."* **He was answered
every single time.** In a slot whose whole contract is *an edit happened;
here is the part you cannot see* — for a press where nothing happened — in
the past tense, about an earlier gesture, truncated to 45 % of the bar
(`NOTES_WIDTH_FRACTION`).

[`super`]'s own header had already ruled on this exact swap, for two other
sentences, in these words:

> *"an operator who reads 'About your last edit' after a gesture that did
> nothing has been told a small lie confidently."*

⇒ So the fix for two of O127's three defects is **not new copy**. It is
these two functions, which put existing sentences in the slot that wears
`⊗` and means *nothing happened*. That is worth a file of prose because it
is the second time this project has proved the same thing: **a control that
answers in the wrong place is indistinguishable, from the operator's chair,
from a control that does not answer at all.**

## Written unconditionally, overwriting whatever was live

[`super::record_text_style`]'s rule, and it matters more here: reflow is a
control an operator presses **twice** when nothing appears to happen. The
second press must produce the second press's sentence, and `LAST` is a slot
rather than a queue precisely so the most recent answer is the visible one.

## Item notes

### `fn record_edit_text`

The operator: *"if I try to edit the edit is not accepted."*

## It belongs in this file, and the seam holds

This module answers *"what does the text caret decline, and who says so?"*
and this is the third such decline — the one raised when the caret's own
**commit** comes back refused. It shares [`record_reflow`]'s call-site shape
exactly: written from **inside** `vector_edit`'s closure, through
`Result::inspect_err`, so the funnel's decline floor lets it stand and the
generic *"That change was refused"* stands aside.

[`record_reflow`]'s note on the ordering is the whole mechanism and is not
repeated here: `vector_edit` takes the floor **before** running the closure,
and `BeforeTheVerb::refused` fills the slot only `if slot.is_none()`. Reflow
was the first verb to use that; this is the second, and the second is what
turns a one-off into the documented route for any verb that learns to
classify its own refusal.

## Why this is a *narrower* decline than the one it sits beside

[`Declined::EditRefused`] — the funnel's own floor — is still there and is
still what every other verb in the shell falls back to. What is different
about `edit_text` is that its error type can now be **classified**:
`pdfcer_core::text_edit::RefusalKind` is a coarse, stable, exhaustively
matchable discriminant, and `crate::text::textedit::EditRefusal::of` joins
it with the one fact the engine cannot see — whether the run the shell
pinned was a single show operator.

⇒ So this recorder is not "a better `EditRefused`". It is what a verb writes
when it has actually *understood* the refusal, and the difference is visible
in the sentence: one says the document is unchanged, the other says why it
is and whether anything can be done.

## Written unconditionally, overwriting whatever was live

[`super::record_text_style`]'s rule, for the reason this file's header
gives: an operator who commits twice must see the second commit's answer,
and `LAST` is a slot rather than a queue precisely so the most recent one is
the visible one.

### `fn missing_character`

# Why reading a variant is licensed here, when this module's own header
forbids it

The header forbids *"matching on `EditError`'s variants"* to derive **the
operator's reason** — *"a second copy of `pdfcer-core`'s taxonomy living in
this crate, which drifts and then tells the operator the wrong reason"*. That
prohibition stands and is honoured above: the category still comes from
`RefusalKind`, exhaustively, and [`crate::text::textedit::EditRefusal::of`]
owns the joining rule.

What happens here is a different act. `RefusalKind` is deliberately coarse —
four buckets, *"stable enough to match exhaustively"* — and `Refusal` carries
**fields the bucket structurally cannot hold**. This reads two of them and
derives nothing: no trigger id is inspected, no `Display` prose is grepped
(exclusion 3 of `check-ui-strings.sh` rules that out in as many words), and
no sentence is chosen from what is read. It is the identical licence
`one_operator` already has — the sentence needs a fact the category does not
carry — with the difference that this fact is the engine's own and is simply
being passed through.

The `_` arm is not laziness: `EditError` is `#[non_exhaustive]`, so an
error the engine adds later must land somewhere honest, and *"this refusal
was not about one character"* is true of every error that is not
`Refused`. Getting that wrong in the other direction — inventing a character
— would put a chooser in front of an operator whose font cannot be
re-encoded at all, and every row in it would refuse.

# What `None` means, and it is a real answer

`Refusal::character` is `None` for the font-classification triggers —
R-INV-2 (symbolic, built-in cmap), R-INV-3 (`/ToUnicode` only) and R-INV-4.
Those are refusals about the font's whole code↔glyph relation rather than
about one scalar, **no other face makes this run re-encodable**, and the
existing `EditRefusal::UnsupportedFont` sentence is the true one for them.
That is why the offer is keyed on the field being `Some` rather than on the
kind being `UnsupportedFont`.

### `fn refused_char_kind`

# Why the shell reads `trigger` here when it reads no other trigger id

`app::status::decline::textedit`'s own header forbids building a second copy
of the engine's taxonomy, and grepping `Display` prose for a cause is exactly
what it forbids. This is neither. It reads **one field of a structured
refusal** — the same licence `Refusal::character` is read on, and for the
same reason: the coarse `RefusalKind` structurally cannot carry it, and
without it the shell says something **false** to the operator.

`RefusalKind` maps `EditError::Refused(_)` to `UnsupportedFont` wholesale, so
these two arrive identically:

| trigger | what is true of the document | what the operator can do |
|---|---|---|
| `TargetAbsent` (R-INV-1) | the character is **not in the font** | change the face |
| `Ambiguous` (R-INV-5, composite) | the character is in the font **twice** — two CIDs map to it | change the face |

The remedy is the same, which is why the face offer is raised for both. **The
sentence is not.** [`crate::text::textedit::font_lacks_the_character`] says
*"The font here was built with only the letters your page already prints"*,
and on an ambiguous character that is simply untrue — the letter is there,
twice over, and pdfcer is declining to guess which glyph he meant. An
operator told the false version would go looking for a missing letter that is
on his page in front of him.

`Pass 256.1`'s own reply names the sentence it wants: *"this letter has two
glyphs in this font; pick another font for it"*. That is what
[`crate::text::textedit::font_has_two_glyphs_for`] says.

⚠ **Test the trigger, not `is_hard()`.** The engine's rustdoc is explicit:
on a composite font `Ambiguous` arrives inside a `Refusal`, and a `Refusal`
is hard by construction, so `is_hard()` answers the simple-font disposition
and is the wrong question here.
