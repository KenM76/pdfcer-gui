# `pdfcer-gui/text/print/position`

## Item notes

### `fn position_heading`

It names the *sheet*, not the preview. The operator is not arranging a
picture on screen; they are choosing which part of an oversized drawing
reaches paper, and the preview is only how they see it. A heading reading
"Preview position" would describe the wrong thing and invite the reading
that this is a view control like the zoom beside it.

### `fn position_mm_suffix`

Whole millimetres are this dialog's unit everywhere — see
[`sheet_from_driver`] — but the entry keeps a decimal, because an
arrow-key nudge steps by a millimetre and a control whose number does not
move when the operator presses a key reads as a control that is not
listening.

### `fn position_frame`

A displacement is meaningless without an origin, and this one's origin is
not the sheet corner — it is *wherever pdfcer put the page*, which is
centred for a page that fits and flush to the top-left corner for one that
does not. So `0, 0` does not mean "at the corner", it means "where pdfcer
chose", and that is the sentence the operator needs in order to read the
Reset button as anything other than a synonym for Centre.

The sign convention is stated for the same reason: positive-is-down is the
device's sense and the preview's, and it is the opposite of the sense a PDF
page uses, so leaving it to be inferred invites exactly one wrong guess.

### `fn position_centre`

This is **not** the same command as [`position_reset`], and the whole
feature turns on the difference. pdfcer places an oversized page flush to
the top-left corner of the printable area, so that as little of it as
possible falls off the sheet. Reset returns to that corner; Centre moves it
to the middle, which crops the drawing evenly on all four edges. Both are
wanted, and an operator choosing what to lose off a big drawing wants the
second one far more often.

### `fn position_reset_all`

Its own control rather than a modifier on Reset, because the two have
different scopes and a job may have a hundred sheets: an operator who has
nudged nine drawings and wants the tenth back needs the narrow one, and an
operator who wants to start over needs the wide one. A single button that
did whichever the modifier key said would make the wide, unrecoverable act
the one nobody can see.

### `fn position_moved_count`

Drawn beside [`position_reset_all`] so the wide button's scope is visible
before it is pressed. One is the commonest count and reads oddly in a
plural, so it gets its own sentence.

### `fn position_extends_past`

Operator request O208, his second clause: *"the hash lines we use to show
what won't be printed should have a line for each edge of the page."* The
hatch answers that on the picture; this answers it as a number, because a
hatched band tells an operator that something is over the edge and not by
how much — and "how much" is the quantity they are adjusting.

# Why this is worded as geometry and never as loss

It says the page *extends past* the printable area. It does not say content
will be lost, because on a 1:1 CAD drawing the overhang is usually empty
paper — which is the whole of operator request O113, and the ink verdict
beside it is what gets to make the claim about content. Two surfaces making
overlapping claims about the same risk is how a dialog comes to contradict
itself, so this one keeps to the measurement it can make honestly.

For the same reason it is never drawn in the warning colour. It is a
readout of a number the operator is steering, not an alarm.

### `fn position_fits_entirely`

A control with a line under it in every other state and a blank in one
reads as a control that failed, which is the same argument
[`paper_auto_nothing_to_measure`] makes.

### `fn position_drag_hint`

The drag is the primary gesture and the buttons are the shortcuts, so the
gesture is what this names. It is here rather than only under the preview
because the preview can be popped into a window of its own, and a hint that
only exists on a surface the operator has moved elsewhere is a hint that is
not there.

### `fn position_page_label`

The controls act on the sheet the preview is showing, not on "the
current page" of the document — the job may be a narrowed range,
odd/even filtered or reversed, so those two are different numbers. Naming
it removes the one ambiguity that would make a per-page setting
untrustworthy: an operator who cannot tell which page a button applies to
will not press it twice.
