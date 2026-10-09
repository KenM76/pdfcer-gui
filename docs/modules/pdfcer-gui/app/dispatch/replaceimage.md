# `app::dispatch::replaceimage` — Format ▸ Replace image

`format.replace_image` asks for a picture file and draws it in place of the
one selected image object (`EditSession::replace_image`).

## Routing

`replaceable_image` is the one test, used by the dispatcher, by
`app::conditions` for `selection.image_replaceable` (`IMAGE_REPLACEABLE`) and
by the Properties panel's button: the selection stands at the Object rung with
exactly one entry and one page object, and the page model reads that object as
`VectorObject::Image` whose source is an image XObject or an inline image. A
form XObject is excluded because the engine refuses it
(`EditError::ReplaceImageOnOther`) — a placed drawing is not a picture. In a
mode that does not edit content, or with no such selection, the press is
declined with `command-declined reason=no-image-selected`.

## The sequence

1. `files::pick_image_source` — the same picker, filters and harness seam
   (`PDFCER_DIAG_IMAGE_PATH`) as Insert image. Cancel does nothing.
2. `dispatch::images::import` — a file that cannot be read or decoded leaves
   the importer's note and stops.
3. A raster becomes `VectorAction::ReplaceImage`. An SVG or EMF is refused
   with `text::images::replace_needs_raster`: the engine replaces an image
   with an image, and a drawing is placed as vector content.

The import runs on the UI thread at the press, as Insert image's does; the
action carries the decoded image in an `Arc`.

## Placements

Three, on one condition: the Format tab's **Image** group, the canvas object
menu (above the Arrange rows, with the `replace-image` glyph), and a button in
the Properties panel's line-and-opacity section (`panels::properties::
strokestyle`, region `properties.stroke.replace-image`). The panel button
presses the command, so all three take one route.
