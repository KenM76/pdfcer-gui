# `panels::redact::appearance` — what a redaction looks like once applied

The operator's **one** choice of fill colour and overlay caption, held for
the whole panel and applied to every mark authored from it.

## This shipped as three `None`s, and the reason it did is worth keeping


* `fill` was honoured only when the shell built the spec itself.
  `EditSession::author_text_matches` hard-coded `fill: None`, so every mark
  from *Find and mark* ignored it. A swatch would have worked on whole-page
  marks and been silently dropped on searched ones.
* `overlay_text` was **written into the PDF and never read**. An operator
  would type *REDACTED*, apply, and get plain black boxes with nothing said.
* `quadding` justifies the overlay text and had nothing to justify.

Both were filed. Both came back **fixed** the same day (`a7210a4`,
`a705d14`), and both replies end *"build the control"*. This module is that.

The lesson survives the unblocking and is the reason this paragraph stays:
**an engine field that exists, is documented, and is written into the file
is not evidence that anything reads it.** Two of these three reached the PDF
the whole time.

## `fill: None` changed meaning, and the change is silent and dangerous

Under the old engine `None` meant *black box*; its doc comment said so.
Under `a705d14` it means **transparent** — ISO 32000-1 Table 192 says an
absent `/IC` leaves the interior transparent, and the old behaviour was
simply wrong.

For this shell that is a behaviour change of the worst possible shape. The
content is still removed, so it is not a security failure — but the
operator sees **no box**, which reads as *the redaction did nothing*, on
the one operation they cannot undo and most need to trust.

So [`RedactAppearance::fill`] here defaults to an explicit
`Some(Color::Gray(0.0))` rather than to `None`. That is the standing rule
for a capability becoming choosable — *a build that omits nothing must
behave as it did before the choice existed* — applied to a default that
moved underneath us rather than to one we changed.

`Transparent` is still offered, because it is what the standard describes
and because there is a real use for it (removing content without announcing
where), but it is a thing the operator chooses rather than a thing they get
by not choosing.
