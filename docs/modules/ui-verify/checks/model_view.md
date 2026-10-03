# `ui-verify/checks/model_view`

`a_3d_model_turns_under_the_pointer` — **a placed PRC model opens in the
viewer, and turns, zooms and jumps to the Top view under the pointer.**

Off-screen, scripted pointer, no OS input. Copies the engine corpus's
four-page document, places its `assembly.prc` (two placed copies of one part)
from the ribbon, and clicks *View…* on its Attachments row.

Oracles, from `model-view-rendered`: the first render covers pixels; a primary
drag across the picture changes `yaw` and `hash`; a wheel changes `zoom`; the
Top button leaves `pitch` above 1.5 radians. `assembly.prc` colours no part,
so `model-view-opened` must count every part `uncoloured` and the first render
must hold no coloured pixel (`chromatic=0`).

Falsified: stopping the drag from changing the yaw fails it ("yaw 0.785 →
0.785"). A screenshot of the viewer's own window is not delivered by the
seam (`shot vp=` goes unacknowledged for this window), so the check keeps no
picture.

The grey arm, falsified twice: painting every part red in
`app::actions::models::assemble` fails it (`uncoloured=0`), and handing red
colours to the viewer's renderer alone fails it on `chromatic` (21208
coloured pixels).
