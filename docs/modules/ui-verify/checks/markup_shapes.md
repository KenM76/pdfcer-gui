# `ui-verify/checks/markup_shapes`

`markup_freehand_and_vertex_kinds` — the four kinds that are not
drag-shaped, and the **one control in this application whose availability is
decided by a gesture in progress**.

# What this is about

Ink, PolyLine and Polygon are engine-ready but **not drag-shaped**; each
needs its own gesture. There are two — a freehand trail
(`canvas::markup::ink`) and a run of clicks with two endings
(`canvas::markup::vertex`) — and neither can be observed by any unit test in
the workspace, because both are **joins**:

| # | Link | Its own test |
|---|---|---|
| 1 | the ribbon click reports the command | `egui-shell`'s `band::render_command` — yes |
| 2 | dispatch routes the id to a `MarkupKind` | `shell::commands::mapping` — yes |
| 3 | the kind arms the canvas tool | `canvas::tool::arm_markup` — yes |
| 4 | `press_kind` gives a vertex kind a **click** and no drag | `canvas::gesture::meaning` — yes |
| 5 | `canvas::interact` routes that click to `vertex::click` rather than to the selection | **nothing** |
| 6 | `app::conditions` publishes `markup.finishable` from the run | its own test — yes |
| 7 | the ribbon reads that condition and enables the control | **nothing** |

Links 5 and 7 are call sites, and **a call site's effect is observable only
in a running window**: every function a join calls can pass its own test
while the join itself is missing. That is why
[`crate::checks::markup_rectangle`] exists for the four-link version of the
same chain.

# The six phases, and why B measures rather than asserts

| Phase | State | Action | Expected |
|---|---|---|---|
| A | Review, Markup tab, nothing clicked | click **Finish shape** | **no** `ribbon-command-invoked` — the control is greyed |
| B | Freehand armed | drag across the page | `markup-tool tool=Markup(Ink)`, then `markup-commit kind=Ink raw=N kept=M` with **M < N**, then `add-markup` |
| C | Polyline armed | three canvas clicks, then Finish | three `markup-vertex n=1,2,3`, then `markup-finish via=command`, `markup-commit kind=PolyLine vertices=3`, `add-markup` |
| D | Polygon armed | **two** canvas clicks, then Finish | **no** invoke — two corners are a line drawn there and back |
| E | …then a third click, Finish, and Finish again | one `markup-commit kind=Polygon vertices=3`, and the **second** press authors nothing |
| F | **Revision cloud** armed | three clicks, then Finish | one `markup-commit kind=`**`Cloud`**` vertices=3` — and NOT `kind=Polygon` |

**Phase F asserts one field**, and one field is the whole of it. A revision
cloud is a `/Polygon` with `/BE` on it, so a control that armed `Polygon`
instead of `Cloud` would place three vertices, finish, author a legal
annotation, render it and add an undo entry — every observable in phases C
through E unchanged. The operator's only symptom is a revision cloud with no
scallop, which reads as an engine rendering bug. `kind=` on `markup-commit`
is the one place the distinction leaves the process.

**Phase B is the only place in this project where the ink simplification can
be measured against a real pointer.** `canvas::markup::ink` §3.3 quotes a
synthetic table from a unit test, and a synthetic curve can be argued with;
`raw=` and `kept=` on the trace of an OS-injected drag cannot. The assertion
is `kept < raw`, which is what a build whose simplification did nothing fails
— and it fails *identically* to a working one on every other oracle, because
both author a perfectly valid `/Ink`.

The drag this harness can deliver is a **straight line**
([`crate::input::Driver::drag`] walks the pointer in eight increments), so
the reduction it measures is the easy case and the numbers are reported
rather than bounded. What the phase establishes is that the code path runs in
the real binary at all; the *quality* of the simplification is the unit
test's subject, and that division is stated rather than blurred.

# Phase D is the falsifier, and here is the build it catches

Everything in A, B, C and E would pass against a build whose
`markup.finishable` was published **unconditionally** — or gated on
`doc.pages`, which is the predicate every other `markup.*` command uses and
therefore the one a copy-paste registration would inherit. The control would
be live, the press would commit, and the operator would have a Finish that
does nothing on almost every press: exactly what `RIBBON_IA.md` P3 forbids,
and exactly what `measure.finish` set the precedent for refusing.

Phase A alone does not catch it either, and the difference is the point:
phase A finds the control dead when there is **no run at all**, which a
`doc.pages` gate would also produce if the document had no pages — and this
check opens one that does. Phase D finds it dead with a run **in progress**,
two clicks deep, at the exact moment the identical two clicks made a
*polyline* finishable one phase earlier. Only a predicate that really asks
`markup::action` — three vertices for a `/Polygon`, two for a `/PolyLine` —
can produce that pair of answers.

Phase E is a second, smaller falsifier: a build whose commit did not **empty**
the run would author the same polygon twice from one gesture, and the operator
would get two identical annotations stacked exactly on top of each other —
indistinguishable on screen and two undo steps to remove.

# Mouse only, and what is therefore NOT covered

Every gesture here is a real `SetCursorPos` + `mouse_event`, because nothing
in this check needs a key.

**Synthetic keyboard input DOES reach the target window** — see
[`crate::checks::add_text`], which types real characters into a caret draft
and asserts they landed, and [`crate::checks::chords`], which presses
declared chords and asserts the command each one resolves to. A chord that
produces no trace line is evidence about *that chord's dispatch*, never
about the machine's ability to type, and a misdiagnosis recorded as fact
protects the defect that produced it.

Two things follow, and both are on the record rather than implied by a green
result:

* **The double-click ending is not driven here.** It is the ending most
  operators will use, and a synthetic double-click depends on two injected
  clicks landing inside `egui`'s double-click window with the harness's own
  settles in between — a timing race that would make this check flaky, and a
  flaky check is worse than an absent one. It is covered by
  `canvas::markup::vertex`'s
  `the_double_click_and_the_command_author_the_same_annotation`, which runs
  **both** endings over identical runs and compares the actions they raise —
  so a driven proof of one ending is a proof about the other by that test's
  equality.
* **Escape's two rungs are not driven** — abandoning a vertex run and then
  retiring the pen. Covered by `canvas::keys`'
  `escape_abandons_a_vertex_run_before_it_puts_the_markup_tool_down` alone.

# Every way this reports SKIP, and why none of them is a pass

* no binary, no `--pdf`, `--no-input` — the harness never began;
* the diagnostic switches did not reach the process;
* the page size could not be read and no `--page-size` was given;
* the Review segment, the Markup tab, or one of the four controls was never
  declared;
* the canvas is not showing page 1, so the harness's one known page size does
  not describe the page it would be clicking on;
* **the freehand drag produced fewer than three trail points** — the harness
  could not deliver enough frames of pointer movement, so `kept < raw` would
  be measuring the harness rather than the simplification.
