# `ui-verify/checks/redact_selection`

`a_selected_object_can_be_marked_for_redaction` — the driven proof of
`OPERATOR_REQUESTS.md` **O60**.

# What this closes

**Ken, 2026-08-30:** *"the redaction tool — am I able to select objects on
the canvas and redact them that way yet? I only tried it when it only worked
with the search box and it didn't work for some things. it just told me it
couldn't."*

He was right. Until now there were two marking routes and nothing between
them: **the search box**, which reaches text pdfcer can read *as text*, and
**mark whole page**, which reaches everything. On a CAD drawing most of what
wants redacting is in the gap — a title-block value drawn as vector strokes,
a scanned stamp, a logo, a signature image. None is findable by typing, so
*"it couldn't"* was true about the route rather than a defect in it.

# Why the oracle is the MARK COUNT and not the trace

`redact-mark-selection-requested` says the shell built some quads. It says
nothing about whether the engine accepted them — and this verb has a whole
family of ways to be accepted and do nothing:

| failure | what the trace would still show |
|---|---|
| the selection's outlines are on another page and filtered out | nothing at all, silently |
| the canvas→PDF hop is wrong and the quad lands off the sheet | a request with a plausible count |
| the quad is built inside-out (the y flip inverts the corners) | the same |

⇒ So the assertion is the **panel's own census of marks**, before and after.
That is the number the operator sees in the review list, and it is the only
one that means *a redaction now exists*.

# And it asserts the mark is NOT applied

Marking is not applying. A `/Redact` annotation removes no content, and the
single most dangerous mistake this feature can produce is an operator who
believes the opposite — they stop reviewing and save a document that still
contains every word.

So the check also asserts that the page's text is **still extractable**
afterwards. A build that quietly applied on marking would raise the census,
look completely correct, and be the worst possible defect in a redaction
tool. Nothing else in this suite would catch it.

## Item notes

### `const INVOKE`

The panel is opened for its **census line**, not to be clicked: it is the
only surface that counts marks, and this check needs the count before and
after. `mode.edit` first, because a mode change reconfigures the dock and
would close a panel opened before it — learned the hard way on the bookmark
clipboard the day before.

`edit.redact`, and there is deliberately **no `view.panel_redact`**. The
panels module says why in as many words: a second id for the same surface
would put it on a tab Read is shown, and Read must not be able to reach a
marking surface at all. The mode taxonomy does that work with no capability
flag and no gate of its own — which is also why this check asks for Edit.

### `const REDACT_BUTTON`

A `ui_rect` region published by the ribbon for every drawn command, named
after the command id. Pressed rather than invoked because
`PDFCER_DIAG_INVOKE` runs at start-up and this verb needs a selection that
does not exist then.
