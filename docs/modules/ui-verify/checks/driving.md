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
