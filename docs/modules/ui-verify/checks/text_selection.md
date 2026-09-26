# `ui-verify/checks/text_selection`

`text_selection_sweeps_and_copies` — the regression test for **a gesture
whose entire behaviour is a drag, on a feature whose entire feedback is a
translucent wash.**

# The defect class this exists for

The sharpest defect class this project has: a grid drawn at a one-point
minor step is ~2,450 hairlines a frame, and **a screenshot cannot catch
that** — 2,450 hairlines and a wash are the same picture. The only way to
see it is to have the running program print the ladder it chose.

Text selection is that trap in a purer form. Its output *is* a wash, drawn
at `canvas::overlay::TEXT_SELECTION_ALPHA` — deliberately low, so the
operator can read the text through it — over the linework of a CAD sheet. A
capture of a page with three words selected and a capture of the same page
with nothing selected are very nearly the same image, and on a dense drawing
they may be the same image to any threshold a pixel oracle could use.

So the oracle is the **trace**, and the application was given a line to say
it with: `canvas-text-selection via=… page=… chars=… quads=…`. `chars=` is
the byte length of the string a copy would put on the clipboard, read from
the same field the copy reads — so a passing check here is a statement about
what would be copied, not merely about what was painted.

# ★ The four-link chain, and which link no unit test observes

| # | Link | Where | Its own test |
|---|---|---|---|
| 1 | a press means text exactly when the mode cannot select content | `canvas::textsel::takes_the_press` | yes |
| 2 | `press_kind` turns that into `DragKind::TextSelect` | `canvas::gesture::meaning` | yes |
| 3 | the state machine carries it across the frames of a drag | `canvas::gesture` | yes |
| 4 | `canvas::interact` builds a `PageContext` from the document and applies the outcome | `canvas::interact` | **no** |

Link 4 is the one a refactor breaks silently, and it is the same shape as
`read_mode`'s link 3: it is a value assembled per frame out of `&OpenDoc`,
and every one of its parts has a plausible wrong answer that compiles. The
`epoch` could be read from the wrong place and stamp every selection stale on
arrival; the `page` could be the strip's first rather than the acting one, so
every canvas point would be converted against another sheet's transform; the
whole `if let` could be skipped on a frame where `page_text()` answered
`None`, leaving a gesture that silently does nothing on exactly the pages
whose content stream is unusual. **None of those breaks a test in the
workspace.**

# The phases, and why the Edit phase is load-bearing

The check is an assertion about a **presence** in Read and an **absence** in
Edit, and `crate::report`'s rule bites on the second: *never treat an absence
as evidence unless you have shown the thing that would have produced it was
working*. So the same document drag is performed in both modes:

| Phase | Mode | Drag | Expected | If it does not hold |
|---|---|---|---|---|
| A | Read | across a band of the page | `canvas-text-selection` with `chars` > 0 and `quads` > 0 | this band had no text; try the next |
| B | Read | — | Escape clears it | — (not driven: keyboard, see below) |
| C | Edit | the **same** drag | **no** new `canvas-text-selection` line | FAIL — the text gesture escaped into the mode whose primary button is the content marquee |

Phase A failing is **not** a failure of the application — it means the
harness swept blank paper, which on a drawing sheet is most of it. So the
band ladder is retried, and if no band has text under it the check reports
SKIP naming exactly that, never PASS.

Phase C is the half that would be easy to omit and is the more dangerous
direction: a build where `takes_the_press` ignored its capabilities would
pass phase A perfectly and would have silently replaced Edit's marquee — the
only content-selection gesture the product has — with a text sweep.

# ★ `quads` > 0 as well as `chars` > 0, and why both

They are the two halves of the one-derivation promise
(`canvas::textsel` §5): the same pass produces the string and the boxes. A
build where the quad grouping silently produced nothing would still copy the
right text and would highlight **nothing at all** — a selection the operator
cannot see, which is indistinguishable from a gesture that did not work. That
is the single most likely way this feature ships broken, and it is one field
on the line.

# Mouse only


# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the page size could not be read and no `--page-size` was given — without
  the page height there is no y-flip;
* a mode segment was never declared, or took no click;
* the canvas is not showing page 1, so the harness's one known page size does
  not describe the page it would be sweeping;
* **no band had text under it** — phase A never succeeded, so there is
  nothing for phase C's silence to be measured against.
