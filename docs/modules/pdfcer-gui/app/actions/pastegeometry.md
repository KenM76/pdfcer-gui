# `app::actions::pastegeometry` — a pasted annotation's geometry keys

## The defect it covers

`EditSession::paste_objects_on_layer` places an annotation carried as
`ClipAnnotation::Raw` by rewriting its `/Rect` and nothing else.
`/QuadPoints`, `/L`, `/Vertices`, `/CL` and `/InkList` hold absolute page
coordinates, and stay where the copy was taken. A pasted `/Link` then draws
at its new box and answers clicks at the old one, in any viewer that honours
`/QuadPoints`. Filed as
`request_G186_a_pasted_annotations_quads_stay_at_the_source.md`.

## The workaround

`carry(&mut ObjectClip, Matrix)` runs on the deserialised clip, just before
`paste_objects_on_layer` in `vector.rs`'s `PasteObjects` arm. For each raw
annotation it translates those five keys by the matrix's `(e, f)`, with the
key list and the per-pair arithmetic of `EditSession::move_annotation`. The
paste trace reports the count as `carried=`.

Limits:

- Only a pure translation is carried. The engine places any other matrix by
  translating the bounds, and discloses that itself.
- Only arrays held inline in the dictionary are moved. A key that is a
  reference into the clip's objects is left alone.
- Modelled markups (`ClipAnnotation::Markup`) are regenerated from their
  spec by the engine and are not touched.

Driven by `a_link_is_moved_resized_copied_and_deleted`. Delete this module
when G186 is consumed.
