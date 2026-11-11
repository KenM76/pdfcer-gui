# `settings_sun_disc`

**Owns:** Settings ▸ 3D's default sun, aimed on the sun disc
(`model3d_look_sun_bearing`, `model3d_look_sun_height`). The check confirms
the disc is drawn there, that a press on it aims the sun, and that Save writes
both numbers.

**Drive.** One launch with `checker-image.pdf`. Settings opens through
`file.settings`, and the check presses the 3D heading. It rolls the page until
`model3d.sun.disc` is declared, presses the disc at 4% across and half way
down, then presses Save and waits for a `settings-saved` line. It presses at
mid-height because the page rolls only until most of the disc shows, so the
disc's foot can sit under the scroll edge where a press reaches nothing.

**Oracle.** Settings has no camera, so its disc keeps the model's front at the
bottom: the left rim is bearing 270° on the horizon. After Save the sandbox's
`preferences.txt` must read a bearing within 1° of 270 and a height of 0 or 1. A
sandbox that already holds bearing 270 is refused, because it would pass with
the press doing nothing.

**Not covered here.** The disc turning with the camera, and the viewer's own
disc, are `an_aimed_sun_lights_the_3d_model_where_it_is_aimed`. Typing a
number into the bearing and height fields is the stock number field's
behaviour and is not driven.
