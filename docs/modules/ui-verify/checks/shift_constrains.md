# `ui-verify/checks/shift_constrains`

`shift_constrains_a_resize` — **Shift preserves aspect**, driven end to end,
and proved by the *difference* between two drags in one process.

# What this is for

`ui-conventions/drag-moves.md` D5 — *"modifiers constrain, and the
constraint is announced"* — was found **absent from every drag in this
shell** by the conventions sweep of 2026-08-20, and Shift-preserves-aspect
is the sharpest instance: it is *the* resize convention, present in every
program in the class for thirty years. An operator who holds Shift and gets
a free-form resize does not conclude that pdfcer chose differently; they
conclude it is broken.

# Why this is a PAIR of drags and not one

Because the only trustworthy oracle for a constraint is a **comparison**.

A single constrained drag reporting `sx == sy` proves nothing on its own:
the south-east grip dragged along a rough diagonal produces near-equal
factors anyway, so a build that ignored Shift entirely could satisfy that
assertion by luck and would then satisfy it in CI for ever. What cannot
happen by luck is *the same travel producing unequal factors without the key
and equal ones with it*.

So the check drags **deliberately lopsided** — far on x, barely on y, both
measured as fractions of the selection box's own extents — twice from the
same grip, with an undo between them, and asserts:

| | assertion | what a wrong build does |
|---|---|---|
| 1 | unmodified: `sx` and `sy` differ materially | if they do not, the fixture or the travel is wrong and the check SKIPS rather than passing — a check that cannot distinguish must not claim to |
| 2 | with Shift: `sx == sy` | a build that ignores the modifier reproduces run 1's unequal pair |
| 3 | with Shift: the kept factor is the **dominant** one | a build that averaged, or that always took `sx`, passes 2 and fails here on the run where y dominates |
| 4 | a `constrain lock=Aspect` line was traced | the arithmetic happened and the operator was told; D5's second clause |

Assertion 3 is the one that earns the lopsided travel. `aspect` keeps the
factor further from unity, and the cheapest wrong implementation — *"take
sx"* — is indistinguishable from the right one on any drag where x happens
to dominate. The check therefore drags **x-dominant** and asserts the kept
factor equals the x factor of the *unconstrained* run, which is a number the
wrong build has no way to produce for a y-dominant travel.

## "Dominant" means RELATIVE travel, and getting that wrong cost a run

`aspect` keeps the factor further from unity, and
`canvas::constrain`'s header derives why that metric *is* relative
travel: `factors` computes `s = 1 + d/extent`, so `|s − 1|` is the travel
expressed as a fraction of the box's own extent on that axis. **Dominance is
therefore a property of the drag AND the shape together, never of the drag
alone.**

Until 2026-08-29 this check chose its travel in **screen pixels** — 90 by 12
— and then asserted that x was dominant. On `SW41177` the selection box is
390.6 × 41.0 px, so 90 px is 0.230 of the width while 12 px is 0.293 of the
height: the drag was **y-dominant in the only space that decides**, the
shell correctly kept `sy = 1.9888`, and the check reported it as *"the wrong
factor"*. The shell was right; the constant was wrong. It is now a fraction
of the shape, so the premise assertion 3 rests on is true by construction on
any fixture. See [`DRAG_X_OF_SHAPE`] for the measured numbers.

# Why the trace and not the pixels

This project's standing rule is that *a trace can say the verb ran and
cannot say the screen changed*, and that every layout, repaint or clipping
defect has exactly one oracle: a rendered screenshot.

This is not a layout defect. What is being asserted is **arithmetic that
reaches the engine** — `resize-commit` carries the two factors and
`move-nodes` proves they became an edit — and a screenshot could not
distinguish `sx=1.42, sy=1.42` from `sx=1.42, sy=1.39` on any shape an
operator would draw. A capture is nonetheless attached on the failure
branch, because the one thing this check cannot see is whether the *caption*
rendered, and a human reading a failure will want to look.

# What it does NOT cover, said rather than implied

The three axis-locked drags — a move, a perimeter vertex and a Bézier
handle — are not driven here. They share `constrain::axis` with this one and
it is unit-tested, but *sharing a function is not the same as reaching it*,
and this project has shipped three features whose parts were all correct and
whose join was unobserved. Named as a gap in `OPERATOR_REQUESTS.md` O14
rather than left to be discovered.
