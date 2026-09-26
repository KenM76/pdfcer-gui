# `pdfcer-gui/app/tests`

## Item notes

### `fn the_memory_backed_toggles_report_their_pressed_state`

The two controls that had none. Both halves are asserted: unarmed must
be *unset*, armed must be set. Asserting only the armed half would pass
on a condition wired to a constant, which is precisely how a toggle
comes to render pressed forever.

### `fn an_armed_tool_stays_pressed_with_nothing_open`

Deliberate, and the opposite of the other conditions in this function:
the armed tool survives closing a document, so a ribbon that forgot
which tool you were in the moment you closed a file would be reporting
something untrue about its own state. The commands are gated on
`doc.pages` separately, so the control is greyed *and* pressed — which
is exactly "this is the tool you are in, and there is nothing to use it
on".

### `fn the_in_form_condition_is_set_only_for_a_form_interior_selection`

The two negatives are the load-bearing half. A condition that were
merely a synonym for `selection.any` would light the control on every
selection in every document, and the operator would meet a button that
declines more often than it works.

### `fn select_the_form_lands_on_the_container_and_it_is_deletable`

Before this command the operator could reach an object inside a form and
could reach nothing else: the deep hit test excludes forms outright, so
the container had no route on the canvas at all. This is that route, and
the assertion that matters is the last one — after pressing it, Delete
has something to delete.

### `fn select_the_form_with_no_form_selected_says_why`

`enabled_when` greys the ribbon item and enforces nothing — every
other route reaches the dispatcher unchecked — so the arm asks again,
and the arm's answer is a sentence rather than silence.

# This test passed for the whole time the sentence was invisible

It asserted `recorded_for_test()` and stopped. That reads the store the
dispatcher writes to, which is one link of a two-link chain — the bar
does not draw what was **recorded**, it draws what `decline::live`
returns, and `live` re-asks each sentence's predicate before handing it
over. `Declined::InsideForm`'s predicate was `selection_in_form`, which
is false by construction on every frame this arm can run, so the
sentence was discarded on the frame it was written. **Every time, in
every build, since the verb shipped**, with this test green.

### `fn delete_on_a_form_interior_selection_removes_it_exactly_as_the_key_does`

⇒ The test was pinning the divergence. Both routes now ask
[`crate::canvas::deleting::subject`], so they cannot differ; what is asserted
here is that they do not.

# The sentence did not disappear — the state it described did

`Declined::InsideForm` is still recorded, by `format.select_form` with
nothing selected (asserted one test up) and by `canvas::moving` on a drag it
cannot route. What no longer records it is a Delete that now works.

### `fn the_selection_condition_follows_the_selection`

The condition powers two surfaces the manifest has been carrying
unwired: the contextual Format tab's *appearance*, and the enable state
of the Delete inside it. It could not be published while the selection
lived in `egui::Memory` — [`PdfcerApp::conditions`] has no
`egui::Context` — so this asserts the consequence of the move rather
than a new policy.

Both directions matter. Publishing it when nothing is selected would
arm a **destructive** command over an empty operand list, which is
defect D1's shape with the worst possible verb behind it.

### `fn the_ribbon_delete_raises_the_delete_action`

`format.delete` was drawn and enabled from the moment the Format tab
landed, and did nothing — the live instance of D1's shape that this
stage is accountable for. It became wirable when the selection moved
onto `OpenDoc`, because [`PdfcerApp::dispatch_token`] has no
`egui::Context` and therefore had no route to a selection in
`egui::Memory`.

Asserted through the real token lookup rather than by calling the arm
directly: the dispatch resolves a token back to an id, so a test that
skipped that step would pass even if the command were never registered.

### `fn the_ribbon_delete_declines_inside_an_object_just_as_the_key_does`

Inside an object the selection names a subpath, and the only wired verb
removes whole objects — one measured CAD export holds an entire drawing
view as a single path object with 1,194 subpaths. The rule lives once,
on `SelectionState::deletable_objects_on`; this asserts that the ribbon
path really reads it rather than re-deriving an operand list of its
own, which is exactly how two spellings of a destructive rule drift
apart.

### `fn the_properties_command_puts_the_panel_on_screen_in_every_mode`

The command was named by File ▸ Document, named by the `objects.row`
context menu, registered in `crate::shell::commands` — and had no arm,
so invoking it traced `command-unimplemented` and did nothing. That is
D1's shape: a control that looks available and is inert.

The mode matters, which is why the test walks all three. The
application **opens in Read**, and Read's default arrangement mounts no
Properties panel at all (`app::modes`' `spec("read")`), so the
interesting case — activate fails, mount, activate again — is the
*first* one an operator meets rather than an edge case. Review and Edit
mount it already and take the cheap path.

Driven through the real token lookup, so a command that stopped being
registered fails here rather than silently taking the `other` arm.

### `fn the_reset_layout_command_restores_the_modes_default_arrangement`

The other command with no arm. `Modes::reset` existed and was tested;
nothing invoked it, so View ▸ Window ▸ Reset layout and the `dock.tab`
context menu both traced `command-unimplemented`.

The test asserts the arrangement is *exactly* the mode's default,
which is a stronger claim than "the closed panel came back": a reset
that produced some third arrangement, or that reset only one dock,
would pass the weaker one.

It resets **before** rearranging as well as after, deliberately. This
application loads the operator's persisted layout at start-up, so the
arrangement a test inherits is whatever is on the machine running it;
the first dispatch is both the assertion that the command works from an
arbitrary starting point and the thing that makes the second half
deterministic.

**`ResetScope::All` is the scope this build passes**, and that is a
decision recorded in the dispatch arm, not an oversight — with no
chooser surface, a control named "Reset layout" that reset half the
layout would be the more surprising failure.

### `fn the_chord_and_the_button_raise_the_same_action`

The structural half of the two-owner fix. `crate::app::keyboard` no
longer knows what `Ctrl+0` *means*; it reads the id out of the manifest
keymap and hands it here, so the chord and the ribbon button land in
one arm by construction.

This asserts the consequence: dispatching the id the keymap binds to
`Ctrl+0` raises exactly what the ribbon's Actual size raises. It would
have failed before the fix — the chord raised `Fit(FitMode::Page)` and
the button raised `ZoomTo(1.0)`, which is the defect in one line.

### `fn the_mode_commands_move_the_ribbon_selector`

`MODES_AND_PANELS.md` Part 1 §6 specifies `Ctrl+1`/`Ctrl+2`/`Ctrl+3`,
and all three `crate::text::commands::mode_*` tooltips print the chord.
Until this arm existed, all three sentences were false: the manifest
bound the chords, nothing dispatched them, and `Ctrl+2` was in fact
doing fit-width from `keyboard::collect`.
