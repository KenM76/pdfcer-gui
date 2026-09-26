# `pdfcer-gui/dialogs/unsaved_host`

## Item notes

### `fn ask_unsaved`

Returns `true` when the question was raised and the caller must
**stop** — the intent is now this window's to resume. `false` means
there was nothing to ask about and the caller proceeds unchanged.

# Why the return value is "did I interrupt you" rather than "may I
proceed"

Both spellings work and only one of them is safe to get wrong. A guard
read as *"may I proceed"* fails **open** when somebody inverts it or
forgets it: the document is destroyed. This one fails **closed** — a
missing `if` means the question is asked and its answer resumes the
intent anyway, so the operator sees one redundant prompt rather than
losing their afternoon.

The already-open guard matters more here than on any other dialog in
this struct: without it, a keymap chord repeated while the question is
on screen would replace the pending intent with a second one, and the
operator would answer a question about Close and get an Open.

### `fn unsaved_cancelled`

Drained by the quit cycle, which needs to know that the operator said
no — a Cancel closes the window and parks no outcome, so
`take_unsaved_answer` reports nothing at all and the cycle would
otherwise re-ask on the very next frame, forever.
