# `app::actions::redactsel` — **redact what is selected on the page**

## What this closes

**Ken:** *"the redaction tool — am I able to select objects on
the canvas and redact them that way yet? I only tried it when it only worked
with the search box and it didn't work for some things. it just told me it
couldn't."*

## Why the search box cannot do what he wanted

The other two routes to a redaction mark are:

| route | what it can reach |
|---|---|
| the search box | **text pdfcer can read as text** |
| mark whole page | everything, indiscriminately |

Nothing in between. And on a CAD drawing the first is far narrower than it
sounds: a title-block value drawn as **vector strokes**, a scanned stamp, a
logo, a signature image and a run in a font whose encoding pdfcer cannot map
are all invisible to a text search. There is nothing to type that would find
them.

⇒ *"It just told me it couldn't"* is the program being honest about a route
that genuinely cannot reach the thing — not a bug, and not something any
amount of retyping would fix. This module is **the route that does not go
through text at all.**

## What this is

Select anything on the page — a path, an image, a text run, several at once —
and mark it for redaction. The geometry comes from the **selection's own
bounds**, which the canvas already computes to draw the outline, so what gets
marked is exactly the box the operator can see around what they picked.

It is `EditSession::add_redaction`, the same verb the *mark whole page*
control uses, with the page's crop box swapped for the selection's bounds.
It needs no new engine capability: the verb takes arbitrary quads.

## Marking is not applying, and this changes nothing about that

A `/Redact` annotation is a **mark**: it removes no content. Applying is a
separate, deliberate, confirmed act (`edit.redact_apply`), and this route
feeds the same review list as the other two — the panel lists every mark
before anything is destroyed.

That is worth stating because a *"redact this"* control on a right-click menu
sounds like it destroys something immediately. It does not, and the wording
says so.

## Why the bounds and not the object's exact shape

A redaction region is quads (§12.5.6.23 `/QuadPoints`), so an L-shaped
polyline could in principle be covered by several. It is deliberately one
box per selected object, for two reasons that point the same way:

1. **A redaction that follows an outline tells you what was there.** The
   silhouette of a signature is a signature; the outline of a part number is
   its digit count. A bounding box discloses less, which is the entire point
   of the operation.
2. It is what the operator can see. The selection outline *is* the box, so
   the mark lands exactly where the preview said it would.

## Item notes

### `fn mark_selection`

One `/Redact` annotation per selected object, all in **one undo entry** —
`vector_edit` wraps the whole loop, so an operator who marks six things and
presses `Ctrl+Z` gets back the state before they started rather than five
marks and a headache.

# What it does when nothing is selected

Nothing, silently. The command is gated on `selection.any`, so a pointer
cannot reach it in that state — and a keyboard route does not exist for this
verb. A sentence here would be describing a state the operator cannot be in.
