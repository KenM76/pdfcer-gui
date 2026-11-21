# `objects_panel_says_where_a_placed_drawing_is_drawn`

**Defect it guards.** A form XObject is one drawing shown wherever a page
places it; a title block on every sheet of a set is usually one form. The
Objects panel lists a placed form like any other object, so an operator
editing inside it cannot tell that the change reaches every sheet. The engine
answers the question document-wide in
`pdfcer_core::text_edit::forms::invocation_map`; unread, the number is only
seen after an in-form text edit.

**Fixture.** `fixtures/shared-title-block.pdf`, built by
`shared-title-block.PROVENANCE.py`: three pages. Form 4 (the title block) is
placed on every page and twice on page 2: four places on three pages. Form 5
(a detail) is placed once, on page 1.

**Steps.** Launch on the fixture with `view.panel_objects` invoked on
opening. On page 0, the `objects-placed` line for `form=4` must say
`places=4 pages=3 lower_bound=0`, the one for `form=5` must say `places=1
pages=1 lower_bound=0`, and the `panel.objects.placed` region must be
published as visible.

**Falsified** by reporting each form's page count as its place count in
`panels::objects::placed::reach`: the check fails reading `places=3` for form 4.

**What it does not prove.** The wording of each line, which is read in
`text::panels::placed`; and the "at least" form, which needs a page whose
scan the engine could not complete.
