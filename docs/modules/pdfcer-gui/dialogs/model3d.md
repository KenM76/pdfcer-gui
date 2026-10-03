# `pdfcer-gui/dialogs/model3d`

**The 3D model viewer.** Opened by *View…* on a PRC row of the Attachments
panel's 3D models section (feature `3d`). Holds the placed meshes
(`app::actions::models::Assembled`) and a camera, and shows the engine's
software rendering (`pdfcer_3d::render_coloured`, each mesh in its own
colour, grey where the file gives none) as a texture.

## Camera

`Orbit { yaw, pitch, zoom, pan, perspective }`, relative to a fitted view.
The model is z-up, as the engine's `3d-render` treats it: yaw 0 looks along +y
(Front), pitch is how far the view looks down, clamped short of ±90° so the
up vector never lines up with the view.

- Direction `(-sin yaw · cos pitch, cos yaw · cos pitch, -sin pitch)`.
- `Camera::fit(bounds, direction, z, perspective, aspect)` frames the model;
  zoom then divides the eye's distance (perspective) or the view height
  (orthographic); pan shifts eye and target along the image's right and up
  by `pan × radius`.
- Named views, in button order: Isometric (45°, 35.26°), Front (0, 0),
  Right (90°, 0), Top (0, 89°), Back (180°, 0). *Fit* resets zoom and pan and
  keeps the side.

Primary drag turns (0.01 rad a point), secondary or middle drag pans, scroll
over the picture zooms (0.05×–50×).

## Rendering

The picture is rendered at the image area's pixel size, capped at 1600 a side
(the renderer is CPU-bound; its cost grows with area), on the theme's
`extreme_bg_color`. It is re-rendered only when the orbit, the size or the
background changes. A render error replaces the picture with its sentence.

Below the picture: the census, the colour note (naming how many parts are
grey), the placement note when the assembly tree could not be read, and the
skipped-part count.

## Use this view on the page; Save picture…

Below the picture, beside *Close*. *Save picture…* is on every model and
queues `AttachmentAction::SaveModelPicture`, which
`app::actions::models::save_picture` writes to a PNG file the operator picks;
the document is not changed. *Use this view on the page* is on a model that
is a `/3D` annotation of its own (`panels::attachments::models::has_own_poster`; a RichMedia asset has
no page picture to replace). The press renders the current orbit through
`render_coloured` at the last rendered picture's shape, 1200 pixels on the
long side, on the engine's default white (the background the engine's own
poster uses, not the theme's), encodes it as PNG and queues
`AttachmentAction::SetModelPoster`, which `app::actions::models::set_poster`
applies as one undo entry. A render failure replaces the picture with its
sentence and queues nothing. Both buttons draw the same picture
(`poster_png`); `PictureFor` says where it goes.

## Lifetime

`DialogsState::model3d`, one at a time; opening another model replaces it,
and changing documents closes it.

## Trace

- `model-view-opened page= parts= uncoloured= triangles= skipped= placed=`
- `model-view-rendered w= h= yaw= pitch= zoom= perspective= covered= chromatic= hues= hash=`
  — `covered` counts non-background pixels, `hash` is FNV-1a of the RGBA.
  `chromatic` counts pixels whose channel spread is at least 48 (a colour,
  not a grey); `hues` counts the twelve 30-degree hue sectors holding at
  least 1% of them. The viewer is an immediate viewport no screenshot
  reaches, so these are a driven check's only view of its colours.
- `model-view-render-failed error=`
- `model-view-poster w= h= yaw= pitch= bytes=` — a picture made for the page.
- Regions: `model3d.image`, `model3d.view.0`…`4`, `model3d.fit`,
  `model3d.close`, `model3d.use_on_page`, `model3d.save_picture`.

Driven by `ui-verify` checks `a_3d_model_turns_under_the_pointer` (an
uncoloured model is grey), `a_coloured_3d_model_draws_in_its_own_colours`
, `the_3d_viewers_view_becomes_the_page_picture` and
`the_3d_viewer_saves_its_view_as_a_picture`.
