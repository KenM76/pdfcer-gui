# `panels::properties::choiceopts` — a choice field's `/Opt` list

`FORMS_PARITY.md` §8.1 row 1, and §7 names it as **one** sibling row rather
than seven features: *add · remove · reorder · rename display ·
export ≠ display · sort · default choice*. Plus the three `/Ff` flags
Acrobat's Options tab draws beside them and this shell could not set — bit
19 `Edit`, bit 23 `DoNotSpellCheck`, bit 27 `CommitOnSelChange`.

## `/Opt` is one property, and that decides the shape of [`section`]

`FieldEdit::with_options` **replaces the whole list** — Table 230 makes
`/Opt`'s order significant, so a per-entry merge has no defined meaning.
Every operation here therefore sends the entire list, and `fieldedit`'s
one-press-one-undo-entry rule is satisfied rather than bent.

The corollary is that this module emits **at most one `FieldEdit` per
frame**, built from the typed draft. Pressing a button steals focus from a
half-typed box, so the box's commit and the button's operation arrive in the
same frame; two pushes would be two undo entries for one press, and the
second — built from the pre-rename list — would silently take the rename
back.

## Sort is one act, and both sides of the boundary now perform it

`NewChoiceField::sort` sorts `/Opt` and sets bit 20. `FieldEdit::with_sort`
sets bit 20 alone, and `edit_field` sorts only when the same edit supplies
a replacement list — a caller that hands over the whole list has no
pre-existing order to destroy. Bit 20 set on its own still reorders nothing
and is disclosed as `sort_claim_unmet`.

This panel always sends the whole list, so it is in the sorting case. It
reorders anyway, because the operator must see the order that will be
written before it is written, and it reorders through
`pdfcer_core::edit::sort_choice_options` rather than through a comparator
of its own: one exported ordering cannot disagree with the gate that checks
it. `FieldEditOutcome::options_sorted` is the tripwire for the case where
it does anyway — see [`crate::app::actions::forms`].

## The duplicate refusal, and which half of it is the shell's

Both `add_choice_field` and `edit_field` refuse a repeated export by name
(`EditError::ChoiceOptionDuplicate`): the fill verb resolves to the first
match, so the second entry would be unselectable for ever.

[`refuse_duplicate`] survives that as the **worded** half, not as a second
authority. An engine refusal reaching `vector_edit` is shown as
`Declined::EditRefused` — *"That change was refused"* — which names neither
the rule nor the value, and the operator's list is long enough that finding
the repeat unaided is the whole difficulty. The shell therefore asks first
and says which value repeated.

It asks with `pdfcer_core::edit::duplicate_choice_export`, the same
predicate `edit_field` refuses on. `G027` was filed because that predicate
was private while the ordering beside it was public, which left this panel
spelling the rule itself and agreeing by construction rather than by
contract; the engine exported it, so the only thing spelled twice now is
the sentence, which is the part that has to be in the operator's
language.

## Rule 4

Nothing here marks the canvas. Removing an option the field is set to leaves
the field showing that answer; `edit_field` returns `value_no_longer_fits`
and the status row says so. Re-pointing the selection would be inventing an
answer the operator did not give.

## Item notes

### `const ROW_REGIONS`

Three because a driven check needs two rows to prove a reorder and a third
to prove it is a swap rather than a rotation. A region per row would cost a
`format!` per row per frame, on a channel-off build too, since
[`crate::diag::ui_control`] takes `&str`. Rows past the third publish
nothing, which is a stated limit rather than a silent one.

### `fn sort_by_display`

Delegates to `pdfcer_core::edit::sort_choice_options`, which is exported so
that a shell cannot hold a second opinion about what "sorted" means: the
same comparator decides the order here, the order `add_choice_field` writes
at placement, and the order `edit_field` tests the bit-20 claim against.

The round trip through [`to_engine`] is the price of [`OptionRow`] being a
third type; the list is an `/Opt` array, so it is bounded by what a person
will read from a drop-down.

### `fn to_engine`

`ChoiceOption::new(export, display)` — that argument order, and it is the one
thing here a reader cannot check by eye, because both halves are `String` and
swapping them compiles. The engine collapses an equal pair to a bare `/Opt`
string itself, so there is no need to choose `plain`.

### `enum Op`

One value rather than a set, because a frame has one press in it. The typed
boxes are not here — they are read out of the draft — so [`Op::None`] still
means the list may have changed.

### `fn touched_for`

`fieldedit`'s header carries why this exists: the engine's refusals arrive
from a direction the request does not name, so the decline is shown against
the control the operator touched.

### `fn refuse_duplicate`

Compared on `export`, not on `display`: it is the export a fill resolves
against, so two options reading *Ontario* and *Ontario (ON)* that both send
`ON` are the broken pair, while two that read the same and send `ON` and `QC`
are merely confusing. §12.7.4.4 permits the second; nothing can select the
second half of the first — and that paragraph is the engine's, not this
panel's: the rule is `pdfcer_core::edit::duplicate_choice_export`, which
`edit_field` refuses on and which this asks with.

This still asks first, because it is the WORDING that is the shell's half:
an `EditError` reaching `vector_edit` renders as *"That change was
refused"*, naming neither the rule nor the value, and finding the repeat
unaided in a thirty-row list is the whole difficulty. Asking with the
engine's own predicate is what makes that a translation rather than a
second opinion.

The list is converted through [`to_engine`] rather than compared in place,
so the values tested are byte-for-byte the ones `with_options` sends.

### `fn box_width`

A measurement feeding a size, which is R128's shape — but it cannot loop:
`available_width()` here is the dock's decision, made before the body draws
and unaffected by anything drawn into it, so the derived width cannot widen
the container that produced it.

`MIN_BOX` is the floor anyway, and the rows are `horizontal_wrapped`, so a
dock squeezed below two boxes plus the buttons moves the buttons to a second
line. That matters because the Properties panel's `ScrollArea` is
`vertical()` only: anything past the right edge is not below a fold, it is
unreachable.

### `fn option_rows`

The two boxes are explicitly sized so the buttons land in the same column on
every row, which is what makes a column of remove buttons readable as a
column.

### `fn enabled_button`

The scope and not a greyed fill: `egui_shell::ribbon::sizing` measured that
a response from an enabled `Ui` reports itself enabled however it is
painted, which both kills `on_disabled_hover_text` and lets the click
through. A greyed control that still fires is worse than a live one.

### `fn default_row`

# `/DV` holds the EXPORT value, and getting that wrong is invisible

`set_choice_value` writes the export value to `/V`, and `edit_field`'s
`value_fit_complaint` checks a selection against `option.export`. `/DV` takes
the same type as `/V` (Table 228), so it holds an export too. A default
written as the display string would look right in this panel, look right in
the drop-down, and fail to match any option the day someone pressed Reset.

No draft: `FieldPropsDraft`'s rule is that a draft exists only for controls
that take typing, and a chooser is a press.

### `fn current_default`

`FieldValue::Choice` rather than `display_text()`, which joins several
selections with `", "` — a sentence, not a value, and it would round-trip a
two-selection default into one option named *"A, B"*. Several defaults are a
real state on a multi-select list box; this chooser offers one, so it reads
the first and the rest are left alone rather than silently discarded, which
is why it reports a change only when the operator moves it.

### `fn editable_row`

# Live in three of the four states, and the fourth is the interesting one

Table 230 makes bit 19 legal only alongside bit 18, and `edit_field` checks
it against the **resulting** field:

| drop-down | typing | control | why |
|---|---|---|---|
| on | either | live | both directions are legal |
| off | off | greyed | turning it on would be refused |
| off | on | live | the file already breaks Table 230 |

The last row is why this is not `add_enabled_ui(combo, …)`. A file can arrive
with `Edit` set and `Combo` clear — pdfcer reads what is there — and greying
the control there would leave the operator looking at a nonconforming field
with no way to fix it. Clearing the flag *is* the fix, and `edit_field`
accepts it because the post-state conforms.

### `fn spell_check_row`

The flag is `DoNotSpellCheck` and the checkbox says *Check spelling*, so
`checked` is `!flag` and the write is `!checked`. Worth the inversion — it is
what every application says — and confined to these lines so there is one
place to read it.

### `fn the_panels_sort_satisfies_the_engines_own_sorted_test`

[`sort_by_display`] delegates to the engine's exported sorter, so this
can no longer drift by accident — it can still drift by someone
re-hand-rolling the comparator, which is what it now guards. Asserted as
the engine's own expression rather than as "it is sorted": `edit_field`
computes `flags.has(SORT) && !effective.is_sorted()` over the
**display** strings, so this asserts `is_sorted()` on exactly that
projection. A case-insensitive or natural ordering would satisfy a human
reading of "sorted" and fail this.

### `fn only_a_repeated_sent_value_is_refused`

Both directions, because a gate that refused both would look correct and
would block a legitimate list: two options that read the same and send
different codes are permitted by §12.7.4.4. What nothing can select is
the second half of a repeated export, which is what `add_choice_field`
refuses.

### `fn to_engine_keeps_shown_and_sent_on_the_right_halves`

Both halves are `String`, so swapping them compiles and produces a file
that opens, reads correctly in the drop-down, and submits the wrong
data — the failure the engine's own note calls something that *"would
silently break forms"*. The only thing between this crate and that is one
line's argument order, so it is asserted rather than read.
