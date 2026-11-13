# `panels::output_preview`

Acrobat's Output Preview, separations half (`OPERATOR_REQUESTS.md` O323 part 2).

## Contract

- **State** lives on `OpenDoc::separations: Option<SeparationPreview>`, not on
  `ViewState`: `ViewState` and `RenderKey` are `Copy`, and the preview holds a
  `Vec<String>`. The key carries `renderworker::key::separation_bits` — 0 for
  none, else a hash of `hidden` with the low bit set — so a change of hidden
  plates re-rasterizes at once (a discrete input) and never reuses a texture
  drawn under another set. With a soft proof
  attached the key also mixes in a digest of the profile bytes and both
  switches.
- **Canvas only.** Print, export and thumbnails pass `None`. Never saved, and
  dropped with the document.
- **One action**, `Action::SetSeparations`, raised by the switch, each ink
  box, and *Show all inks*. The apply arm traces
  `output-preview-set on= hidden=`.

## Why the panel waits for a texture drawn under the preview

The ink list is `Diagnostics::inks`, which the engine fills only for a render
under a preview. Reading it from the texture on screen after a click would
list the previous preview's plates, or none. `RenderKey::drawn_under` gates
the list, the disclosures and the `output-preview` trace line, so the trace is
also the driven proof the render was redone.

## Disclosure

Off-canvas only (R8b): `inks_forced_subtractive` and `inks_unmatched` on the
panel; `status-group:output-preview` in the status bar while the preview is
on, naming how many inks are hidden. The canvas is drawn as the engine
returns it — no marking.

## Trace

`output-preview on={0|1} inks=a|b|… hidden=…` on change (`-` for none,
spaces as `_`). Regions: `panel:output-preview`, `output-preview.on`,
`output-preview.ink:<name>`, `output-preview.show-all`.

## Inks under the pointer

While the preview is on and the pointer rests on the page, each ink row shows
its tint there as a percentage and a line beneath gives where (page points,
lower-left origin) and the total area coverage — the sum of every plate,
Acrobat's figure for a printer's ink limit. The reading comes from
`canvas::inkhover`; moving the pointer off the canvas (to the panel, say)
clears it, as Acrobat's does.

Trace: `output-preview-coverage` on change — `none`, `reading`, `failed`,
`off_page`, `no_inks`, or `page= x= y= inks=Cyan:1.000|… total=`.

## Overprint

`output-preview.overprint` says whether the page sets overprint and whether
it changes the print, from `PageOverprint::from_diagnostics` on the render
under the preview (which always simulates overprint). A region render answers
for the part drawn and says so. Trace: `output-preview-overprint set= effect=
whole=`.

## Simulation profile (`output_preview::proof`)

A *Simulate* list under the inks: *None*, the document's output intent when
it has one (`SoftProof::from_output_intent`), and every installed CMYK
printer-class profile (`icc_profiles::installed_profiles`). The choice rides
`SeparationPreview::soft_proof` through the same `Action::SetSeparations`, so
hiding an ink keeps the proof and the proof keeps the hidden inks.

- *Simulate paper colour* implies *Simulate black ink*, as in Acrobat.
- `ProofUi` (on `PanelsState`, carried across `forget_document`) reads each
  profile once and remembers per document serial whether it has an output
  intent. That answer is not re-read after an edit to the document's output
  intents within one open.
- Disclosure, on the panel: a refused profile in the engine's words; no black
  point compensation (both switches off then looks like black ink); spot inks
  on the page are not proofed. The status line adds that colours simulate a
  press profile.

Trace: `output-preview-proof source=<none|intent|installed|other> paper= black=
bpc=`. Regions: `output-preview.proof`, `output-preview.proof:<none|intent|i>`,
`output-preview.paper`, `output-preview.black-ink`.

## Coverage warnings (`output_preview::warnings`, `canvas::coverage`)

*Total area coverage over* (default 300%, up to 400) and *Rich black from black
at* (default 95%), each a switch and a threshold, under the simulation profile.
The thresholds are kept while a rule is off (`WarnUi`, carried across
`forget_document`); the rules switched on live in `canvas::coverage`, which the
panel sets with `coverage::set`.

- `canvas::coverage` checks the visible part of the current page at screen
  density with `pdfcer_render::coverage_warning_masks`, one render for both rules, on a
  thread, once the view has held still for 0.15 s; the overlay shown stays
  until its replacement arrives. Region and scale come from
  `canvas::ocrink::visible_raster`, so both overlays render the same region.
- The mask is rendered under the preview's hidden plates and without its soft
  proof: a proof changes the screen colour, not the ink. Rich black is painted
  over total area coverage, in the colour beside each rule's switch, at the
  *Warning opacity* shown while a rule is on (10–100%, default 100). Opacity
  is a paint setting, so changing it does not re-check.
- The panel says per rule *found on screen*, *none on screen*, *checking* or
  *could not be checked*: an answer for the part of the page drawn, as the
  overlay is. A refused check is not retried until the document, page or
  rules change.
- With a rule on, the status line adds that coverage warnings are painted
  over the page. The overlay goes with the preview: switching it off clears
  them.

Trace: `output-preview-warn tac=<n|-> rich=<n|-> opacity=<percent>` then `found=<checking|refused>`
or `found_tac=<0|1|-> found_rich=<0|1|->`; `coverage-mask page= tac= rich=
tac_box= rich_box= px= ms=` per finished check (flagged pixel counts; boxes in
canvas points, `-` for none); `coverage-mask-refused`. Regions:
`output-preview.tac`, `output-preview.rich-black`, `output-preview.warn-opacity`,
`coverage-overlay`.
