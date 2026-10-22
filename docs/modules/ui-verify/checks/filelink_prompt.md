# `ui-verify/checks/filelink_prompt`

`a_file_link_asks_or_opens_here` — **a `/Launch` link raises the Launch File
prompt and honours its answer, and a `/GoToR` opens the PDF it names here, at
the place it names.**

## What it drives

Writes `payload.exe` (a few bytes, never run) and a copy of the fixture as
`other.pdf` into the run's output folder, beside the opened copy of
`fixtures/non-navigation-links.pdf`, and launches with `PDFCER_DIAG_NO_LAUNCH`
so a hand-off to Windows is traced, not performed. Clicks through the scripted
pointer, so it runs under `--no-input`.

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | the `/Launch` click traces `launch-decision decision=ask` naming `payload.exe`, and `launch.body` is drawn | the link only says where it points, or opens without asking |
| 2 | Do Not Open traces `launch-answer open=false dont_ask=false` and no `external-open` | refusing opened the file |
| 3 | a second click asks again; *Do not show this message again* + Open traces `open=true dont_ask=true` and `external-open kind=file` naming `payload.exe` | the answer or the hand-off is lost |
| 4 | a third click traces `decision=open`, draws no prompt, and hands the file over | the checkbox did not turn asking off |
| 5 | the `/GoToR` click traces `filelink-open page=2` with a path ending `other.pdf` | the PDF did not open here, or `/D (tree-target)` was not resolved in the opened file's names (it names that file's third page) |
