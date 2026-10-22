# `resubset-pair.pdf` — one face embedded twice, as a Word document holds it

Copied byte for byte from `D:\Dev\pdfcer\fixtures\synthetic\resubset\pair.pdf`.
The engine generates it with `tools/gen-resubset-fixtures.py` (in
`D:\Dev\pdfcer`); that script's header is the authoritative account of how it
is built.

3,662 bytes, one page. The synthetic face `pdfcerAugFace` (the one in
`augment-fonts/face.ttf`) is embedded twice under two subset tags: a simple
TrueType font showing `A`, and a Type0 / Identity-H font over a CIDFontType2
showing glyph 3 (`C`), each with its own descriptor and its own copy of the
program. Cutting the fonts with merging leaves one program holding `.notdef`,
`A` and `C`.

Used by `dialogs::recompress`'s tests and by
`tools/ui-verify/src/checks/compact_fonts.rs`: switching on "Cut embedded
fonts down to the characters used" must cut one program, merge the other copy
into it, say so, and save that copy.
