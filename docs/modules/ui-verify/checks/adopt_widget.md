# `ui-verify/checks/adopt_widget`

`adopt_widget` — **insert a form's page to CREATE orphaned widgets, then
register one back into the document.**

Driven off the desktop through the scripted pointer, so it runs under
`--no-input` while the operator uses the machine.

# The fixture is a STATE, and the insert makes it

An orphaned widget is a `/Widget` annotation in a page's `/Annots` that no
`/AcroForm` field reaches. `EditSession::insert_pages` carries every field
whose widgets all arrive (`InsertOutcome::fields_carried`), so a whole form
inserted whole leaves no orphans. It still leaves them in three cases
(`InsertOutcome::orphaned_widgets`): a field split across inserted and
non-inserted pages, a widget whose field the source's `/AcroForm /Fields`
does not list, and the repeat copy of a page named twice.

This check uses the second. `fixtures/unlisted-fields.pdf` (built by its
`PROVENANCE.py`) has three merged field-widgets on one page and lists one in
`/Fields`; the insert carries that one and orphans the other two, each with
its own `/T`. Those are the recoverable kind: `adopt_widget` registers them
losslessly. A split field would orphan **bare kids**, which have no name of
their own, and the phase asserting a resolved name could not pass.

The two halves stay one check because the state only exists inside one
session.

# What an operator meets if this regresses

A box drawn on the page with a border and a background, indistinguishable
from the field beside it, that swallows every keystroke — a visible control
that is silently inert, arriving through a **document** instead of a ribbon.

# Why the button's LABEL is asserted and not only its presence

The row reads *"Register as `Address`"* rather than *"Register"* because for
a merged field-widget the name is in the file and not on screen: the widget
belongs to no field, so no field row names it. A bare *"Register"* is a guess
the operator is asked to accept.

The harness cannot read a label off the screen, so the panel traces its
decision per row (`adopt-row named=`) and this check asserts on that.
Presence alone would pass on a build where the preview was never asked.

# Phases

| Phase | Does | Expected |
|---|---|---|
| A | Pages ▸ Insert from file, with `PDFCER_DIAG_INSERT_PATH` naming the fixture; press Insert | `insert-pages` traced, its disclosure asks for re-registering |
| B | View ▸ Forms panel, open Tab order | `forms-tab-page` census with a non-zero unclaimed count |
| C | read the per-row preview trace | at least one row knows the name it would register under |
| D | scroll the first row into the panel, press Register | `adopt-widget-requested`, then `adopt-widget … epoch=` |
| E | re-read the census | one fewer unclaimed widget than before |

Edit mode is rung at launch through `PDFCER_DIAG_INVOKE=mode.edit`; the
ribbon items are clicked, so their reachability is part of what passes.

## Item notes

### `const TARGET`

`fixtures/blank-overhang.pdf`, copied into the run's output folder: one page
and no form, so `acroform_created` is also exercised and the insert never
writes into a file the repository holds.

### `const MAX_SCROLL`

A bound rather than a loop: "scroll until you find it" over a list that does
not contain it is a hang, and a hang in a suite is a failure with no message.

### `fn repo_fixture`

Resolved from `CARGO_MANIFEST_DIR`, so the check reads this repository's
fixtures wherever it is cloned. `None` becomes a SKIP with a reason rather
than a crash mid-suite.

### `fn unclaimed_total`

Summed across the pages of the **latest** frame rather than taken from one
page: where the insert lands is the insert's business, and a hard-coded page
index would break on a change to the insert position that is not a defect.

### `Hand::open_tab_order`

The section defaults closed — it is a diagnostic, not the panel's main job —
so a check that assumed it open would report the whole feature missing on a
correct build.

### `Hand::open_from_tab`

Opens each collapsed group on the tab until the item is declared, because the
off-desktop window is narrower than the operator's and the band collapses
groups to fit it.
