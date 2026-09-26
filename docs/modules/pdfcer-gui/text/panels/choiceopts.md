# `pdfcer-gui/text/panels/choiceopts`

## Item notes

### `fn column_sent_hover`

It says the two are **usually the same** because that is the state the
operator is looking at, and an editor showing two identical columns with no
explanation reads as a mistake rather than as a capability.

### `fn row_up`

`\u{23f6}`, not `\u{25b2}` BLACK UP-POINTING
TRIANGLE, and the reason is
[`super::bookmarks::bookmark_collapsed_glyph`]'s: U+25B2 is in
`icons::glyphs`'s genuinely-absent row, so it would draw as a
substitution box in front of the operator. The U+23F4-U+23F7 block is
supplied by `emoji-icon-font`, and taking both halves of the pair from
one face is what stops a missing glyph reading as a direction.

### `fn row_remove`

`\u{00d7}` MULTIPLICATION SIGN, for
[`crate::text::find::close`]'s reason rather than a new one:
`\u{2715}` MULTIPLICATION X is in `icons::glyphs`'s absent
row, and U+00D7 is supplied by `Ubuntu-Light`.

### `fn row_remove_hover`

It says what removing an option does to an answer already given, because
that is the consequence the button's appearance cannot carry and pdfcer
deliberately does not repair: re-pointing a selection would be inventing an
answer the operator never gave.

### `fn no_options_yet`

Not *"No options."* — the fact worth stating is the **consequence**, which
is that the field cannot be filled in at all. `EditSession` allows a
zero-option choice field and discloses it, for the same reason.

### `fn flag_sort`

**"Keep sorted", not "Sorted"**, because the control does both halves: it
puts the list in order now and keeps a new option in order as it arrives.
The flag on its own sorts nothing — Table 230 makes it *"intended for use by
writers, not by readers"* — so a label naming only the flag would describe a
checkbox that appears to do nothing.

### `fn already_first_hover`

R9 wants every greyed control explained, and this one is
self-explanatory only to someone who has already noticed which row
they are on. It names the state rather than the rule, because the
state is the whole reason.

### `fn flag_editable_needs_combo_hover`

Table 230 makes bit 19 *"shall be used only if"* bit 18 is set, so the
engine refuses the press by name. Greyed rather than absent because this is
R9's **temporary** unavailability — one checkbox above makes it available —
and the hover names that checkbox.

### `fn flag_editable_without_combo_hover`

The control stays live in this state, which is the whole point: clearing the
flag is the only way to make the field conform, and greying it would leave
the operator looking at a defect they cannot fix.

### `fn flag_spell_check`

The flag is `DoNotSpellCheck`, so the checkbox shows its **inverse**. Worth
the inversion: *"Check spelling"* is what Acrobat's field properties says and
what every other application says, and a checkbox labelled "Do not check
spelling" is read wrongly by roughly half of everybody.

### `fn flag_spell_check_hover`

It says **pdfcer does not spell-check**, because the setting otherwise reads
as a promise this program makes. It is a note in the file for whichever
reader fills the form in.

### `fn note_duplicate_sent`

# This sentence exists because the engine's refusal names nothing

Both `add_choice_field` and `edit_field` refuse a repeated export, so the
document is safe either way. What the engine's refusal cannot do is reach
the operator as anything but the funnel's un-categorised line — *"That
change was refused"* — and a person looking at a thirty-row drop-down needs
the value, not the verdict. The panel asks first and says which one.
