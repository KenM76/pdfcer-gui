# `app::actions::vector::replaceimage` — commit Replace image

One `EditSession::replace_image(page, object, image, ImageFit::Contain)` call
through `vector_edit_on_page`, so one undo entry (`CommandKind::ReplaceImage`)
and a page-scoped re-render.

## Why Contain

The new image keeps its own shape, centred in the old image's extent. Stretch
would distort a picture of a different aspect ratio without the operator
asking; Insert image makes the same default. There is no fit choice on this
command: a stretch is one resize away.

## The selection

The engine draws the new image at the old one's point in the content stream,
so the object keeps its index and the selection stands. After a Contain
replace the object's extent is the letterboxed box, so a second replace fits
inside that, not the original.

## Disclosures

Off-canvas, in the status notes:

- the new picture kept its shape and does not fill the old box
  (`replace_letterboxed`), when the engine reports `letterboxed`;
- `placement_disclosures` for resolution, distortion and re-encoding — called
  with `letterboxed: false`, because its letterbox sentence speaks of "the box
  you gave it";
- `source_decoding_notes` for what a GIF or TIFF decode dropped;
- always, `replace_old_data_kept`: other placements of the old XObject keep
  drawing it and an incremental save keeps its bytes, so replacing is not
  removal. Redaction is the verb that removes content.

## Trace

`image-replaced page= object= old= new= model=` — `old` and `model` are the
image XObject the page model names at `object` before and after the edit (0
for an inline image), `new` the engine's `image_id`. `model == new` is the
read-back that the replacement landed in the selected slot. Written only when
the engine succeeded; a refusal is the funnel's `replace-image-refused` line.
