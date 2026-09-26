# `panels::forms::rows` — one field, one row

The per-field half of the Forms panel: what a text field, a check box, a
radio group, a choice list and a rich-text field each look like, and which
[`FormEdit`] each of them can raise.

The form-wide half — the disclosures, the recompute and reset sections, the
two whole-form buttons — is in [`super`], and the split is by *scope*
rather than by size: a control that acts on the whole form and a control
that acts on one field answer to different rules about placement, about
disclosure and about when they may be offered at all.

## Every unfillable row is DISABLED AND EXPLAINED, never hidden

`RIBBON_IA.md` R83. An operator scrolling past a signature field should see
that pdfcer knows it is there; a row that vanishes teaches nothing, while a
disabled one with a sentence beside it teaches what would enable it.

The reason is asked in a fixed order — see [`block_reason`] — because a
field can be blocked several ways at once and the most specific answer is
the useful one.

## A check box with no ON state is disabled, never offered and refused

`pdfcer_core::forms::Widget::on_states` lists the button on-state names a
widget's `/AP` `/N` subdictionary defines, **excluding `Off`** (§12.7.4.2.3).
Two rules follow from that exclusion:

- **Never ask `on_states` whether an `Off` appearance exists.** A predicate
  of the shape `on || widgets.any(|w| w.on_states.contains("Off"))` reduces
  to `on`, because the right-hand disjunct is false by construction — it
  compiles, it reads like a real check, and it answers the wrong half of the
  clicks. The fact itself lives on `Widget::has_off_appearance`, which core
  keeps deliberately separate for exactly this reason.
- An **empty** `on_states` means there is no ON state at all.
  `EditSession::set_button_state` refuses any name but `Off` that no widget
  defines — `EditError::FieldStateUnknown` — so the box is drawn disabled
  with [`crate::text::forms::form_field_no_on_state_note`] beside it. R83
  rather than a disclosure after the fact: the control that would always
  error is not offered at all.

## Deliberately absent: field creation, deletion, renaming, widget moving

A Rename editor, a per-widget Delete, a whole-field Delete and a
grouping-node roster are `Edit ▸ Forms` **authoring** commands
(`edit.form_create_field`, `edit.form_manage_fields`). They answer to core's
*structural* certification gate rather than the fill gate, and a reader that
fills a form does not create fields in it. They land with the commands that
name them.

## Item notes

### `fn row_label`

Prefers `/TU` — what a screen reader announces for an interactive field —
so the operator reads the same string an assistive technology speaks rather
than two different names for one field. The raw name is always in the
tooltip, because `/TU` may be absent, may differ, and is not what a data
file matches on.

A blank-but-present `/TU` falls back to the name rather than producing an
unlabelled row: the file has technically supplied one, and honouring it
literally would leave the operator with a control identified by nothing.

### `fn blocked_row`

The value is shown in a **disabled text box** for a text field and as a
plain label otherwise, which is the old shell's shape and survives review:
a box that looks like every other box, greyed, says "this is the same kind
of thing and you may not type in it". A label would say "this is a
different kind of thing", which is not true.

A field holding nothing draws no value at all rather than an empty label,
so a read-only empty field is not indistinguishable from a rendering fault.

### `fn rich_text_row`

# Why it is not editable in place

**Correctness, not fidelity.** pdfcer cannot author `/RV`, and appearance
generation for these fields is bound to `/RV` rather than `/V` (§12.7.3.4
and §12.7.3.3, both `shall`). Writing plain text and leaving `/RV` behind
would make conforming readers rebuild the appearance from the OLD text —
the document would display words nobody typed.

So the row shows the value read-only and offers a **disclosed downgrade**:
convert the field to a plain one. Deliberate, named and lossy, which is why
it is a button the operator presses rather than something that happens when
they start typing. The disclosure travels with the capability: the button is
never offered without the sentence that says what it costs.

# Why the `/RV` is parsed every frame

The alternative is a cache keyed by field name, which would then need
invalidating on every edit, undo and reload — a correctness problem in
exchange for parsing a few hundred bytes of XML on a panel that is already
re-laying-out its whole field list. Measure before trading one for the
other.

### `fn mirrored`

`live` is the whole frame's answer — one field, or none
([`RowContext::live_canvas_draft`]) — and this is the per-row half of the
question: the name has to match, because a live draft for *Address* says
nothing whatever about the *Name* row it is being asked beside.

# A pure function, for the reason [`commit`] is one

The rule it states — *the page wins for the field the page is typing into,
and for no other* — is one line of code and two ways to get it wrong, both
silent: mirror unconditionally and every text row in the form shows one
field's draft; mirror never and the panel lags a whole typing gesture behind
the page. Neither is visible in a diff and neither needs an `egui::Ui` to
demonstrate, so it is tested rather than looked at.

### `fn check_row`

Immediate commit, no draft: a check box has one atomic change and no
intermediate state to protect, so the sixty-undo-entries argument that
governs the text rows does not apply.

See this module's header for why an on-state-less box is drawn disabled
rather than being offered and refused.

### `fn check_on_state`

The **first** state any widget offers, in widget order. A check box has one
ON state by definition (§12.7.4.2.3), so a second would be a malformation;
taking the first rather than asserting there is exactly one means a
malformed file still gets a working control instead of a refusal.

Extracted so [`check_row`]'s two paths — the live control and the disabled
one — read the same answer, rather than one of them re-deriving it.

### `fn radio_row`

A radio GROUP is one field with several widgets, each carrying its own
on-state name; the field's `/V` is whichever name is selected, or `Off`. So
the control is one exclusive cluster over the **distinct** on-states, NOT a
check box per widget — which is exactly the shape
`set_button_state(fqn, on_state)` takes.

Duplicates are real and meaningful — two kids sharing an on-state name is
what `RadiosInUnison` describes — but they are ONE choice to the operator,
so they get one control, and core turns every kid with that name on
together for free.

### `fn radio_states`

Order is the document's, never sorted: the widgets are laid out on the page
in an order the form's designer chose, and re-ordering the cluster would
make the panel disagree with what the operator is looking at.

A `Vec` with a linear `contains` rather than a set, deliberately: a radio
group has a handful of options, order must be preserved, and a set that
preserved insertion order would be a dependency for nothing.

### `fn choice_row`

# Options are displayed in `/Opt` ORDER, never sorted

§12.7.4.4: a conforming reader SHALL display them in the order they occur
in `/Opt`. Re-sorting is a conformance violation, not a presentation
choice, and the `Sort` flag is an instruction to the **writer**.

# `/V` stores the EXPORT value and the operator must see the DISPLAY one

`/Opt` entries may be `[export display]` pairs, so rendering `/V` verbatim
shows an operator `MX` where the form says `Mexico`. Every read of a
selection goes through the export → display mapping below; a fixture whose
export differs from its display is what makes the omission visible at all.

A `/V` that matches no option is a real state — set by another program, or
left behind when the option list changed. A single-select field shows it as
stored, with
[`crate::text::forms::form_field_choice_value_not_listed`] beside it;
showing blank would claim the field is unanswered when it is not. A
multi-select field has no box to show it in, so [`multi_choice_ticks`]
drops it and
[`crate::text::forms::form_field_choice_multi_value_not_listed`] says so.

### `fn multi_choice_ticks`

The selection is rebuilt by asking each **option** whether it is selected,
never by copying `/V` and editing it. That is the rule
[`crate::canvas::forms::choosing::wanted`] follows, and it is not a style
choice: `/V` may hold an entry matching no option, `set_choice_value`
refuses such a value as `ChoiceValueNotInOptions`, so copying it forward
turns the operator's first tick into a refusal naming a value they never
touched. Dropping it is the only outcome a check-box stack can express,
since there is no box to show it in.

`/V` may legally hold an export or a display string, so both are matched;
only exports are returned, which is what `SetChoice` writes and what the
boxes then compare against.

The returned flag is the disclosure's trigger — the drop is invisible on
the page, so it is owed words. Empty entries are not counted: they are
"unanswered", not "unlisted".

### `fn leaving_an_untouched_field_writes_nothing`

The second half of [`commit`]'s condition, and the one that is easy to
drop. Reading a form means tabbing through every field in it; if that
wrote a command per field, an operator who merely *looked* at a
forty-field form would have forty undo entries and a modified document
they never edited.

### `fn typing_does_not_commit_until_focus_leaves`

`TextEdit::changed()` fires per keystroke and `fill_text_field` pushes
one undo entry per call, so committing on change would make one typed
word a dozen undo steps — and a dozen appearance regenerations, each
re-rasterizing the page.

### `fn the_pages_draft_reaches_its_own_row_and_no_other`

Both halves are asserted because both are silent failures: without the
first the panel goes on showing the value from before the operator
started typing on the page — two boxes disagreeing about one field — and
without the second every text row in the form shows whatever is being
typed into one of them, which is worse than the lag it replaces.

### `fn a_value_the_options_do_not_list_is_dropped_and_disclosed`

The whole point of rebuilding from `/Opt`. Carrying `Kiribati` forward
would make the operator's next tick arrive at `set_choice_value` as
`["Kiribati", "MX"]`, which it refuses as `ChoiceValueNotInOptions` -
a refusal naming a value the operator never touched, in answer to a
gesture that was valid.

### `fn a_selection_stored_by_display_string_is_recognised`

`/V` may hold either half of an `[export display]` pair, so matching
only exports shows a filled field as empty — and then reports the
operator's own answer as a value the document does not list, which is
the same disclosure firing on a false premise.

### `fn emptying_a_filled_field_is_committed`

The case a `!draft.is_empty()` guard would silently swallow: emptying a
field the document has a value for is exactly as much an edit as typing
into an empty one, and it is the gesture an operator makes to correct a
mistake.
