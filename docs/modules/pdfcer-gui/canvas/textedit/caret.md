# `canvas::textedit::caret` — **where the caret is, and what a key does to
the text around it**

## Why this is its own file

R2, on 2026-08-20, when the caret pushed `canvas::textedit` past 1,500
lines. It is a real seam rather than a convenient cut, and the test for that
is the one this project uses everywhere: **everything here is a pure
function of a `&str` and an index.** No `egui::Context`, no document, no
page, no PDF. The rest of `textedit` is about *placing* a caret on a page
and *committing* what was typed into it; this file is about the string.

That split is worth having for a second reason. A caret is the most
convention-bound object in any editor — every operator alive already knows
what Home does, what Ctrl+Left does, and that Delete eats forwards while
Backspace eats backwards. Conventions are testable as arithmetic, and a file
with no window in it can assert every one of them in microseconds.

## The defect this file is the answer to

Until 2026-08-20 there was **no caret index at all**. `insert` extended the
end of the draft and `backspace` popped its last character, so the painter
drew its line at the right edge of the run's glyph box — because that is the
only position an append-only draft has. The operator:

> *"the cursor just sits at the end of a text line. It can't be moved to the
> center of an existing text block."*

Exactly right, and it made editing existing page text nearly useless: a
title-block cell reading `SHEET 1 OF 4` could only be changed by deleting it
back to `SHEET ` and retyping the rest.

## The invariant every function here maintains

```text
caret <= text.chars().count()
```

Held by **clamping on entry** rather than by asserting, so a caller that
hands over a stale index — one taken before the text was replaced — gets the
nearest sensible position instead of a panic. A panic in a caret would take
the whole window down over a keystroke, which is a spectacularly bad trade
for a class of bug whose worst honest outcome is a cursor one character from
where it should be.

## Characters, never bytes

Every operation here is expressed in keystrokes and one keystroke is one
`char`. A byte index would make Left-arrow over `é` — two bytes — either
move half a character or need a decode at every use, and a byte truncation
of a multi-byte character is a **panic** in Rust rather than mojibake.

The cost is that every operation is O(n) in the draft's length. A draft is
one show operator — a table cell, a label, one line of a note — so n is tens
of characters, and the alternative is a byte index plus a boundary check at
every call site.

## What is deliberately NOT here

**A selection.** There is no anchor, no range, no Shift+arrow, no Ctrl+A.
That is a second feature with its own drawing, its own replace-the-range
semantics in every edit below, and its own interaction with the clipboard —
and folding it in half-done would give the operator a highlight that some
keys respect and others silently ignore. It is recorded as its own row in
`OPERATOR_REQUESTS.md` rather than left as an implied gap.
