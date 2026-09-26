# `pdfcer-gui-base/text/merge`

## Item notes

### `fn the_success_sentence_names_what_went_in_and_what_came_out`

The property that lets an operator notice a dropped source in the one
sentence they were going to read. Asserted as "both numbers appear"
rather than against the wording, because the wording will change and
the property must not.

### `fn the_success_sentence_says_the_originals_survived`

Not obvious from the outside: *combine* is a word that could mean
*consume*, and an operator who has just pointed pdfcer at four drawings
deserves to be told in the same breath that they are all still there.

### `fn merged`

Both counts, and the pair is the point. A combine that silently dropped
a source writes a perfectly good PDF; the only thing on screen that would
differ is the number of files it says it read. An operator who chose three
and is told "2 files" has been told about a defect in the one sentence they
were going to read anyway.

The page count is the engine's own `AssembleReport::pages` rather than a
sum this module computed, so it reports what was **written** rather than
what was intended — which is the whole difference between a report and a
restatement of the request.

### `fn failed_source`

The name and not the whole path: the sentence appears on a one-line status
row, a Windows path is routinely eighty characters, and the operator chose
these files a moment ago and knows where they are. If the stem is
unreadable the full path is used, because a sentence that names nothing is
worse than a long one.

### `fn failed`

The engine's own error text is deliberately **not** carried. It goes to
the trace, where a reader diagnosing a machine they cannot see will find it,
and it is not operator copy — the same split `crate::text::status`'s
`save_copy_failed` makes, for the same reason: a `Display` impl is written
for a programmer.

What the sentence does say is the part the operator needs and could not
otherwise be sure of: **nothing was written**. A failed combine that left a
half-written file behind would be the frightening outcome, and this says it
did not happen.
