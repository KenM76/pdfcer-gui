# `pdfcer-gui/dialogs/model3d`

**The 3D model viewer.** Opened by *View…* on a PRC row of the Attachments
panel's 3D models section (feature `3d`). Holds the placed meshes
(`app::actions::models::Assembled`) and a camera, and shows the engine's
software rendering (`pdfcer_3d::render`) as a texture.

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

Below the picture: the census, the flat-colour note, the placement note when
the assembly tree could not be read, and the skipped-part count.

## Lifetime

`DialogsState::model3d`, one at a time; opening another model replaces it,
and changing documents closes it.

## Trace

- `model-view-opened page= parts= triangles= skipped= placed=`
- `model-view-rendered w= h= yaw= pitch= zoom= perspective= covered= hash=`
  — `covered` counts non-background pixels, `hash` is FNV-1a of the RGBA.
- `model-view-render-failed error=`
- Regions: `model3d.image`, `model3d.view.0`…`4`, `model3d.fit`.

Driven by `ui-verify` check `a_3d_model_turns_under_the_pointer`.
