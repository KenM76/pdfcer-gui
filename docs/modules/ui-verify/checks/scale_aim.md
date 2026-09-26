# `checks::scale_aim` — getting to a zoom rung, and staying on the target


## The seam is a real subject boundary, not a size-driven cut

`scale_sweep` is now one thing: **what the mouse can do once you are
there.** Click-select, drag, marquee, nodes, handles, pan — a battery of
probes, each of which changes when a *gesture* changes.

This file is the other thing: **how you get there and stay on the point.**
The zoom ladder, the closed-loop re-aim, the tier reading, and the
vocabulary those three share. It changes when the *zoom or position model*
changes — a new render tier, a new position anchor, a different zoom
gesture — and it is entirely uninterested in what a press means.

⇒ The two change for different reasons, which is this project's stated test
for a module boundary (`canvas::interact`'s header makes the identical
argument about composition versus interaction).

## Why the aiming half is worth reading alone

Because it is the half that can make every measurement in the other file a
statement about the harness. The 2026-09-05 sweep filed *"clicking directly
on the content the zoom is anchored to selected nothing"* at five rungs;
what had happened is written in [`re_aim`]'s own header — the correction
moves the **pointer**, and Ctrl+wheel holds the point under the pointer
fixed, so an error the correction cannot close is magnified by every
further notch rather than reduced. It was a limit of this loop and not
anything the application did. A reader who wants to know whether a
scale-sweep finding is real starts here, and [`aim_residual`] is the number
that answers it.

## The dependency runs one way

`scale_sweep` uses this module; nothing here knows that `Rung`, the report
or any probe exists. The three trace names this layer reads live **here**
rather than upstairs, and are `pub` so the battery can share the ones it
also needs — one definition, imported upward, so a rename in the
application cannot leave one file corrected and the other quietly reading a
name that is never printed.
