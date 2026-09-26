# `pdfcer-gui/text/forms/authoring`

## Item notes

### `fn a_resize_that_could_not_redraw_says_so`

The three cases are asserted as a set, because the defect was that two
of them shared one sentence: `field_widget_moved` keyed on `resized`
alone, so a check box whose appearance was stretched rather than
rebuilt was told *"its contents were redrawn to fit"* — the opposite of
what the outcome it was reading said.

The assertions are on the CLAIM rather than on the wording, because the
wording will change and the claim must not: the unsatisfiable case must
say the contents are stretched, and must not say they were redrawn.

### `fn a_property_change_names_what_was_touched_and_not_a_move`

Every border, caption, visibility and colour edit ended with *"The box
was moved."*, because the handler pushed that line unconditionally. The
claim asserted here rather than the wording: what the operator touched
must appear, and the word *moved* must not.

### `fn only_a_resize_claims_the_artwork_is_stretched`

The engine's string for a colour edit it could not repaint ends *"The
geometry did not change, so nothing is stretched."* — and the shell
prefixed its own assertion of a stretch onto it, so the operator read
two sentences contradicting each other in one line.

### `fn a_move_owes_no_disclosure_either_way`

Deliberate, not an omission: a translation changes no length, so an
appearance carried across a move is exact and there is nothing to
disclose. Only a resize can be unsatisfiable.

### `fn the_merge_disclosure_says_what_it_means_for_the_operator`

Asserted rather than left to review because the tempting rewrite — "the
field was merged with an existing one" — is shorter, is what the engine
calls it, and tells an operator nothing about what will happen when they
type. This test fails if the sentence stops saying that both change.

### `fn a_rename_and_a_delete_say_different_things_about_other_peoples_buttons`

They pin the *wording*, which is where both sentences could go wrong in
a way no reader would notice: that the two do not borrow each other\'s
alarm or reassurance, that both admit JavaScript was not handled, and
that the rename count is worded as *places* rather than *buttons*.

They do **not** prove the sentences are ever produced. That is
`crates/pdfcer-gui/tests/action_targets_are_disclosed.rs`, added the
same day: it drives `rename_field` and `delete_field` against
`fixtures/action-names-field.pdf` — hand-authored for this, because the
counters see only targets written as **fully-qualified name strings**
and `submit-button.pdf` names its target by object reference, which the
traversal is structurally (and correctly) blind to — and asserts the
counts are non-zero, the sentences follow from them, and a rename of a
field **nothing names** reports zero.

⇒ What is still uncovered is the **frame**: that the status bar draws
the line. `tools/ui-verify` is where that belongs.

This paragraph was corrected once already. It read *"the chain is not
driven"* for as long as it took to build the fixture, which is the right
thing for a limit to say while it is true and the wrong thing to leave
standing afterwards.
**A rename says pdfcer rewrote buttons the operator did not touch, and
a delete says it could not.**

Both counts shipped with the verbs and **neither was read for three days**.
`FieldRename::action_targets_retargeted` and
`FieldDeletion::action_targets_orphaned` land on `rename_field` and
`delete_field` — verbs this shell already called — so the capability was
present, reachable, and silent. The engine flagged both as rule-4
obligations in the reply that shipped them.

### `fn both_sentences_admit_that_javascript_was_not_handled`

⚠ A sentence claiming the rename was handled everywhere would be false on
exactly the documents most likely to carry scripts — a form with logic in
it. The operator cannot act on the detail; what they can act on is knowing
to look.

### `fn the_retarget_count_is_worded_as_places_not_buttons`

The engine states it at the field: one button naming a field three times
counts three, and one field named by three buttons also counts three —
*"pdfcer does not distinguish them"*. So the sentence says **places**, which
is true under either reading, and never **buttons**, which is true under
only one.
