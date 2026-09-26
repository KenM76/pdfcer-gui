# `ui-verify/checks/left_rail`

`the_left_rail_is_reachable_and_constant_width` — the strip O123 part 7 asked
for, proved on a running build.

# What this is for — `OPERATOR_REQUESTS.md` **O123** part 7 and **O126**

> *"the navigate selectors and some other related selection controls (lasso
> tool when we implement one, etc) and these will fold up into a drop down
> arrow if space becomes scarce."*

> *"also add rotate pages to that area, and those should be available in
> every mode including read."*

# Why this check has to exist, and why it is THIS check

**The rail is the surface that hides an unreachable panel.** Bookmarks,
Layers and Signatures can ship **unreachable** — each with a rail entry,
each publishing a perfectly healthy rectangle, and **every gate green** —
whenever the dock's rect channel publishes *layout* while the checks reading
it treat the answer as *visibility*. Converting that channel to visibility
was a **precondition** for scheduling the rail at all, on exactly the ground
that no driven check can otherwise tell a working rail from that defect.

So every region this check reads comes through
`crate::diag::ui_rect_visible`, and a `declared` here is a claim about
**reachability**, not about layout. That is what makes the check worth
running rather than a re-statement of the unit tests.

# The five assertions, and why none is redundant

Each names the build it fails on — that is what makes it an assertion
rather than a description.

1. `dock.left.toolrail` is on screen in **every** mode. Fails on a rail
   reserved but not drawn, and on a rail that only exists in Edit.
2. The panel-tab rows in [`TABS`] are reachable in **every** mode. Fails on
   an unreachable panel tab — the one thing the fold ladder may never
   produce.
3. The rail's **x-extent is the same in all three modes**, and is
   `WIDTH_PTS`. Fails on a rail sized from its widest word: the R128
   fit-zoom loop, which a unit test can only assert about a number the
   renderer might not use.
4. `pages.rotate_*` are reachable **in Read**. Fails on a build that
   quietly mode-gated them back, which is a silent reversal of an operator
   decision.
5. `view.tool_node` is **absent in Read** and present in Edit. Fails on an
   authoring control sitting on a mode whose own dispatch refuses it.

Assertion 3 is the one that cannot be had any other way. The unit test
`the_width_is_constant_at_every_rung_and_every_budget` asserts that the
**planner** reports a constant; it cannot assert that the **renderer** used
it. Only a rect from a running build can, and only by comparing across
modes whose contents differ — which is why this check switches modes rather
than measuring once.

# NOT RUN

**Nothing in this file has been executed against a running binary.** The
operator's standing instruction is that missing driven verification must not
stop the work or the release — it must be *named*. It is named here and in
the report, and until a run happens every claim above is a claim about the
code rather than about the program.
