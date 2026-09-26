# `dialogs::print::preview` — a zoomable, pannable picture of the real sheet

## Why a picture rather than a number

pdfcer diverges from Acrobat here on purpose: Acrobat clips silently when
content falls outside the printable area, and pdfcer says so. That
divergence is worth nothing if the GUI reduces it to a count an operator can
look past. The whole reason `pdfcer-print` reads real device geometry
instead of guessing a bounding box is so this picture can be exact — drawing
only the SHEET and not the PRINTABLE AREA would show a page fitting that
will not.

**And the geometry is only half of it.** A preview that draws the correct
rectangles and fills the placed one flat answers *"where will the page
sit"* and never *"what is on it"*, which is the half an operator checking a
margin actually needs. The page content at (4) below is not decoration.

## What is drawn, outermost first

Four rectangles and one hatch, in this order, because each is *inside* the
previous and the nesting is the information:

1. **The sheet** — `DeviceGeometry::physical_pt`, the whole piece of paper.
2. **The printable area** — inset by the driver's own unprintable margins.
   This, not the sheet, is what constrains the job. A preview that showed
   only the sheet would show pages fitting that the hardware will crop.
3. **The placed page** — `Placement`'s offset and scale, applied *within*
   the printable area.
4. **The page's real content**, rendered through the same options the
   spooler uses (see [`super::render_options`]) and drawn into (3).
5. **A hatch over what will be lost** — over the part of the overhang that
   actually carries ink, and over nothing else. Hatched rather than filled:
   a hatch means *"this will happen and has not happened yet"*, which is
   exactly a pre-print clip. A solid fill reads as something already done.

## (5) is ink-aware — operator request O113

`Placement::clipped` is a *geometric* verdict: the page box exceeds the
printable rectangle. Hatching the whole overhang on the strength of it
shouts about losing something on every 1:1 CAD drawing while nothing is
being lost — *"the area that isn't printed is just empty border."*

**A disclosure that is technically true and practically false is the worst
kind.** An operator who sees the same red band on every drawing learns to
ignore it, and then does not see it on the one sheet where the border really
does have a title block in it.

The hatch now asks [`super::ink::InkMask`] — a downsample of the very raster
drawn at (4) — what is in the band, and covers the ink extent within it. No
ink ⇒ no hatch. [`hatch_lost_content`] holds the geometry, `super::ink`
holds the pixel test and the measurement behind its threshold, and
[`Overhang`] is how the caption is kept from contradicting the picture.

## The preview owns NO scroll area, deliberately

Zoom is Ctrl+wheel and pan is a primary-button drag. Neither competes with
the dialog's own [`egui::ScrollArea`]: per
`D:\dev\rag\egui\egui_0.35_zoom_with_keyboard_vs_app_zoom_chords.md` egui
splits wheel input at the input-state level, so a wheel event carrying the
zoom modifier surfaces as `zoom_delta()` and contributes nothing to
`smooth_scroll_delta()` — the two cannot fire from one gesture. A plain
wheel over the preview therefore belongs unambiguously to the dialog, and
there is no nested consumer to race it. **Scroll-to-pan was rejected for
exactly that reason**: it would have made the preview a scroll consumer
and put the question back.

## Colours: chrome for the diagram, pass-through for the page

Everything this file paints *except the page bitmap* is chrome — a diagram
of a piece of paper — and takes its colour from [`egui::Visuals`], the
same discipline [`crate::canvas::overlay`] states. The page bitmap is
**document content** and is drawn with a white multiplier, which
`painter.image` treats as "draw the pixels as rendered". Any palette role
there would mean restyling the application restyled the operator's page.
