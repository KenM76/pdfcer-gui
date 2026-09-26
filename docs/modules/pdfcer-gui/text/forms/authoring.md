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

### `fn form_noun_choice`

"Drop-down list" rather than "choice field", which is the PDF spec's word
(`/Ch`) and means nothing to anyone who has not read it. The operator's
standing tie-breaker — *make it work the way other programs do* — applies to
vocabulary as much as to behaviour, and every program calls this a drop-down.

### `fn form_field_added`

It names the kind rather than saying "field added", because five commands
place five different things and a generic confirmation cannot tell an
operator that the button they pressed was not the one they meant.

### `fn form_field_merged`

The single most important sentence in this file, and the one with the least
visible cause. In PDF a fully-qualified name *is* the field's identity: two
widgets carrying the same one are one field with two appearances on the
page, and typing into either changes both.

The page looks exactly as it would if they were independent. So this is
stated plainly, with what it means rather than with the word "merged" —
which is the engine's word and describes the mechanism, not the consequence.

### `fn form_field_no_tooltip`

Not a scolding and not a warning: leaving it blank is a legitimate decision
and the engine accepts it as one. What the operator may not know is the
consequence, which is entirely invisible on screen — a screen reader has
nothing to announce for this control but its type.

### `fn form_field_no_options`

Authorable, and empty. Worth saying because an empty list renders as a
control that opens and shows nothing, which reads as a broken field rather
than an unfinished one.

### `fn form_field_tagged_document`

Covers both `tagged_document` and `structure_tab_order` in one sentence,
deliberately: they are two symptoms of one situation, and an operator who
gets two lines about the same thing reads the second as a separate problem.

It says what is true rather than what to do, because pdfcer cannot yet fix
it and a line that recommended an action it does not offer would be worse
than one that reports a fact.

### `fn form_field_renamed`

It names `descendants_renamed` when there are any, and that is the whole
reason this takes two arguments. Renaming a field that has children renames
their fully-qualified names too — `Address` becoming `Postal` turns
`Address.Line1` into `Postal.Line1` — because a qualified name is built from
the parent chain. The operator renamed one thing and several changed, and
every one of those is a name an FDF import or a filling script keys on.
Nothing on the page says so.

### `fn form_field_actions_retargeted`

# Why a rename owes a sentence at all

Renaming a field looks like a local act. It is not: `/ResetForm` and
`/SubmitForm` name their targets in `/Fields`, and `/Hide` names its in
`/T`, all as fully-qualified **name strings**. A rename that did nothing
else would leave every button naming the old name pointing at nothing — so
`rename_field` rewrites them.

**That repair is correct, invisible, and not what the operator pressed.**
Buttons elsewhere in the document now hold different bytes because of a
rename, and no view in this shell shows an action's target list. It is the
canonical rule-4 case: an inference the operator cannot see.

⚠ **The number is ACTIONS, not buttons, and the sentence must not imply
otherwise.** The engine says so at the field: one button naming a field
three times counts three, and one field named by three buttons also counts
three — *"pdfcer does not distinguish them"*. So this says *"places"*,
which is true under both readings, rather than *"buttons"*, which is true
under only one.

**JavaScript is not repaired and is not counted.** `R55` requires every
script carrier to round-trip byte-identical, so a form whose logic lives in
a script is not fixed by this — and a sentence claiming the rename was
handled everywhere would be false on exactly the documents most likely to
carry scripts. The clause is one short sentence because the operator cannot
act on the detail; what they can act on is knowing to check.

### `fn form_field_actions_orphaned`

# The asymmetry with a rename is the whole point

A rename can repair an action, because the field still exists under a new
name and that name is known. A **deletion** cannot: there is no name left to
point at. So the engine counts the broken references and repairs nothing,
and its own comment is blunt about what that leaves — *"each one is a button
that will do less than it says when pressed."*

⚠⚠ **This is the more serious of the two, and it is the one the operator
meets later, on somebody else's screen.** A form whose Reset button quietly
stopped resetting one field is not a form anybody notices until it matters,
and nothing in the saved file records that pdfcer knew.

⇒ So the sentence names a **consequence**, not a count of internals: the
buttons will do less than they say. A bare number would read as bookkeeping
about pdfcer rather than as a fact about the operator's document.

It does **not** offer to fix them, because pdfcer cannot — repairing would
mean deciding what a Reset button that named a deleted field should now
reset, and that is the operator's judgement rather than a default. Naming
the problem and stopping is the honest end of this.

### `fn form_widget_deleted_last`

`delete_widget` removes the field when its last widget goes, and that is
right: a named field nothing draws is a field nothing can fill. It is still
a larger outcome than the button promised, so it is said.

### `fn field_sort_claim_unmet`

pdfcer will not reorder an `/Opt` list it was not given, and this
sentence is why that is the right refusal rather than an omission.
`Sort` *"intended for use by writers, not by readers"* and requires a
conforming reader to display the options *"in the order in which they occur
in the Opt array"* — so `Sort` is a **claim about provenance**, not an
instruction. Setting it over an unsorted list makes the file say something
untrue; silently sorting would change what the operator sees in a
drop-down without being asked.

So the operator gets the flag they asked for and the sentence that says what
it now claims. Both, which is Rule 4's *render normally, report separately*.

### `fn field_options_reordered`

A tripwire rather than an everyday disclosure. `edit_field` sorts an `/Opt`
list when the same edit supplies the list and sets the Sort flag, and the
properties pane sorts with the engine's own exported comparator before
sending — so the engine finds nothing to move and this never fires.

It fires if the two orderings come apart, which is the failure the
comparator was exported to prevent and which neither side's tests can see,
because each stays internally consistent. Worded for the operator anyway:
the thing they can act on is that the list on screen was not the list
written, and Rule 4 owes them that off-canvas whether or not the cause is
theirs.

### `fn field_widgets_affected`

The engine's scope table, taken verbatim from Acrobat's own scripting
model: some properties *"apply to all widgets that are children of that
field"* and others *"are specific to individual widgets"*. Required,
read-only, the tooltip and the type flags are all in the first group — one
write, every placement.

The operator is looking at **one** box. A field drawn in three places has
just changed in three places, two of which may be on other pages, and
nothing on screen would otherwise say so. `widgets_affected` is reported by
the engine *"to be shown"*, in its own words.

Said only when the count is above one. On the overwhelming majority of
fields it is exactly one, and a bar that narrated that would stop being
read.

### `fn field_appearance_not_repainted`

The one disclosure here that is about something the operator can SEE
and will misread — [`AppearanceOutcome::RecordedNotPainted`], the state the
two older outcome fields could not name. The value is in the file and what
is on screen has not changed.

# `resized` picks the sentence, and getting it wrong CONTRADICTS the engine

This function said *"This box was resized and its artwork could not be
redrawn, so it will look stretched"* for every unpaintable outcome, because
until `Pass 308.0` a resize was the only edit that could produce one. A
colour-only edit now can, and for that case the engine's own string ends
*"The geometry did not change, so nothing is stretched."* — so the shell was
prefixing a denial with its own assertion of the same claim, in one
sentence, on the operator's status line.

⇒ §12.5.5 derives the appearance matrix from the appearance box's corners
and the `/Rect` corners, so only a changed **extent** makes the old stream
stretch. Where the extent did not change, the artwork is exactly as correct
as it was; what is wrong is that it does not reflect the edit.

### `fn field_widget_moved`

It names which of the two happened, because the engine distinguishes them
and the consequences differ: a move keeps the baked artwork exact and free,
a resize rebuilds it. An operator who dragged a corner and one who dragged
the middle have done different things to the file.

### `fn field_widget_property_changed`

The line [`field_widget_moved`] was giving for every non-geometry edit,
and it said **"The box was moved."** A border style, a caption, a visibility
flag and — since O202 — a colour all reached it, because the caller pushed
that sentence unconditionally and `resized` is `false` for all of them. The
receipt named an act the operator had not performed.

`touched` is the control they actually pressed, carried from the panel for
the same reason a refusal carries it: after the fact nothing else can say
which one it was.

`regenerated` is added rather than assumed. A colour change rebuilds the
appearance stream and a visibility flag does not, and an operator who just
watched a check box redraw itself is owed the difference from one who did
not.

### `fn field_siblings_untouched`

The mirror of [`field_widgets_affected`], and the reason the two exist as a
pair: a *field* edit changes every box and a *widget* edit changes one, so
an operator working on a field drawn in three places needs to know which
kind of control they just used. `siblings_untouched` is the engine's own
count, reported *"to be shown"*.
