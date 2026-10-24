# `ui-verify/checks/word_gap_band`

`word_gap_band_re_extracts_the_page` — O312, driven off the desktop through
the scripted pointer.

Edit ▸ Content carries an *Auto* switch and a *Gap width* slider for where
extracted text gets its spaces. A control that saves the setting and leaves
the page's text as it was is the defect: a click on the page still splits
words the old way. So the check reads what the **page's text** was built
under, not what the setting says.

The band traces `word-gap-band auto= ratio= page=` whenever its state
changes: the switch and slider (`Settings::word_gap_auto`,
`Settings::word_gap_ratio`), and the threshold the current page's text used
with whether it was measured (`PageText::word_gap_ratio`,
`word_gap_measured`), or `unbuilt`.

## Phases

Each phase presses its control, then clicks the third line of text so the
page's text is built, and polls until the band answers.

| Phase | Does | Expected |
|---|---|---|
| A | launch in Edit text mode, open the Edit tab | Auto off (the default), the page built at exactly the fixed width |
| B | click the slider's track a quarter along | a different width, and the page rebuilt at it |
| C | press the Auto switch | Auto on, the page measured |
| D | press the Auto switch again | Auto off, the page rebuilt at the fixed width |

The slider is clicked on its track, `TRACK_AT` points in from the declared
rect's left edge: the rect also holds the value box, and its centre lands
there, where a click opens typing instead of moving the knob.

The Content group is opened first when the narrow off-desktop window has
collapsed it. Each launch has its own profile directory, so the settings this
writes reach no other check.

## The fixture

`fixtures/word-gap.pdf` separates its words with `TJ` displacements of 0.3 em
and no space glyphs. The engine's Auto threshold needs a valley in the page's
glyph-gap ratios; a page written with space glyphs has none, keeps the fixed
width and reports `measured=false`, which phase C could not tell from Auto
never being applied. This page measures 0.150 em, distinct from the fixed
default and from phase B's width. The generator,
`fixtures/word-gap.PROVENANCE.py`, states the arithmetic.

## Falsification

| control | expected verdict |
|---|---|
| `built_under` requires `measured` false under Auto | FAIL at phase C (driven) |
