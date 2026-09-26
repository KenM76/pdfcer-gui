# `ui-verify/checks/display_two_rows`

`the_display_buttons_stack_in_two_rows` — the View tab's page-display group
is laid out two-high, not four-across.

# The operator's ask, `OPERATOR_REQUESTS.md` O97

> *"our display buttons should be on two rows to save space."*

Four small commands — single page, continuous, facing, facing continuous —
that had been sitting in one long row and pushing everything to their right
toward the overflow.

# Why this is a DRIVEN check and not a unit test, when the layout is
pure arithmetic

It looks like the ideal unit-test subject: `egui_shell::ribbon::plan`
computes the wrap, it takes a group and returns rows, and it is tested. But
the thing that can break is not the arithmetic — it is whether the **hint
reaches it**, and there are four places the chain can be cut, none of which
the planner's own tests can see:

1. `prefer_rows: 2` is absent from the group in `built_in.ron`, or the
   regenerated manifest silently drops it;
2. the RON round-trip loses the field, because `Group`'s `Default` supplies
   `None` and a missing key is indistinguishable from an unset one;
3. the band builder reads the group's items but never asks for its hint;
4. the hint arrives and the **fits-already short-circuit runs first** — the
   planner's ordinary rule is "one row if it fits", and four small buttons
   fit at almost any width, so a `prefer_rows` that is consulted *after*
   that test is a `prefer_rows` that never does anything.

The fourth is the one that matters, and it is why this check pins the
window to a **fixed wide viewport** before measuring. At a narrow width the
group might wrap for the ordinary reason and the check would pass over a
build where the hint is dead. At 2560 pt there is abundant room, so **the
only reason these four can be on two rows is that somebody asked for it.**

A fixed viewport rather than "maximise", because maximise gives whatever
monitor the run happens to be on — reproducible on one machine and not
across two, and silently weaker on a laptop.

# The oracle is geometry, not pixels

Every ribbon item publishes its rectangle. Four rectangles falling into
exactly two distinct vertical bands, two per band, is the whole assertion —
no screenshot, no colour, nothing that a theme change could disturb. That is
unusually cheap for a layout claim and is worth saying, because the
surrounding notes on this feature had assumed it would need a rendered
image.

# What a passing run does NOT prove

That the rows are in a sensible **order**, or that the group is narrower than
it was. Order is a manifest question and is asserted where the manifest is
tested; width is what two rows buys and is not independently measured here,
because "narrower than the one-row version" would need a second run of a
build that does not exist.

## Item notes

### `const VIEWPORT`

**The width IS the precondition of the assertion**, not a convenience.
See the module header's point 4. It also does not steal the desktop:
`PDFCER_DIAG_VIEWPORT` switches `with_active` off, so the window lays out
fully without taking focus.

### `const SAME_ROW_PT`

Deliberately small. Buttons on one row share a `y` exactly in `egui`'s
layout, so any tolerance at all is generous; 4 pt allows for the harness
rounding a scaled coordinate and nothing else. A large tolerance here would
quietly merge two genuinely-stacked rows on a compact theme and report the
feature missing on a correct build.
