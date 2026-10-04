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

## `stamp` — a picture as a comment

`stamp(doc, page, rect, &Picture, author, opacity)` is the apply half of
`Action::StampPicture`, raised where the mode authors markup but not content:
a paste in Review, Markup ▸ Paste picture as stamp, and a drop in Review. One
`vector_edit`, so one undo step:

| Picture | Verb | Signed, dated, pen opacity |
|---|---|---|
| Raster | `add_image_stamp(page, rect, &ImportedImage, &MarkupOptions)` | yes: the note is `annots::signed_note("", author)`, `/CA` is the pen's opacity |
| SVG | `add_svg_stamp(page, rect, &ImportedSvg)` | no — the verb takes no options (request G107); when the operator has an author name or an opacity set, the status line says the stamp is unsigned and opaque |
| EMF | `add_emf_stamp(page, rect, &ImportedEmf)` | the same |

On a page with `/Rotate` the stamp is turned by the same angle about the
rectangle's centre (`set_annotation_rotation`) inside the same edit, so it
reads upright the way the operator sees the page; `customstamp.md` carries the
argument. The trace line is `picture-stamp-placed kind= id= page= rotate=
turned= signed=`.

Driven: `a_copied_picture_pastes_as_a_stamp_in_review` (paste, raster) and
`a_dropped_picture_stamps_in_review` (drop, raster and SVG).
