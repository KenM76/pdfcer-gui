# `pdfcer-gui/dialogs/tests`

## Item notes

### `fn an_answered_window_survives_its_own_close`

Four rows because the predicate has two inputs and each combination is a
real state: an open unanswered window (the normal case), an open window
that has just been answered (`answered` wins and it is kept — it cannot
arise today, since answering closes both windows, but the rule must not
depend on that), a cancelled window (retired, and it answers nothing —
which is what makes the ✕ non-destructive), and the one that cost the
day.

### `fn no_document_means_no_dialog`

The guard that stops a keyboard chord from enumerating the spooler —
a call that blocks on a network printer — to populate a window that
would be closed again on the next frame.

### `fn the_apply_dialog_is_guarded_on_both_counts`

Both guards matter more for this dialog than for any of its neighbours,
because opening it runs a full rewrite of the document. The second
assertion is the one with teeth: a rebuild would re-run that work *and*
discard the operator's two acknowledgements, throwing away the reading
they have just done on the one report in this program that has to be
read before a control is pressed.

### `fn no_document_means_no_diagnostics_dialog`

Its command is gated on `doc.open`, so the ribbon cannot reach this
state — but a chord can, and without the guard the dialog would be built
and then closed by [`DialogsState::show`] on the very next frame. A
window that flickers is harder to diagnose than one that never appears.

### `fn opening_the_diagnostics_report_twice_leaves_the_first_one_alone`

Nothing would be lost — it holds no configuration, and it reads the
texture live — but the window would jump back to the centre and the
findings list back to the top, which for an operator half-way down a
census is the program losing their place. About's argument, one dialog
over.

### `fn no_document_means_no_recognition_dialog`

Same guard as print's, and it matters for a different reason: the
dialog captures the page index and the document path on construction,
so one built against `Status::Empty` would have neither and would be a
window that could only refuse.

### `fn about_opens_without_a_document_and_survives_one_closing`

The one property that would have been lost by reusing print's shape.
`open_about` takes no `Status` precisely so this cannot regress by
someone adding a guard "for consistency"; the assertion is here so
that if they do, something says why it was not consistent in the first
place.

### `fn opening_about_twice_leaves_the_first_one_alone`

Nothing would be *lost* — it holds no configuration — but the window
would jump back to the centre and the attribution list back to the
top, which for an operator reading it is the program losing their
place.
