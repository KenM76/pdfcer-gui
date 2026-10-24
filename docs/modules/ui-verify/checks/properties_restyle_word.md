# `ui-verify/checks/properties_restyle_word`

`properties_italic_restyles_the_word_at_the_caret` — O308, driven off the
desktop through the scripted pointer, on `fixtures/paragraph.pdf`.

The operator selected characters inside a block with the I-beam and found
Bold, Italic, Underline and Font did nothing. The ribbon's Bold is covered by
`ctrl_b_bolds_the_word_at_the_caret`; this check covers the Properties panel,
which is a separate route (`app::dispatch::textformat::restyle_or_runs`).

It launches with the Text tool armed and Properties shown, clicks a caret into
the word `drawing`, presses the panel's Italic and requires:

1. the panel draws `properties.text.italic` while the caret is in the block;
2. no `text-style-declined` and no `text-style-applied` (the whole-run
   route) after the press;
3. a `text-span-style-applied` with `applied` non-zero and the word still
   selected (`reselected=true`).

`change=weight` in that line is right: Italic is a weight-and-slant change.

## Falsification

| control | expected verdict |
|---|---|
| the build before `textoperand::caret` existed (the panel drew no Text section during an edit) | FAIL at step 1 (driven) |
