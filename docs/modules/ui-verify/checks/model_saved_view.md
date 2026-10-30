# `ui-verify/checks/model_saved_view`

`the_3d_viewer_opens_on_the_files_view` — **a 3D model whose file saves an
opening view opens in the viewer on that view, and *File's view* returns to
it from a named one.**

Off-screen, scripted pointer, no OS input. The fixture is written by the
check (`crate::fixture::pdf_of`): one page, one `/3D` annotation whose PRC
stream is the engine corpus's `assembly.prc` and saves `Corner` first,
the stream's `/DV`. Its `/C2W` looks along `(0.6, 0, -0.8)` with `+y` up:
no named view looks that way, and a z-up viewer could not hold that up.
Opened in Edit with the Attachments panel showing, then *View…*.

Oracles, from the trace:

- `model-view-opened … file-view=1`.
- The first `model-view-rendered` line's `dir` and `up` (the camera's unit
  look direction and up) are parallel to the view's (cosine above 0.999).
- After *Isometric* they are not: without this a return could not be told
  from never leaving.
- `model3d.view.file` is declared, and after it `dir` and `up` are the
  view's again.

Falsified: discarding the file's view when the viewer opens (`opening`
`None`) fails it on `file-view=0`, on the first picture's direction, and on
the missing *File's view* button.

*Saved views.* The stream saves two more views after `Corner`: `Side`, whose
`/C2W` looks along `-x` with `+y` up, and `Bare`, which saves no camera. As
the viewer opens it traces `model-view-saved-views count=3 cameras=2
default=Some(0)`. The check opens `model3d.saved_views`, clicks
`model3d.saved_view.1`, and requires the next picture's `dir` and `up` to be
`Side`'s. Falsified: a menu item that does not set the camera leaves the
picture on `Corner` and fails on the direction.
