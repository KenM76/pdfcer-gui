# `pdfcer-gui/canvas/moving/nudge/tests`

## Item notes

### `fn arrows`

`count` rather than a `bool`, because a held arrow arrives as several
`Event::Key { pressed: true, repeat: true }` in one frame and *"one undo
entry per keypress"* is a claim about exactly that.

### `fn the_up_arrow_moves_the_mark_up_the_page`

PDF user space has y increasing upward from the bottom-left (§8.3.2.3) and
canvas space has it increasing downward, so this asserts a positive `dy` for
the key labelled Up — the one assertion whose failure would be invisible in
review and obvious in one second of use.

### `fn a_quarter_turned_page_nudges_along_the_axis_the_operator_sees`

The claim that makes routing through [`super::super::page_delta`] worth the
parameter: on a page carrying `/Rotate 90`, screen-up is page-**left**, so an
Up arrow must produce a displacement along **x** and not along y. A nudge
that hard-coded `dy = +1` passes every test above and moves a mark sideways
on the first landscape drawing it meets.

### `fn shift_and_alt_are_not_this_gestures_to_take`

`Alt+Up` and `Alt+Down` are bound in the built-in keymap to
`pages.move_up` / `pages.move_down`. `egui`'s `consume_key` matches with
`Modifiers::matches_logically`, which **ignores extra Shift and Alt** — so a
nudge written the obvious way would have reordered the page *and* moved the
mark from one press, and no test injecting a bare arrow could have seen it.

Shift is refused for a different reason and the same effect: it already means
*constrain to one axis* on this canvas, three gestures over.

Falsified: deleting the `shift || alt` line in [`super::step_for`] turns
both halves of this red.

### `fn a_held_arrow_raises_one_move_per_repeat`

Three press events in one frame raise three moves of one point, not one move
of three. That is what Illustrator, InDesign and Acrobat all do, and the
alternative needs a notion of *gesture end* a keyboard does not offer.

### `fn a_real_focused_text_field_keeps_its_arrow_keys`

This is the test `DEFECTS.md` D1 did not have. It builds an actual
[`egui::TextEdit`], focuses it, and asserts
`Context::text_edit_focused()` is **genuinely true** before asserting that
nothing was nudged — because a test that presses an arrow into a bare
`Context` with no widgets passes whatever the guard says, which is precisely
how the founding defect shipped.

Falsified: replacing [`crate::canvas::textedit::composing`] with a
constant `false` in [`super::keys`] turns this red on the *second* assertion
while the first stays green — which is the proof that the first assertion is
doing its job.

### `fn a_canvas_draft_keeps_its_arrow_keys`

The second claimant, and the one `text_edit_focused()` answers `false` for.
The caret this shell paints sits in PDF space at the glyphs' own scale, so it
is deliberately not an `egui::TextEdit` — which is why the predicate has to
be [`crate::canvas::textedit::composing`] and not egui's own.

Asserting `!text_focused && composing` is the whole content of that gap: the
second instance of D1 cost the operator the space bar because a guard asked
egui alone, and an arrow key is where the same gap costs a caret its
movement.

### `fn a_locked_mark_refuses_and_says_why`

§12.5.3 Table 165 bit 8. The sentence matters as much as the refusal: the
Properties panel's standing sentence for a locked mark is about its
*appearance*, so an operator pressing an arrow has been told nothing relevant
and silence would be the standing cross-cutting defect.

### `fn a_ce_dimension_is_not_nudged_by_this_verb`

Moving one has to re-measure it, which `move_annotation` does not do and
refuses by name. The pointer makes the same fork one module over
(`dimdrag` claims it, `annotdrag` does not), so this is that fork restated
for the keyboard.

### `fn a_mode_that_cannot_author_markup_nudges_nothing`

`author_markup`, not `edit_content` — Review has the second `false` and the
first `true`, and it is the mode an operator is in *because* they are working
on comments. The Delete rung learned this the hard way.

### `fn an_empty_selection_says_nothing_and_page_content_says_something`

The split the refusal catalog argues for. Arrow keys are pressed constantly
for reasons that are not about a selection, so a bar that narrated every
stray press would stop being read — but an operator who picked a line out of
the drawing and pressed a key every drawing program binds is asking a
question.

### `fn a_frame_with_no_page_declines_with_a_sentence`

`page: None` is what a frame with no page on screen genuinely hands over.
The alternative — assuming an unrotated letter page — would author a move in
units nothing on screen agrees with.

### `fn a_held_arrow_that_refuses_reports_once`

A refusal is a property of the selection and the mode, so every repeat of a
held key would refuse identically. Writing the same sentence per repeat would
make the status row flicker and would put forty identical lines in a trace a
harness has to read.

### `fn a_nudge_never_alters_the_selection`

[`super::super::tests`]' first invariant, restated for the keyboard. The mark
stays selected so a second press nudges it again — an operator correcting a
position presses four times, and a gesture that dropped the selection after
the first would make that impossible.
