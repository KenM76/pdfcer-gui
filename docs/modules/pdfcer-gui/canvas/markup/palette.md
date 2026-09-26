# `canvas::markup::palette` — **Acrobat's own markup colours**, measured
rather than chosen


> *"Also make sure you've used the same default colours and style look for
> these things as Adobe."*

This module is the answer's data half: the ten colours Adobe Acrobat itself
authors comments in, and the grid the Style swatch offers them from.
[`super::pen`] is the answer's behaviour half.

## WHERE THESE NUMBERS COME FROM — the whole point of this header

A colour written into `/C` reaches the operator's saved file. Under this
project's standing **claim-bearing copy** rule — *verify the source, don't
invent* — a document colour presented as *"what Adobe uses"* is a claim, and
a plausible-looking hex triple sourced from memory or from a blog would be
exactly the invention that rule forbids.


```text
HKEY_CURRENT_USER\Software\Adobe\Adobe Acrobat\DC\Annots\cAnnots\<subtype>\cstrokeColor
    t0 = RGB            the colour space
    d1, d2, d3          the three /DeviceRGB components, 0.0 – 1.0
```

That key is where Acrobat stores the *tool default properties* each markup
tool draws with — the values its own **Properties ▸ Make Properties Default**
writes — so it is the same number Acrobat would put in `/C`, not a
description of one.

### The measurement, verbatim

| Acrobat key | `d1, d2, d3` | as bytes | this module |
|---|---|---|---|
| `cSquare`, `cCircle`, `cLine`, `cLine:LineArrow`, `cPolyLine`, `cPolygon`, `cPolygon:PolygonCloud`, `cInk`, `cSquiggly`, `cStamp` | `0.858826, 0.203918, 0.145096` | `219, 52, 37` | [`MARKUP_RED`] |
| `cHighlight`, `cInk:InkHighlight`, `cSound` | `1.000000, 0.384308, 0.000000` | `255, 98, 0` | [`HIGHLIGHTER_ORANGE`] |
| `cUnderline` | `0.074509, 0.450974, 0.909805` | `19, 115, 232` | [`UNDERLINE_BLUE`] |
| `cStrikeOut`, `cFreeText\cstrokeColor` | `0.972549, 0.392151, 0.392151` | `248, 100, 100` | [`STRIKEOUT_PINK`] |
| `cText` (sticky note), `cFileAttachment`, `cHighlight:HighlightNote` | `0.588242, 0.262741, 0.988235` | `150, 67, 252` | [`NOTE_PURPLE`] |
| `cCaret` | `0.752945, 0.215683, 0.768631` | `192, 55, 196` | [`CARET_MAGENTA`] |
| `cFreeText\crichDefaults\ctextColor` | `0.023529, 0.541183, 0.109802` | `6, 138, 28` | [`FREETEXT_GREEN`] |
| every `ctextColor` | `0.000000, 0.000000, 0.000000` | `0, 0, 0` | [`BLACK`] |
| `cFreeText\cfillColor` | `1.000000, 1.000000, 1.000000` | `255, 255, 255` | [`WHITE`] |

### Why this is Acrobat's FACTORY default and not Ken's last click

`HKCU` is a per-user store, so the honest first question is whether these are
the operator's own past choices rather than Adobe's shipped values. Three
pieces of evidence say factory, and they are recorded because the reader who
doubts this table deserves the reasoning rather than an assurance:

1. **Unrelated subtypes share exact values.** `cHighlight`, `cInk:InkHighlight`
   and `cSound` all hold `1.0, 0.384308, 0.0` to six places. Nobody sets a
   *Sound annotation's* colour by hand, and certainly not to bit-for-bit
   agreement with the highlighter.
2. **Ten shape subtypes agree to six decimal places.** A user who recoloured
   a rectangle would move `cSquare` and leave `cProjection` behind.
3. **Every component lands on an exact 1/255 boundary.** `0.858826 × 255 =
   219.0006`; `0.384308 × 255 = 98.0`; `0.074509 × 255 = 19.0`. These are
   byte values a designer picked and a float store round-tripped, not values
   a colour wheel produced.

### THE SURPRISE, and it is the reason a measurement beat a memory

**Acrobat's highlighter is ORANGE, not yellow.** `1.0, 0.384308, 0.0` is
`#FF6200`. Everything anyone "knows" about PDF highlighting says yellow, this
shell has shipped `(1.0, 1.0, 0.0)` since the pen existed, and the reasoning
written into [`super::pen::Pen::default`] said *"yellow … because that is what
every PDF reader draws it in"*. That sentence was written from memory and it
was **wrong about the program the operator actually compares against**.

Yellow is still in the grid — as [`CLASSIC_YELLOW`], sourced honestly as *this
shell's own shipped highlighter*, one click away — because an operator who
wants yellow must not have to leave the palette to get it. What changed is
which of the two is the **default**, and the answer to *"is it the same as
Adobe"* is now measured rather than assumed.

## Why the stored form is BYTES and not Acrobat's own fractions

Acrobat's registry holds `0.858826`; this module holds `219`. `219 / 255 =
0.858823…`, which differs from Acrobat's stored number in the sixth decimal
place and is invisible at any output resolution that exists.

The byte form wins because it makes **one** value the answer to two questions.
A colour reaches `/C` down two paths — as a shipped default, and as a cell the
operator clicked in the grid — and the grid can only offer a `Color32`, which
is bytes. Storing fractions would make the shipped orange and the clicked
orange different numbers in the file for no reason an operator could ever see,
and *"why is my second highlight a different colour from my first"* is a bug
report nobody could reproduce.

⇒ The fractions are preserved in the table above, which is the record. The
bytes are the code, which is the behaviour.

## What is deliberately NOT here

* **A line width.** Acrobat's `cAnnots` tree carries no width, thickness or
  border key at all — searched for `width`, `thick` and `border` across the
  whole `DC` tree, and the only hits were print and multimedia settings. So
  there is no measured Adobe number to match, and [`super::pen::Pen::default`]
  keeps this shell's own 2 pt with the argument written out there rather than
  adopting a number nobody sourced.
* **A dimension colour.** `cLine:LineDimension` exists in the same tree and is
  deliberately not read: a **ce dimension** is not markup, it has its own style
  verb and its own pen (rule 15), and folding Acrobat's dimension default into
  the markup palette would be the exact conflation that rule forbids.
* **A redaction colour.** `cRedact` is likewise present and likewise not
  markup; `crate::text::redact` owns that surface and its own vocabulary.

## Item notes

### `fn every_shipped_default_is_one_click_away_in_the_grid`

The property that makes the palette a palette rather than a decoration:
an operator who changes the highlighter to red and wants it back must be
able to click the value it shipped with. If a default is not in the grid
there is no way back to it except by remembering three numbers and typing
them into the full picker — which is the state a "restore defaults" bug
report describes.

⚠ Note which cell is load-bearing here, because it is not the obvious
one: [`CLASSIC_YELLOW`] is **not** a default any more (the highlighter
ships at [`HIGHLIGHTER_ORANGE`]), so removing the yellow row would not
fire this. The cells this test actually protects are the five a slot
ships at.

Falsified by changing [`ACROBAT`]'s orange cell to `[254, 98, 0]` — one
byte off the highlighter's default: the assertion fired naming
`Highlighter`. Restored.

### `fn each_constant_is_the_registry_value_it_claims_to_be`

This is the test that keeps the module header honest. The header claims
each byte triple is the registry's float triple; a typo in either would
make the claim false and nothing else would notice, because a wrong-but-
plausible red still looks like a red.

The tolerance is a **half a byte** — the largest error a correct
conversion can have — so a value that is one byte out fails.

Falsified by changing [`UNDERLINE_BLUE`]'s green from 115 to 116: the
assertion fired on the `cUnderline` row. Restored.

### `type Reading`

A named type because the tuple is three unrelated things and clippy
is right that an inline `[([u8; 3], (f64, f64, f64), &str); 7]` is
unreadable — but the *shape* is the point of the test, so it is named
rather than simplified away.
