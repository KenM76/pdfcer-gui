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

## Item notes

### `fn default`

Black is written **explicitly**, and that is the whole point: see the
module header. Leaving it `None` would inherit the engine's new
transparent default and silently change what every existing operator's
redactions look like.

### `fn the_shipped_fill_is_an_explicit_black_box_not_a_none`

The regression test for the engine's default changing underneath this
shell. `a705d14` made `RedactSpec::fill = None` mean **transparent**
where it had meant black — correctly, per Table 192 — so a shell that
passed `None` would silently stop drawing the box over every redaction
its operators applied. The content would still be removed, which is why
nothing would fail; the operator would simply see no evidence that
anything had happened, on the one operation they cannot undo.

Asserted against the ENGINE's type rather than against `Fill::Black`,
so it fails if the mapping is what breaks rather than the default.

### `fn a_blank_caption_is_no_caption`

The rule lives in one place so no call site can send `Some("   ")` to
the engine, which would author an `/OverlayText`, burn in a box of
spaces, and count as a caption in the report.

### `fn a_caption_on_a_dark_fill_is_flagged`

The engine hard-codes black text in the `/DA` it authors and told us so
when it shipped the feature. Black-on-black is the case an operator
reaches by accident — keeping the default fill and typing a caption —
so it must be flagged rather than special-cased as "well, that is the
default".

### `fn a_fill_with_no_caption_is_never_flagged`

The other half, and the one that stops the warning becoming permanent
furniture: the default appearance is a black box with no caption, which
is the commonest redaction there is and has nothing wrong with it.

### `const MAX_OVERLAY_CHARS`

**64 characters.** Not a format limit — `/OverlayText` is a PDF string and
has none worth naming — but a legibility one: the engine lays the caption
out inside the marked box and auto-sizes it into a 4–12 pt clamp, so a
sentence over a one-line mark becomes unreadably small or is clipped. 64 is
comfortably longer than every caption anybody actually writes (*REDACTED*,
*Exemption 5*, a case number) and short enough that the operator meets the
bound while typing rather than in the applied document.

### `struct Appearance`

Held on [`super::RedactUi`] — the panel's own state, not the document's —
because it is a property of *what the operator is about to author*, exactly
like `canvas::markup::Pen`. It is read at the moment a mark is created and
never afterwards.

# Why the fill is an enum and not an `Option<Color>`

Because the engine's `Option<Color>` has three operator-facing meanings and
only two of them are colours: *black*, *some other colour*, and *no box at
all*. An `Option` in the UI would put "transparent" and "the operator has
not picked yet" into the same value, and this is the feature where that
ambiguity is least affordable.

### `fn caption_would_be_legible`

# This exists because the engine told us it would not be

`a7210a4`'s reply carries the warning verbatim:

> **Black-on-dark is illegible, and it is our gap, not yours.** The
> `/DA` we author hard-codes black text. Wire a fill-colour picker, let
> someone choose a dark red, and the caption will be black on dark red
> — we saw it in our own verification render. **There is no
> overlay-text colour on the API yet.**

So the panel must not let an operator walk into it silently. The
predicate is deliberately crude — relative luminance against a
mid-point — because the failure it guards is crude: black text on
anything dark. A precise WCAG ratio would imply a precision the engine
cannot honour, since the text colour is not ours to set.

**Black fill with a caption is the loudest case** and is not special-
cased away: black on black is invisible, and an operator who chooses
the default fill and types a caption deserves to be told before they
apply rather than after.

### `fn show`

# It edits in place and raises nothing

No `Action` and no return value, exactly as `canvas::markup::swatch` does
for the pen. The funnel's invariant is that no path runs from a widget to a
**document**, and this touches none: it sets what the *next* mark will be
authored with. There is nothing to undo and nothing to order against.

# Collapsed by default

The shipped appearance — a plain black box, no caption — is what almost
every redaction wants, so these controls should cost nothing until an
operator asks for them. This panel has already shipped its primary verb
below the bottom of its own pane once, and everything added to it now is
measured against that.
