# `pdfcer-gui/text/status/resize`

## Item notes

### `fn resize_not_rebuildable`

`OPERATOR_REQUESTS.md` O51. Two sentences, chosen by whether the drag was
proportional, and the split is the whole value of the message: **only one of
the two switches helps in each case**, and naming the wrong one would send
the operator to a control that changes nothing.

| drag | what fixes it |
|---|---|
| proportional | *Scale line weight* — the resize then comes out **exact** |
| not proportional | nothing fixes it; only *Allow the artwork to distort* proceeds |

**It does not say "cannot".** The operator resized a shape and got
nothing; what they need is the next click, not a diagnosis. Both sentences
name a switch by the words on it, and the non-uniform one is honest that the
result will be imperfect rather than dressing the option up.

Neither sentence mentions appearance streams, placement matrices or
§12.5.5. The *reason* is real and is written down in `canvas::scaling`; what
belongs in a status bar is what to do. A sentence that explained the matrix
would be correct, unactionable, and too long to read where it appears.

*"pdfcer did not draw this shape"* is in the uniform sentence because it is
the part an operator can verify and act on — shapes pdfcer drew resize
perfectly, so the message quietly tells them the difference between the two
kinds of object on their page.

### `fn resize_fixed_size_marker`

# Move is the remedy, and the sentence says so first

A sticky note's box has no size a reader honours — it draws the icon at one
size and reads the box only for **where**. The operator who dragged a
geometry field and got nothing needs the verb that does work on this
object, which is a drag of the note itself. Neither sentence says
"cannot": the object has a property, position, and the sentence names it.

# Why two sentences

The engine's error carries a `why` that is either the subtype's rule (a
`/Text` is always fixed-size, 12.5.6.4) or the annotation's own `NoZoom`
flag (12.5.3). The first is not the operator's to change; the second is,
in principle — a flag can be cleared — but this shell offers no flag
editor, so the sentence states the fact without promising a switch. When
one exists it belongs in the `by_flag` sentence and nowhere else.

# Neither sentence mentions `NoZoom` by its PDF name for a sticky

An operator who placed a sticky note did not set a flag and would not know
what one is; "drawn at one fixed size" is the fact in their terms. For the
flag case the name is kept, because a foreign producer set it and the
operator may be looking at the file elsewhere.
