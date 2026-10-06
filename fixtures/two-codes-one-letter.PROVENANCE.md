# `two-codes-one-letter.pdf` — a font whose character map gives one letter two codes

Copied byte for byte from
`D:\Dev\pdfcer\fixtures\synthetic\text\cidfonttype2-partially-injective-tounicode.pdf`
(2,216 bytes, SHA-256 `4068be45cd8c4709ff020500f734bf289c62e740cec74850ffe75075411b443e`).
The engine generates it with `cidfont_partially_injective_tounicode` in
`tools/gen-cidfont-nocmap-fixtures.py` (in `D:\Dev\pdfcer`); that function is
the authoritative account of how it is built.

One page, 612 x 792 pt. `/F0` is a `/Type0` font over the embedded
`/CIDFontType2` subset, `/Identity-H`. Its `/ToUnicode`
maps codes `<0001>` and `<0002>` to `A` and `<0003>` to `B`. The page shows
`<0001>` (`A`) at 48 pt from (72, 600) and `<0003>` (`B`) from (72, 540), each
in its own show operator. The outlines are generated boxes, so the file
carries no third-party licence.

Used by `tools/ui-verify/src/checks/two_codes.rs`: an edit to the `A` is
refused for the `A`, and the `B` still edits.
