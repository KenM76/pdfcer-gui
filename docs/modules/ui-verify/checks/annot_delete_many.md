# `ui-verify/checks/annot_delete_many`

`several_annotations_delete_as_one_undo_step` drives `fixtures/annots-align.pdf`
(see `annot_multiselect.md` for its geometry) under `--no-input`, with
`mode.edit` and a two-entry `PDFCER_DIAG_SAVE_PATH` queue. It has no SKIP arm.

It also relies on the locked Square `/NM (LO)` `[20 20 80 50]` (`/F 132`).

1. Require `status-group:edit-disclosure` not yet declared.
2. Click the square; Shift-click the circle, the link and the locked square,
   requiring `annot-toggled after=2`, `3`, `4`.
3. Press Delete; require `delete-annotations-applied deleted=3/3
   kept_locked=1 one_step=true`, then the disclosure declared (the kept
   locked one is said). A Delete that acts on the primary only traces
   `delete-annotation` and no `delete-annotations-applied`.
4. Save a copy; require SQ, CI and LK off the page's `/Annots` and LO and
   the ce dimension still on it. A saved copy can keep a deleted body, so the
   check reads the page's `/Annots`, not the file's objects.
5. One `Ctrl+Z`; save a copy; require all five on the page. Unfolded deletes
   bring back only the last.
