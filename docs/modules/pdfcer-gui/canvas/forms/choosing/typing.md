# `canvas::forms::choosing::typing` — the **editable** combo box

`/Ff` bit 18 `Combo` **and** bit 19 `Edit`: a drop-down whose value need not
be one of its options. `FORMS_PARITY.md` §8.1 row 15.

## Contract

One entry point, [`type_into`], called from [`super::choose`] before any of
its own work. It owns the whole frame for an editable combo and answers the
same thing `choose` does — whether this frame's primary press belonged here.

Every outcome still leaves as [`FormEdit::SetChoice`], the same command a
pick raises, because `pdfcer-core`'s `set_choice_value` already resolves a
value against `/Opt` first and falls to a free-text branch only when it
matches nothing (`edit.rs`, `editable_combo`). So a typed string and a
picked row are one verb, and nothing here has to decide which the operator
meant — the engine decides, from the file.

## Why this is a separate surface rather than a flag on [`super::choose`]

A plain combo box is a **focus ring plus a popup**: it draws nothing over
the widget, reads the vertical arrows to move a highlight, and every value
it can hold is already in the list. An editable one is a **live text box
with a drop button**: it covers the widget, owns the vertical arrows for
its caret, and its value may be a string that exists nowhere in the file.

Those differ in what has focus, in which keys mean what, in what is painted
over the page, and in when a write happens. Threading a boolean through
`choose` would have put a two-armed `if` at each of those four points and
called it one function.

What *is* shared is shared by call: [`super::list`] draws the popup,
[`super::wanted`] resolves a pick, and [`super::super::textbox`] dresses the
text box — so an editable combo's list looks exactly like a plain one's and
its text box honours `/Q` and `/MK` `/BG` exactly as a `/Tx` field does.

## The measured behaviour this reproduces

Photographed in Acrobat Pro on `fixtures/all-field-kinds.pdf`, which carries
`ComboEdit` for the purpose (`tools/acrobat-form-study.ps1`):

* **Unfocused, it is indistinguishable from a plain combo box** — the same
  `/AP`, the same square drop arrow. Nothing here paints until it is
  focused, which is rule 4 and also what the photograph shows.
* **A click in the text area gives a caret and selects the whole value.**
  It does **not** open the list. The value came from a list, so replacing
  it wholesale is the common act.
* **A click on the drop button opens the list**, flush under the field,
  field-width, square-cornered, unshadowed — [`super::list`]'s geometry
  already, because that was measured from the same photographs.
* **A focused editable combo shows a chevron where the `/AP` drew a square
  button.** That is not decoration: the text box covers the appearance
  stream, so a field that drew no arrow of its own would lose the only mark
  that says it is a drop-down at the moment the operator starts using it.

## Item notes

### `fn commit_typed`

The whole of what bit 19 buys: `set_choice_value` matches the string
against `/Opt` first, so a typed *"Large"* is the same command as a picked
*"Large"*, and a typed *"Extra large"* is a free-text value the engine
stores as its own export. Nothing here has to tell the two apart.

One function because it is reached from three exits — Enter, focus loss and
Escape — and *"tabbing through a field writes nothing"* has to mean the same
thing at all three. [`display_matches`] is the half that makes it true when
the field's `/V` is an export whose display text is what the box shows.

### `fn display_matches`

Without this, opening an editable combo whose `/V` is an export that
differs from its display would commit the *display* on the way out — a
write the operator did not ask for, on every field they merely looked at.

### `fn chevron`

`accent_pair`, never a named colour — `tools/gates/check-theme-colors.sh`
forbids the second and `check-plate-colour.sh` requires that an `on_accent`
ink state the plate it is drawn on. Both are satisfied by taking the pair
together, which is also the only way the contrast is gated.

### `fn arrow_strip`

Taken from the **height**, so the button is the square Acrobat draws and
scales with the field rather than with the zoom. Capped at half the width
so a wide-and-short field does not end up all button; floored at one unit
so the arithmetic below never produces an inverted rectangle.

Called on the **page** rectangle to decide what a click meant and on the
**screen** rectangle to decide where to draw. Those two disagree slightly
for a widget small enough that [`crate::canvas::forms::boxes::editor_rect`]
grew it to the legible minimum, and that is accepted: the alternative is a
hit region derived from a rectangle the operator cannot see.

### `fn arrival`

Called by [`super::focus_choice`] so the decision lives beside the
arithmetic that defines the button, rather than being re-derived at the
focus site.
