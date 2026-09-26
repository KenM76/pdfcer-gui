# `pdfcer-gui/canvas/forms/tests`

## Item notes

### `fn an_edit_reseeds_the_draft_and_a_different_document_discards_it`

The difference from the panel, pinned. Dropping the focus on an epoch
change would take the caret out of a field the operator had just
clicked into, because clicking field B while A is focused commits A and
therefore moves the epoch on the very next frame.

### `fn escape_is_claimed_exactly_once`

Claimant 0's contract. A flag that survived its reading would spend the
*next* Escape as well — which the operator would experience as a press
that failed to ascend the selection ladder for no visible reason.

### `fn the_panel_reads_the_pages_draft_only_while_the_page_holds_the_keyboard`

The 2026-09 review's row **A12c**, pinned from both sides. The positive
half is the fix: a stored [`Focus`] whose editor owns the keyboard is
published to [`crate::panels::forms::rows`], so the panel row beside the
field stops showing the value from before the gesture started.

The three negative halves are the safety argument, and each of them is
a way for this fix to become a worse defect than the one it replaces:

* **No `egui` focus** — a `Focus` outlives the frames the editor is
  actually focused for, and a panel is drawn on every frame. Publishing one
  then would let a stale canvas draft overwrite a live panel one, which is
  A12c pointing the other way.
* **A different revision** — an edit landed, and the panel drops its own
  drafts on exactly that boundary (`FormsUi::load`). Mirroring across it
  would hand back the one value that key exists to discard.
* **A different document** — every field name means something else.
