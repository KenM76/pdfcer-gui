# `canvas::input` — reading one frame's pointer: what it landed on, what it is panning, and where the gesture is kept

## Why this is a module rather than four functions at the bottom of [`super`]

Rule R2's 1,500-line ceiling forced a split when the rulers landed, and
this is the seam it forced — the same way it produced [`super::trace`] when
Phase 4 added the strip, and [`super::strip`] alongside it. Both of those
headers record that the forced seam turned out to be a real one, and so
does this.

Everything here answers **"what is the pointer doing this frame?"**, and
every one of them is a question with a single, local answer:

| function | question |
|---|---|
| [`probe`] | what a click landed on, at every rung of the selection ladder at once |
| [`pan_delta`] | whether *either* of the two panning gestures is in flight, and how far it moved |
| [`load_gesture`] / [`store_gesture`] | where the in-flight press lives between frames |

What is left behind in [`super`] answers a different question — *how is the
frame composed?* — and it is a question about layout, the scroll area, the
strip and the order the overlay is painted in. Nothing here needs any of
that: [`probe`] needs a provider and a mapping, [`pan_delta`] needs an
input state and a rect, and the two `Memory` accessors need a `Context`.

## The one thing that is still in `egui::Memory`, and why

[`GESTURE_MEMORY_KEY`]. The selection moved off `Memory` and onto
`OpenDoc` at stage S4 because it is **document-scoped** state and `Memory`
outlives documents; the argument, and the address-as-identity hazard that
came with the workaround, are in [`crate::app::state::OpenDoc::selection`].

A gesture is the opposite case and it is worth being explicit about why.
The drag that is happening *right now* is genuinely frame-local UI state.
It has no meaning across a document, and a gesture that survived one would
be a drag continuing over a file it did not start on. Keying it in `Memory`
means it cannot: `Memory` is per-`Context`, and every document change
starts the next frame with no press in flight — by construction, with
nothing to compare and nothing to forget.
