# `pdfcer-gui/text/commands/tests`

## Item notes

### `fn all`

Maintained by hand, and that is the point: adding a command means
adding a line here, and the count assertion in
`crate::shell::commands` cross-checks this list against the
registry, so a command that is registered but never appears here
fails a test rather than shipping with unreviewed copy.

### `fn every_command_has_a_label_and_a_tooltip`

P3 reserves greying for temporarily unavailable and requires that
it always be explained on hover. A command with no tooltip cannot
honour that, and the salvage source shipped four such controls on
the Measure tab.

### `fn no_two_commands_share_a_label`

The defect this prevents shipped: `edit_text_tool_button()` and
`add_text_tool_button()` both returned the literal `"Aa"`, and the
two adjacent buttons in the Content group were distinguishable only
by icon and tooltip. Two identical labels side by side is not a
style problem; it is two controls the operator cannot tell apart.

The check is deliberately global rather than per-group. A label
duplicated across two tabs is less confusing than one duplicated
within a group, but it is still a search result with two answers,
and the moment customization lets an operator move a command
between tabs the per-group version of this rule stops holding.

### `fn tooltips_are_sentences_and_labels_are_not`

A label is a name and takes no trailing period; a tooltip is prose
and does. Stated as a rule in [`crate::text`] and worth checking,
because the two conventions sit two lines apart in this file and
the wrong one is easy to copy.

### `fn the_content_tools_have_real_labels`

`RIBBON_IA.md` §5.4 requires `Aa`, `I⁺ Aa` and `Obj` to become
real words. This asserts the outcome rather than trusting that
nobody copies the old literals back in — `Obj` is not a word, and
it was the label on one of the three primary editing tools.
