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

## Item notes

### `const ENABLED_KEY`

Application-scoped rather than per document, like the armed tool and the
pick filter: it is a statement about how this operator works, not about a
file, and re-answering it per document would be a question asked again for
no new reason.

### `fn inside_a_container_a_leaf_resolves_to_itself`

Without this the feature would be a cage: the operator could select a
container and never anything in it, which is strictly worse than the
behaviour it replaces.

### `struct Entered`

Carries the page and the document epoch with the object index for the reason
in the module header: the record invalidates itself rather than relying on
five call sites to clear it.

### `fn enabled`

# Why the default is ON, when the operator asked for a checkbox

Because the checkbox exists so the behaviour can be turned **off**, and the
behaviour is what every program in the class does. A default of `false`
would ship the convention switched off and leave the complaint that produced
this row — *"I'm still not entirely clear how to reliably get to a point
where I can edit nodes"* — answered only for an operator who found a
checkbox first.

### `fn sync`

# Why this is not [`set_enabled`], which looks like the same function

`set_enabled` is what a **press** calls, and it also leaves whatever
container the operator is inside — because switching the mechanism off while
scoped to a container would leave the next click resolving by a rule that is
no longer switched on.

This runs every frame from `app::frame`, changed or not. Leaving on every
call would make entering a container impossible; leaving on every *change*
would make it identical to `set_enabled` and the two would not need to exist
separately. It does neither — it writes the value and nothing else, and the
consequence lives on the press path where the operator's intent is.

⇒ The direction is strictly `Prefs` → memory. The only writer of the
persisted answer is the dispatch arm an operator's press runs, so there is
one source of truth and one mirror of it.

### `fn entered`

Returns `None` — and clears nothing — when the record names another page or
another document. Reading is not the place to write; the record is replaced
the next time one is written and is harmless meanwhile.

### `fn leave`

The `bool` is what makes Escape composable. `canvas::keys` consults this as
its **last** claimant — one press clears the selection, a second steps back
out of the container — which follows that ladder's own rule of retiring the
most transient thing first: a selection inside a title block is remade by
every click, while the fact that the operator is working inside it survives
all of them.

### `struct Scope`

# Why a value rather than reading the context at each call site

The pick helpers in [`crate::canvas::input`] are pure functions over a
`&dyn CanvasTargetProvider`, deliberately: they are the most heavily
unit-tested code in this crate and they answer *"what did this click
land on?"* without a running application. Handing them an `egui::Context`
to consult would make every one of those tests build a context and would
put a global read in the middle of a hit test.

⇒ So the scope is read **once**, at the surface that has the context, and
travels as two facts. Every consumer then resolves identically by
construction — which is the property that matters, because a press and the
click that follows it must agree about what is under the pointer or a drag
starts on one thing and selects another.

### `fn resolve`

The whole rule, in one function so that the click path, the press path
and any future caller cannot each have their own version of it:

| switched on | target | inside its container | resolves to |
|---|---|---|---|
| no | anything | — | itself — the behaviour before O70, unchanged |
| yes | a page object | — | itself |
| yes | a **leaf** | no | its **outermost container** |
| yes | a **leaf** | yes | itself |

The third row is the whole feature and the fourth is what stops it
from being a cage: once you have entered a title block, clicking its
lines selects its lines.

### `fn scope`

The document slot comes from `crate::pagedrag::active`, which the frame
publishes before any surface draws — the same source the Pages panel uses
to know which document it is showing, so *"which document is this?"* has
one answer in this crate rather than two.
