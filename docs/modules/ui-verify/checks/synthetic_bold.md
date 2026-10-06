# `ui-verify/checks/synthetic_bold`

`bold_drawn_by_a_stroke_reads_as_bold` — **text made bold by a stroke shows
Bold pressed, and Ctrl+B takes the bold off.**

Off-screen, scripted pointer, no OS input. On a copy of
`fixtures/synthetic-bold.pdf`: one line of Helvetica 12 pt drawn in render
mode 2 with a 0.264 pt line width, the ratio pdfcer writes for a synthesized
bold. A click inside the first word opens a caret, the Format tab is raised,
then Ctrl+B.

Oracles:

- The last `ribbon-item-selected id=format.bold` (shell trace) before Ctrl+B
  says `selected=1`.
- The `text-style-ladder` line after Ctrl+B carries `removed=bold`.

The read behind both is `canvas::textedit::weight::at`, which hands the
glyph's provenance, line width included, to `synth::detect_at`.

Falsified: passing a line width of 0 to `synth::detect` instead leaves Bold
not drawn pressed, and Ctrl+B asks for bold (`rung=standard-14
requested=bold`); both oracles fail.
