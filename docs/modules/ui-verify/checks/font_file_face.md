# `ui-verify/checks/font_file_face`

`a_font_file_restyles_swept_text` — **sweep text, press Font file…, pick a
font file, and the text is redrawn in that face.**

## What it drives

1. `paragraph.pdf` (pinned by `fixture::text_point_target`), Edit mode,
   Properties open.
2. `T`, then a 60 pt sweep along the first line.
3. Scrolls Properties until `properties.text.font-file` is on screen, and
   captures the swept text box.
4. Presses the button. `PDFCER_DIAG_FONT_FILE_PATH` answers the picker with
   `C:\Windows\Fonts\comic.ttf`, so no native dialog opens.

## What it asserts

| link | evidence |
|---|---|
| the press asked for a file | `font-file-picked source=env` |
| the plan was built and the engine accepted it | `text-style-applied … change=face-file applied>=1`, a `format-text` line, and no `text-style-declined` |
| the page now draws another face | at least 2% of the swept box's pixels differ between the captures before and after |

The pixel assertion is the one no trace can stand in for. The engine can
accept an embedded plan and the page can still draw the old face, if the
canvas never re-renders or the renderer ignores the new resource. Comic Sans
was picked because every Windows install carries it and none of its glyphs
match the fixture's Helvetica. The selection highlight is present in both
captures, so it cancels out.

## Falsified

With the constant pointed at `fixtures/paragraph.pdf`, the check fails on
`text-style-declined … font-file=… detail=this file is …`. That is
`plan_subset` refusing a non-font file, and it proves the decline branch
works too.

## Skips

- `comic.ttf` absent: the machine's precondition, not the program's.
- The sweep selected nothing: the harness's aim, not the program's.
- No Text section drawn: `restyle_text` owns that defect.
