# `ui-verify/checks/attachment_clip`

`an_attachment_moves_between_two_open_documents` — **the gap the
verb-coverage gate found on its first honest run.**

# Where this came from


⇒ **This check is the answer to "how would anyone have known?"** and the
shape is worth keeping: the gate found the gap, and the gate cannot close
it, because a gate reads source and a capability is a thing you drive.

## Why it must cross a document boundary

A same-document copy-then-paste would exercise every line of the code and
**would not test the defect**. The defect was *"an attachment cannot be moved
between two open documents"*; a check confined to one document is a check
that passes on a build where the clipboard is a per-document field.

That is this project's standing failure mode wearing a new hat — a check
whose subject is narrower than the report it answers.

## And why the disclosure is asserted by its ABSENCE here

`attach_file` builds its name-tree patch with
`entries.retain(|(k, _)| k != &name_bytes)` before pushing, so a same-named
attachment is **replaced** — silently, recoverable only from the earlier
revision. The panel says so before the press, and the region
`attachments.paste.replaces` is that sentence.

In the *second* document there is no file of that name, so the note must
**not** be drawn. Asserting the absence is the more useful half: a build that
drew the warning unconditionally would be crying wolf on every paste, which
is how an operator learns to stop reading warnings — and it would pass any
check that only asserted the warning appears when it should.

## The sequence

| # | step | oracle |
|---|---|---|
| A | attach a file to document 1 | `attach-file` and a census of 1 |
| B | press Copy | `attachment-copied name=… bytes=…` |
| C | open document 2 and go to its Attachments panel | a census of 0 |
| D | the Paste control is drawn, and the replace note is NOT | `attachments.paste` declared, `attachments.paste.replaces` absent |
| E | press Paste | `paste-attachment-requested … replacing=false`, then a census of 1 |

Step D's first half is also an R9 assertion: before step B the clipboard
is empty and `attachments.paste` must be **absent**, not greyed. That is
checked at the top, and it is the control point — without it, a build that
always drew the button would pass every later step.

## Item notes

### `const INVOKE`

NOT `click_mode_segment` plus a ribbon click. The ribbon shows one tab at
a time and this check leaves the operator on whichever tab the mode selector
last drew — which on the first run was File, so
`ribbon.item.edit.attachments` was simply not on screen and the check
skipped on a build where the feature worked. A region absent because it is
on another tab looks exactly like one absent because the feature is missing.

The seam reaches the command wherever it lives, which is the same reason
`checks::attachments` uses it.

### `const REPLACES_REGION`

The first version of this check asserted `attachments.paste.replaces` was
**absent** in the second document, and FAILED against a correct build —
because `ui_rect` is a change log and the panel had legitimately declared
that name several frames earlier, in the FIRST document, where a file of
that name really was present. A region that stops being drawn does not
un-declare itself.

⇒ The panel now publishes exactly one of two names every frame, and this
check reads whichever came last. **An absence assertion became a presence
assertion**, which is the only kind a change log can answer. The finding
is already in `D:/dev/rag/egui/` under
`a_change_log_ui_rect_trace_cannot_report_that_a_widget_stopped_being_drawn.md`,
and this reproduced it within the hour.

### `fn census`

`None` and `Some(0)` are different answers and the distinction is the
whole of the ask-then-toggle rule above: *"the panel is not on screen"* and
*"the panel is on screen and this document has no attachments"* look
identical to any check that collapses them, and the remedies are opposite.
