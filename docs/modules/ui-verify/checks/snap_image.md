# `snap_image` — a measure tool snaps to a placed image

`measure_tool_snaps_to_an_image` opens `image-box.pdf` (one image filling
x 100..300, y 100..200), arms Linear in Review, and hovers twice:

1. Blank paper at (360, 260): the `measure-hover` line must say
   `source=none`, so the second hover's answer is not a constant.
2. Just inside the image's top-left corner at (101, 199): the line must say
   `snap=1 source=image`.

The fixture has no paths, so a snap there can only come from the image's
quadrilateral, which the engine gives through `EditSession::page_snap_geometry`
when the caller passes the page's decomposition. `source` is the candidate's
`SnapSource`, named by `canvas::measure::resolve::source_name`.

The hover helper is `snap_markup::hover`, shared with
`measure_tool_snaps_to_a_markup`.
