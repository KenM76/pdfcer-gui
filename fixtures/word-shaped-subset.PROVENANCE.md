# `word-shaped-subset.pdf` — a Word-shaped subset whose program outlines more than it shows

Copied byte for byte from `D:\Dev\pdfcer\fixtures\synthetic\text\word-shaped-subset.pdf`.
The engine generates it with `tools/gen-word-subset-fixture.py` (in
`D:\Dev\pdfcer`). That script's header is the authoritative account of how
it is built.

2,075 bytes. Two pages, 612 x 792 pt. Page 1 shows `ABC` at 24 pt from
(72, 600); page 2 shows `A`. One font, `/F0`: a simple `/TrueType` with
`/WinAnsiEncoding`, no `/ToUnicode`, and `/Widths` covering only codes 65..67.
Its program also outlines `D` and U+2013, and leaves `E` as an empty slot. The
outlines are rectangles, not letterforms.

Used by `tools/ui-verify/src/checks/subset_glyph.rs`. Typing `D` must commit:
the program outlines it, and the shell passes the engine its embedded-program
reader.
