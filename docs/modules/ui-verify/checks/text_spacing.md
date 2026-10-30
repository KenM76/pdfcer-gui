# `ui-verify/checks/text_spacing`

`text_spacing` — **sweep text, open Properties › Spacing and position, press
Superscript, and the file changes.**

## What it guards

The fold under the Text section (`panels::properties::textspacing`) carries
tracking, word spacing, width, baseline shift and the three position buttons.
Each raises `Action::TextStyle` with a `StyleChange` that `stamp` turns into
`FormatRequest::char_spacing`, `word_spacing`, `h_scale`, `rise` or `script`.
A fold that draws but whose buttons raise nothing looks identical in every
unit test. Superscript is the driven row because it needs no typed value.

## How it drives

1. It opens the text fixture from `fixture::text_point_target` with
   `PDFCER_DIAG_INVOKE=mode.edit,file.properties`.
2. It arms the text-sweep tool (`T`), drags 60 pt along the baseline and
   requires a `canvas-text-selection` line with `chars` > 0.
3. It finds the `properties.text.spacing` header, scrolling the Properties
   panel up to six notches, and clicks it open.
4. It clicks `properties.text.script.super`.
5. It polls for `text-style-applied` or `text-style-declined` (20 s ceiling).

It passes when `text-style-applied` reads `change=script` with `applied` > 0
and a `format-text` line follows.

## Falsification

With `chosen = Some(shift)` removed from `script_row`, the check fails: no
summary line follows the press.
