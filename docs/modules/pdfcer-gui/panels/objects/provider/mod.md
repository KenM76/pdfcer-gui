# `panels::objects::provider` — front-to-back page object decomposition

The thin `pdfcer-gui` adapter that plugs `pdfcer-core`'s read-only vector
object model (`pdfcer_core::vector`) into the shell. The shape is fixed:
this adapter CALLS INTO the object model, which stays GUI-free; the adapter
owns the trait impl and the object model owns none of it.

## What lives here vs in core (GUI–core separation)

ALL geometry — decomposition, hit-testing, marquee enclosure — is
`pdfcer_core::vector`, in PDF user space. This module owns exactly two
things core cannot: (1) the **coordinate-space translation** between the
canvas's device convention and PDF user space, and (2) the [`TargetId`] ↔
object-index encoding. The translation reuses the SAME transform
[`pdfcer_render::page_device_geometry`] computes to rasterize the page (at
scale 1.0, which *is* canvas space), inverted — so selection geometry and
the render agree by construction, exactly as
[`crate::viewer::canvas_to_pdf_space`] does (this provider is the
batched, object-model-backed sibling of that per-point bridge).

## Single-page by design

The canvas shows one page at a time and only ever queries
`view.page_index`, so a provider is built for the **current page** and
rebuilt on page change / edit. A query for any other `page_index` returns
nothing — cheap, and it keeps the decomposition off the hot path of a
large document (only the visible page is decomposed, not all N).

`RIBBON_IA.md` §5.2 names this as the reason continuous page display is
"a larger build than it looks": this file returns nothing for any page
but the current one, and continuous mode needs a page *range*. That is
`GUI_ROADMAP.md` Phase 4 work, and it is a real change here rather than a
wiring change above.

## [`TargetId`] encoding

A [`TargetId`] is the object's index into
[`pdfcer_core::vector::PageObjects::objects`] (paint order), cast to
`u64`. Consumers treat it opaquely; only this module mints and decodes
it.

---

# Who reads this surface

| Method group | Consumer |
|---|---|
| [`ObjectModelProvider::build`], [`page_objects`](ObjectModelProvider::page_objects) | the Objects panel's row list |
| [`part_kind`](ObjectModelProvider::part_kind), [`part_count`](ObjectModelProvider::part_count), [`subpath_count`](ObjectModelProvider::subpath_count), [`text_run_count`](ObjectModelProvider::text_run_count) | the Objects panel's **object → part → point** nesting |
| [`subpath_node_points`](ObjectModelProvider::subpath_node_points), [`object_node_points`](ObjectModelProvider::object_node_points), [`subpath_handle_points`](ObjectModelProvider::subpath_handle_points) | the Objects panel's point rows and the Properties panel's node readout |
| [`hit_test_all`](ObjectModelProvider::hit_test_all), [`hit_test`](ObjectModelProvider::hit_test), [`hit_test_rect`](ObjectModelProvider::hit_test_rect), [`bounds`](ObjectModelProvider::bounds) | the canvas selection layer, through [`crate::canvas::target::CanvasTargetProvider`] |
| [`part_hits`](ObjectModelProvider::part_hits), [`subpath_hits`](ObjectModelProvider::subpath_hits), [`text_run_hits`](ObjectModelProvider::text_run_hits), [`nearest_node`](ObjectModelProvider::nearest_node), [`nearest_handle`](ObjectModelProvider::nearest_handle) | click-to-select and the level ladder |
| [`part_bounds_canvas`](ObjectModelProvider::part_bounds_canvas) and friends | the selection outlines |
| [`object_sample_points`](ObjectModelProvider::object_sample_points) | the measure tools' snap query and the Taubin best-fit circle |

**Every one of them is under test below**, independently of the trait: a
method proven here cannot be broken by a change to how the canvas reaches
it.

## Two invariants this file is responsible for

* **[`TargetId`] is minted here and nowhere else.** It is the *encoding*,
  and the encoding belongs with the thing that mints it — `canvas`
  re-exports this rather than defining a second one, because two id types
  over one index space is exactly the divergence these docs warn about.
* **The tolerance is a per-query parameter, never a baked constant.**
  [`tests::selection_tolerance_is_honoured_per_query_not_baked_in`] holds
  it there. Declaring a second copy of a screen-pixel tolerance in this
  module would put one rule in two places, which is the cause of the
  defect that test guards rather than a way to guard it.
