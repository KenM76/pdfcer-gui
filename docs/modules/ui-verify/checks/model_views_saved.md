# `ui-verify/checks/model_views_saved`

`the_3d_viewer_saves_its_views_into_the_file` — **the 3D viewer's *Save
views in the file* writes the named views, as the operator's axes set them,
into the model's file, and the model then opens on the view chosen.**

Off-screen, scripted pointer, no OS input. The fixture is the one
`model_saved_view` writes: one page, one `/3D` annotation whose PRC stream
(the engine corpus's `assembly.prc`) saves one view, `Corner`, looking along
`(0.6, 0, -0.8)` with `+y` up. Opened in Edit with the Attachments panel
showing, then *View…*.

Steps and oracles, from the trace:

1. The first `model-view-rendered` looks along `Corner`. This is the
   control: a viewer that opens there and later opens looking down `-y`
   has read something new from the file.
2. *Up* ▸ `+Y`, *Top*, *Save views in the file*. `model-views-set` carries
   `after=5`: the five named views, and no sixth because Top is one of them.
3. *Save views in the file* again from the same window: a second
   `model-views-set` with `before=5`. The window's listing row is the one it
   opened on, which the first write changed; a write that compared the whole
   row would refuse here.
4. *Close*, then *View…* again. The new `model-view-opened` carries
   `file-view=1`, and its first `model-view-rendered` looks along `(0, -1, 0)`:
   Top about `+y`, read back from the file through `default_3d_view`.

The reopen reads the session the edit changed, the same objects Save
writes; no reader other than the engine's is involved.

Falsified: writing the views with view 0 as the default (`set_3d_views(.., Some(0))`)
reopens the model on Isometric, looking along `(-0.58, -0.58, -0.58)`, and fails
step 4's direction.
