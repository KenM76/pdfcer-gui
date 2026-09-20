---
name: adding-a-second-route-is-an-audit-of-the-capability
description: A capability reachable by one route has untested other routes — wiring a second door reliably finds a divergence the first was hiding
metadata:
  type: feedback
---

**When you give a capability a second route, expect to find that its existing
routes disagree.** Budget for it; do not treat it as scope creep.

**Why:** 2026-08-28, twice in one afternoon.

- Adding a **form-field context menu** exposed that the Delete *key* deleted a
  selected widget and `format.delete`, the *command*, did not. Two Deletes
  acting on different things — the exact defect `app::keyboard`'s single
  dispatcher exists to make impossible — invisible because the command's only
  route was the Format tab, which is not drawn for a form selection.
- Adding a **`canvas.text` menu** for reflow exposed that a caret is in neither
  `SelectionState` nor the object hit test, so no existing menu could ever
  resolve for one.

Both were live divergences, in shipped code, found within minutes of the second
door being cut.

**How to apply:**

- The mechanism is simple: a capability's gate, operand derivation and refusal
  wording get written once per route, and only the exercised route is kept
  honest. A route nobody uses is a route nobody notices is wrong.
- **Check the gate first.** `format.delete` was `enabled_when("selection.any")`
  and a form field is not in `SelectionState`, so every item on the new menu
  would have resolved *disabled* — and an all-disabled egui menu **does not
  open at all**. The feature would have shipped as "right-click does nothing".
- Then check the **operand**: does the new route's dispatch reach the same
  `if let` ladder the old one does? Read them side by side; do not assume.
- Then check the **refusal**: if the old route declines silently in a state the
  new one can reach, the new route inherits a silence somebody will report as
  a bug.

★ The corollary, which is the useful half: **if you want to know whether a
capability is really wired, add a route to it.** It is cheaper than an audit and
it finds the same things.

Related: [[the-canvas-is-the-primary-surface-never-a-panel]] — the reason a
second route keeps being needed at all is that the panel route is never the
one he reaches for.
