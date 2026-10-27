# `ui-verify/checks/rich_copy`

`a_copied_paragraph_pastes_as_one_formatted_paragraph` — in Read mode, four
clicks select the six-line paragraph of `paragraph.pdf`, and Copy (the scripted
pointer's `copy` event, which egui receives for Ctrl+C) places the selection as
formatted text. It runs under `--no-input`, and `PDFCER_DIAG_CLIPBOARD_DIR`
sends the placement to `rich-copy-capture/` so the operator's clipboard is
untouched.

## Steps

1. Launch `paragraph.pdf` in Read mode, off the desktop.
2. Click four times inside line 2 (page point 120, 668). Require a
   `canvas-text-selection via=paragraph` line. If it is missing, the run is an
   error, not a failure, because `a_fourth_click_selects_the_paragraph` owns
   that defect.
3. Send Copy and judge.

## Judging

For that copy:

- There is no `text-copy-declined` line, and there is a `text-copied` line.
- The capture holds `1-HTML_Format.bin`, `2-Rich_Text_Format.bin` and
  `3-CF_UNICODETEXT.bin`, which is the placement order.
- The HTML starts with the CF_HTML `Version:` header, and the RTF starts with
  `{\rtf1`.
- The trace line says `paragraphs=1`, and the UTF-16 text contains
  `of a sheet in one file`. That is the end of line 1 joined to the start of
  line 2 by one space.
- Both formatted captures name the page's font, `Helvetica`.

A plain-text copy fails twice over. It places only `CF_UNICODETEXT`, so the
first two captures are missing. It also keeps a line break where the lines are
drawn, so the joined phrase is absent.

## Falsification

Driven against the v0.98 build, which copies plain text, the check fails on the
missing captures.
