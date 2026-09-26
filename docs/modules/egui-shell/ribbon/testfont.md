# `egui-shell/ribbon/testfont`

## Item notes

### `const EXTRAS`

`…` appears in command labels ("Open…"), `⏷` is the overflow
affordance's chevron, and `�` is what `epaint` looks for as its
replacement character — supplying it keeps a "failed to find
replacement characters" warning out of the test log and gives any
unmapped character a visible, measurable width rather than nothing.

### `fn advance_for`

Deliberately irregular. See the module header: a monospaced synthetic
font would make every string's width a function of its length, which is
the one property real proportional text does not have and the one the
ribbon's arithmetic must not assume.

### `fn glyf_and_loca`

Every glyph is one closed contour: a rectangle from `x = 50` to
`x = advance − 50`, `y = 0` to `y = GLYPH_TOP`, four on-curve points,
no hinting instructions. All four coordinate flags are bare `0x01`
(`ON_CURVE`), which means the deltas that follow are signed 16-bit —
the long form, chosen because it is the one with no special cases.

Each record is 34 bytes, which is even, which is what short `loca`
(`indexToLocFormat = 0`, offsets stored halved) requires.

### `fn cmap`

Format 4 rather than the simpler format 12 because it is the one every
shaper and every parser has supported since 1996; a synthetic font is
not the place to discover which of `skrifa`'s or `harfrust`'s subtable
preferences applies.

Four real segments plus the mandatory `0xFFFF` terminator:

| Segment | Characters | Glyphs |
|---|---|---|
| 0 | U+0020 – U+007E | 1 – 95 |
| 1 | `…` | 96 |
| 2 | `⏷` | 97 |
| 3 | `�` | 98 |
| 4 | U+FFFF | 0 (required end marker) |

`idDelta` is `(glyph − character) mod 65536` and `idRangeOffset` is
zero throughout, which is the direct-mapping form of the table.

### `fn assemble`

`tables` must already be in ascending tag order — the format requires
the directory to be sorted, and parsers binary-search it. Each table's
data is padded to a four-byte boundary; the recorded length is the
*unpadded* one, as the specification says.

Checksums are written as zero. `skrifa` does not verify them, and a
test font that computed them would be spending code on the one field
nothing in this stack reads.

### `fn the_synthetic_face_loads_and_measures_real_text`

The self-test of the harness. Everything in
[`super::super::width_tests`] is worthless if this is not true, and
"worthless" here means "passing" — which is why it is asserted
rather than assumed.
