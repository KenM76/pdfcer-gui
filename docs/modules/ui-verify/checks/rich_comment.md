# `ui-verify/checks/rich_comment`

`a_formatted_comment_says_it_is_formatted` — a sticky note whose body is also
stored as rich text (`/RC`, ISO 32000-1 12.7.3.4) shows a line in the Comments
panel naming that formatting. The panel shows the plain `/Contents`, so without
the line a formatted note reads flat and nothing says editing it drops the
formatting.

# What it drives

`fixtures/rich-comment.pdf` (ignores `--pdf`; built by
`rich-comment.PROVENANCE.py`): one page, two `/Text` notes covering both forms
12.7.3.4 permits.

- Object 5: `/RC` as a string — "Check the **weld** <red>size</red>" — with
  `/DS (font: 12pt Helvetica)`.
- Object 6: `/RC 7 0 R`, a stream holding an italic paragraph.

Launched with `PDFCER_DIAG_INVOKE=mode.review,markup.comments`, so the panel
is the front tab of its stack and draws.

1. Owed: the last `comments-panel` census carries `rich=2`.
2. Owed per note: a `comment-rich-note id=N state=formatted words="…"` line
   whose comma-separated words include, for 5, `bold`, `12 pt`, `Helvetica`
   and `#FF0000`, and for 6, `italic`. The words come from the same function
   that writes the operator's line (`text::richtext::formatting`).

# Falsified

- With the row's call to `panels::comments::rich::line` removed, step 2 fails:
  no line for note 5.
- With the census counting rows *without* `/RC`, step 1 fails: `rich=0`.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`; it runs under `--no-input`.
