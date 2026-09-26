# `text::embed` — what the Embed-fonts window says before it changes
anything

The copy for [`crate::dialogs::embed`].

## This window exists to be READ, not to be filled in

⚠ **Corrected 2026-09-05.** This paragraph read *"It has no settings … there
is no useful way to make it configurable either"*, and that sentence was
used in `OPERATOR_REQUESTS.md` **O47** as the reason not to let the operator
decide whether pdfcer's own standard-14 faces may stand in for his. The
window now has exactly one control, and the reasoning that kept it out was
wrong rather than merely outdated — `dialogs::embed`'s header carries the
whole account.

What survives, and still governs every string in this file: `embed_fonts`
takes a request the shell has already resolved, so almost every word here is
a **report of what would happen** rather than a field to fill in, and the
window is a confirmation rather than a form. The one control is a *consent*,
not a configuration: it changes what the report says, and the report is
still what the operator is reading.

That shape is chosen because of what an embed is: it puts font **programs**
into a document permanently, changes its size, and can invalidate a PDF/A
claim. There is no honest way to offer that as a one-click ribbon verb.

## The three things it must say, in this order


**1. What will be embedded**, because that is the operator's answer.

**2. What will NOT be, and why, per font.** The engine's `EmbedBlocker` has
eight variants and they mean very different things — a Type 3 font cannot be
embedded at all, a font with no donor needs a folder, a composite needs a
different verb. Collapsing them to *"3 fonts could not be embedded"* would
throw away the only part an operator can act on.

**3. The PDF/A claim.** A document identifying as PDF/A that gains an
unembedded-to-embedded change is a document whose claim may no longer hold,
and `pdfcer-core` is explicit that it is **choosing** this disclosure rather
than matching Acrobat — parity there is an open research gap. A choice made
on purpose is one this shell repeats rather than quietly drops.
