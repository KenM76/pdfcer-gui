# `app::actions::addtext` — placing NEW page text, and the width question

One verb, `Action::CommitAddText`, and the one decision it has to make that
nothing upstream can: **how wide is the text the operator just typed?**

## Why it is its own file

The same seam [`super::funnel`] is cut along: [`super::apply`] is a
**router** — it answers *"which module handles this action?"* — and this
answers *"what does placing text mean?"*. The arm below decides something;
a router arm should not.

## A PDF HAS NO PARAGRAPH, and that is the whole subject

Every visible line of text in a PDF is its own show operator at its own
absolute position. There is no object that means *"this text, flowing"*. So
something has to decide where a second line starts — a **width to wrap
against** — and that decision has to be made by the time the engine is
called, because `AddTextRequest` offers exactly two shapes:

| request | what the engine does with `\n` |
|---|---|
| **point** — `new(page, origin, text)` | **refuses it, by name.** `\n` has no code in any standard encoding, so `encode_str` raises `Refusal { trigger: TargetAbsent, character: Some('\n') }` and the whole add fails |
| **boxed** — `…with_box(x, y, w, h)` | splits on it: each `\n` is a hard paragraph break, each paragraph is wrapped independently to the box's width, top-anchored from the box's top edge |

⇒ **A multi-line add MUST be boxed.** That is not a preference, and it is
why dragging a rectangle was the multi-line gesture in the first place: a
drag says how wide.

## Where the width of a CLICKED multi-line draft comes from

The operator: *"can the enter key create new lines when we are editing or
creating text?"* Enter inserts a line break at a **clicked** caret too — and
a click has no extent, so this arm meets a multi-line draft carrying no
width of its own.

The shell's own standing rule, from `canvas::textedit::place`'s header, is
that a width may not be **invented**: *"a click would have to invent a
width."* That rule is kept. The width is not invented — it is **read off the
operator's own page**:

```text
left   = where they clicked
right  = the crop box's right edge
top    = where they clicked          (so the first line starts at the caret)
bottom = the crop box's bottom edge
```

Every number is a fact about their document. Nothing here chooses a margin,
a column width or a default; the sheet does.

The promotion is **conditional**. A single-line point add is still a point
add, byte for byte, taking the path it would take if this rule did not
exist — which is what keeps the common case unchanged and makes the rule
impossible to regress into. See [`request`]'s three-way match.

## The promotion is DISCLOSED

A rectangle that is not drawn is a rectangle the operator cannot see, and
where a long line breaks depends on it. `crate::text::textedit::point_text_became_a_block`
is the sentence, shown once at the commit and not while typing, and it names
the gesture that puts the width back in their hands.

It rides the ordinary disclosure list — the same `Vec<String>` the engine's
own disclosures travel in — rather than getting a channel of its own,
because an edit *did* happen and this is the part of it they cannot see.
That is precisely what `⚑ About your last edit:` is for, and it is the
distinction O127's other two defects are both on the wrong side of.
