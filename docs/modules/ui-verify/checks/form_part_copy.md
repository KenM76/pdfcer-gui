# `ui-verify/checks/form_part_copy`

`a_part_of_a_placed_drawing_can_be_copied_and_pasted`, on
`fixtures/form-parts.pdf`. Drives with the scripted pointer, so it runs under
`--no-input`. The copy writes the OS clipboard; the check saves it first and
puts it back afterwards (`os_image_paste::ClipGuard`).

PASS needs, in order:

1. A click and a double-click at (240, 120) select leaf 1, the polyline.
2. Copy writes `clipboard-copy kind=form-leaves page=0 leaves=1 items=1`.
3. Edit ▸ Paste writes `paste-objects-applied page=0 pasted=1`.

## Falsified

- The dispatcher sending the parts to the page copy: FAIL at step 2, no
  `clipboard-copy` line (the page copy refuses them as inside a drawing).
- Not caught: skipping the in-memory store. The paste then reads the clip
  the copy also put on the OS clipboard, and lands the same.

## What it does not prove

- Where the paste lands, and at what size; the oracle is the engine's count.
- Cut, and a mixed selection of page objects and parts.
- Dragging the selection to another window.
