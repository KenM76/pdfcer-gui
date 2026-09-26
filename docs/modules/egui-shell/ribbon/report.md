# `egui-shell/ribbon/report`

## Item notes

### `fn the_reported_names_are_a_stability_contract`

These strings are consumed by a harness in another tool, possibly
in another repository, by literal comparison. A rename is
therefore a breaking change with no compiler to catch it: the
harness keeps building, its assertions simply stop matching
anything, and a test that matches nothing passes.

Pinning the exact spellings makes a rename a deliberate act with a
failing test in front of it. If this test is ever updated, the
harness's selectors have to be updated in the same change.

### `fn a_reporter_with_no_sink_never_builds_a_name`

This is the zero-cost claim, and the only way to observe it is a
side effect inside the closure that is supposed not to run. If
this fails, every rect call site in the paint loop is allocating a
`String` per frame to throw it away.
