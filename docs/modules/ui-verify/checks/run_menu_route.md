# `ui-verify/checks/run_menu_route`

**The right-click route to one line inside a block of text** — O188(A),
driven end to end on the real binary.

One check: `the_right_click_offers_the_line_you_clicked`.

# The operator's report, and what it is actually about

`OPERATOR_REQUESTS.md` O188's (A) row, left open after the refusal sentence
landed:

> ⚠ **(A) is still open.** The delete is reachable only through the Points
> tool (`A`, then click); a double-click on text opens the caret instead,
> which is O70's ruling and correct. The new sentence tells him Delete
> works, but only after he has tried to drag. **A route he can find
> *before* failing is owed.**


⇒ The fix is a row in the canvas object menu: *Select this line of text*,
offered when the right-click landed on a line of a text object that has
more than one. This check is the assertion that the row exists, that it is
about the line the pointer was on, and that pressing it lands on that line.

# Why a unit test cannot make this claim, and what it would miss

Every mechanism below has unit tests and they were all green while the
route did not exist. The reason is that the operand is **parked in
`egui::Memory` on the frame of the right-click and read back on the frame
of the press**, and there is no in-process test in this project that can
own two frames of a real popup:

| mechanism | its unit test proves | what it cannot see |
|---|---|---|
| `canvas::runmenu::pick_at` | the three gates answer correctly for a given provider | that anything calls it on a right-click |
| `canvas::runmenu::park`/`parked` | a value survives a round trip through `Id`-keyed memory | that the key is still live by the time the row is pressed |
| `shell::menus` item table | the row is registered against `canvas.object` | that `shown_when` ever resolves true in a running frame |
| `dispatch::format` | the arm calls `select_part` when `resolve` answers | that `resolve` is reached at all |
| `SelectionState::select_part` | the entry list and level are set | that the operator can get there |

The middle row is the one that bites. `MenuHost::with_conditions` sets
`canvas.run_select_offered` **per click**, from a pick taken on that click,
and a condition that is never published is simply absent — which reads as
*false*, which drops the row, **silently and with every test green**. That
is R8's mechanism working exactly as designed and it is indistinguishable
from the feature not shipping.

# The chain, and why each link needs its own assertion

The operand crosses four subsystems and three frames. A single end-state
assertion (*"the selection is at the Part rung"*) would be satisfied by a
build in which the menu row does nothing and the Points tool happened to be
armed, so the links are asserted **in order**, each with its own message,
and a build that breaks one fails at that one.

| # | step | oracle | what a wrong build produces |
|---|---|---|---|
| 1 | Edit mode | `ribbon-mode-selected` | Read refuses a content click by design (DEFECTS.md D6) |
| 2 | click the third line | `canvas-selection … level=Object` | the aim is not on a glyph |
| 3 | **control**: the bar has said nothing about a rung | no [`RUNG_EVENT`] line yet | a fossil that would satisfy step 8 |
| 4 | right-click the same point | `canvas-menu context=canvas.object` | the view menu, i.e. the hit test missed |
| 5 | the pick was taken **on that frame** | `text-run-menu pick=line:N/6 offered=true` | `offered=false`, i.e. nothing to offer |
| 6 | **the row is on screen** | `menu.item.canvas.object.format.select_text_line` | O188(A), unfixed: no route |
| 7 | the press found the parked operand | `text-run-command pick=line:N/6 outcome=raised` | `outcome=declined`, i.e. the memory key died with the popup |
| 8 | the ladder entered the Part rung on **that** run | `selection-set … part=N level=part via=select-text-line` | a different N, i.e. the operand was re-derived and drifted |
| 9 | the bar says which line | `status-rung kind=text part=N held=1 of=6` | the rung is entered and nothing discloses it |
| 10 | the bar drew it | `ui-rect status-group:selected` | a sentence computed and never painted |

**Steps 8 and 9 carry the same `N` as step 5, and that is the real
subject of this file.** Each individual line could be produced by a build
that re-picks from scratch at press time — which would be wrong in exactly
the way that is hardest to see, because it works on a one-line document and
picks the wrong line on a thirty-six sheet title block. Asserting that the
index is *the same number all the way through* is what makes this a check
of the parked operand rather than three separate checks of three
mechanisms.

# Why step 9 needs a trace line and could not use the rect


⚠ It is emitted through `diag::trace_changed`, which de-duplicates on the
rendered line. So step 9 is asserted as *"the count was zero before the
press and there is a line after it"* rather than as *"a line follows the
mark"* — an earlier gesture producing the identical clause would suppress
the later one and a mark-relative assertion would report a defect that is
not there. Step 3 is that control, and without it step 9 is vacuous.


# The fixture, pinned here and not read from `--pdf`

`fixtures/paragraph.pdf` — 612 × 792, one `BT`…`ET` block holding **six**
`Tj` operators at 12 pt, baselines at 700, 684, 668, 652, 636 and 620, all
starting at x = 72. The same file `move_line_of_text` uses, for the same
measured reasons: one text object with several runs, legible at fit zoom.


The aim is **(120, 672)** — inside the *third* line, whose baseline is
668 — and the third and not the first on purpose. A build whose pick
ignores the pointer and returns run 0 is a real and tempting defect (it is
what a `.first()` over the run list does), and it is invisible when the aim
is on the top line. So this check asserts `N != 0` as well as `of == 6`.

`N` and `of` are **line** numbers. On this fixture that coincides with
the show-operator count — see [`EXPECTED_RUNS`] for why that is a stated
weakness of this check rather than a silent one.

⚠ A missing fixture is a **FAIL**, not a SKIP: it is committed to this
repository, so its absence is a broken checkout rather than an unavailable
precondition, and a SKIP would say the opposite.

# ⚠ Falsification recipe — run this before believing a PASS

A check nobody has seen fail is a claim, not a measurement. Each of these
must turn this check red, at the step named:


   ⚠ **This recipe has already earned its keep, on the day it was
   written.** Run against the first version of the oracle it PASSED: the
   `status-rung` line was emitted from ABOVE that `match`, keyed on the same
   `PartKind` the arms are keyed on, so it went on being written by a build
   that disclosed nothing — and the check reported a working route on a
   program where the operator stands on one line of six and the status bar
   never says so. The emission now lives INSIDE the two arms. An oracle one
   statement away from the thing it claims to measure is an assertion both
   outcomes satisfy, and the only way to find one is to run the recipe.

(3) and (4) are the two that matter. (1) and (2) break loudly and would
be noticed by a person opening the menu; (3) and (4) are silent, and a
check that cannot distinguish them from a pass is not measuring the
feature.

## Item notes

### `const MODE`

The shell's default is Read, where a canvas click on content is refused BY
DESIGN — and Read's right-click resolves `canvas.read_object`, a two-row
menu (O71) that does not carry this command at all. A check that skipped
this step would report the mode gate as a missing row.

### `const PAGE_SIZE`

Pinned with the fixture rather than read back, for the same reason: a
mapping derived from a page size this check did not choose would silently
aim somewhere else.

### `const EXPECTED_RUNS`

A hard fact about a 976-byte file committed to this repository, quoted in
the module header from its own content stream. It is asserted rather than
read back, because `of` is the denominator the operator is shown — *1 line
of 6* — and a build that counted wrongly would state a wrong number to him
while every mechanism in the chain still worked.

⚠ **This number cannot tell lines from show operators**, and saying so is
the point of the warning. `paragraph.pdf` writes a `Tm` in front of all six
of its `Tj`s on six distinct baselines, so its run count and its line count
are both 6 — a build that had never been re-keyed satisfies this assertion
unchanged. The oracle that CAN tell them apart is
`provider::line::tests::runs_sharing_a_baseline_are_one_line` (3 runs, 2
lines); its driven twin is `move_line_of_text`, whose fixture holds five
runs in four lines.

### `const MODE_DECLINE_EVENT`

Named only in a failure message, as one of the readings of a press that
produced no [`COMMAND_EVENT`] line. A check that did not mention it would
send a reader hunting a dispatch gap when the mode had simply changed.

### `const ROW_REGION`

Publishing is the only possible answer for a popup: a context menu is
drawn **at the pointer** and `egui` may flip it to any of several
alignments to keep it on screen. There is no fraction of the window it can
be hard-coded to and no layout a harness could re-derive.

### `fn run_of`

`None` for `pick=elsewhere` and for anything malformed. The caller
distinguishes those from the raw line rather than from this return: a parse
failure and an honest *"the pointer was not on a line"* are different
findings and must not share a message.

Parsed rather than `Debug`-matched. A `{:?}` rendering of the pick would
make this harness depend on a Rust enum's formatting, which is the defect
recorded as *never `Debug`-format a field a machine reads* — a check there
reported the opposite of the truth while quoting the truth in its own
message. `RunPick::word` writes `line:N/M` as a deliberate, stable token for
exactly this reason.

### `fn drive`

The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
not hold (FAIL), `Ok(None)` is a pass.
