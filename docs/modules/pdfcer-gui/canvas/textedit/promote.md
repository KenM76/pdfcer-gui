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
glyph slices concatenated, lines joined by `joint`.

## Breaks and wraps

The model does not say which line ends were typed (G163). `joint` decides
each: a `
` when the next line's first word, after the paragraph's widest
space glyph, would have fitted on the line within the wrap width; otherwise a
space. `edit_block_text` packs greedily, so a space there would pull the word
up and merge two lines broken by hand, and anywhere else a space reproduces
the wrap. A break typed after a line already too full for the next word reads
as a wrap; an edit that then shortens the line merges the two.

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
- **`MixedLooks`** — the paragraph's glyphs carry more than one look. The
  rewrite sets every character in the first run's font, size and colour, which
  would silently flatten the others. The look compared is (font resource, `Tf`
  size, fill colour) from provenance, a subset of the engine's own `same_look`,
  which is not public. This is a workaround, reported to the engine.
- **`Unrewritable`** — `edit_block_text_preview` refuses the text the break
  would produce (the same refusals the commit would raise); the engine's
  sentence is carried.

Each decline traces `text-edit-enter-declined page= run= reason=` with one of
`no-text`, `no-cells`, `no-paragraph`, `run-not-in-paragraph`, `run-split`,
`run-text-differs`, `mixed-looks`, `engine-refuses`. A promotion traces
`text-edit-promoted page= run= block= len= caret=`.

`widen` declines the same way, plus `one-line` for a paragraph of one line,
and leaves the run draft. It traces `text-edit-widened page= run= block=
lines= breaks= len= ms=` or `text-edit-widen-declined page= run= reason= ms=`.

## Options

`options(doc, wrap)` is the `BlockEditOptions` both the preflight and the
commit use: the typing disposition with the installed faces
(`installed::augmented`), no fallback or replacement face, which
`edit_block_text` forbids, and the wrap width.

The wrap is the paragraph's box width plus `WRAP_SLACK` (0.5 pt). The engine's
default is the box width itself, against which the box's own widest line can
measure over and split (G162). In a table cell the wrap is `None`, so the
engine keeps the cell's inner width, which an explicit width would override.
`Anchor::Block` carries the wrap to `TextAction::CommitBlock`.
