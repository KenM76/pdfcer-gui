# `pdfcer-gui/canvas/forms/textbox`

## Item notes

### `fn lay`

# `/Q`, and it is the one placement property this editor reads

[`super`]'s §3 refuses to make this box a facsimile of the rendered widget,
and the refusal is **arithmetic**: the overlay is a font substitution by
construction, so a box pretending to be the appearance stream would put the
caret where the glyph is not going to land, and would be wrong by more the
longer the string.

That argument is about glyph advances and it does not reach quadding, which
states which **end** of the box the run is anchored to and says the same
thing in any font. An editor that read `/Q` nowhere would type every field
left-aligned, and a centred or right-aligned form would re-lay itself out
the moment the value committed. The whole of the rule is
[`super::boxes::editor_align`].

# `/MK` `/BG`, the second property, admitted by the same test

A live box painted `extreme_bg_color` — near-white under every light preset
— makes a pale-yellow or shaded field **turn grey the moment the operator
touches it** and turn back a gesture later. Nothing in the file has
changed; the only thing that changes is the colour of the thing being
looked at, which is what pdfcer's rule 4 forbids. The engine's own
`Widget::background` doc names this editor as its intended consumer.

**The fill and the ink arrive together, and that is not tidiness.** A
document-derived fill under a theme-chosen foreground is `DEFECTS.md` D2's
second shape — *a foreground assigned for a fill the text is not on* — and
it is how the old GUI shipped near-white headings on light grey.
`Theme::foreign_fill_pair` measures the pair and answers `None` when no
theme ink reads on that fill; `None` means **paint neither**, so an
unreadable field keeps the theme's own readable box rather than becoming a
tinted one the operator cannot read their own typing in.

What is deliberately NOT tinted: the focus ring. A ring is the *cursor*,
which rule 4 admits in full — it says where the keystrokes are going, not
what the document contains.

# The refusal is traced, because an operator cannot see one

Three outcomes reach this point and only two of them are visible. A field
with no `/BG` keeps the theme box, which is right and expected. A field
WITH a `/BG` that the theme has no readable ink for **also** keeps the
theme box — identical on screen, a different fact about the file — and
rule 4 is explicit that an inference the operator cannot see still owes an
off-canvas report. This is that report, in the place this shell puts
machine-readable ones.

[`Spec::trace`] is `Some` only on the frame the caller seats the caret, so
the line is one per opened editor rather than one per frame. Components are
printed as decimals rather than `Debug`-formatted, because a driven check
reads this line and a `{:?}` tuple is a shape that changes when the type
does.

### `fn seat`

# Two seatings, and the difference is measured rather than chosen

A `/Tx` field seats the caret at the **end**: the click that asked for the
editor was consumed by the page (see [`super`]'s §4), so there is no click
position to place a caret from, and selecting all would turn the operator's
next keystroke into a deletion of the field's contents.

An editable combo box seats it as a **select-all**, because that is what
the product class does and because the gesture means something different
there. Acrobat, photographed on `fixtures/all-field-kinds.pdf`: a click
anywhere in an editable combo's text area gives the field a white box, a
chevron drop button, and the existing value highlighted end to end. The
field's value came from a list, so replacing it wholesale is the common
act; in a text field the operator is usually amending what is there.
