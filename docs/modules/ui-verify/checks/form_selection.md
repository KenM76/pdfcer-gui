# `ui-verify/checks/form_selection`

`a_click_inside_a_form_selects_what_is_drawn_there` — the driven proof of
the operator's headline defect.

# The defect


> *"There are obviously more than one item on the page, but when I click on
> one of the objects all I get is the page selected. When I double click on
> an object it doesn't select — it still only has the whole page selected."*

He was reporting the truth, precisely. His file wraps its visible body in a
**form XObject**, `pdfcer-core` did not descend into one, and the form's
`/BBox` is a clipping extent (§8.10.1) rather than a claim about ink — so a
page-sized form sat in paint order above everything drawn before it and won
every click at every point. He was selecting a real object. It was the
wrapper.

# Why a unit test is not enough here, and this is R1's own argument

`panels::objects::provider_tests` proves the hit test against a real
decomposition, and those tests are good: they were falsified by putting the
shallow query back, which turns three of them red. They still cannot see
four things that stand between the engine's answer and the operator's
screen, and every one of them has been a real defect on this project:

| | the failure it would hide |
|---|---|
| the **mode gate** | Edit is required for content selection, and a check that did not switch modes once reported the gate as a selection defect |
| the **coordinate hop** | canvas space, window points, DPI scaling and the page raster's own rect. A wrong page height mirrors every click about the page centre and hit-tests something plausible |
| the **dock geometry** | a panel width changes the canvas rect, which has silently invalidated harness coordinates before |
| the **click actually reaching egui** | in-process injection would not exercise the focus machinery a person's click does |

*"The tests pass"* is not a report of working software. That is the rule
this project was founded on, and this file is its discharge for the form
work.

# The oracle


`first=` is `object:N`, `leaf:N` or `none`. The kind is spelled out rather
than implied, because `objects[7]` and `leaves[7]` are different things in
the same document.

# The sequence

1. open the pinned fixture — a 200 × 200 pt page whose **only** page object
   is a page-sized form holding three 40 × 40 squares;
2. click the Edit mode segment, because a Read-mode canvas click on content
   is refused by design (`DEFECTS.md` D6);
3. click the **centre of the middle square**, through the OS;
4. assert `first=leaf:` — the object painted inside the form;
5. click a **gap between two squares**, still inside the form's page-sized
   box;
6. assert the selection is **empty**.

**Step 6 is the half that is easy to lose and expensive to lose.** It
forbids the tempting "fall back to the shallow hit test when the deep one
finds nothing" repair, which would answer a click on blank paper inside a
page-sized form with the form — the operator's original complaint, restored,
for the case that produces it most often. A check that asserted only step 4
would stay green through that regression.

# Why this check pins its own fixture and ignores `--pdf`

The same reason `ocr` does, learned the same week: a check whose subject is
*"what does a click inside a form select"* cannot take an arbitrary
document. On a drawing with no forms — the operator's own SolidWorks export
has **zero** — the honest answer is *"there was nothing to descend into"*,
which is neither a pass nor a defect. A suite-wide `--pdf` is a convenience
for the checks that need *some* drawing; this one needs a specific shape.

The fixture is the engine's `forms-xobject/page-sized-form.pdf`, read from
the read-only corpus at `D:\Dev\pdfcer`. It is the file the engine built to
reproduce this operator's report, so the check and the fix are aimed at the
same target by construction.

# Where the aim comes from

The squares' page-space boxes are known from the fixture and stated as
constants below rather than discovered at run time, and the geometry hop is
`crate::coords::CanvasMapping` as every other driven check uses. A literal
screen coordinate is never written: `crate::coords`'s header records what
that cost the last time somebody tried.

## Item notes

### `const ON_A_SQUARE`

The **middle** one deliberately: it is furthest from every page edge, so a
small error in the coordinate hop lands on paper rather than off-window,
and the check fails with "selected nothing" rather than with "the click
went outside the client area", which are different diagnoses.

### `fn engine_fixture`

The path is derived, not configured. `D:\Dev\pdfcer` is READ-ONLY to this
project and its corpus is the only place this shape exists, so the check
reads from it and writes nowhere near it. `None` rather than a panic turns
a missing corpus into a SKIP with a reason instead of a crash mid-suite.

### `fn drive`

The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
not hold (FAIL), `Ok(None)` is a pass. An author who reaches for `?` gets a
SKIP, which is the safe default — the unsafe default would be a pass.

### `fn aim`

Its own function so the two call sites cannot hop differently — the class of
error `crate::coords` exists to prevent, and the one a literal screen
coordinate always is.

### `fn last_first`

The **last** line rather than a count of new ones. `canvas-selection` is
emitted through `diag::trace_changed`, so a click producing the same
selection as the previous one emits nothing — a consumer that counted lines
would read a legitimate no-change as a dropped event. Reading the last line
asks the question this check actually has: *what is selected now?*
