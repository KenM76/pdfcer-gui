# `pdfcer-gui::app::dispatch::snapshotclip`

The first rung of `edit.copy` (O272). While a snapshot box is laid, Copy means
the box: the operator laid it to say which area they want, and no other operand
— a selection, a field, page text — can be what the chord is about while it is
there. The box exists only while View ▸ Snapshot is armed, so the rung costs
nothing when the tool is down.

## Contract

- `owns_copy(app)`: a document is open and `OpenDoc::snapshot` is set.
  `dispatch::clipboard::copy_or_cut` asks it before every other rung, for
  `edit.copy` only. Cut has no meaning for a box and falls through to the
  ordinary rungs.
- `copy(app)` calls `clipboard::snapshot::copy_snapshot` and reports:
  - on success, trace `clipboard-snapshot-copy dpi= asked= w= h= vectors=
    formats=`, where `vectors=` is `cut`, `withheld` or `refused` and the
    formats are comma-joined in placement order, and the status sentence
    `text::snapshot::copied`, which names the pixel size and the dpi, says when
    the dpi was lowered to fit, says whether the vectors went and why not, and
    carries the engine's notes;
  - on refusal, trace `clipboard-snapshot-copy-refused reason=` with one of
    `no-box`, `render`, `would-degrade`, `clipboard`, and the page copy's
    refusal sentence.
- Reached from the Copy chord and from the box's right-click menu,
  `canvas.snapshot`, whose one row is `edit.copy`. Copy is allowed in every
  mode, so the box copies in Read.
