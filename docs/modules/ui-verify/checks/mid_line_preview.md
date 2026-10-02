# `ui-verify/checks/mid_line_preview`

`a_key_typed_mid_line_previews_where_it_commits` — a letter typed into the
middle of a line a word processor wrote in pieces is previewed with the words
after it moved over to make room, exactly where the commit puts them.

Fixture `fixtures/word-fragmented-lines.pdf`. Its first line,
`Date Premises Required____`, is one `TJ` at y=700 from x=72. The window is
off the desktop and driven by `ScriptedPointer` only, with
`PDFCER_DIAG_INVOKE=mode.edit,edit.text`, so the check runs under `--no-input`.

## Steps

1. Click inside `Date`, press `Home`, then `ArrowRight` four times.
2. Shoot the untyped line.
3. Type `X` and shoot the preview.
4. Press `Escape` (which commits), move the pointer away, and shoot the
   committed line.

## Judgement

The measure is the share of ink columns, within the line's band (PDF points
x 66–260, y 709–698), that are inked in exactly one of two images. A column
is inked when some pixel in it is dark and not accent blue, so the draft's
outline and caret are not counted.

- **Precondition (skip):** the last `text-edit-shaped` line names `chars=28`
  and `shaped=1`. Otherwise the preview never drew in the line's own face and
  there is nothing to compare.
- **Control (skip):** the untyped line against the commit must differ by at
  least 0.3. That proves the band can see a word move.
- **Fail:** the preview against the commit differs by 0.2 or more.

On the fixed build the control measures 0.42 and the preview 0.10. On the
falsified build the preview measures 0.40.

## Falsification

Forcing `canvas::textedit::splice`'s tail to be empty draws `X` over the space
and leaves `Premises Required____` unmoved until the commit. The preview then
measures 0.40, and the check fails.
