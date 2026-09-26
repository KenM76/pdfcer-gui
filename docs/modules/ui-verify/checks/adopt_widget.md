# `ui-verify/checks/adopt_widget`

`adopt_widget` — **insert a form's pages to CREATE orphaned widgets, then
register one back into the document.**

# The only check in this suite that needs two features, because the
fixture is a STATE and not a file

An orphaned widget is a `/Widget` annotation in a page's `/Annots` that no
`/AcroForm` field reaches. There is no fixture PDF that contains one and
there should not be: it is not a shape a producer writes, it is a shape
**pdfcer itself makes**. `EditSession::insert_pages` copies everything
reachable from a page, and a page's `/Annots` reaches its widgets — but
`/AcroForm` is a **catalog** entry, so it is not in the set of objects being
copied. A source with 12 fields inserted into another document arrives as 13
inert boxes and no form at all.

So this check drives Insert pages first, not as setup but as **half the
subject**: the defect it exists to catch is exactly *"pdfcer made these and
then could not undo it"*, and a hand-authored fixture would prove the
registration works on a shape pdfcer never produces.

It is also why the two halves cannot be split into two checks. The state
only exists inside one session.

# What an operator meets if this regresses

A box drawn on the page with a border and a background, indistinguishable
from the field beside it, that swallows every keystroke. This project's
recurring failure — a visible control that is silently inert — arriving
through a **document** instead of a ribbon.

# Why the button's LABEL is asserted and not only its presence

`adopt_preview` shipped so the row could read *"Register as `Address`"*
instead of *"Register"*, and the engine's framing is the reason it matters:
for a merged field-widget the name is **in the file and not on screen** —
the widget belongs to no field, so no field row names it. A button that says
only *"Register"* is a guess the operator is being asked to accept.

The harness cannot read a label off the screen, so the panel traces its
decision per row and this check asserts on that. Presence alone would pass
on a build where the preview was never asked and every row said *"Register"*
— which is precisely the state this morning's request was filed about.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | Edit ▸ Insert pages, with `PDFCER_DIAG_INSERT_PATH` set to a form | `insert-pages` traced, `orphans>0` in the disclosure |
| B | open Forms ▸ Tab order | `tab-order-unclaimed` census with a non-zero count |
| C | read the per-row preview trace | at least one row knows the name it would register under |
| D | press the first row's Register | `adopt-widget-requested`, then `adopt-widget … epoch=` |
| E | re-read the census | one fewer unclaimed widget than before |

## Item notes

### `const SOURCE`

# `demo-form.pdf`, and the first choice was wrong for a reason worth
keeping

It was `multi-widget-form.pdf`, picked because the engine's measurement
("11 of 13 merged, 2 bare kids") suggested a fixture with both shapes. The
driven run answered `orphaned_widgets = orphaned_widgets_unrecoverable = 3`
— **every** orphan was a bare kid, so the row that resolves a name never
appeared and the phase asserting it could not pass.

The fixture's own name says why, once you know what the two shapes are. A
*multi-widget* field is one field with several `/Kids`, and a kid IS a bare
kid the moment `insert_pages` drops its `/Parent`. A **merged**
field-widget (§12.7.3.1) is the opposite arrangement: one dictionary serving
as both, which is what a form with one widget per field is made of.

So the fixture that exercises the recoverable path is the plain one. Worth
recording rather than just changing, because "pick the fixture whose name
mentions the feature" is the intuition that produced the wrong answer here.

### `const MAX_SCROLL`

A bound rather than a loop, because "scroll until you find it" over a list
that does not contain it is a hang, and a hang in a suite is a failure with
no message.

### `fn engine_fixture`

The path is derived, not configured. `D:\Dev\pdfcer` is READ-ONLY to this
project and its corpus is the only place these shapes exist, so the check
reads from it and writes nowhere near it. Returning `None` rather than
panicking is what turns a missing corpus into a SKIP with a reason instead
of a crash in the middle of a suite.

### `fn unclaimed_total`

Summed across the pages of the **latest** frame rather than taken from one
page. The insert puts the form's sheets somewhere in the middle of the
document, and which page they land on is the insert's business, not this
check's — a check that hard-coded a page index would break on a change to
the insert position that is not a defect.

### `fn open_tab_order`

It defaults closed deliberately — the section is a diagnostic, not the
panel's main job — so a check that assumed it open would report the whole
feature missing on a correct build.
