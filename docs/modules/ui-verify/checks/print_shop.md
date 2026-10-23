# `ui-verify/checks/print_shop`

`print_shop` — the Render diagnostics window names the page's colour findings,
and View ▸ Display ▸ Skip tiny details leaves out what is too small to see,
draws everything when off, and says on the status bar how much it left out.

# What it drives

`fixtures/print-shop.pdf` (ignores `--pdf`), built by
`fixtures/print-shop.PROVENANCE.py`, with
`PDFCER_DIAG_INVOKE=mode.edit,tools.render_diagnostics`.

1. `render-findings` (traced by the Render diagnostics window): every engine
   divergence counter as `key=count`. Owed: `color.spaces_unresolved=1`,
   `shading.refused=1`, `color.patterns_unpainted=1` (a `/Pattern` fill naming
   no pattern the page holds), and `shown` at least 3, so all three reached
   the list as sentences.
2. `canvas-tiny skipped=0 mode=0` before the toggle: off skips nothing.
3. Scripted clicks on `ribbon.tab.view`, then
   `ribbon.item.view.skip_tiny_details`.
4. `canvas-tiny skipped=400 mode=1` after it: each of the fixture's 400 forms
   is 0.2 pt square, under half a device pixel at fit.
5. `status-group:tiny-details` declared: the canvas no longer shows what will
   print, and says so.

# Falsified

- Removing `ink::findings` from `notes::findings` drops `shown` below 3 and
  fails step 1.
- A fixture without the unresolvable pattern fill reads
  `color.patterns_unpainted=0 shown=2` and fails step 1.
- Not copying `skip_tiny_details` into the render request leaves `skipped=0`
  after the press and fails step 4.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.

# `ink_picker`, in the same file

Tools ▸ Diagnostics ▸ Ink picker reads the inks at a click. Same fixture,
`PDFCER_DIAG_INVOKE=mode.edit` (the Tools tab is an Edit-mode tab).

1. Scripted clicks on `ribbon.tab.tools`, then `ribbon.item.tools.ink_picker`;
   owed: `ink-picker armed=true`.
2. A click at page point (320, 110), cyan alone: owed `ink-probe source=ink
   c=1.000 m=0.000`, then `ink-picker-shown kind=inks` (the tool strip showed
   the reading).
3. A click at (400, 152), the overprinted magenta over the cyan: owed
   `source=ink c=1.000 m=1.000`. A knocked-out overprint reads m=1 c=0.

Falsified: dropping `with_ink_probe` from the options fails step 2 with
`source=failed`; mislabelling an ink reading as `screen` fails step 2's
`kind=inks`.

# `overprint_preview`, in the same file

View ▸ Display ▸ Overprint preview starts on, and turning it off reaches the
render and the status bar. Same fixture; its magenta square sets `/OP true`
over cyan, so the engine reports one overprint request.

1. Owed with no gesture: `canvas-overprint mode=1` with `requested>0` and
   `effective>0` — on by default, and simulation changed the picture.
2. Scripted click on `ribbon.tab.view`; `ribbon.item.view.overprint_preview`
   must then be declared (a FAIL naming the declared items if not).
3. Click it; owed `canvas-overprint mode=0` with `requested>0` and
   `effective=0` — the engine's own counter says nothing was simulated, which
   is the pixel oracle an off-screen window cannot photograph.
4. `status-group:overprint` is declared: the status bar says the canvas no
   longer shows what will print.

Falsified: the build before the switch emits no `canvas-overprint` line and
fails step 1.

# `output_preview`, in the same file

View ▸ Panels ▸ Output Preview lists the page's inks and hides one. Same
fixture, whose page uses only the four process inks.

1. Scripted click on `ribbon.tab.view`; `ribbon.item.view.output_preview`
   must be declared (a FAIL naming the declared items if not). Click it.
2. Click `output-preview.on`; owed `output-preview on=1` whose `inks` holds
   Cyan, Magenta, Yellow and Black. The panel emits it only from a texture
   drawn under the current preview, so the line is the engine's plate list.
3. Click `output-preview.ink:Magenta`; owed `hidden=Magenta` — again only from
   a texture drawn under that preview, so the render was redone.
4. `status-group:output-preview` is declared.

Falsified: the build before the panel has no `view.output_preview` and fails
step 1.

## `output_coverage`

Opens View ▸ Panels ▸ Output Preview, switches the preview on, then — after
the dock has opened, so the canvas mapping is final — rests the scripted
pointer on the two `AIMS` points and waits for an
`output-preview-coverage … inks=` line at each. Owed: `Cyan:1.000` and
`Magenta:0.000` on cyan alone, both `1.000` on the overprinted magenta, and a
`total` at least their sum. Falsified on a control build with the canvas call
removed: it fails with `output-preview-coverage none`.
