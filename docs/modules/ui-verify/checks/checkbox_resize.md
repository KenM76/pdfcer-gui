# `ui-verify/checks/checkbox_resize`

`a_resized_check_box_is_redrawn_not_stretched` — **drag a check box bigger
and its border stays the weight it was.**

# The report


> *"Form shape outlines of checkboxes and such scale when I drag them
> larger."*

## The cause was neither of the two the row first guessed

The border did not thicken because pdfcer wrote a bigger `/BS /W`. It
thickened because **nothing was rewritten at all**: the engine rebuilt a
field's appearance for Text and Choice fields only, and a check box is a
`/Btn`. So the artwork pdfcer itself had drawn — at the original size, with a
hard-coded 1 pt stroke — was kept, and §12.5.5's placement matrix stretched
it into the new box. Drag a 12 pt check box to 40 pt and its 1 pt border
draws at about 3.3 pt.

That was an engine gap, filed rather than worked around, and answered by
`pdfcer-core` **Pass 187.0**: a `/Btn` appearance **pdfcer authored** is now
redrawn at the new size, and a foreign one refuses by name rather than
stretching. The shell's half is passing the operator's three scale answers
through `WidgetEdit::with_resize`, which the same Pass made possible.

## Why the oracle is `regenerated=`, not a pixel

**A screenshot cannot tell the two apart.** A border that thickened because
`/BS /W` changed and one that thickened because the placement matrix scaled
pdfcer's own artwork are *the same pixels*, and so are a redrawn 1 pt border
at the new size and a lucky crop. The distinguishing fact is which of three
things the engine did, and it says so:

```text
edit-widget-applied field=… widget=0 resized=true regenerated=true stale=false
```

| field | meaning | the defect's value |
|---|---|---|
| `resized` | the extent changed | `true` — it always was |
| `regenerated` | the appearance was **rebuilt** at the new size | `false` |
| `stale` | the engine says the artwork no longer fits | `false` either way |

⇒ `regenerated=false` on a resize IS the operator's complaint, stated
exactly. That is the same argument `markup_move` makes for reading `keys=`
and `scale_switch` for reading `stroke=`: where the picture is identical,
the trace is the only oracle that exists.

And note what a weaker check would have passed. `edit-widget-applied`
being present at all, or `resized=true`, is true on the broken build —
this check must read the third field or it is measuring nothing.

## The sequence

| # | step | oracle |
|---|---|---|
| A | drag out a check box, big enough to have grips | the widget census names it |
| B | clear, then click it | `form-field-selected field=…` |
| C | drag a corner grip outward | `resize-widget-commit … grip=…` |
| D | the engine redrew it | `edit-widget-applied … regenerated=true` |

Step A **drags** rather than clicks, and that is not a stylistic choice.
A clicked check box is authored at its default 14 pt, which on this sweep's
1584 pt sheet at fit zoom is four pixels — smaller than one grip's hit
square, so there is no corner to aim at and the gesture under test cannot be
started. The drag route is the operator's own (*"click to place the position
or drag a box for size"*, O53) and it makes the box big enough to have
corners.
