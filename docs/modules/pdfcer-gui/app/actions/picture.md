# `app::actions::picture` — the apply half of Insert image

`insert(doc, page, rect, fit, &Picture)` runs one engine verb through the
edit funnel (`apply::vector_edit`), so each placement is one undo step with
its disclosures on the status line:

| Picture | Verb | Disclosures |
|---|---|---|
| Raster | `EditSession::add_image` with `NewImage` (contain unless `fit` is stretch) | effective dpi, below screen resolution, letterboxed, distorted, recompressed, source decoding |
| SVG | `add_svg(page, rect, &ImportedSvg)` | "placed as vector artwork", a stretch when `distorted`, the import notes |
| EMF | `add_emf(page, rect, &ImportedEmf)` | the same |

Then `select_newest` selects the page's last object in paint order, which is
what every placement verb appends. Unselected, the operator's first press on
the new object would start a marquee instead of a resize. The object count
comes from the model rebuilt after the edit; a page that no longer decomposes
leaves the selection alone rather than naming an index that may be another
object.

Driven: `insert_image_places_a_drawing` (all three kinds).
