# `pdfcer-gui/dialogs/sign/sections`

## Item notes

### `fn field_list`

The two disclosures below a row — the `/Lock` and the `/SV` — are
drawn **here, before the press**, and that is the whole point of reading
them out of the document rather than waiting for `SignReport`. Signing a
locked box freezes fields the author nominated; a consequence the
operator learns about after the file is written is a consequence he did
not consent to.

### `fn kind_section`

`Pass 10.12`. §2d of [`crate::sign`]'s header argues why this is a radio
pair here rather than a second ribbon command.

**The certifying option is ABSENT, not greyed, on a document that
cannot carry one** — and the sentence explaining why is drawn in its
place. R9's *explained* branch: both of the engine's certification
refusals are states of the document that are knowable when this window
opens, so meeting one by pressing rather than by reading is a failure
this surface can avoid outright.

⚠ And the sentence ends *"you can still add an ordinary signature"*,
which is the half that stops the note being read as a refusal of the
whole window.
