# `ui-verify/checks/print_shop`

`print_shop` — the Render diagnostics window names the page's colour findings,
and View ▸ Display ▸ Skip tiny details leaves out what is too small to see,
draws everything when off, and says on the status bar how much it left out.

# What it drives

`fixtures/print-shop.pdf` (ignores `--pdf`), built by
`fixtures/print-shop.PROVENANCE.py`, with
`PDFCER_DIAG_INVOKE=mode.edit,tools.render_diagnostics`.

1. `render-findings` (traced by the Render diagnostics window): the roster's
   `key=count` pairs. Owed: `cs_unresolved=1`, `shadings_refused=1`, and
   `shown` at least 2, so both reached the list as sentences.
2. `canvas-tiny skipped=0 mode=0` before the toggle: off skips nothing.
3. Scripted clicks on `ribbon.tab.view`, then
   `ribbon.item.view.skip_tiny_details`.
4. `canvas-tiny skipped=400 mode=1` after it: each of the fixture's 400 forms
   is 0.2 pt square, under half a device pixel at fit.
5. `status-group:tiny-details` declared: the canvas no longer shows what will
   print, and says so.

# Falsified

- Removing `ink::findings` from `notes::findings` drops `shown` below 2 and
  fails step 1.
- Not copying `skip_tiny_details` into the render request leaves `skipped=0`
  after the press and fails step 4.

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
