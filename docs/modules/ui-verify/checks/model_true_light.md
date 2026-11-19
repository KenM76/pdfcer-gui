# `ui-verify/checks/model_true_light`

`the_3d_viewer_draws_at_a_true_light_level` — the 3D viewer's *True light
level* draws the picture at an exposure fixed in lux, a higher *Full white
at* draws the model darker, and the picture drawn in lux is not the metered
one.

# What it drives

`fixtures/lux-room.prc` (`fixtures/lux-room.PROVENANCE.py`), opened with
*View…* under `--no-input` with a `ScriptedPointer`. Its two point lamps,
3000 cd and 300 cd, keep their candela through `3d-convert`, and the model
has no ambient fill, so the engine can draw it in lux.

The oracle is the `model-view-rendered` line, the viewer being an immediate
viewport no screenshot reaches: `exposure=metered|fixed|refused`,
`white-lux`, and the pixel fields read back from the drawn picture on
either path, `luma` (the mean Rec. 709 luma of the pixels that are not the
background) and `hash`.

Each state is compared at its settled picture, the first line whose
`gpu-bounce` is not `waiting`. A picture drawn before the card's bounce
arrives is darker than the same state settled.

1. The opening picture must be `exposure=metered`: the switch is off by
   default.
2. Press `model3d.true_light` on the options menu and close it. A picture
   with `exposure=fixed white-lux=500.0` must follow, and its `luma` must
   differ from the metered picture's by at least 4. A changed `hash` alone
   would pass on the picture's height, which the note said under it changes.
3. Click `model3d.white_lux`, select its text, type 1000 and press Enter. A
   picture with `exposure=fixed white-lux=1000.0` must follow, with `luma`
   at least 4 below the 500 lux picture's.

A picture that never reaches `exposure=fixed` fails with the newest picture
line and `model-view-true-light` line; `exposure=refused` there names an
engine refusal, whose reasons are said under the switch.

# Falsified

Each assertion was turned red by a sabotage of the shell:

- `TrueLight::exposure` ignoring the field (always 500 lux) fails on the
  white point never reaching 1000.
- Drawing at 500 lux while the trace says 1000 fails "darker".
- Drawing metered while the trace says fixed fails "differs from metered"
  (luma 38.0 against 37.9).
