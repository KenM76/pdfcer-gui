# `ui-verify/checks/title_build_stamp`

`the_title_bar_carries_the_build_time` — the window title ends with the date
**and the time** this binary was compiled.

# The operator's ask, `OPERATOR_REQUESTS.md` O101

> *"also in the next release add the local compilation time to the top bar at
> the end of the date you added."*


⇒ So the failure this guards against is not cosmetic. A title that silently
lost its time would put the project back to spending mornings on defects that
do not exist in the build on disk.

# The one check in this suite that needs no input at all

The title is published as `window-title "..."` whenever it changes, and the
window is placed with `PDFCER_DIAG_VIEWPORT`, which lays out a real window
**without taking focus**. So this reads a trace line from a launched process
and asserts on it — no pointer, no keyboard, nothing that competes with
whoever is using the machine.

That is worth naming rather than just doing: most of this suite is gated on
`--allow-input` and therefore on the operator being away from the desk. A
check that can run at any time is a check that can run *often*.

# What is asserted, and why the zone rule is the interesting part

`PDFCER_BUILD_TIME` has two producers and they disagree about zone:


A packaged build's time is already local, so its offset is noise to somebody
standing in that zone and is dropped. A dev build's is UTC, and showing
`06:25` bare would invite reading an hour that is not the wall clock — so
`UTC` is kept. `build.rs`'s own sentence is the rule: *a stamp that says the
wrong hour is worse than one that says a true hour in a named zone.*

⇒ Hence the third assertion: **a raw offset must never survive into the
title.** That is what the obvious simpler implementation — truncate to
sixteen characters and stop — would get wrong in one direction, and what
"just print the whole stamp" would get wrong in the other.

# What a passing run does NOT prove

That the time is *correct* — that the stamp matches when the binary was
actually compiled. Nothing observable from outside can establish that, and
`build.rs` owns it. This asserts the **shape** reaches the operator, which is
the part that has silently regressed before.
