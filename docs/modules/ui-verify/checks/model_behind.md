# `the_3d_viewer_walks_seen_from_behind`

**Owns:** walking seen from behind (O409, O420): the Behind switch, the
camera behind the walker, the figure drawn where the walker stands, and the
game camera — a left drag orbits, the wheel pulls in.

**Drive.** One launch, the 3D model opened from its page.

1. Press Walk; read `model-view-rendered`'s `eye=`, `from=`, `dir=`, `up=`
   and `body=`.
2. Press `model3d.behind`; read the same line, `model-behind-arm`, the
   `model-figure` lines and the `model3d.figure` region.
3. Left-drag across the middle of `model3d.image`, a fifth of its width;
   read `model-walk-moved`'s `yaw=` and `look-yaw=`. Then one wheel step
   over the picture; read `model-behind-arm` again.
4. Press Behind again; read the line once more.

**Oracle.**

- With Behind off, `from` equals `eye`.
- With it on, `model-nav-prefs behind=true` is traced. The camera sits at
  least 0.3 eye heights behind the eye along `dir`, and above it along `up`.
- No `model-figure` line comes before the switch. After it,
  `model3d.figure` lies inside `model3d.image`, its centre within a sixth of
  the picture's width of the middle, and its top below the upper third.
- After the drag, `look-yaw` (the camera's turn off the heading) is more
  than 0.05 rad and `yaw` is unchanged: the camera orbited and the walker
  kept its heading. After the wheel, `model-behind-arm eye-heights=` differs.
- Off again, `from` equals `eye` and no new `model-figure` line follows.

**Falsified** by drawing no figure, which fails naming no `model3d.figure`.
Making the left drag turn the walker (the right drag's behaviour) fails
the orbit assertion with `look-yaw` 0. The first drive also failed for a real cause: the picture was not redrawn on
the switch.

**Not covered.** The figure's pixels are not checked: the viewer is an
immediate viewport, which the harness cannot photograph, so the figure is
read from the region its painter declares. A wall pulling the camera in is
covered by `behind`'s unit tests.
