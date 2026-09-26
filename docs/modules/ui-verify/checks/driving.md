# `ui-verify/checks/driving`

`checks::driving` — the moves that **every check which drives the ribbon**
has to make, in one place.

# Why this module exists

[`crate::checks::markup_rectangle`] was the first check to click a ribbon
control, and it had to invent five small things to do it: read the last
rect the application declared for a name, list the names it *did* declare
for a SKIP reason, re-parse the same captured stderr under the **shell's**
line prefix, measure a control's fill out of a capture, and compare two
fills. All five are properties of *driving an `egui-shell` ribbon*, not of
markup.

The second and third such checks — [`crate::checks::measure_linear`] and
[`crate::checks::read_mode`] — needed the same five, and a third copy of a
function is where copies start to disagree. So they live here, with the
reasoning that shaped them carried across rather than summarised.

`markup_rectangle` deliberately keeps its own copies. Rewriting a check
that is already known to detect its defect, in the same change that adds
two new ones, would mean the three checks stopped being independent
evidence of each other at exactly the moment the harness grew. This module
is a *widening*, not a refactor; when someone next has cause to touch
`markup_rectangle` for its own sake, folding it onto these is a one-line
change per helper.

# The two diagnostic channels, and why a check reads both

One captured stderr file, two vocabularies:

| Channel | Switch | Prefix | Says |
|---|---|---|---|
| the shell | [`SHELL_DIAG_ENV`] | [`SHELL_TRACE_PREFIX`] | a segment/tab/control took a click |
| the application | the profile's `diag_env` | the profile's `trace_prefix` | what the application did about it |

The split is not an accident of this build. `egui_shell::verify`'s header
explains that one environment variable name lets a harness arm tracing on
*any* `egui-shell` application without first discovering its name, and the
prefix is the application's so two crates' lines never blur together. The
consequence for a check is the thing that makes a failure attributable: a
present `ribbon-command-invoked` with an absent application-side effect
names the application's dispatch and nothing else, and an absent
`ribbon-command-invoked` means no click was ever delivered — which is a
SKIP, because a check that could not deliver a click has learned nothing.

## Item notes

### `const MAX_BAND_SCROLLS`

A bound rather than a `while true`, because a harness that hangs reports
nothing at all. It is deliberately far above any real tab — the widest tab
in `RIBBON_IA.md` has nine groups — so hitting it is a defect in the
application (an arrow that never retires) rather than a tab this helper
cannot search, and the two must not be confused.

### `fn at_this_stop`

S3 gave the band a middle rung: when it runs short of width a whole group
folds into a single captioned button, its items reachable through that
button's popup. They are on the ribbon, they are one click away — and they
**publish no rect**, exactly like a scrolled-away group's, which is why this
search exists at all.

`export_dxf_writes_the_pages_geometry` went red on it and the failure read
as a lost command: *"the File tab declares no
`ribbon.item.file.export_dxf`, on the band or in the overflow."* The command
was not lost. The harness was asking about two of the three places it could
be, and the window the harness opens is 1,100 pt wide — precisely the width
at which the Export group collapses.

It is checked at **every scroll stop**, not once, because collapsing
happens before scrolling: a group that scrolls into view can arrive already
collapsed, and looking for its items on the band would find nothing.

# Errors

If the trace cannot be read, or a click cannot be delivered.

### `fn rewind_band`

The left arrow is drawn **only** while the band is scrolled off its start
(`egui-shell`'s `ribbon::band`: `if scrolled > 0`), so its absence is the
termination condition rather than a count this helper would have to keep in
step with the application's.

On a band that is already at position zero this costs one trace read and
no clicks, which is why `declared_or_in_overflow` can call it
unconditionally.

# Errors

If the trace cannot be read, or a click cannot be delivered.

### `fn collapsed_groups`

A collapsed group publishes `ribbon.group.<tab>.<id>.collapsed` — a
deliberately distinct name from the expanded `ribbon.group.<tab>.<id>`, so
that a check can tell *"on the band, collapsed"* from *"on the band"* and
from *"gone"*. This is the consumer that distinction was created for.

### `fn a_regions_last_declaration_is_the_one_that_is_used`

The same property [`crate::checks::markup_rectangle`] pins for its own
copy. Pinned twice on purpose: the two copies exist to be independent,
and an independent copy with no test of its own is not independent
evidence, it is an untested duplicate.

### `fn the_application_and_shell_streams_do_not_contaminate_each_other`

If a future prefix change made one a prefix of the other, this test is
what says so — and the symptom otherwise would be a check that reads a
`ribbon-command-invoked` that is not there, or misses one that is.

### `fn the_threshold_separates_pressed_from_unpressed_under_both_palettes`

The second assertion is the one that matters: `AA_LARGE` is 3.0 and
these fills are 1.5:1 and 1.3:1 apart, so a check written against the
harness's usual legibility oracle would report "no difference" about a
control that is visibly blue.
