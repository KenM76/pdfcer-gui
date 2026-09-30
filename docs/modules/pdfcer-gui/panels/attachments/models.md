# `panels::attachments::models` — the 3D models section of the Attachments panel

Lists every U3D, PRC or STEP model in `/3D` and `/RichMedia` annotations
(`pdfcer_core::threed::list_3d_with_notes`), one row each: page, format, saved
views, whether it has a preview picture, and where it is kept when that is
not its own annotation. Each row has a *Save model…* button, which pushes
`AttachmentAction::SaveModel`. The listing's caveats (unwalkable page tree,
truncation at `MAX_3D_ARTWORKS`, annotations with no data) show as small
notes. A document with no 3D content draws nothing.

Trace: `models-section count= without_stream= truncated=` every frame the
panel draws; the first Save button publishes region `models.save`.

Strings: `pdfcer_gui_base::text::panels::models`.
