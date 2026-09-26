# `panels::properties::refusedchar` — the refusal, the character it names,
and the face that can type it

`OPERATOR_REQUESTS.md` **O141**. The operator, while trying to fix a typo:

> *"if the character isn't available in a pdf are we able to change to a
> different font?"*

**Yes**, and this module is the connection that makes the answer reachable.
Four pieces have to meet, and each belongs to someone else:

| piece | where it lives |
|---|---|
| the engine refuses by name, before touching bytes | `text_edit::encoding` — `CompositeEncoding::encode_str`, and the simple-font arm's four `Refusal` sites |
| the refusal carries **which character** | `Refusal::character`, `Option<char>`, public |
| a face chooser that offers the standard fourteen | [`super::face`] |
| `set_font` accepting a face the page does not carry | measured: `set_font=AAAAAA+Arimo-Bold->Helvetica-Bold`, after which the `€` goes in |

## Why a PANEL block and not a sentence in the status bar

[`crate::app::status::decline`] already words the refusal, and it cannot
carry this one. `Declined::line` returns `&'static str` — so the slot cannot
interpolate the character the engine named — and `disclosure_line` truncates
what it draws to 45 % of the bar and hangs the rest on hover. A route that
ends in a hover is a route the operator does not find.

[`super::disclose`]'s header settled this exact question for the text tools'
other refusals and its answer transfers word for word: *"A dock panel's
width is the dock's, decided before the body draws, so text wrapped inside it
cannot drive a width and R128 does not apply"*, and *"a refusal is a fact
about the last thing the operator tried to do to the document in front of
them. Properties is where this application already puts facts of that
shape."* The bar keeps its elided line — reworded to name the obstacle and
point here — and the readable copy, the character, and the control live in
the panel.

## Rule 4, and the half of it that binds here

`D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 is *fuzzy,
never sneaky*: pdfcer may make an inference and may never make one silently.
It **forbids marking the drawing** and **requires** the report off the
canvas.

Swapping the face is the operator's own instruction, so the changed
letterforms are not pdfcer marking its own uncertainty: nothing here badges,
tints or outlines a substituted letter, and the editing canvas goes on
looking exactly like the saved file.

But a standard-14 face is a **name**, not a font program — the engine says so
itself, *"no font program is embedded and no bytes of glyph outline were
added"* — so the client's reader supplies the letterforms. **That is the part
the operator cannot see on his own screen**, and it is exactly the case rule
4's surviving half exists for. It is discharged twice, off the canvas, both
times before he can act on it:

1. [`crate::text::panels::face::face_addable_disclosure`], drawn as a
   visible label in this block, **above** the chooser — a disclosure sits
   above the thing it qualifies, because a caveat below a list arrives after
   the operator has already drawn a conclusion;
2. the same sentence again inside the chooser's popup, drawn by
   [`super::face::popup_body`].

And a third time after the fact, without a line of code here: the engine's
own `format_text` disclosure — *"'Helvetica-Bold' was NOT a font resource
here, so pdfcer ADDED one as `/pdfceF6` … no font program is embedded"* — <!-- old-name-exempt: a PDF resource / BaseFont name the ENGINE writes into the file, quoted verbatim from its own output. It is data in the file format, not prose about this project, and the engine deliberately stopped the rename at that boundary because changing it would alter the bytes of every document pdfcer has ever produced. Same ruling as O141's. -->
arrives verbatim in [`super::disclose`], four lines above this block.

## The offer is UNTESTED against the character, and says so

`preview_font_resources` coverage-tests the characters **already in** the
run, not the one the operator is about to type, so a row here can be a face
that then refuses their `€`. [`super::face`]'s header carries the reason this
is not silently filtered — `FontPreflight`'s `R221` forbids this crate
re-deriving the encoding rule, and a second copy would drift from the commit
path — and the standing ruling on this surface is the Bold button's: *offer
it, and surface the disclosure*. That is
[`crate::text::panels::face::refused_char_no_face`], drawn instead of the
chooser.

## The state machine, and why it is three states rather than one

```text
  (nothing)  --refusal naming a character-->  Offer
  Offer      --the operator picks a face--->  Swapped     (the edit epoch moved)
  Offer      --any other document change-->   (nothing)
  Swapped    --any document change-------->   (nothing)
```

**`Swapped` is not a courtesy.** The face swap is itself an edit, so it
retires the offer — and a block that vanished at that moment would leave the
operator at a caret with nothing telling them to type the character again,
which is a route that ends one gesture short of what it promised. It is also
the state in which the operator most needs a sentence, because on a
metric-compatible swap (his own file moved by **0.005 pt**) the page looks
exactly as it did and nothing on screen says the font changed.

Retirement is `doc.edit_epoch`, the number every other stamped read in this
panel uses, and it is honest in both directions: an undo moves it, a save does
not, and any edit the operator makes instead of taking the offer ends the
offer — which is right, because the refusal it reports is then two gestures
ago.

## Where the two halves are written and read

The refusal is recorded by `crate::app::status::decline::textedit`, from
inside `vector_edit`'s closure, at the same moment it writes the status bar's
sentence — one event, two surfaces, one classification. It travels through
[`PENDING`], a thread-local, for the reason
[`crate::app::status::decline`]'s own store is one: the writer is the
dispatcher and the reader is a body that is handed `&OpenDoc` **shared**, so
there is no `&mut` path between them and inventing one would trade
`panels`' founding invariant for a parameter.
