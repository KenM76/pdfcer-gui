# `pdfcer-gui/canvas/textedit/shapedblock`

An open paragraph (`Anchor::Block`) drawn in place, in each run's own font and
colour, as the commit will write it (O295). It fills the same `Shaped` that
`shaped` draws for a single run, so painting, the cover, the caret and the
cache are shared.

## Contract

- **Layout.** `EditSession::edit_block_text_preview(page, block, text,
  &promote::options(doc))` is the rewrite `edit_block_text` would commit,
  without the splice. Each of its runs, one per look, is outlined by
  `pdfcer_render::edit_preview::preview_outlines` with the page renderer's
  `FontEnvironment`.
- **Reading order.** Glyphs from every run are pooled, assigned to the line
  whose baseline is nearest, and sorted by (line, x). The draft's characters
  are then paired with them in order.
- **Caret stops: one per character, then one after the last.**
  - A character whose glyph is next takes that glyph's origin.
  - A whitespace character with no glyph (the space where a line wraps, a
    typed break, a collapsed second space) takes the next glyph's origin if
    that glyph is on the same line, else the end of its line. The end is the
    line's origin plus its natural width, or its last glyph origin if
    further.
  - A `\n` moves the following characters to the next line; a trailing `\n`
    puts the last stop at the next line's start.
  - Any other unmatched character, or glyphs left over, is `Unpaired`.
- **Ink.** `inks` carries each glyph's run colour; `ink` and `invisible` come
  from the first run. `up` is the largest em vector of any glyph.
- **Bounds.** `bbox` is the union of the new and old block boxes, so the
  cover hides the text being replaced even when the rewrite is shorter.
- **Fallbacks,** each to the shell-font box with its status sentence: empty
  text or a pairing failure is `Unpaired`; the engine refusing is `Refused`;
  a run with unsupported font machinery is `NoOutlines`.
- `shaped::refresh` calls this under `diag::muted`, so the preview's engine
  traces do not stand in for the commit's.

## Verified by

`an_open_paragraph_shows_each_words_look` (the red bold word stays red while
the paragraph is open) and `enter_breaks_a_paragraph_on_the_page`.
