# `the_3d_viewer_walks_seen_from_behind`

**Owns:** walking seen from behind (O409): the Behind switch, the camera
behind the walker, and the figure drawn where the walker stands.

**Drive.** One launch, the 3D model opened from its page.

1. Press Walk; read `model-view-rendered`'s `eye=`, `from=`, `dir=`, `up=`
   and `body=`.
2. Press `model3d.behind`; read the same line, `model-behind-arm`, the
   `model-figure` lines and the `model3d.figure` region.
3. Press it again; read the line once more.

**Oracle.**

- With Behind off, `from` equals `eye`.
- With it on, `model-nav-prefs behind=true` is traced. The camera sits at
  least 0.3 eye heights behind the eye along `dir`, and above it along `up`.
- No `model-figure` line comes before the switch. After it,
  `model3d.figure` lies inside `model3d.image`, its centre within a sixth of
  the picture's width of the middle, and its top below the upper third.
- Off again, `from` equals `eye` and no new `model-figure` line follows.

**Falsified** by drawing no figure, which fails naming no `model3d.figure`.
The first drive also failed for a real cause: the picture was not redrawn on
the switch.

**Not covered.** The figure's pixels are not checked: the viewer is an
immediate viewport, which the harness cannot photograph, so the figure is
read from the region its painter declares. A wall pulling the camera in is
covered by `behind`'s unit tests.
