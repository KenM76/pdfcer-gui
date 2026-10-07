# `a_redraw_says_what_it_did_to_a_fields_text`

**Defect it guards.** Every verb that redraws a text or choice field as a side
effect — a property edit, a box edit, a turn, Reset, Redraw values — returns
the engine's `LayoutDisclosure`: the auto size it chose and its bound, a `/DA`
colour drawn black, characters drawn as `?`. Unread, a restyle silently
changes how big the text is, and only a fill says so.

**Fixture.** `fixtures/autosize-fields.pdf`, built by
`autosize-fields.PROVENANCE.py` from three-text-fields: every widget's `/DA`
asks for size 0 (auto), and the `/AcroForm` carries `/NeedAppearances true`,
without which Redraw values has nothing to redraw and pushes no command.

**Steps.**
1. Field arm: launch on `FieldOne` with Edit mode and Properties open, pick
   the first Alignment option. The `edit-field` line's `disclosures=` must
   contain *“FieldOne” was redrawn at an automatic text size*.
2. Form arm: launch with the Forms panel open in Review and click
   `forms.regenerate`. The `form-regenerate-appearances` line's `redrawn=`
   must contain *A field of this form was redrawn at an automatic text size*.

Both arms require `status-group:edit-disclosure` to be declared. Values are
read from the raw line after ` key=`, because the sentences contain spaces.

**Falsified** by deleting the `redraw_notes` call in `edit_properties`
(field arm FAIL, `disclosures=none`) and by emptying `Applied::redrawn`
(form arm FAIL, `redrawn=none`).

**What it does not prove.** The `Width` and `Floor` sentences, the colour and
character sentences (unit tests in `text::forms::redraw`); the box-edit, Turn
and Reset routes, which call the same formatter. The fixture's short text in a
25 pt box is height-bound, which takes the general sentence.
