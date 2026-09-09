# HANDOFF — `resize_scales_a_shape` is red, and four explanations for it were wrong

> ## ★★★ RESOLVED, 2026-09-08 late evening — the press never reached the canvas
>
> **The cause was egui's default FLOATING scrollbar.** It allocates no width,
> draws nothing until hovered, and `interact`s the rightmost/bottom **10 pt**
> of the scroll area with `CLICK | DRAG` *after* the content
> (`egui-0.35.0/src/containers/scroll_area.rs:1316–1334`); a press anywhere on
> that band centres the handle on the pointer (`:1405`) — a scroll jump. At
> fit zoom the page's right edge sits 6 pt inside the viewport edge, so the
> sheet border's SE grip was drawn on top of an invisible control that took
> the press first. The trace that named it (one new line, `canvas-gesture`,
> in `gesture/mod.rs`):
>
> ```text
> canvas-press   grip=SE … drag=Resize            ← understood
> canvas-gesture started=0 dragging=0 … origin=1  ← never delivered to the canvas's Response
> canvas rect=… off=[442.3 1045.9]                ← scroll jumped 453 pt on the press frame
> canvas-unavailable reason=nothing-visible       ← page left the screen
> ```
>
> **⚠ Explanation 2 below was RIGHT in substance and was killed by a wrong
> comparison.** *"428 × 302 px is nowhere near the canvas"* compared the
> outline to the window; the canvas viewport was **444 × 592** and the outline
> filled its width. A disproof is a measurement too — record what it compared
> against (`feedback_a_disproof_is_a_measurement_too…` in agent memory).
>
> **Fix** (`canvas/present.rs::scroll_style`): solid bars, `foreground_color`,
> 10 pt, `ScrollBarVisibility::AlwaysVisible` — beside the page, never over it,
> the Acrobat/Word convention. **That exposed a second defect on its first
> run:** with a bar of real width, `fit::placement` measured the centre against
> `inner_rect` and placed it against the pre-area outer, and the page crept
> **7.4 px per frame** (half the 14 pt allocation) because the exact-compare
> resize gate was held open by a 0.1–0.5 pt central-panel width jitter. Fixed
> by measuring ONE viewport before the area — `inner_avail = available −
> allocated_width()`, style set first — and deriving the fit viewport, `vp`,
> and the frame record from it. `CanvasFrame::outer == viewport` now by
> construction; its doc carries the history.
>
> Both findings are in `D:/dev/rag/egui/` (`a_floating_scrollbar_is_an_invisible_control…`,
> `a_measure_and_place_pair_against_two_viewport_sizes…`). The check's own
> whole-page warning now describes the real mechanism and tells the reader
> NOT to re-aim — a page-sized selection is the regression test.
>
> **Still open, measured, not chased:** the central-panel width oscillates
> 0.1–0.5 pt frame to frame (`central-panel rect max.x` 732.0/732.3/732.4) with
> `dock.right` unchanged, which flickers the fit zoom in its last digit
> (0.1737↔0.1739) and runs `fit::placement` every frame. Harmless now that the
> pair is consistent (±0.3 px shimmer), but it is a loop and its source is
> unknown. Instrument, don't theorise.

**Written 2026-09-08, end of session. Shell `7361d79`, engine `00ddbb1`.**
**The PC is free — drive.**

> ## ★★★ UPDATE, SAME EVENING — THE INSTRUMENT WAS BUILT AND IT ANSWERED ON ITS FIRST RUN
>
> §4 asked for a trace of the assembled `Press`. It exists now
> (`canvas-press`, `canvas/pressing.rs`), plus a second at the resize arm
> (`canvas-resize-arm`, `canvas/interact.rs`). Re-running the failing check
> produced this, and it **narrows the fault to one link**:
>
> ```text
> canvas-press grip=none handle=0 … drag=TextSelect click=1
> canvas-press grip=none handle=0 … drag=Marquee    click=1
> canvas-press grip=Move handle=0 … drag=Move       click=1
> canvas-press grip=SE   handle=0 … drag=Resize     click=1     ← the press is RIGHT
>
> canvas-resize-arm …                                            ← ZERO. never fires.
> ```
>
> ### What that eliminates
>
> | | |
> |---|---|
> | the grip hit test | ✅ **works** — `grip=SE` |
> | the `handle`-outranks-`grip` precedence (§4's hypothesis) | ✅ **not the cause** — `handle=0` |
> | `press_kind`'s rung order | ✅ **works** — `drag=Resize` |
> | `canvas::resizing::drag` | **never called** — no ghost, no `resize-*` line of any kind |
>
> ⇒ **§4's hypothesis is dead too. That is five.** The press is understood
> perfectly and the fault is entirely downstream of it.
>
> ### Where it now is, exactly
>
> Between `press_kind` returning `DragKind::Resize(SE)` and
> `canvas/interact.rs`'s `GestureOutcome::Resize` arm — i.e. inside
> **`canvas::gesture`'s drag state machine** (`gesture/mod.rs`, and
> `gesture/outcome.rs:425` where `DragKind::Resize` becomes
> `GestureOutcome::Resize`).
>
> ★ And the drag became **nothing at all**, not something else: the trace holds
> no move, no transform and no marquee outcome for that gesture. So the
> question is not *"which other kind claimed it"* — it is *"why did the machine
> produce no outcome"*.
>
> ⚠ `gesture/mod.rs`'s own contract is the first thing to read:
> *"`press_kind` is consulted **only** on the frame a drag starts"*, and
> *"a press starts a drag and returns `Idle`. Nothing else."* **The four
> `canvas-press` lines are HOVER states** — the slot fires on change, so what
> is recorded is the sequence of meanings as the pointer travelled, not
> necessarily the meaning sampled on the press frame.
>
> ⇒ **The next instrument is one line**: trace what `GestureState` latched when
> the drag started. If it latched `Move` or `Marquee` from a frame before the
> pointer reached the corner, that is a stale-sample defect and it explains
> everything. **Measure it; do not conclude it.** Five stories have died here.


> **Read this whole file before touching anything.** Its value is not the
> hypothesis at the bottom; it is the four things that are already *disproved*,
> each of which was confident, specific and reached by reading code instead of
> running it. Re-deriving any of them costs an hour.

---

## 1. State, every number re-measured for this file

| | | measured with |
|---|---|---|
| tests | **3,894 passing, 0 failing** | `cargo test --workspace`, summed through `awk` |
| gates | **32 of 32, 0 skipped** | `bash tools/gates/run-all.sh` |
| driven checks registered | **196** | `Box::new(` inside `checks/roster.rs`'s `all()` |
| engine | `pdfcer-core` **0.49.0 at `00ddbb1`** | `Cargo.lock` |
| shell | `7361d79`, on `origin/main` | `git log -1` |
| request channel | **198 files open, 40 of them `request_*`** | `ls …/open \| wc -l` |

⚠ **Nothing was released on 2026-09-08.** The last published build is
`v0.5.0-dev.20260908.5` (OneDrive `pdfcer-gui2`), from `3ce70a5` at 13:05 —
**before** the day's five fixes. Everything since is committed and unpublished,
deliberately, because of the red this file is about.

---

## 2. The failing check, and exactly how to reproduce it

```bash
cargo build --release -p pdfcer-gui -p ui-verify
mkdir -p target/scratch/drive && cp target/release/pdfcer-gui.exe target/scratch/drive/

./target/release/ui-verify \
  --exe target/scratch/drive/pdfcer-gui.exe \
  --pdf fixtures/a1-titleblock.pdf \
  --doc-point 0,300,500 \
  --check resize_scales_a_shape \
  --out target/scratch/uv
```

⚠ **Copy the exe to `target/scratch/drive/` first.** Never drive the published
build — the harness's side effects land in the operator's own saved state.

⚠ **The staleness guard will refuse** if the sources are newer than the binary.
That is correct and is not the bug. Rebuild, or pass `--allow-stale` only when
you *intend* to drive an older build (as when bisecting, §4).

### What the check does

1. clicks a mode segment to reach **Edit**;
2. clicks the page at `--doc-point` to select an object;
3. reads the last `canvas.selection-outline` rect from the trace, takes its
   **bottom-right corner** as the SE grip, and drags from there outward;
4. asserts a `resize-commit` event appears.

### What happens

> `aimed at a selection 428 x 302 screen px. ★ THE GRIP DRAG COMMITTED NOTHING
> AND DECLINED NOTHING.`

**No resize event of any kind reaches the trace** — not a commit, not a
decline, nothing.

---

## 3. ★★★ FOUR DISPROVED EXPLANATIONS — do not re-derive these

Each was reached by reading source and reasoning. Each was wrong. This is the
most useful section in the file.

### ❌ 1. "The 2026-09-08 `ghost_box` change broke it"

**Disproved by building `50eebe6` in a `git worktree` and running the same
check against it — identical failure.** It predates the day's work entirely,
and no earlier session recorded ever running this check, so how old it is is
unknown.

### ❌ 2. "The selection is the whole page, so the grip is at the page corner and the drag aims off the canvas"

Genuinely plausible: `properties-panel` reports the selected object as
**`2383.94 × 1683.78`** — *exactly A1* — so the click really does select the
sheet's own border rectangle.

**Disproved:** the outline on screen is **428 × 302 px**, nowhere near the
canvas. The grip is comfortably reachable and the drag stays on the canvas.
The check now prints its own aim and fires an explicit whole-page warning; the
warning does **not** fire.

### ❌ 3. "An annotation is selected, so the markup rung claims the grip"

This one had real evidence behind it: `canvas.rotate-handle` is published **4
times** in the failing trace, and this shell draws a rotate handle for markup
annotations.

**Disproved:** `annot-select` appears **zero** times, and `properties-panel`
reports `object=0 kind=Path`. It is page content. The rotate handle is
legitimate for content too — content rotates through `transform_objects`, so
`grabbable`'s content arm answers `GripSet::all()`.

### ❌ 4. "The `canvas.selection-outline` double-publish makes the harness aim at the wrong rect"

**Real, and not the cause.** `canvas.selection-outline` genuinely *is*
published twice per frame with two different rectangles — once per selected
entry in the loop, and once for the grip box at `overlay.rs`'s
`if let Some(box_) = grip_box(…)` — so a consumer asking for "the" selection
outline gets whichever won the race. **But for a single selection the two
differ by a fraction of a pixel** (296.0 vs 296.2 in the trace), which cannot
make a grip miss.

⇒ Still worth fixing on its own merits, separately. The call site argues for
the shared name deliberately (*"the name is the SELECTION's rather than the
grips'"*) — and that argument was made when only one of the two existed, so it
needs **re-deciding**, not just renaming.

---

## 4. The live hypothesis — labelled as one, because four siblings already died

The press path, read out (all line numbers current at `7361d79`):

```
canvas/pressing.rs:723        assembles gesture::Press { grip, handle, markup_grip, … }
canvas/gesture/meaning.rs:587 press_kind(press, caps)
  … meaning.rs:1091           MARKUP rung:  match grip { Some(g) if g.is_resize() => Resize(g) }
  … meaning.rs:1108           CONTENT rung: match (handle, grip) {
                                              (None, Some(grip)) if grip.is_resize() => Resize(grip),
canvas/gesture/outcome.rs:425 DragKind::Resize(grip) => GestureOutcome::Resize { … }
canvas/interact.rs:969        the Resize arm → canvas::resizing::drag(…)
```

★★★ **Note the content rung's pattern: `(None, Some(grip))`. `handle` OUTRANKS
`grip`.** `meaning.rs` says so in its own words — *"a handle sits INSIDE the
selection's box … deeper than anything a grip describes, so it wins."*

**HYPOTHESIS (unverified):** the selected object is a **rectangle**, so its
corner *node* lies exactly on the corner of its own bounding box — which is
exactly where the SE grip is. If node/handle hit-testing answers `Some` there,
`handle` wins, the press is not a Resize at all, and whatever the handle rung
does at the Object rung does nothing and says nothing.

⚠ **Do not act on this before measuring it.** It is the fifth story of the day
and the previous four were equally coherent.

### The instrument that would settle it, and it does not exist yet

**Nothing traces the assembled `Press` or the resulting `DragKind`.** That
absence is the whole reason this is a mystery rather than a diagnosis: the
failure is *"no resize event"*, and there is no line anywhere saying what the
press was understood to be instead.

⇒ **First piece of work: add one.** At `canvas/pressing.rs:723`, after
`press_kind` returns, trace the inputs and the outcome — something like:

```
canvas-press grip=SouthEast handle=none markup_grip=0 widget_grip=0
             node=none tool=Select drag=Resize(SouthEast) click=1
```

Then re-run the check and read it. Three outcomes, three different files:

| the trace says | the fault is in |
|---|---|
| `grip=none` | the grip hit test — `handles::grip_at_in`, or the box it is given |
| `grip=SouthEast drag=Handle(..)` or `drag=none` | **the hypothesis above** — precedence in `meaning.rs` |
| `drag=Resize(..)` | `canvas::resizing::drag` or the commit path in `interact.rs:969` |

★ Write it as a **permanent** trace line, not a temporary `dbg!`. Every press
on this canvas is currently unobservable, and this is the second time in two
days that has cost an hour.

---

## 5. Before you decide it is harness-only

The check may yet be wrong rather than the application. Two cheap ways to tell,
both worth doing **before** the instrument work if you want a fast answer:

1. **Aim at a small shape instead of the sheet border.** The current
   `--doc-point 0,300,500` on `a1-titleblock.pdf` selects the A1 border. A
   fixture with a small rectangle would separate *"resize is broken"* from
   *"resize is broken on a page-spanning path"*. ⚠ I tried
   `fixtures/square-no-appearance.pdf --doc-point 0,150,650` and the click
   selected nothing — that fixture's geometry is unknown to me, so find the
   real coordinates first (`pdfcer list-objects`, or the Objects panel).
2. **Drive it by hand.** Open the release build on a drawing, select a shape,
   drag a corner. If it resizes, the check is the subject. Ninety seconds, and
   it is the one instrument that cannot be wrong about what an operator sees.

---

## 6. What else is outstanding

> ★ **2026-09-09 — the central-panel width jitter is CLOSED.** It was egui's
> own solid scroll bar fading in inside the Comments body, overshooting its pane
> by a rounding residue, and `Panel::show` sliding the whole side inward by the
> excess. Found by `egui-shell/src/dock/overflow_probe.rs`, reproduced with
> pure egui in `dock/scroll_fade_repro.rs`, fixed in `dock/stack.rs`
> (`new_child`, not `scope_builder`). RAG:
> `D:/dev/rag/egui/a_solid_scrollbar_fading_in_overshoots_its_pane_…`. The
> `RESIZE_FLOOR_PT` floor in `fit.rs` stays, re-documented.

### Waiting on the engine

| filed | subject |
|---|---|
| `request_an_ink_stroke_has_readable_vertices_and_no_way_to_edit_them.md` | freehand-stroke node editing. Asked whether the refusal is a decision or a not-yet; if it is a decision we owe him a sentence saying so |
| `note_the_sticky_note_refusal_is_correct_and_its_sentence_is_false.md` | low priority, no reply needed. The `/Text` refusal still claims *"pdfcer did not draw it"* about a marker pdfcer drew |

### Ready to publish the moment the red is settled

Five operator-facing fixes are committed and unshipped — page-preview
checkbox + timeout box, text-box newlines, text-box resize, the resize preview
on markup, and sticky-note handles removed. `OPERATOR_REQUESTS.md` O150–O158.

⚠ **Refresh `FEATURES.md` and smoke-launch before packaging** — both are
standing rules and both have been skipped before.

---

## 7. The methodological finding of the day, because it is the actual lesson

**Four wrong explanations, all reached by reading code carefully and not
running it.** Not one of them was careless; each cited real source, real line
numbers, and a real mechanism. They were wrong anyway.

The engine team independently corrected a fifth on the same day — a request of
ours asserted a code branch was fine, and their test found it broken on **four**
cases where we predicted three, because *"the branch was correct given real
newline bytes and never received any."*

⇒ The rule this session earned, now in agent memory:

- **Measure the intersection, not one condition** —
  `feedback_count_a_condition_and_you_will_reason_about_the_other_one`
- **A check whose input is chosen for convenience tests the assertion** —
  `feedback_a_check_whose_input_is_chosen_for_convenience_tests_the_assertion`
- **A wrong refusal sentence hides a defect in whoever believes it** —
  `feedback_a_wrong_refusal_sentence_hides_a_defect_in_whoever_believes_it`
- **An `||` between two required conditions asserts neither** —
  `feedback_an_or_between_two_required_conditions_asserts_neither`

★ And the one that applies to *this* file: **a check that cannot say what it
aimed at cannot be believed about what it found.** `resize_scales_a_shape` now
reports its aim first. That change is what killed explanation (2), and it is
the reason explanation (5) will be cheaper to kill if it is also wrong.
