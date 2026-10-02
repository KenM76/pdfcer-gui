# `augment-subset.pdf` and `augment-fonts/face.ttf` — a subset and the face it was cut from

Copied byte for byte from `D:\Dev\pdfcer\fixtures\synthetic\text\augment\`
(`subset-in.pdf` and `face.ttf`). The engine generates both with
`tools/gen-augment-face-fixtures.py` (in `D:\Dev\pdfcer`); that script's
header is the authoritative account of how they are built.

`augment-subset.pdf`, 1,849 bytes: one page, 200 x 100 pt, showing `ABC` at
24 pt from (10, 40) in `/F0`, a simple `/TrueType` with `/WinAnsiEncoding`
and no `/ToUnicode`, embedding the subset `ABCDEF+pdfcerAugFace`. The subset
outlines only `A`, `B` and `C`.

`augment-fonts/face.ttf`, 1,020 bytes: the synthetic face `pdfcerAugFace` the
subset was cut from, outlining `A` to `E`, an acute and a composite `Eacute`,
with the same hinting programs. The outlines are polygons, not letterforms,
and the face is generated, so it carries no third-party licence.

Used by `tools/ui-verify/src/checks/augment_subset.rs`. Typing `D` must add
the glyph to the subset from `face.ttf` and commit in the line's own font.
