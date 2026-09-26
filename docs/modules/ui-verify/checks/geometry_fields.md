# `ui-verify/checks/geometry_fields`

`geometry_fields_resize_a_shape` — **the typed X/Y/W/H route**, driven end
to end against the operator's own drawing.

# What this is for

`FEATURES.md` names this capability *"Editable geometry — X/Y/W/H in the
Properties panel, typed rather than dragged"*. It is built out of the same
machinery as the eight grips, and this check exists to stop it ending the
same way that machinery can end: **drawn, cursored, and committing
nothing.**

# ★ Why this is not covered by `resize_scales_a_shape`

Because the two routes share only their *last* link. The grip check proves
that `resizing::action`'s output reaches the engine; this one proves that a
**panel** can reach `resizing::action` at all, and the four links in between
are entirely different code:

| # | link | its own test |
|---|---|---|
| 1 | the Properties panel draws a geometry section when content is selected | `properties::geometry` — the plan, not the draw |
| 2 | the section's bounds come from the object's **anchors** | unit-tested, on synthetic points |
| 3 | the draft survives a frame and is not re-seeded under the operator | ★ **nothing** — it is a per-frame `sync` and a stale stamp is invisible to a unit test |
| 4 | Apply turns the draft into a command instead of staying greyed | ★ **nothing** |
| 5 | the command reaches the engine | shared with the grip check |

Link 3 is the one that would fail silently and plausibly. `sync` runs
**every frame**, and if its stamp were wrong in either direction the symptom
is not a crash: too eager and the operator's typing is wiped between the
keystroke and the button press, so Apply is permanently greyed and the
feature reads as *"the fields do nothing"*; too lazy and the fields describe
an object that has since changed. Both are states a running window shows in
two seconds and no unit test can construct, because the thing under test is
*the sequence of frames*, not the function.

# ★★ Why the harness SCRUBS the field instead of typing into it

An `egui::DragValue` takes a number two ways: click-then-type, or drag to
scrub. Typing needs a double-click to enter text mode, a select-all, a
keystroke per digit and a commit — six OS-level events whose failure modes
(a double-click misread as two clicks, an IME, a stuck modifier) are the
*harness's* and would be reported as the application's.

Scrubbing is one drag, and it is **arithmetically checkable**: the field
moves by `pixels × SPEED`, and `SPEED` is a named constant in the panel
precisely so this check can assert the number rather than assert that
something changed. ★★ **A harness assertion is a claim about the program
*and* about the harness**, so between two routes to the same state the
honest one is whichever owns fewer failure modes of its own.

# The oracle

`transform-objects-applied`, the same line the grip check ends on, **plus**
a `resize-scale` whose `sx` exceeds 1. The second is what distinguishes this
from a check that could pass on a build where Apply raised a *move* and no
scale at all — which is exactly what a `plan()` with its width comparison
inverted would do, and it would look like a working button.

★★ Deliberately **not** `resize-commit`, which is the *gesture's* line and
carries the grip that was dragged: the typed route never writes it, so a
check asserting on it reports "Apply committed nothing" against a build
where Apply works perfectly. ★★★ The rule that follows is worth more than
the check: **when two routes must agree, the trace line they are judged by
belongs in the one function both of them call** — here `resizing::action` —
and not in either route's own code, or the instrument measures the route
instead of the claim.
