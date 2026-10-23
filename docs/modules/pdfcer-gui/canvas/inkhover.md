# `canvas::inkhover`

The inks under a resting pointer, for the Output Preview panel
(`OPERATOR_REQUESTS.md` O323).

## Contract

- Called once per canvas frame from `canvas::interact` with the hovered
  page-space point, or `None` off the canvas. Does nothing while
  `OpenDoc::separations` is `None`, and clears its reading.
- **Rest, then read.** A probe starts after the pointer has stayed within
  1.5 screen pixels for 0.12 s; any move cancels the probe in flight. One
  region render interprets the whole content stream (seconds on a dense
  sheet), so probing every frame of a sweep would queue that work.
- The probe is `canvas::inkpick::prepare` — the picker's 3 × 3 device-pixel
  region and settings-funnel options — composited under the document's
  `SeparationPreview`, so an RGB page splits into the same CMYK plates the
  panel lists. The engine samples before hiding plates, so hidden plates keep
  their numbers.
- A reading is valid for one document serial, page and `edit_epoch`.
- `Tints` maps a plate name to its tint: process names index
  `InkProbe::cmyk` through `pdfcer_render::PROCESS_INKS`; anything else is
  looked up in `InkProbe::spots`. `total` is their sum, `None` without an ink
  buffer.

## Trace

`ink-hover page= x= y= source= c= m= y= k= alpha= spots= srgb=` per finished
probe (the picker's format), `ink-hover-cleared` when the reading is dropped.

Driven by `output_coverage` (`docs/modules/ui-verify/checks/print_shop.md`).
