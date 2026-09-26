# `ui-verify/checks/redact_preview`

`the_apply_report_lists_the_text_it_will_destroy` — the driven proof of
`OPERATOR_REQUESTS.md` **O217**, fourth bullet.

# What this closes

Until this block the apply report answered *how much*: regions, pages,
characters, content streams. A number cannot be checked against an
intention. Redaction is the one verb in this program whose mistake survives
the undo stack, so *"what will be removed is visible before it is
committed"* is a safety control and not a convenience.

The case that makes it load-bearing: the shell's unit of text selection is
the visual line, and `G032` records that the engine groups a line with no
horizontal-gap criterion, so a bill-of-materials row welds its item number,
part number, description and quantity into one selectable thing. An operator
who marks the quantity has marked the row, R8b forbids saying so on the
canvas, and this list is the only place he can be told.

# ★★★ What the oracle is, and the two things it cannot see

The dialog publishes one trace line and one region:

```text
pdfcer-diag redact-apply-removed-text state=listed entries=1 chars=24 lines=4
pdfcer-diag ui-rect name=redact-apply-removed-text rect=[..]
```

`state` is the branch the derivation took, carried out of it rather than
re-derived, so the line reports what was drawn rather than what a second
reading of the report would have drawn. The region is published through
`diag::ui_rect_visible`, so *drawn and readable*, *drawn behind the fold*
and *not drawn at all* are three different observations rather than one
absence.

**It carries no text, deliberately.** `PDFCER_DIAG` writes to stderr and
gets redirected into files; a redaction surface that copied the operator's
confidential strings into a second file would have undone its own job. So:

| this check proves | this check cannot prove |
|---|---|
| the block exists in the build and was laid out | that the quoted strings read correctly |
| it found text rather than reporting none | that the quotation marks are the right glyphs |
| it listed at least as many characters as the fixture put on the page | the order of the entries |

`chars` is what makes the middle row more than a presence assertion. The
fixture's page 1 carries exactly one known string, so a build that listed
*some* text rather than *that* text reports a count below its length and
fails here. That is the strongest content assertion available without
putting content in the log.

# ★★ Why a whole-page mark rather than a selection

The selection route has its own check
(`a_selected_object_can_be_marked_for_redaction`). This one wants a mark
whose contents are known *to the harness*, and the only marking gesture with
that property is the one that takes everything on a page the harness itself
generated.
