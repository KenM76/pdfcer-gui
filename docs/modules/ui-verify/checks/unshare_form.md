# `ui-verify/checks/unshare_form`

The two driven checks of *"give this page its own copy"* — that the command
has an operator route which is **not** the ribbon, and that it tells the
truth about the document in front of it.

| check | fixture | asserts |
|---|---|---|
| `the_context_menu_gives_this_page_its_own_copy_of_a_shared_form` | `shared-across-two-pages.pdf` | the row is in the menu, the press copies, and the measurement found the other page |
| `the_unshare_declines_when_nothing_else_draws_the_form` | `page-sized-form.pdf` | the press **changes nothing** and says so |

# Why there are two, and why the first one was not enough

Until 2026-08-29 there was one check, it was named
`…_of_a_shared_form`, its pass note read *"Every other invocation site keeps
naming {original} and is byte-identical"* — and it was pinned to
`page-sized-form.pdf`, **a document with exactly one invocation.** There
were no other invocation sites. The check asserted a sentence about a
population of zero and passed.

That is not a cosmetic error in a note. It is the same defect the *feature*
had, reproduced in the instrument that was supposed to catch it: the command
shipped asserting *"This drawing is drawn on other pages too"* on a document
it had never counted, and the check that drove it shipped asserting the same
thing about the same absent pages. **A check whose fixture cannot exhibit
the condition under test measures nothing**, and this one had a name
promising it did.

⇒ So the shared case now uses a fixture that is actually shared, and the
unshared case — which is now a **real behaviour**, a worded decline that
changes nothing — gets a check of its own rather than being the accident
that the shared check happened to be running.

# What this is for


> *"So you can offer the button. Please un-suppress it rather than leaving
> the suppression in place — a control withheld on the strength of a note
> that has since been withdrawn is exactly the kind of thing that stays
> withheld for months."*

## Why the audit's own instrument cannot answer this, which is the
## whole reason this file exists

`tools/verb-coverage.py` greps this crate for each engine verb's name.
`EDITABLE_SURFACES.md` states the limit of that measurement in as many
words:

> **A hit means the NAME appears**, not that a reachable operator route
> calls it. A call site behind a condition nothing sets is a hit here and
> dead in the running program. Only `tools/ui-verify` answers that question.

⇒ So the audit that found this gap **would report it fixed the moment the
identifier appeared in a source file**, whether or not a single click could
reach it. This check is the other half: the verb is called because a person
pressed something.

## The route under test is the CONTEXT MENU, deliberately

`OPERATOR_REQUESTS.md` **O53** rules that a command must not exist only on
the ribbon. That rule is doing more work for this command than for most, and
the reason is about *when* the operator needs it rather than about
consistency:

An operator who needs to unshare is, by construction, **mid-gesture**. They
have clicked inside a title block, they are about to type into it, and the
moment the choice is worth anything is *before* that keystroke — because
afterwards the edit is already in the one shared stream and every sheet has
it. The Format contextual tab is the correct second home. The pointer is the
first.

And the ribbon route is the one a check could pass on while the useful
one was broken: a Format-tab click proves a band item dispatches, which
`font_group` and its neighbours already prove for that tab. Nothing before
this file had ever **pressed a context-menu row** — see the harness gap
below, which is the finding this check turned up.

## The harness gap this check found, and had to close


It stopped there because it had to. `shell::menus::MenuHost::attach_with`
called `egui_shell::menu::Menu::attach` — the convenience constructor that
takes *no optional capabilities at all* — so pdfcer's context menus drew rows
and published **no `ui_rect` for any of them**. There was no coordinate to
aim at, so no check could press a row, so the entire "does the menu row
actually do the thing" question was unaskable.

⇒ That is the same shape `field_menu`'s own header records one layer up:
*"a gesture with no driver is a gesture R1 cannot reach, and the gap left no
failing test behind to advertise itself."* There the driver was missing;
here the **target** was. Both are invisible to a green suite.

`MenuHost::attach_with` now supplies a rect sink, so every row of every
pdfcer context menu publishes `menu.item.<context>.<command id>` through the
same `crate::diag::ui_rect` channel the ribbon and the status bar use. This
check is the first consumer; every future menu check inherits it.

Publishing is the only possible answer for a popup, and
`egui_shell::menu::report`'s header says why: a context menu is drawn **at
the pointer**, and `egui` may flip it to any of several alignments to keep
it on screen. There is no fraction of the window it can be hard-coded to and
no layout a harness could re-derive.

## The oracle: `unshare-form-applied`, and what a wrong build gets wrong

`app::actions::xobject` traces
`unshare-form-applied page=… original=… copy=… moved=…` on the success path.
Three of those four fields are load-bearing here:

| field | what a wrong build reports |
|---|---|
| `original=` | the **innermost** enclosing form instead of the outermost — the operand `EditError::FormNestedInAnotherForm` exists to refuse. On this flat fixture the two coincide, which is why the check also asserts the number against the page's own object list rather than merely against itself |
| `copy=` | the same number as `original`, i.e. nothing was allocated |
| `moved=` | `0`, i.e. the page's `/XObject` names were not re-pointed and the copy is an orphan |

**The absence of the line is the interesting failure**, not its content.
A build where the menu row is greyed, where the dispatcher has no arm, where
`containing_form_object` returns the innermost form, or where the engine
refuses, all produce **no line at all** — and each of those is a state in
which the operator presses a row and the page looks exactly as it did. That
is why this check exists and a unit test would not do: on this command,
*"nothing visibly happened"* is what **success** looks like too.

## Why these checks pin their own fixtures and ignore `--pdf`

`form_selection`'s reason, unchanged and for the same subject: a check whose
subject is *"what happens to a form XObject"* cannot take an arbitrary
document. On a drawing with no forms — the operator's own SolidWorks export
has **zero** — the honest answer is *"there was nothing to unshare"*, which
is neither a pass nor a defect.

And here the fixture is not merely *a* document with a form: **it is the
condition under test.** Sharedness is a property of the file and of nothing
else, so each of these two checks is defined by which file it opens, and
swapping them would swap what each one proves without changing a line of
assertion code. Both are read from the read-only corpus at `D:\Dev\pdfcer`.

### `shared-across-two-pages.pdf` — the shared case

Two 200 × 200 pt pages, both with `/Resources << /XObject << /Fm0 6 0 R >>
>>`, both drawing object 6 at `1 0 0 1 20 20 cm` — a 40 × 40 blue square
from (20, 20) to (60, 60). One form, **two invocation sites, on two distinct
pages**, which is the smallest honest model of the operator's thirty-six
sheet title block.

⇒ The numbers it makes assertable: `places=2`, `pages=2`, **`other=1`**,
`moved=1`. A build that had not counted could not produce `other=1`, and a
build that counted the wrong thing produces `other=2` — the off-by-one that
forgets to subtract the page being unshared, which is invisible on any file
with several sheets and wrong on every file with one.

### `page-sized-form.pdf` — the unshared case

One 200 × 200 pt page whose only page object is a page-sized form holding
three 40 × 40 squares. It is invoked **once**.

This file used to be the shared check's fixture, and the comment that
justified it read: *"It is invoked once, not thirty-six times, and that is
fine — `unshare_form` does not require a form to be shared, and refusing to
privatise a singly-invoked form would be a rule nobody wrote."* Both
sentences were true about the **engine**, which is a verb and does what it
is told. Neither was a defence of the **shell**, which had told the operator
their drawing was on other pages and dirtied their document to give them a
byte-identical clone of it. The rule nobody had written is now written:
`crate::…::UnshareRefusal::NotShared`, and this file is the fixture that
proves it fires.

## The sequence

Steps 1–4 are identical for both checks and live in [`open_and_press`]; only
the fixture, the aim point and step 5 differ.

| # | step | oracle |
|---|---|---|
| 1 | open the fixture, click the Edit mode segment | `ribbon-mode` |
| 2 | click the centre of a square drawn by the form | `canvas-selection first=leaf:` |
| 3 | right-click the same point | `canvas-menu context=canvas.object` |
| 4 | the unshare row is on screen and clickable | `menu.item.canvas.object.format.unshare_form` |
| 5a | shared: it copies, having measured the fan-out | `unshare-form-measured other=1` then `unshare-form-applied` |
| 5b | unshared: it declines, and nothing is edited | `unshare-form-measured other=0` then `unshare-form-declined`, and **no** `unshare-form` funnel line |

Step 4 is O53's assertion and step 5 is the audit's. Neither substitutes for
the other: a row that is drawn and does nothing passes 4, and a command
reachable only from the ribbon would pass 5 if this check pressed a band
item instead.

## Why the decline needs a POSITIVE oracle, and gets one

The obvious way to check a decline is to assert that
`unshare-form-applied` did not appear. It is worthless, and it is worthless
for the reason this file's own header already gives about the *success*
case, read backwards: **every possible breakage produces that same
absence.** A greyed row, a dispatcher with no arm, a menu that never opened,
a click that missed the square, a build with the feature deleted — all of
them are "no applied line", and all of them would pass a check written that
way. It would be a check that could only ever succeed.

⇒ `app::actions::xobject::fanout` therefore writes **two** lines the moment
the walk runs: `unshare-form-measured` on both paths, carrying the numbers
the decision was made from, and `unshare-form-declined … reason=not-shared`
on the declining path only. The unshared check asserts both are present,
that `other=0`, **and** that the `unshare-form` funnel line — which
`vector_edit` writes once per committed edit — is absent. Together those
say: the press arrived, the document was measured, the verb chose not to
act, and nothing was written. No one of them says it alone.

## Item notes

### `const SHARED_FIXTURE`

See the module header. This is the file that makes "shared" a fact rather
than a claim — object 6 is drawn from page 0 *and* from page 1, so
unsharing page 0 leaves exactly one other page on the original and the
disclosure has a real number to state.

### `const PAGE`

Stated rather than read, for `form_selection`'s reason: each file is a
handful of objects of hand-written syntax, and a page size that changed
would change what every constant below means. They agree at 200 × 200,
which is why one constant serves both.

### `const ON_THE_SHARED_SQUARE`

Its content stream is `q 1 0 0 1 20 20 cm /Fm0 Do Q` over a form whose
`/BBox` is `[0 0 40 40]`, so the square spans (20, 20) → (60, 60) and its
centre is (40, 40). Forty points from two page edges at a zoom that fits a
200 pt page to a full-size canvas is a comfortable margin — and the point
is well inside the page box, which is what `CanvasMapping::doc_to_window`
refuses to clamp for.

### `const ON_A_SQUARE`

The **middle** one, exactly as `form_selection` aims: it is furthest from
every page edge, so a small error in the coordinate hop lands on paper
rather than off-window — and a failure then reads "selected nothing" rather
than "the click went outside the client area", which are different
diagnoses.

It matters twice here rather than once, because the same point is
right-clicked. A popup opened near an edge is repositioned by `egui`, which
is exactly the case the published rect exists to survive — but a check
should not be *testing* that incidentally while trying to test something
else.

### `const FUNNEL`

The unshared check asserts this line is ABSENT, which is its proof that the
document was not touched: no `edit_epoch` bump, no undo entry, no dirty
flag. `Trace::events` matches the whole first token, so this never collides
with the three suffixed names above — the property
`tools/gates/check-trace-names.py` exists to hold.

### `struct Scenario`

Its existence is the point made in the module header: these two checks
differ in **which file they open**, and almost nowhere else. Bundling the
three facts that vary keeps [`open_and_press`] identical for both, so a
change to the gesture sequence cannot be made for one case and forgotten
for the other.

### `fn engine_fixture`

The path is derived, not configured, and `None` rather than a panic —
`form_selection`'s helper verbatim in shape, for the reason its own docs
give: `D:\Dev\pdfcer` is READ-ONLY to this project, and a missing corpus is a
SKIP with a reason rather than a crash mid-suite.

### `struct Pressed`

The `Session` travels because the process must stay alive for the caller to
read the trace it wrote — dropping it kills the application, and a trace
read afterwards would be whatever happened to be flushed.

### `fn open_and_press`

Returns `Ok(Ok(Pressed))` when the row was pressed, `Ok(Err(failure))` when
an assertion up to and including step 4 did not hold, and `Err(skip)` when a
precondition was absent. The three-way split is the suite's SKIP/FAIL/PASS
rule made structural: an author who reaches for `?` gets a SKIP, which is
the safe default — the unsafe default would be a pass.

### `fn other_pages`

`None` means the line is absent or malformed, and both callers treat that as
the same finding: **the walk did not run**, which is the state the feature
shipped in and the state these checks exist to prevent returning to.

### `fn aim`

Its own function so the click and the right-click cannot hop differently —
the class of error `crate::coords` exists to prevent. Both gestures in this
check aim at the *same* screen point, and that is load-bearing: the menu
must open over the thing that was selected, not over a second guess at where
it is.

### `fn last_first`

The **last** line rather than a count of new ones, for
`form_selection::last_first`'s reason: `canvas-selection` is emitted through
`diag::trace_changed`, so a click producing the same selection as the
previous one emits nothing, and a consumer that counted lines would read a
legitimate no-change as a dropped event.
