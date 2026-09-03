---
name: the-canvas-is-the-primary-surface-never-a-panel
description: If the engine can do it, the object must be clickable and editable ON THE CANVAS — a properties panel is a supplement, never the route
metadata:
  type: feedback
---

**Anything the engine can do to an object, the operator must be able to do by
clicking that object on the canvas.** A properties panel is a *supplement* — a
precise route for typed values — and is never the answer to *"how do I move
this."*

**Why:** Ken, 2026-08-28, on finding he could not select a checkbox he had just
placed:

> *"Note that if the engine is capable, I should be able to select the object and
> do all of the ordinary editing one would expect a GUI editor to be able to do.
> It is great that there is a properties box that allows this, but **always
> always always** I need objects on the canvas to be clickable and editable as
> one would expect given our research of other programs."*

The triple *always* is the emphasis he chose. This is a standing acceptance
criterion, not a preference about one control.

**How to apply:**

- When an engine verb lands, the question is **"what gesture reaches it on the
  canvas"** — not "which panel gets a field". Four numbers and an Apply button
  are a form for editing a rectangle; **dragging is how a person moves a box.**
- Ordinary GUI editing means, at minimum: **click to select, drag to move, grips
  to resize, Delete to remove, context menu for the rest.** If the engine
  supports a verb and no gesture reaches it, that is an unbuilt feature — say so
  in those words rather than pointing at the panel.
- **The panel is not evidence the feature exists.** Twice now a capability was
  reachable only by typing into the Properties panel and reported as done: form
  field geometry, then annotation geometry. Both read as missing to him.
- Check the *whole* interaction, not the one verb: after placing an object, is
  it selected? Is the tool still armed? He met this as *"I can't select it"* when
  the cause was that the placement tool stayed armed and ate the next click.
  See [[scope-a-request-to-the-whole-expected-behaviour]].

**★ The tell that this rule is being broken:** a driven check needs a step the
operator would never know to take. `dragging_a_form_field_moves_it` has to press
Escape before it can select what it just placed, and that step was written into
the harness as a workaround with a comment saying *"exactly as a markup pen
does"* — treating it as normal. It was the defect, recorded as scenery.

⇒ **When a harness needs a workaround to reach an ordinary gesture, that
workaround is a bug report.** Read it as one.

Related: [[use-the-conventional-interaction-never-invent-one]] — the convergence
of the product class is the spec, and it covers what happens *after* an action
as much as the action itself.
