# `pdfcer-gui/app/state/heldpreview`

## Item notes

### `const HELD_PREVIEW_MAX`

A wall-clock backstop, because the epoch test alone can fail to fire at all:
a render that fails, a page that will not rasterise, or any path that leaves
`page_texture_epoch` behind strands the hold, and the operator is left
looking at a selection-coloured tracing of their drawing with no way to clear
it. A stuck preview is worse than a late one — it is indistinguishable from a
corrupted document.

Four seconds is roughly four times the measured whole-page raster on the
operator's hardest drawing, so it cannot fire on a render that is merely
slow.

### `const HELD_PREVIEW_GRACE`

This is the only thing separating *"not applied yet"* from *"refused"*.
Actions are drained after the frame that raised them, so there is a real
window — one frame, ~16 ms — in which a hold is legitimate and the epoch has
not moved; there is also a state in which the epoch never moves at all,
because the engine refused the edit. By epoch the two are identical; by
elapsed time they are not remotely alike. 250 ms is fifteen frames at 60 Hz,
far longer than the real window can be and far shorter than a refusal stays
wrong for.

Getting it wrong ships a preview of a move that did not happen, over a
document that disagrees with it, for the full [`HELD_PREVIEW_MAX`] — a
picture of a lie rather than a picture that is late.

### `const CATCHING_UP_AFTER`

# Why a threshold rather than "whenever it is behind"

The picture is behind after **every** edit — for a few milliseconds on a
simple page, for a second or two on a dense one. A sentence that appeared
every time would flash on and off on every keystroke, and a status line that
flickers is one the operator stops reading. That costs every *other*
sentence the bar carries, which is a far larger loss than this one is a gain.

400 ms is past the point where a person notices a wait and starts wondering
whether the program heard them. Below it, saying nothing is the correct
behaviour and not merely the cheap one.
