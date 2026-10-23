# `panels::output_preview`

Acrobat's Output Preview, separations half (`OPERATOR_REQUESTS.md` O323 part 2).

## Contract

- **State** lives on `OpenDoc::separations: Option<SeparationPreview>`, not on
  `ViewState`: `ViewState` and `RenderKey` are `Copy`, and the preview holds a
  `Vec<String>`. The key carries `renderworker::key::separation_bits` — 0 for
  none, else a hash of `hidden` with the low bit set — so a change of hidden
  plates re-rasterizes at once (a discrete input) and never reuses a texture
  drawn under another set.
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
