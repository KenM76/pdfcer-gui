# `ui-verify/checks/read_mode`

`read_mode_refuses_canvas_edits` — the regression test for **a mode gate
that is a pure function, tested as a pure function, and whose entire value
lies in something no pure function can observe: what a real click does to a
real canvas.**

# The defect class this exists for

The operator asked for it in one sentence, recorded in
`app::modes::capability`'s header:

> *"in read mode the document shouldn't allow editing and should allow only
> selecting of objects that acrobat reader would allow."*

⚠ The shape of the failure this guards against: **clicking a line in Read
selects it, dragging it moves it, Delete deletes it** — three edits in a
mode whose entire purpose is that it authors nothing. Each of the three is a
separate gesture route, so the mode is only as closed as its leakiest one.

The gate that closed it is four links, and every one has a passing unit
test:

| # | Link | Where | Its own test |
|---|---|---|---|
| 1 | a mode's capabilities are derived from its **tab list** | `app/modes/capability.rs` | yes |
| 2 | the running application asks for them, per frame, from the **ribbon's** mode | `app/gating.rs::capabilities` | yes |
| 3 | they reach the canvas and are handed to the gesture machine | `canvas::show` → `canvas::interact`'s `Frame::caps` | — |
| 4 | `press_kind` returns `PressMeaning { click: caps.edit_content, … }` and the machine swallows the click | `canvas::gesture::meaning` | yes |

Link 3 is the one with no test, and it is the one a refactor breaks
silently. `Frame::caps` is a field on a struct built once per frame; a build
that sampled it from `self.modes` instead of from the ribbon would be one
frame stale on exactly the frame a stray click is most likely (the pointer
is already down over the chrome), and a build that defaulted it would get
[`Capabilities::FULL`] — because `Default for Capabilities` is `FULL`, on
purpose, so that a test which does not mention modes is not silently
asserting one. **A `..Default::default()` added to `Frame` would reopen this
defect completely and break no test in the workspace.**

# ★ Why the Edit half is load-bearing, and not a courtesy

The whole check is an assertion about an **absence**: no
`canvas-selection via=click` line after a click in Read. `crate::report`'s
rule for that is blunt — *never treat an absence as evidence unless you
have shown the thing that would have produced it was working* — and the
reason is that the most likely way to write this check wrong is to write one
that passes against a build where the click simply missed the page. Empty
paper produces the same silence as a working gate, and so does a click that
landed on the grey surround, on a panel, or on a page that had not finished
rastering.

So the same document point is clicked twice, in two modes, and the check
only reaches a verdict when the second click **does** select something:

| Phase | Mode | Click | Expected | If it does not hold |
|---|---|---|---|---|
| A | Read | at `P` | **no** `canvas-selection via=click` | FAIL — Read edited the document |
| B | Edit | at `P` | `canvas-selection via=click` with `sel=` > 0 | this point had no content; try the next `P` |
| C | Read | — | `mode-capabilities … cleared_selection=true` | FAIL — the selection survived into Read |

Phase B failing is **not** a failure of the application. It means the
harness aimed at empty paper, which `crate::coords` documents as
symptom-identical to a broken hit test and which has already produced one
filed-then-retracted defect in this codebase. So the pair is retried at the
next candidate point, and if no candidate has content under it the check
reports SKIP naming exactly that — never PASS.

# Phase C, and why it is a separate fact

Refusing a press and retiring what is already there are two mechanisms, and
`app/gating.rs`'s header says why neither is sufficient alone:

> Refusal alone leaves an armed pen drawing a crosshair over a page it
> cannot draw on, and eight resize handles on a selection nothing will move.
> Retirement alone leaves every gesture available for as long as the
> operator stays put.

Phases A and B test the refusal. Phase C tests the retirement, and the
defect it closes is not "Delete works in Read" — it is the *outline and
eight resize handles left on the page*, which are visible controls the
operator can aim at and which would do nothing. That is precisely the
*"visible control, silently inert"* failure `MODES_AND_PANELS.md` Part 1
forbids, and it is why `on_mode_capabilities_changed` clears the selection
on the way in: so that `press_kind` never has to refuse a grip the operator
is looking at.

# ★ The one thing about the trace that shapes this whole file

`canvas-selection` is emitted through `crate::diag::trace_changed`, which
**suppresses a line identical to the last one written to the same slot**.
That is right for the application — a marquee dragged across a sheet would
otherwise bury the events around it — and it is a trap for a check that
clicks the same object twice and expects two lines.

It is why the phases are interleaved per candidate rather than run as
"calibrate first, then test". In this order the *first* successful
selection of the run is the only `sel=` > 0 line that has to appear, so
there is never a previous identical line to suppress it. Misses are
immune by construction: a miss clears the selection and prints `sel=0`, and
a second miss printing nothing at all leads to the same conclusion the line
would have.

Written down because a future reader restructuring these phases into the
obvious "find a content point, then run the three phases" shape would get a
check that FAILs at phase B against a working build, and the cause would be
four modules away.

# Mouse only

Nothing here needs a key. Every assertion below is a press, a drag or a
ribbon click, which keeps this check independent of the input-synthesis
question entirely.

⚠ Synthetic keyboard input *does* reach the target window — see
[`crate::checks::add_text`], which types real characters into a caret draft
and asserts they landed. A chord that produces no trace is evidence about
the **keymap**, never about the machine; reading it as an environment limit
is how a whole class of assertions goes unwritten and unchallenged.

★ **The Delete key is gated by the same `Capabilities::edit_content`**
(`canvas::keys::canvas_keys` receives `caps` on its `Keys` argument), and it
is covered by unit test rather than driven here — named so the gap is on the
record rather than implied.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the page size could not be read from the fixture and no `--page-size` was
  given — without the page height there is no y-flip;
* a mode segment was never declared, or was declared at no usable size, or
  took no click — [`crate::checks::driving::click_mode_segment`] carries
  the reasons and the argument for each;
* the canvas is not showing page 1, so the harness's one known page size
  does not describe the page it would be clicking on;
* **no candidate point had content under it** — phase B never succeeded, so
  phase A's silence proves nothing and is not reported as though it did.
