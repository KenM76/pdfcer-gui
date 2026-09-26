# `app::state` — what is open

One thing lives here: [`Status`] and [`OpenDoc`] — the shape of "what, if
anything, is open", and everything one open document owns.

## What is NOT here, and where it went

The **raster bookkeeping** — the per-frame decision about whether the
cached page texture is still a picture of what the operator is looking at,
and if not whether to re-rasterize now or wait for a zoom gesture to settle
— was the second half of this file until Phase 4 and now lives in
[`crate::render::settle`]. Phase 4 made it considerably larger (one texture
became a texture plus a bounded strip cache, and one staleness question
became two), and the seam is the one this header had already named:
everything here answers *"what is open, and what is the operator looking
at?"*, and everything there answers *"what does the picture need to be, and
what should be done about it this frame?"*. Only the second belongs in
`render/`, beside the worker it schedules.

The **fields** it reads are still declared below, because that is what
bounds their lifetime and is the whole point of their living on the
document; only the methods moved.

The two **derived caches** an open document owns — the page decomposition
and the font inventory — live in [`crate::app::cache`], along with the
four methods that read them. Their fields are still declared on [`OpenDoc`]
below, because that is what bounds their lifetime and is the whole point of
their move onto the document; only the types, the accessors and their
argument moved out.

The seam is a real one rather than a cut made to satisfy rule R2's
1,500-line gate: everything left in this file answers *"what is open, and
what is the operator looking at?"*, while everything in `cache.rs` answers
*"what expensive derived value do several surfaces need, and how is it
computed once?"* — a different question, with its own shared hazard
(staleness against [`OpenDoc::edit_epoch`]), its own shared device (a `Cell`
key beside a `RefCell` payload) and its own documented exemption from the
`&mut`-only mutation rule. `cache.rs`'s header carries that argument in
full. Splitting anywhere else would have left half an explanation on each
side of the cut.

## Three ways to fail, three ways to say so

[`Status`] distinguishes what most viewers conflate, and the
distinction is carried across from the old shell because it is one of
the things pdfcer does that its competitors do not:

- [`Status::Failed`] — the *file* is wrong: damaged, truncated, not a
  PDF. "Something is wrong with your document."
- [`Status::Unsupported`] — the file is fine and **pdfcer** is not
  finished. `pdfcer-core` detects such a file and refuses it cleanly
  rather than misparsing it into plausible-looking garbage. Presenting
  that as "failed to open" would tell the operator a lie about their own
  file.
- [`Status::NeedsPassword`] — a third thing again: pdfcer *can* decrypt
  this document and has not been told how.

The branch between them is made on **structured error data** from
`pdfcer-core`, never by matching on a message string. That is exactly what
"core errors are stable, structured diagnostics" is *for*, and it is what
makes the distinction reliable rather than a heuristic that decays as error
prose is edited.

The branch itself now lives in [`crate::app::lifecycle`], with
`is_unsupported_structure` and the two methods that move `Status` between
these variants. This file kept the **shape** of the answer (the enum, and
why it has these variants); that one has *when each one is produced*. See
that module's header for the seam.

## Rendering happens on state change, never per frame

egui redraws continuously; rasterizing a PDF page at 60 Hz would be
absurd. The canvas holds one cached [`PageTexture`] for the **current
page**, plus — under a continuous page-display mode only — a bounded cache
of the *other* visible pages ([`OpenDoc::strip_rasters`]). Each texture
carries the [`RenderKey`] it was rendered from — page, raster scale,
annotation visibility, layer-override generation — and staleness is that
key compared against the one the view wants, with deliberately no second
field list to keep in step with it (see [`RenderKey`]'s own docs; a key
compared on one side and not the other is a control that ticks and redraws
nothing).

**The comparison, the zoom debounce and the strip's scheduling all live in
[`crate::render::settle`]**, which carries the full argument for both
staleness policies. What lives here is the state they read.

[`OpenDoc`] also carries the page decomposition and the font inventory,
moved off `crate::panels::PanelsState` at S4 so the document's own
lifetime bounds them and no identity key is needed — see
[`OpenDoc::page_objects`] — and, from the same stage, the **canvas
selection**, moved off `egui::Memory` for the same reason. See
[`OpenDoc::selection`].
