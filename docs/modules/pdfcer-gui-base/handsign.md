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

## The ledger

The engine writes the signature as ordinary content and records nothing tying
it to the field (request **G073**). `Ledger` reconstructs *"is field F
hand-signed?"* from undo depths:

- `placed(field, depth)` after the commit: the entry is live at that depth.
- `undone(depth)` / `redone(depth)` after a history step: an entry is applied
  iff its depth `≤` the current undo depth.
- `reconcile(redo_depth)` with `redo_depth == 0` drops every unapplied entry,
  because a new edit has cleared the redo stack and that placement can never
  come back. Run every frame and before each history step, which is what
  separates a redo from a new edit at the same depth.

Known limits. Past the engine's undo cap (256) every new commit evicts the
oldest entry and the depth stops rising, so depths stop naming commands: undoing
a later edit there can mark a placement undone while its ink is still on the
page. The failure is a *sign here* tag shown on a box that is signed, never a
signature hidden or removed. After a save and reopen every box shows its tag
again until G073 lands.
