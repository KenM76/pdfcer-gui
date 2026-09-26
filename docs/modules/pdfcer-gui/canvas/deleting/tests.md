# `pdfcer-gui/canvas/deleting/tests`

## Item notes

### `const SIX_LABELS`

The obvious spelling — `decompose` over a hand-written `BT /F1 10 Tf …
(12.5) Tj … ET` — produces a `TextObject` with **zero runs**, silently.
`close_text_run` drops a run whose `current_run` bounds are empty, and
bounds come from glyph widths, which come from a resolved font; a bare
`ContentStream` has no `/Resources` and therefore no font.

That is not a curiosity, it is a trap with teeth: `text_run_count` then
answers `0`, `text_run_delete_would_move_next` answers `false` because it
requires `runs.len() > 1`, and a test written that way **passes for the
wrong reason** — it asserts a routing decision made over an object that has
no parts. `provider::tests::part_kind_and_part_count_answer_for_every_object_kind`
hits the same wall: its part count is `0` there, so it asserts the KIND and
records in a comment that the number is not a measurement.

So the text rungs are exercised against documents on disk. The path rungs
are not, because path geometry needs no font.

**Six runs, all explicitly placed** — one text object holding several
labels, which is what makes "delete a label" and "delete the text object"
two different acts.

### `const ROTATED_INHERITING`

The engine's own §9.4.2 fixture (`text/runs-inherited.pdf`) cannot serve
here: its inherited runs are horizontal, so each one shares its
predecessor's baseline and lands inside its predecessor's LINE. Deleting
that line takes both fragments and orphans nothing, so no line of it can
reach the refusal. Rotation is the only way a line can begin on another
line's advance.

### `fn against`

Takes a closure rather than returning the provider because `page_objects`
hands back a `Ref` borrowed from the `OpenDoc`, and a helper returning one
would have to return the document too.

### `fn selecting_one_label_deletes_that_label_and_not_the_object`

A label is selected — the Part rung on a text object — and Delete must reach
`delete_text_run` with that run's index. The variant is the whole claim: a
build that answered [`DeleteSubject::Objects`] here would pass any test that
only asked whether *something* was deleted, and would remove all 237 labels
on the operator's sheet.

### `fn deleting_a_label_that_would_move_the_next_one_is_refused_by_name`

Line 2 of the fixture is the rotated `Delta`, and line 3 is `Epsilon`
riding on its advance. Removing line 2 would slide line 3; the engine
refuses with `DeleteWouldMoveNextRun`, and this refuses first, from the
same `positioned_by` flag, so the operator gets the remedy instead of a
cause-less decline.

Aimed at the ROTATED pair deliberately. `Alpha`/`Beta` inherit too, but a
horizontal inherited run lands on its predecessor's baseline and is
therefore inside its predecessor's LINE — so deleting that line takes both
fragments and orphans nothing. Rotation is the only way one line can begin
on another line's advance.

### `fn several_selected_points_refuse_and_say_how_many`

`move_nodes` takes a slice and `delete_node` is singular, so a multi-anchor
delete would be N commands and N undo entries — and each excision renumbers,
so the second index would be planned against offsets the first invalidated.
Acting on the entered one alone is the `selected_nodes_on` defect: four
anchors highlighted, one removed, nothing said.

### `fn several_selected_lines_refuse_and_say_how_many`

The asymmetry is the point: `moving::eligible` builds `MoveSubject::TextLines`
out of exactly this selection, because `move_text_run` rewrites an operand
in place and renumbers nothing. `delete_text_run` excises a show operator, so
the second index of a loop would address a line the first moved. A shell that
looped both would corrupt a drawing on the press after the one that worked.

### `fn the_object_rung_needs_no_object_model`

The asymmetry `subject`'s docs argue for: the Object rung is answered from
the selection alone, so a page that will not decompose can still have its
objects deleted. A signature that demanded a provider would have invented a
limit.

### `fn a_part_inside_a_form_declines_because_the_engine_has_no_verb`

Measured against the locked engine rather than assumed: its six
form-interior verbs are five moves and one whole-object delete. There is no
`delete_subpath_in_form`. Naming it is what turns a dead key into a limit
the operator is told about — and the sentence points at what does work.
