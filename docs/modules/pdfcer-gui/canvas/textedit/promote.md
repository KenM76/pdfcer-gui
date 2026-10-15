# `canvas::textedit::promote`

A run draft edits one show operator, and a show operator holds no line break.
Enter in a run draft therefore opens the run's **paragraph**: `open` re-anchors
the draft as `Anchor::Block`, whose commit is `EditSession::edit_block_text`,
and the keystroke then inserts its line break as in any multi-line draft.
`widen` does the same on a click: a run in a paragraph of two or more lines
opens as the paragraph, so text written as several lines re-opens as one
draft.

## The paragraph

The paragraph is the one `edit_block_text` will be handed: recognised with
`EditableTextModel::recognize_with_cells(&text, &reflow_recognition_options(),
&cells)` over the provenance extraction, as `reflow::block_of_run` does, and
found with `block_at(TextPosition::new(run, 0))`. Its text is each line's
glyph slices concatenated, lines joined as `line_ends` says.

## Breaks and wraps

The model is built `with_line_marks(LineMarks::scan(..))` over the page's
content, and `EditableTextModel::line_ends` says of each line end whether it
is a `Break` (joined with `\n`) or a `Wrap` (joined with a space).
`line_end_source` says how it knows: `Marked` when the block carries the
break marks `edit_block_text` writes, so a paragraph pdfcer has rewritten
keeps every hand break exactly; `Inferred` otherwise (Add text writes no
marks; nor does any other producer), when the engine judges a break from
whether the next line's first word would have fitted. An inferred paragraph
raises the `breaks-inferred` draft note, from `place` on a click and from
`edits` on Enter, because a break typed after a nearly full line reads as a
wrap and the next edit would merge the two lines.

A marked blank line arrives as one `Break`: `line_ends` carries no count (G183), so a
re-open keeps one line break where there were two.

While walking it, `open` records where the run's characters sit. The draft's
text replaces exactly that slice, so the operator's edits to the line so far
survive and the caret and selection move by the length of the text before it.
The draft's undo history is re-based the same way (`DraftHistory::embed`), so
`Ctrl+Z` after the break restores the paragraph with the line as it was, never
a line-only text inside a paragraph draft.

## When it declines

`open` leaves the draft untouched and answers an `EnterRefusal`:

- **`NoParagraph`** — no text, unreadable cells, no paragraph at the run, the
  run's characters split across lines or interleaved with another run's, or
  the slice no longer matches the run's original text.
- **`Unrewritable`** — `edit_block_text_preview` refuses the text the break
  would produce (the same refusals the commit would raise); the engine's
  sentence is carried.

Each decline traces `text-edit-enter-declined page= run= reason=` with one of
`no-text`, `no-cells`, `no-paragraph`, `run-not-in-paragraph`, `run-split`,
`run-text-differs`, `engine-refuses`. A promotion traces
`text-edit-promoted page= run= block= len= caret= source= looks=`.

A paragraph of several looks opens like any other. `edit_block_text` keeps
each surviving character's look, gives an inserted one its neighbour's, and
discloses when it fell back to the first run's; that disclosure reaches the
status row through `app::actions::blocktext`. `looks=` counts (font resource,
`Tf` size, fill colour) from provenance, an unset fill read as the black
`DeviceGray 0` it renders as (the rewrite writes a colour only where the
inherited one differs), a subset of the engine's
`BlockEditReport::looks`, so a driven check can tell a mixed paragraph from a
uniform one.

`widen` declines the same way, plus `one-line` for a paragraph of one line
and `recognised-word` for an invisible (OCR) run, and leaves the run draft. A
recognised run (a line, or a word on a page an older pdfcer recognised) is
edited as its own run so its preview keeps the run's size and scale, as the
OCR layer (`canvas::ocrink`) renders it. It traces `text-edit-widened page= run= block=
lines= breaks= len= source= ms= looks=` or `text-edit-widen-declined page= run= reason= ms=`.

## Options

`options(doc)` is the `BlockEditOptions` both the preflight and the commit
use: the typing disposition with the installed faces
(`installed::augmented`), no fallback or replacement face, which
`edit_block_text` forbids, and the engine's own wrap width. The engine fits a
line within `FIT_TOLERANCE`, so a paragraph rewritten with its own text keeps
its widest line whole (G162); in a table cell it keeps the cell's inner width.

## Glyph places

The walk records, for each glyph of the opened paragraph, its byte span in
the paragraph's text and its `TextPosition` on the page; they ride on
`Anchor::Block` as `places`. `position(places, byte, end)` maps a byte of the
paragraph's text to the page on a glyph boundary — a start rounds back into
the glyph it falls in or the next one, an end forward past the glyph it falls
in or the previous one — so a range crossing a wrapped line ends on one run
and starts on the next. The line separators have no glyph. The map describes
the text as opened; once the draft's text differs from `original` it no longer
holds, and the Format tab's restyle refuses with `ParagraphEdited` rather than
guess.
