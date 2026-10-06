# `handsign::picture` — a signature from a picture file

`OPERATOR_REQUESTS.md` **O287**, the *Picture* tab: a scan or photo of the
operator's signature, PNG, JPEG, BMP or TIFF, placed in the box.

## What is carried

`SigPicture` holds the file's bytes unchanged and the clear-white choice.
`SigPicture::image` imports through `pdfcer_core::image_import::import` each
time it is asked, so what the preview shows and what is placed are the
engine's decoding of the same bytes; nothing is re-encoded here.

## Transparency

- A picture with its own transparency (a PNG alpha channel) keeps it: the
  importer carries it as a soft mask and `add_image` writes it.
- *Make white see-through* sets `ImportedImage::color_key_mask` to the range
  `[0.92·max, max]` per component: a colour-key `/Mask` (ISO 32000-1
  §8.9.6.4). A scan's paper is rarely exactly 255, hence the threshold.
- It is offered only where it can apply (`can_clear_white`): grey or RGB
  samples with no transparency of their own. A CMYK or indexed picture, or
  one already transparent, is left as it is and the option is greyed with the
  reason on hover.
- A new picture starts with white cleared when it can be: the paper around a
  signature is what the operator least wants laid over the form.

## Proportions

`ink_size` is the picture's displayed pixel size (`display_size_px`, which
honours an EXIF turn). It is the signature's proportions for the fit rule in
`handsign::place`. The picture is then placed stretched to the resulting
rectangle (`NewImage::stretching`), so a placement the operator reshaped with
Shift is honoured exactly.

## The preview

`preview(image)` builds the one-page document `blank::picture_page` makes,
and draws it with `pdfcer_render::render_page_with` on a transparent backdrop,
`PREVIEW_PX` (480) pixels on its longer side. The engine draws it, so a
cleared white previews cleared and the preview cannot disagree with the page.

## The remembered copy

`hand-signature-picture.bin` beside `settings.txt`: a header line
`pdfcer-hand-signature-picture clear-white=0|1`, then the file's bytes. Any
other content is ignored whole. Written and deleted by the same *Remember my
signature on this computer* tick as the drawn and typed copies.
