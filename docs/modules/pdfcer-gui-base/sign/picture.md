# `pdfcer-gui-base/sign/picture`

The picture a visible digital signature is drawn with, read once and shared.

# Contract

- `AppearancePicture::read(path)` reads the file and imports it with
  `pdfcer_core::image_import::import`. The error is the engine's sentence, or
  the file system's.
- `image()` is the imported picture that `sign::prepare` copies into
  `SignRequest::appearance_image`. The engine ignores it for an invisible
  signature, and `Authored::appearance_image` is `None` then as well.
- Equality is identity (`Arc::ptr_eq`), which keeps `Authored` `Eq` without
  comparing pixels.
- `size()` is the picture's pixel size. `sign::prepare` widens the default
  box leftward by `picture_widening(w, h)`, which is the picture's width at
  the box's 60 pt height, capped at 180 pt. The engine gives a picture at most
  half the box, so the text keeps its 180 pt.
