# `pdfcer-gui/dialogs/model3d`

**The 3D model viewer.** Opened by *View…* on a PRC row of the Attachments
panel's 3D models section (feature `3d`). Holds the placed meshes
(`app::actions::models::Assembled`) and a camera, and shows the engine's
software rendering (`pdfcer_3d::render_coloured`, each mesh in its own
colour, grey where the file gives none) as a texture.

## Camera

`dialogs::model3d::orbit`. `Orbit { yaw, pitch, zoom, pan, perspective,
axes, framed }`, relative to a fitted view. `axes` is a front and an up: yaw
0, pitch 0 looks along front, yaw turns about up, pitch looks down from it,
clamped short of ±90° so up never lines up with the view.

- Direction `-sin yaw · cos pitch · right + cos yaw · cos pitch · front -
  sin pitch · up`, with `right = front × up`.
- Named views use z-up axes (front +y), as the engine's `3d-render` treats a
  model with no view of its own.
- The file's opening view (`pdfcer_core::threed::default_3d_view`: the
  annotation's `/3DV`, else the stream's `/DV`, else its first `/VA`) gives
  `Orbit::saved` when it carries a `/C2W` camera: axes are its
  `SavedViewAim::direction` and `up` (up orthonormalised against the
  direction; a view whose two coincide is not offered), yaw and pitch 0,
  perspective unless it asks for orthographic, and `framed` when its aim is
  `ViewFit::Framed`.
- `Camera::fit(bounds, direction, up, perspective, aspect)` frames the model;
  while `framed` and orthographic, `SavedViewAim::frame` then moves it onto
  the view's own centre and height;
  zoom then divides the eye's distance (perspective) or the view height
  (orthographic); pan shifts eye and target along the image's right and up
  by `pan × radius`.
- Named views, in button order: Isometric (45°, 35.26°), Front (0, 0),
  Right (90°, 0), Top (0, 89°), Back (180°, 0). *Fit* resets zoom and pan,
  drops the file's framing and keeps the side.
- *File's view* (`model3d.view.file`), first in the row and drawn only when
  the file's view carries a camera, returns to `Orbit::saved`. The viewer
  opens there; without one, at Isometric.
- While the file's framing applies, the line under the hint discloses it:
  *Shown from* `SavedViewAim::source`, which names the view and says its
  orthographic scale was read as the bound side spanning 1/scale camera
  units, an interpretation the standard leaves open.

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
poster uses, not the theme's) and queues its RGBA samples as
`AttachmentAction::SetModelPoster`; `app::actions::models::set_poster`
builds the image with `ImportedImage::from_rgba8` and applies it as one undo
entry. A render failure replaces the picture with its sentence and queues
nothing. Both buttons draw the same picture (`poster`); `PictureFor` says
where it goes, and *Save picture…* alone encodes it as PNG (`png`).

## Lifetime

`DialogsState::model3d`, one at a time; opening another model replaces it,
and changing documents closes it.

## Trace

- `model-view-opened page= parts= uncoloured= triangles= skipped= placed= file-view=`
  — `file-view=1` when it opened on the file's view.
- `model-view-rendered w= h= yaw= pitch= zoom= perspective= framed= dir= up= covered= chromatic= hues= hash=`
  — `dir` and `up` are the camera's unit look direction and up, `x,y,z`;
  `covered` counts non-background pixels, `hash` is FNV-1a of the RGBA.
  `chromatic` counts pixels whose channel spread is at least 48 (a colour,
  not a grey); `hues` counts the twelve 30-degree hue sectors holding at
  least 1% of them. The viewer is an immediate viewport no screenshot
  reaches, so these are a driven check's only view of its colours.
- `model-view-render-failed error=`
- `model-view-poster w= h= yaw= pitch= bytes=` — a picture made for the page.
- Regions: `model3d.image`, `model3d.view.file`, `model3d.view.0`…`4`, `model3d.fit`,
  `model3d.close`, `model3d.use_on_page`, `model3d.save_picture`.

Driven by `ui-verify` checks `a_3d_model_turns_under_the_pointer` (an
uncoloured model is grey), `a_coloured_3d_model_draws_in_its_own_colours`
, `the_3d_viewers_view_becomes_the_page_picture` and
`the_3d_viewer_saves_its_view_as_a_picture`, and
`the_3d_viewer_opens_on_the_files_view` (the file's view and the way back to
it).
