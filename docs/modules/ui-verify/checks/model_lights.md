# `ui-verify/checks/model_lights`

`the_3d_viewer_switches_lights_off_and_on` — a glowing part, a folder of
glowing parts and a lamp in a 3D model are switched off and back on from the
part list, from the folder the viewer makes up for lights, and from the
right-click menu, and each switch changes the picture.

# What it drives

`fixtures/glow-room.prc` (`fixtures/glow-room.PROVENANCE.py`): root 0 over
floor 1, box 2, bulb 3 (glows), Fittings 4 over tube-a 5 and tube-b 6 (both
glow), lamp 7. The model carries two lamps: a point lamp and an ambient
light. Opened with *View…* under `--no-input` with a `ScriptedPointer`.

On opening, the last `model-view-lights` line must read `gathered=3` (the
bulb is beside parts that do not glow; the tubes are alone in a folder of
their own and are not gathered), a non-zero `lamps`, and `lamp-nodes=0|0`:
the file stores both lamps on the root, so the root's row carries a light
box. The picture is the
`hash` of the last `model-view-rendered` line after the viewer settles.

Each step clicks one region, reads the `model-view-lights` line it traces,
and requires the picture to differ from the opening picture exactly when
something is off.

| Step | Region | `dark` | `lamps-off` |
|---|---|---|---|
| 1 | `model3d.part.glow.3` | 3 | - |
| 2 | `model3d.part.glow.3` | - | - |
| 3 | `model3d.part.glow.4` | 4 | - |
| 4 | `model3d.part.glow.4` | - | - |
| 5 | `model3d.lights.part.3` | 3 | - |
| 6 | `model3d.lights.part.3` | - | - |
| 7 | `model3d.lights.lamp.0` | - | 0 |
| 8 | `model3d.lights.all` | - | - |
| 9 | `model3d.lights.all` | 3 | 0, 1 |
| 10 | `model3d.lights.all` | - | - |
| 11 | `model3d.part.glow.0` | 0 | - |
| 12 | `model3d.part.glow.0` | - | - |

Steps 11 and 12 also require `lamps-drawn=0` and then `lamps-drawn=2`: the
root's box switches the lamps stored on it, while their own rows stay on.

Then the bulb's row is clicked to select it, and the picture with it selected
becomes the reference, because the selection is drawn on the picture.
Right-click the row and `model3d.menu.light_off`: `dark=3`, picture differs.
Right-click and `model3d.menu.light_on`: `dark=-`, picture returns.

# Falsified

The build before the switches fails at opening: it traces no
`model-view-lights` line. The build before folder lamps fails at opening
too: its line carries no `lamp-nodes`.
