# `canvas::textedit::route` — where a text-tool click goes

## Contract

`route::click` is the one path from a text-tool click (the single click in
`canvas::clicking`'s text arm, and the double-click descent) to
`textedit::click`. In order:

1. **A note.** If the click opened or closed a note's pop-up, any open draft
   settles and no caret opens. The pop-up is where a text box comment's words
   are read and changed. Traced `text-click-routed to=note`.
2. **A form field.** If the armed tool fills forms (`forms::offer`) and the
   click is inside a fillable widget, any open draft settles and the form
   overlay's own click focuses the field. Traced `text-click-routed to=field`.
3. **The caret.** `textedit::click` with `Click::on_image` set when an image is
   the topmost picture under the click. On a page with no text, an image click
   refuses as `Refusal::PictureOfText` instead of becoming Add text.

A refusal is raised as `Action::DeclineOnCanvas(CanvasDecline::TextClick(..))`,
so it lands in the status bar's decline slot, and traced
`text-edit-declined reason=…` (with ` via=double-click` from the descent).

## Why the decline slot

The decline slot can carry a button: `Declined::remedy` names the command that
removes the cause (File ▸ Recognise text… for a picture of text), and the bar
draws it only when that command is registered in this build.

## Why the text tool fills forms

A click on a field with the text tool means *type here* in every program the
operator has used. `forms::boxes::types_into_fields` names the two tools that
do it; in Edit mode they skip the field-authoring selection, which belongs to
the Select tool. Add text is excluded: it places new text wherever it is
clicked.
