# `pdfcer-gui/app/conditions/tests`

## Item notes

### `fn the_history_conditions_follow_the_session`

# The defect this exists for

This function published neither for the whole life of the project, with a
comment saying they were "deliberately absent". The obvious way to land
them — set them beside `doc.open` and move on — arms both controls
permanently, and a build that did that is **indistinguishable from a
correct one** in every scenario that presses undo *after* an edit. The
first two assertions here are the only ones that tell the two apart.

# Why it drives the real actions rather than setting fields

Because the thing under test is a **join**, and this crate has been bitten
by exactly that before: `every_armable_tool_kind_reports_a_pressed_state`
exists because Phase 7 shipped a measure tool with four passing unit tests
and no `conditions` call site, so the button never lit up. A test that set
a boolean and read a condition would prove that `set.set` works.

So the log is filled by [`crate::app::actions::Action::CommitMarkup`] going
through `vector_edit` and emptied by [`crate::app::actions::Action::Undo`]
going through the same funnel — the two paths an operator's gesture takes
— and the conditions are read through the **registered commands'** own
predicates, so a rename of either condition string fails here rather than
silently greying a control forever.

### `fn every_armable_tool_kind_reports_a_pressed_state`

# The defect this exists for, which shipped

`app::conditions` published the armed **markup** kind and did not
publish the armed **measure** kind. Phase 7 had `CanvasTool::Measure`,
`arm_measure`, `measure_command` and a dispatch arm using its inverse —
all four with passing unit tests — so Linear armed the tool, placed a
dimension the engine accepted, and the button never lit up. It was found
by `ui-verify` driving the real window, because the missing link was a
**call site**, and no unit test observed two adjacent links being
connected.

Iterating `MarkupKind::ALL` and `MeasureKind::ALL` is what stops the
same omission recurring: a fifth kind added to either enum with no
`selected_condition` fails here rather than shipping as a control that
arms without looking armed. A list of ids spelled out in this test would
have to be remembered, which is the thing that was not.

### `fn the_text_markup_controls_need_a_live_text_selection`

Reading `set.is_set("selection.text")` would assert that this function
agrees with itself. What matters is whether the *control* comes alive,
which is the registration's predicate and this function's publication
joined — the same join `every_armable_tool_kind_reports_a_pressed_state`
exists for, and the same join `ui-verify` had to find in a running window
when the measure tools armed without lighting up.

Three states, and the third is the one a build would plausibly get wrong:
a selection made *before* an edit is not an operand, because its recorded
boxes may now sit over other glyphs — and a control that is live while
the press would decline is the disagreement `selection.bounds` was
invented to prevent one command over.

### `fn the_formattable_condition_is_the_union_of_the_two_selections`

It is the contextual Format tab's `visible_when`, and the tab now
carries controls for two unrelated kinds of selection:

* spelled `selection.any` — the object selection — a text sweep would
  raise **no tab**, so the Font group could not be reached at all and
  `format_text` would be a capability with no surface. That is what
  shipped between the panel landing and this condition existing;
* spelled `selection.text`, the tab would vanish on an ordinary object
  selection, taking the Delete it has carried since it shipped with it.

So the assertion is a truth table rather than a pair of positives, and
the **fourth row** — both at once — is the one that could not happen
when `selection.text`'s own note was written (*"the two are mutually
exclusive by construction"*) and can now: `takes_the_press` answers true
for an armed text tool in **any** mode, so an operator who clicks an
object and then presses `T` and sweeps has both.

### `fn delete_is_not_offered_in_a_mode_that_cannot_perform_it`

# What was wrong

`selection.delete_permitted` asked only whether the *engine* would
refuse. It never asked whether the *mode* would. That was defensible
while nothing could be selected in a read-only mode, and
`canvas::keys` wrote its own guard anyway, saying why:

> "Delete is safe because nothing can be selected" holds only for as
> long as its other half does, and the other half is in a different
> file.


# Why the table has a Review row for annotations

Row 4 is the load-bearing one. It is what stops a future
simplification collapsing this ladder to `caps.edit_content` and
silently taking **Review's markup Delete** off the ribbon while the
Delete key kept working. One predicate per capability: `author_markup`
guards the annotation rung, `edit_content` guards the content rung, and
neither stands in for the other.

### `fn the_font_groups_visibility_follows_the_mode_and_its_enablement_the_sweep`

`mode.edit_content` is **visibility** — a mode that cannot change page
content does not have a mislaid ability to restyle text, it does not
have the ability, so the controls render nothing. `selection.text` is
**enablement** — inside Edit the capability is present and only the
operand is missing, which greys and explains itself on hover.

# Why both are needed, stated as the two one-condition builds

With only `selection.text`, sweeping text in **Read** — which Read does
with the plain select tool, because copying is not authoring — would
draw an enabled Bold that the mode gate must then refuse. With only
`mode.edit_content`, Edit would draw an enabled Bold with nothing
swept: a control that does nothing on almost every press, which is the
placeholder shape P3 forbids.

Asserted through `Capabilities::for_mode` and the shipped manifest,
not through a hand-made `Capabilities` value, so a mode taxonomy edit
that gave Read `edit_content` fails here as well as wherever else it is
wrong.

### `fn in_edit_the_text_tool_makes_the_text_markup_controls_reachable`

# What was wrong, and why it was a rule violation rather than a gap

Edit shows the Markup tab, so `markup.underline`, `markup.strikeout` and
`markup.squiggly` were **drawn** there — and `selection.text` could never
be true in Edit, because `canvas::textsel::takes_the_press` gave the press
its text meaning only where the mode could *not* select content. So three
controls rendered, greyed, in every Edit session for the life of the
build, with no state that could ever enable them.

`RIBBON_IA.md` **P3** reserves greying for *temporarily* unavailable and
says an absent capability renders nothing. Permanently greyed is neither,
and it could not be fixed by hiding: a command lives on exactly one tab,
and the Markup tab is in **both** Review and Edit, so hiding them in Edit
would have required a per-command per-mode visibility rule that this
manifest does not have and that would have been a mechanism invented to
conceal a gap rather than to close one.

# Why this test is worth its length

It is the only assertion in the workspace that joins **four** things that
each have their own passing tests: the mode's capabilities, the armed
tool, the condition, and the dispatch. `the_text_markup_controls_need_a_
live_text_selection` above proves the condition-to-enable join and says
nothing about the mode; `canvas::textsel`'s tests prove the press rule and
know nothing about the ribbon. A build that armed the tool and left
`press_kind` reading the mode first would pass both of those and fail
here.

The **negative** half is asserted first and is what makes the positive
half mean something: with the tool down, the same mode with the same
selection must still refuse, because that is the state the operator was
in before this feature and it must not have been quietly widened. Note it
is the *press rule* that is asserted there, not the condition — the
condition reads only whether a selection exists and is live, and in Edit
without the tool no gesture could have made one.

### `fn finish_is_not_offered_with_no_document_open`

The one condition published from inside the `Status::Open` arm that is
about a gesture rather than about the document, and this is the reason
it is inside it. A circular pick set lives in `egui::Memory`, which
**outlives documents** — that is the property the armed-tool conditions
below it are published outside the arm to preserve. Here it is exactly
the hazard: the action Finish raises names a page, and with the document
closed there is no page for it to name. A live control that raises a
`CommitDimension` against nothing is the placeholder shape this project
refuses.

Both directions are asserted, because the first alone would pass on a
build where the condition was never published at all.

### `fn markup_finish_needs_a_document_and_enough_corners_for_its_kind`

The document half is a near-copy of the test above, deliberately: the two
conditions have the same shape, the same hazard and the same argument for
living inside `Status::Open`, so a build that got the scope right for one
and wrong for the other is what a near-copy catches.

What is **not** a copy is the last section, and it is the interesting
half: this condition is where the polygon/polyline difference reaches the
operator. `markup::action` needs **three** vertices for a `/Polygon` and
two for a `/PolyLine`, so the same two-click run leaves the ribbon's
Finish live for one tool and greyed for the other. Asserting it here
rather than only in `markup::vertex` is the point — the rule is worth
nothing until it reaches the control.

### `fn a_certified_document_withholds_delete_for_a_selected_form_field`

# The defect, and why it is invisible without the second document

The publication read
`doc.selected_field.is_none() && annotdelete::refuses_selected(doc)`.
With a field selected the first conjunct is **false**, so the whole
expression is false and the condition was set **unconditionally for
every selected field on every document** — a gate that is a no-op by
construction. `format.delete`'s `visible_when` on the `canvas.field`
menu therefore resolved *shown* on a certified fillable form, and the
press it invited deleted nothing and said nothing.

⇒ Both halves are asserted, against a fixture **pair** that differs in
exactly one dictionary (`tools/gen-certified-fixture.py`), because the
negative half alone is satisfied by a build that withholds Delete
always — which is a worse defect than the one being fixed: a control
absent where it would have worked leaves the operator no gesture that
reports it.

Driven through `app.conditions()` and `open_path` rather than by
calling the derivation, for this module's standing reason one test up:
what is under test is the **join** between the query and the published
name, and a test that read the derivation would prove `set.set` works.

### `fn the_markup_style_group_follows_the_kind_of_annotation_and_the_mode`

# The defect this exists for, and it is the one nothing else could catch

`selection.markup_restylable` is spelled in **four** places: here in
`app::conditions` (which sets it), `manifest::format::MARKUP_VISIBLE_WHEN`
(which draws the five items on it), `catalog::format::MARKUP_RESTYLABLE`
(which enables the five commands on it) and the `KNOWN` list in
`shell::commands::tests` (which vouches that somebody publishes it).

Three of the four agreeing is not enough, and the failure is silent in the
worst direction: `KNOWN` is a hand-written list, so a typo *in the publisher*
leaves the other three consistent, the condition permanently unset, and the
**group permanently absent**. Nothing would be greyed, nothing would trace,
and R9 makes absence the correct rendering of an unavailable capability — so
the bug would look exactly like the design. `every_armable_tool_kind_reports_a_pressed_state`
exists in this file because Phase 7 shipped a measure tool with four passing
unit tests and no `conditions` call site; this is the same join, asserted
before it can be missed.

⇒ It reads the condition through the **registered commands' own predicates**
rather than by name, exactly as `the_history_conditions_follow_the_session`
does, so a rename on either side fails here rather than silently withholding
a control forever.

# Rule 15 — a ce dimension must not light it

A ce dimension is also an annotation and is also selectable, and its verb is
`set_dimension_style`. Handing one to `set_markup_style` regenerates it as a
bare line with its label and witness lines gone. The `AnnotKind::CeDimension`
case below is that rule asserted, and it matters because a ce dimension's
`/Subtype` **is** `/Line` — identical to an arrow's — so a build that filtered
on the string would pass every test that only ever selected a square.
