# `a_copied_picture_pastes_as_a_stamp_in_review`

**Defect it guards:** a picture copied in another program cannot be put on
the page in Review, where pasting content is refused and nothing offers it
as a stamp.

## What it drives

Off-screen, scripted pointer, on a copy of `four-pages.pdf`. The clipboard is
snapshotted, set to a 64×32 CF_DIB at 96 pixels per inch (48×24 pt), and
restored afterwards.

1. Ctrl+2 (Review). Hover the first page at `osp::FIRST` and paste.
2. Ctrl+Z.
3. Markup tab, then Markup ▸ Paste picture as stamp.
4. Ctrl+1 (Read). Hover and paste again.

## Oracles

| step | requires |
|---|---|
| 1 | a `clip-pasted kind=image as=stamp` line, 48×24 pt within 0.5, centred on the pointer (`osp::lands`), then a new `custom-stamp-placed` line |
| 2 | a new `undo-applied` line |
| 3 | a second `as=stamp` line of the same size and another `custom-stamp-placed` |
| 4 | no new `clip-pasted` line and a `command-declined id=edit.paste` |

## Falsification

Placing the picture as page content in Review (the Edit branch) traces
`as=content` and fails step 1. Dropping the `rect_at` centring fails
`osp::lands`.
