# `ui-verify/checks/style_donor`

`a_donor_face_makes_bold_real` — **a font file offered from the Properties
panel turns a synthesized Bold into the real face, embedded.**

Off-screen, scripted pointer, no OS input. On a copy of
`fixtures/subset-font-floor.pdf`, whose one run `ABC` is set in
`SUBSET+pdfceSubsetDemo` with no bold face on the page and no standard-14
sibling, so Bold forecasts synthesis. `fixtures/augment-fonts/face.ttf` is
copied to `pdfceSubsetDemo-Bold.ttf` in the output folder, a name the
engine reads as the run's family in bold, and `PDFCER_DIAG_FONT_FILE_PATH`
answers the file picker with it. The Text tool is armed and Properties
shown; a click in the run opens a caret.

Steps and oracles:

1. Before any donor, the Properties panel draws
   `properties.text.style-donor`.
2. Clicking it traces `text-style-donor-added donors=1`, and the next bold
   `text-style-forecast` says `rung=supplied-face`.
3. Clicking `properties.text.bold` traces `text-style-ladder
   rung=supplied-face`.
4. Ctrl+S saves, and the appended revision names `+pdfceSubsetDemo-Bold`,
   in plain bytes or inside a Flate stream.

Falsified: dropping the donors from the span commit in
`textstyle::span::apply` leaves the forecast at `supplied-face`, but the
press binds another rung and oracle 3 fails.
