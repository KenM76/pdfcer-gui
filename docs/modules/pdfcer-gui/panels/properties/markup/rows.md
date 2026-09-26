# `pdfcer-gui/panels/properties/markup/rows`

**One function per markup property row.** Each takes the value the
parent already read and offers exactly one edit.

# Why this is its own file


What the parent keeps is the judgement: `section` decides there is a
selection worth a panel, `markup_rows` reduces the annotation to a
[`Current`] and asks [`MarkupStyleSupport`] which of these rows the
subtype even has. **A row that should not exist is never called** — it is
not called and then disabled, which is R9 (`no placeholders`), and it is
why none of these functions takes a `support` argument.

# The contract every row in this file honours

1. **It pushes an [`Action`], never an edit.** Nothing here touches an
   `EditSession`. The action goes through
   `app::actions::funnel::vector_edit`, which is the single place an
   engine refusal becomes a sentence the operator reads.
2. **A `Set` and a `Clear` are different writes and both are offered**
   when the key is present. [`StyleEdit::Set`] writes the key;
   [`StyleEdit::Clear`] removes it and restores the standard's default.
   A control that could only `Set` makes the key a one-way door, and the
   difference shows in another viewer even when it does not show here.
   `colour_row` carries the full argument; the others cite it.
3. **Clear is absent, not greyed, when there is nothing to clear.** Same
   rule, same reason — greying is reserved for *temporarily*
   unavailable.
4. **It reads `super`'s vocabulary and defines none of its own.**
   [`Current`], [`MIN_WIDTH_PT`], [`MAX_WIDTH_PT`] and [`DASH_WIDTH`] all
   live in the parent. The child reaching up is what keeps one owner for
   each; a constant copied down here would be the start of two answers to
   the same question.

# ⚠ `swatch_of` is NOT here, and it looks like it should be

It converts an annotation's `/C` or `/IC` into something a swatch can
show and reports whether that cost a narrowing. Its callers are the
parent's gathering `match` and [`super::textannot`] — neither is a row.
It is a colour-space conversion, and a file about controls is the wrong
home for one.
