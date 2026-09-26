# `canvas::smart` — **click selects the whole thing; double-click goes
# inside it**

## The request


> *"we should have a checkbox in navigate for a Smart-Selector option, in
> Edit Mode this makes it so if I click on an object and it is enabled to be
> selected in the Smart Selector it puts it in a bounding box with handles to
> move, resize and rotate, if a click selects an object that is made of
> multiple objects (group, form, etc) a double click should bring me further
> down the chain, until a double click reaches the bottom and lets me edit
> the nodes. If I recall this is similar to how Inscape does things and we
> should follow that convention."*

He named the convention, so **the convention is the spec** — this shell's
standing rule about never inventing an interaction model. What follows is
Inkscape's group context, applied to the only container a PDF page actually
has.

## What this changes, and it is the opposite of what it sounds like

It sounds like *"add a way to go deeper"*. It is not. Before this module a
click on a title block selected **one line inside it**, because the deep hit
test returns a form XObject's interior and excludes the form itself:

```text
provider::hit_test_all → [Leaf(1180), Leaf(1181), …]     // never Object(the form)
```

⇒ So the operator could reach the 10,256 leaves of a wrapped CAD drawing and
could not reach the drawing. The container was addressable only through a
Format-tab command (`format.select_form`) that they had to know existed.

**This module makes a click select the container and a double-click enter
it**, which is what every drawing program in the class does and what he
described. The interior is not less reachable than it was — it is one
double-click away instead of zero — and the container is reachable at all
for the first time.

## The chain, and where each rung already lived

| rung | reached by | who owns it |
|---|---|---|
| the **container** (a form XObject) | a click | this module |
| an **object inside it** (a leaf) | double-click *into* the container | this module |
| a **part** (a subpath, a text run) | double-click again | `SelectionState::descend` |
| a **node** (an anchor) | double-click again | `SelectionState::descend` |

Only the first two rungs are new. The bottom two are the ladder this shell
has had since S4, and they are deliberately untouched: a container is a
**scope**, not a fourth `SelectionLevel`. Adding a rung above `Object` would
have meant re-reading every `match` on that enum and every assertion that
*"Object is the rung a click starts on and Escape returns to"*.

## Why a scope rather than a rung, in one sentence

Because a form XObject **is a page object**. Selecting it is an ordinary
Object-rung selection of `TargetId::Object(paint_order)`; being *inside* it
is a fact about what the next click will resolve to, not about what is
selected. Inkscape models it the same way — the group context is a property
of the canvas, and the selection inside a group is an ordinary selection.

## Rule 4 — what is drawn and what is only said

Entering a container draws **nothing extra on the page**. The selected
object gets the selection outline it would get anywhere, which is a cursor
affordance and explicitly allowed. *Which container you are inside* is
disclosed **off-canvas**, in the status row, exactly as rule 4 requires and
exactly where Inkscape puts it (*"Entered group g1234"*).

A screenshot of the canvas while inside a container differs from a
screenshot of the same document saved and reopened only by where the pointer
is and what is selected. That is the one-line test, and it passes.

## Leaving, and why there are five ways

Every one of these leaves the container, and each is somebody's habit:

1. **Escape**, as the last claimant on that key — one press clears the
   selection, a second leaves. See `canvas::keys` for why the scope is the
   outermost rung of one ladder rather than a competitor for the press.
2. **A click outside the container's bounds** — Inkscape's other one.
3. **Changing page.** The record names a page; carrying it to another sheet
   would silently scope clicks to a form that is not there.
4. **Changing document.**
5. **Leaving Edit mode**, because the whole gesture is content selection and
   `caps.edit_content` is what grants that.

3–5 are enforced by **the record carrying its own page and document
epoch** rather than by five call sites remembering to clear it. That is the
`dialogs::placing` lesson applied one module over: a state that five routes
must remember to clear is a state one of them will forget.
