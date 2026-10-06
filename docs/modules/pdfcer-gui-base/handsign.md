# `handsign` — the drawn signature, its fit, its remembered copy, and the session ledger

`OPERATOR_REQUESTS.md` **O269**. A form-filler's signature: ink on the page,
not a certificate signature.

## The fit rule

`fit(mark, rect)` places a mark inside a signature box, both y-down, same unit.

- Scale is `min(0.95·W / inkW, 0.95·2H / inkH)`: the mark may rise to **two
  box heights**, as Acrobat Fill & Sign does, because form signature boxes are
  often a single text line high and a signature squeezed into that is unreadable.
- Placed height `≤ 0.9H`: centred vertically. Taller: it **stands on the
  lower edge** (`0.05H` above it) and rises above the box, the way a pen
  signature overruns the line it was written on.
- Left inset `0.03W`; the mark is left-aligned, as handwriting is.
- Pen width `clamp(0.04 · placedHeight, 0.6, 2.5)` points, so a small box gets
  a fine line and a large one a bolder line without either going blotchy.
- A one-point stroke (a dot over an *i*) becomes two equal points: the engine's
  Ink verb rejects an empty stroke, and round caps draw two equal points as a dot.
- A mark with no extent above 0.5 is not a signature; `fit` returns `None`.

## Simplification and normalisation

Strokes are reduced by Ramer–Douglas–Peucker at `SIMPLIFY_TOLERANCE` (pad
points) before placing, so the content stream carries a few dozen points per
stroke, not every mouse sample. `normalised` scales the longest side to 1 so a
remembered mark is independent of the pad's size.

## The remembered copy

`hand-signature.txt` in the settings store's directory, one stroke per line as
`x,y` pairs. A separate file, not a preference key: it is personal data the
operator may want to delete by hand, and it is the only thing *Remember my
signature on this computer* writes. Unticking the box and placing deletes it.
A damaged file is ignored whole (`from_text` returns `None`), never partly used.

## Which boxes are signed

Every placement wraps what it writes in the engine's hand-signature tag
(`/pdfc_HandSig <</Field (name)>> BDC … EMC`, `pdfcer_core::hand_sig`), so the
document itself records which field a mark signs. `HandSigned` holds the marks
`hand_sig::hand_signatures` last found (`SignedMark`: field, page, bounds in
PDF user space; the page's canvas overlay selects and adjusts them), the field
names derived from them, and the edit epoch it measured at; `app::handsigned::refresh` re-measures whenever the epoch moves, on the
pages carrying an unsigned `/Sig` box only. Undo, redo, save and reopen need no
bookkeeping: each changes the content, and the next measurement reads it. A
mark whose objects were all deleted leaves an empty sequence the engine does
not count, so its box shows its tag again.

