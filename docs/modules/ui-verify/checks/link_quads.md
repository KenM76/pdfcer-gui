# `ui-verify/checks/link_quads`

# `a_link_is_moved_resized_copied_and_deleted`

The fixture is `fixtures/link-quads.pdf`, built by its `PROVENANCE.py`. It is
one 400 × 400 page with one URI link at `/Rect [100 180 300 220]`. Its
`/QuadPoints` are inset inside the box: 20 pt in from each side and 5 pt in
from top and bottom. That inset makes "the quad followed the box" measurable:
a quad equal to the box would pass even if it had been rebuilt from `/Rect`.

The check is driven with `PDFCER_DIAG_INVOKE=mode.edit`.
`PDFCER_DIAG_SAVE_PATH` answers the save picker, and
`PDFCER_DIAG_CLIPBOARD_DIR` captures the clipboard, so the operator's own
clipboard is never touched.

1. Click (200, 200) and require `annot-select subtype=Link`.
2. Drag the body to (200, 120), which moves the box to `[100 100 300 140]`.
   Require `move-annotation-applied`.
3. Drag the top-right grip from (300, 140) to (340, 160). Require
   `resize-annotation-applied`.
4. Press Edit ▸ Copy and require `clipboard-copy annots=1`. Press Edit ▸
   Paste and require `paste-objects-applied annots=1`.
5. Press File ▸ Save a copy and require `save-copy`. Read the saved bytes.
   The links are those that the last definition of a page lists in
   `/Annots`, each read from its object's last definition. Require two
   links. For each link, its `/QuadPoints` must be the fixture's quad
   fractions mapped into its current `/Rect`, within 1 % of the box's width
   or height.
6. Click (150, 130), inside the moved link, and press Delete. Require
   `delete-annotation`. Save again and require one link.

The `/Annots` filter in step 5 is needed because an appended save keeps a
deleted object's original body in the file. Defined is not the same as on
the page.

## Falsified

- With the shell also translating the clip's geometry keys before the
  engine's paste (which moves them itself), the check fails at step 5: the
  pasted link's `/QuadPoints` moved twice.
- With delete turned into a successful do-nothing edit, the check fails at
  step 6: the trace says `delete-annotation` and the file still holds two
  links.
