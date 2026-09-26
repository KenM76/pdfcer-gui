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
