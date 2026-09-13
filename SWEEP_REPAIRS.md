# SWEEP_REPAIRS.md — the harness repairs the driven sweep earned, and why each one is a defect in the CHECK

**What this file is for.** `tools/ui-verify/` drives the real binary, and R1 of
this project's charter makes it the only oracle that counts. The consequence
nobody plans for is that **a driven failure is a claim about two programs** — the
application and the check — and in this project's measured experience the check
is wrong more often than the application. **Two full sweeps, and the score so
far is twenty-one to nil.** The first — 213 checks, 2026-09-12, `passed=162
failed=11 skipped=40` — produced eleven FAILs and not one of them was a defect
in the application. The second — 220 checks, 2026-09-13, `passed=185 failed=4
skipped=31` — produced **four**, and all four are rows in this file: R4
(`zooming_does_not_throw_away_where_the_operator_panned`), R6
(`text_edit_on_a_real_drawing`), R8
(`the_page_still_renders_at_every_decade_of_zoom`) and R9
(`the_wheel_turns_pages_when_the_operator_asks_it_to`).

★★★ **Read the two tallies as one fact, not two.** Seven of the first sweep's
eleven failures are gone and eleven of its skips are back on, and **not one check
went from green to red.** That is what makes the remaining four safe to ship
against: they are the residue of a shrinking list, not the leading edge of a
growing one.

So this file exists to stop a harness defect from being *reported as an
application defect*, which is the expensive failure: it sends the next session
to debug working code, and it publishes a confident diagnosis naming a mechanism
that is often already repaired. R9 below is exactly that — a check asserting
page-turning against a one-page document, whose failure text accused a
snapshot mechanism that had been fixed hours earlier.

**Timing, and why these rows pile up instead of being fixed as they are found.**
Every repair here is a `.rs` edit. Two staleness guards — `tools/ui-verify/src/main.rs`'s `refuse_if_self_is_stale` and
`launch.rs`'s `staleness_complaint` — refuse to drive a binary older than its
sources. That is correct behaviour and it is what keeps a sweep honest, but it
means **editing any `.rs` mid-sweep turns every remaining chunk into a usage
dump.** The 2026-09-13 sweep held the real cursor and keyboard from 22:42 to
00:22 — one hour forty for 220 checks. So findings are parked here, in markdown,
and applied in one pass after `=== SWEEP-DONE`.

**How to use it.**

1. A row is **deleted** when its repair lands, in the same commit.
2. Each row states *what the sweep measured*, *why the obvious repair is wrong*,
   and *how to falsify the repair*. The third part is not optional: this project
   has shipped several checks that could not fail, and a repair that is not
   falsified is a second one.
3. Rows are numbered in the order they were found, not by priority. The only
   suite-wide one is R1 (a shared liveness detector).

   ★★★ **This clause used to name a second suite-wide blocker, "the File-tab
   route", and it did not exist.** Two checks skipped saying a click on
   `ribbon.tab.file` produced no `ribbon-tab-activated tab=file` line; that was
   read as one broken route in the ribbon and promoted to suite-wide on the
   strength of the second sighting. Driven and measured 2026-09-13: the ribbon's
   tab handler emits that line **unconditionally for every tab**, and the real
   cause was each check's own setup putting a real OS window in front of its own
   click. Fixed, and the check now PASSES — see the correction after the
   2026-09-12 table below.

   ⚠ **And the clause that did the promoting cited a record that has never
   existed.** It said the route was *"already on the housekeeping list from an
   earlier sweep"*; `git log -S "File-tab route" --all` names exactly one commit,
   which introduced all five mentions of the phrase at once. A row that cites its
   own prior existence is making a checkable claim, and this one was false in the
   commit that wrote it.

★ **The standing prior, earned:** when a driven check fails, read the trace
before reading the check's own failure message, and check what the check was
*asked to do* — its `const FIXTURE`, its `--pdf`, its aim point — before
believing what it says about the application. A check's fixture is part of its
assertion, not part of its plumbing.

---

## R1 — the panic excuse must name the event the application published

### What the sweep measured

`the_page_still_renders_at_every_decade_of_zoom` failed on the whole-harness
thread-panic guard (`launch.rs:557-570`), not on its own assertion — which
passed at every rung, `drawn=1` from 141 % to 3,099,517 %.

The panic is **expected**: `pdfcer-render` Pass 296.0 catches `tiny-skia`'s
panic and returns `RenderError::RasterizerLimit`, and the engine's reply says
in writing that *"the panic hook is deliberately NOT silenced … your
thread-panic guard will therefore keep seeing the text"*.

And the application **says so**. `render/worker.rs:806-818` has a named arm
that traces

```
raster-limit scale=509704.3 region=1 panic=<third-party text>
```

— measured, exactly once, at `uv-full/zoom-gallery.trace.txt:9610` — and shows
the operator `canvas_zoom_past_rasterizer()`, the actionable sentence, rather
than the engine's.

### Why NOT `session.expect_thread_panic()`

It exists (`launch.rs:457`) and **has no callers anywhere in the workspace** —
measured by grep; the only file naming it is `launch.rs` itself. It sets one
bool and excuses **every** panic for the rest of the session.

⇒ An assertion both outcomes satisfy measures neither. With a blanket excuse,
a check driving the zoom ladder would go green on a build where the render
worker died of something else entirely — which is the precise failure this
guard was written for on 2026-09-10, when a dead worker was misattributed to
the shell and a correct module was nearly rewritten.

### The repair — replace it, do not add to it

Because there are zero callers, the blanket form can be **deleted** rather than
deprecated. The only way to excuse a panic becomes *naming the diagnostic event
the application published about it*:

```rust
/// Tolerate a worker-thread panic **only when the application reported it**
/// as the named diagnostic event.
///
/// The bare "a panic is fine here" form this replaces excused every panic for
/// the rest of the session, which is an excuse the WRONG outcome also
/// satisfies: a worker dying of an unrelated bug would have been waved
/// through by a check that only meant to tolerate the rasterizer ceiling.
///
/// Naming the event makes the excuse falsifiable — if the shell ever stops
/// converting the panic into `raster-limit`, the check that relies on it goes
/// red, which is exactly when somebody needs to know.
pub fn expect_thread_panic_reported_as(&self, event: &'static str) -> &Self {
    self.panic_excused_by.set(Some(event));
    self
}
```

and in `trace()` (`launch.rs:556`), the excuse is evaluated **against the trace
that was just read**:

```rust
if let Some(panic) = Self::thread_panic_in(&trace) {
    let excused = self
        .panic_excused_by
        .get()
        .is_some_and(|ev| trace.lines.iter().any(|l| l.event == ev));
    if !excused { …existing fatal error, plus: if an event WAS named, say
        which one, and that the application never published it… }
}
```

`Trace.lines: Vec<TraceLine>` with `pub event: String` — measured at
`trace.rs:167-174` and `:68-78`, so the lookup costs a scan of what was already
parsed.

### The two call sites

- `checks/zoom_gallery.rs`, after `Session::launch` at `:498` —
  `session.expect_thread_panic_reported_as("raster-limit");`
- `checks/deep_zoom.rs`, `zooming_past_the_pixmap_ceiling_still_renders`
  (`:80`) — same line. It SKIPPED this sweep, so it has not been observed to
  need it; add it anyway and say in the comment that it is provisioning, not
  observation.

⚠ **Do not match on `panic_message`.** The engine states it is third-party text
and explicitly not a contract, and `text/mod.rs:859-866` already carries a unit
test whose whole job is to catch the engine putting it back into `Display`.

### Falsify it

Plant a build where the named arm in `worker.rs:806` is replaced by the
wildcard `Err(e) => Outcome::Failed(e.to_string())`. `raster-limit` then never
appears, the panic is no longer excused, and the check must go **red**. If it
stays green the excuse is not reading what it claims to read.

---

## R2 — merged into R9

R2 was the first sighting of the wheel-paging check's one-page fixture; **R9**
below is the measured write-up of the same defect, with the trace and the
repair. Kept as a stub rather than renumbered, because R3 exists only as the
app defect this one exposed and the numbering is cited elsewhere.

---

## R3 — the APP defect R9's bad fixture exposed on the way past — a toggle offered where it can do nothing

`app/status/page_box.rs:234` `wheel_toggle` copies **one** of the two
availability clauses — `is_continuous()` — and not `pages.len() > 1`. On a
one-page document the toggle draws, clicks and flips, and can never act.

```rust
// canvas/paging.rs
/// Does the choice EXIST here? Not "which way is it set" — that is the
/// operator's answer and this function must not know it.
pub(super) fn choice_exists(doc: &OpenDoc) -> bool {
    !doc.view.display.is_continuous() && doc.pages.len() > 1
}
pub(super) fn flips_pages(doc: &OpenDoc) -> bool {
    doc.prefs.wheel_paging.flips() && choice_exists(doc)
}
```

`wheel_toggle` returns early on `!choice_exists(doc)`. Then the two doc comments
asserting parity (`page_box.rs:213-219` and the `paging.rs` tests) describe a
mechanism instead of an intention. Lesson already written to
`D:/dev/rag/egui/a_control_that_copies_one_clause_of_its_predicate_is_offered_where_it_can_do_nothing.md`.

---

## R4 — `zooming_does_not_throw_away_where_the_operator_panned` is measured at a point the view cannot reach

### Settled, with numbers — the measurement is Appendix A §3c, below

The check's probe sits **6.4 pt below the bottom edge of the sheet**. From
about 4494 % upward the pan is on its clamp (`at_y` equals `display` to within
half a pixel at every rung above it), so the viewport's top edge is pinned to
the page's bottom edge and the centre sits **half a viewport below it in SCREEN
pixels** — 289.4 px measured, against a half-viewport of 289.15 px.

A fixed screen distance is a shrinking page distance, so the held point must
slide by `(V/2) * (1/z0 - 1/z1)`. Predicted 0.8197 pt, measured 0.8189 pt. The
tolerance is 0.1825 pt. **The application is doing the only thing geometry
allows.** Below the clamp the held point is flat to 0.01 pt over 200 frames.

Both named causes in the check's own failure text are stale: O24e's clamp is
gone (`canvas/geometry.rs:758` returns unclamped and says so), and the drift is
not O24f either — the failing tier is `scroll`.

### The repair — three edits

1. **Move the probe inside the sheet.** The pan that produced this anchor puts
   it outside the page; a check about *holding the operator's place* must ask
   about a place on the drawing. Pick a fraction of the page box, not a
   fraction of the pan.
2. **Add a rung that tests the clamp deliberately**, and assert the CORRECT
   clamped behaviour rather than absence of drift: when the requested anchor is
   unreachable, the held point may move, and must move by **no more than**
   `(V/2) * (1/z0 - 1/z1)` — i.e. never further than the overshoot. That is a
   real invariant, it is falsifiable, and it is the one this rung was
   accidentally measuring all along.
   ★ It needs the viewport rect, which the trace already publishes as
   `ui-rect name=canvas-viewport` — the check reads it today.
3. **Rewrite the failure sentence.** Naming two fixed mechanisms sent this
   investigation to `zoom_anchor_offset` for an hour. The new text must say
   which of the two regimes the run was in — *inside the pan range* or *on the
   clamp* — because the same number means opposite things in each, and the
   check can tell them apart from data it already has (`at` versus
   `page_extent * zoom`).

⚠ **O186 part 1 is now UNREPRODUCED, not fixed.** This measurement removes the
only evidence we had for Ken's *"the cursor still jumps at deep zoom"*. A new
probe, inside the sheet, on both tiers, at the zooms he actually works at, is
owed before anything is built or reported to him.

---

## R5 — the ALONE table

`tools/ui-verify/sweep-full.sh` needs `layers_search_narrows_the_list` and
`new_document_makes_a_page` added, with their `--pdf` fixtures. **Never edit
that file while it is running** — bash reads a script by byte offset.

### R2, measured precisely

- `checks/wheel_flips_pages.rs:169-172` takes the document from `ctx.pdf` and
  errors only when there is none. It never asks how many pages it got.
- `fixtures/four-pages.pdf` **exists** (measured by `find`), as does
  `fixtures/four-pages-unrotated.pdf`.
- The check already establishes the single-page DISPLAY through the ribbon at
  `:203` (`ribbon.item.view.page_single`), and says in its own comment that
  doing it through the control is deliberate. That half is right and stays.
- Its note at `:215-218` reports *"single-page display, on page 1 of the
  document"* — the sentence that made a one-page fixture look like a correct
  starting state.

⇒ Two edits, not one:

1. Pin the fixture: `fixture::workspace_root().join("fixtures/four-pages.pdf")`,
   ignoring `ctx.pdf`, with the reason in a comment — *paging cannot be
   measured on a document with one page, so the operator's `--pdf` is the wrong
   input for this question no matter what he passes*.
2. Assert the precondition it assumed: read `pages=` off the status line and
   fail with a named complaint below 2. **The note must state the count**, not
   just the current page — a precondition that is reported as a page number
   cannot be read as an absence.

---

## R6 — `text_edit_on_a_real_drawing`: wrong fixture, an invented explanation, and an incomplete oracle

Measured in full in Appendix A §4, below. **The application behaved
correctly**; all three faults are in the check.

1. **Pin the fixture.** `D:/dev/pdfTests/SW41177/SW41177.pdf`, ignoring
   `ctx.pdf`, with a SKIP-if-absent guard — the Tranche C repair already owed.
   That file must NOT be copied into `fixtures/`; it is Ken's own drawing and
   the check exists precisely to be measured on one.

2. **Delete the invented sentence at `:403`.** Replace the two-way
   `shares` test with a three-way one, keyed on what the trace actually
   distinguishes:
   - `text-edit-shares-line` present ⇒ the multi-run shape, disclosure owed;
   - a `Run` anchor with no `shares` line ⇒ genuinely a single-run line;
   - `text-edit-became-add` present ⇒ **there was no run at all**, and the
     check must say so and fail the aim, because a check about EDITING text
     that lands on blank paper has not measured its subject.
   ★ Never write the flattering branch of a two-way test as a positive claim.

3. **Enumerate `add-text` in the completion oracle at `:575`** — or, better,
   make aim-failure fatal at step 2 so the Add path is never reached by this
   check at all. Both: the oracle should still name it, because a check that
   cannot recognise a correct outcome will eventually meet one.

4. **Rewrite the failure sentence.** *"the shell built no plan"* is the exact
   opposite of what happened and would have sent the next reader into
   `textedit/commit.rs` for an afternoon.

### Re-measured 2026-09-13, chunk 161 of the second full sweep — every clause holds

It failed again, identically, and the trace is unambiguous
(`target/scratch/uv-full/text_edit_real.trace.txt`):

```text
:616  pdfcer-diag text-edit-became-add reason=no-run-under-the-click
:617  pdfcer-diag text-edit-caret kind=Edit page=0 origin=1998.1,320.9 len=0
:760  pdfcer-diag add-text page=0 n=1 epoch=1 disclosures=new run uses a bundled
      Standard-14 face 'Helvetica' by name+code — no glyph embedding
```

So, in order: the click landed on **blank paper** (`no-run-under-the-click`),
the shell said so in the trace, it converted the gesture to an Add with a
zero-length caret (`len=0`), it raised `add-text`, **and it disclosed the font
substitution off-canvas exactly as R8b rule 4 requires.** That is four correct
behaviours in a row.

★★★ **The check reported it as "the shell built no plan".** It greps for
`edit-text`, `text-edit-commit` and `edit-text-refused`, and `add-text` is in
none of those three — so a complete, disclosed, correct outcome reads to the
oracle as silence. This is the exact shape of
*an assertion both outcomes satisfy*, inverted: an oracle that cannot
**recognise** a correct outcome is as broken as one that cannot detect a wrong
one, and it is worse in practice because it manufactures a defect report.

And the fixture is still the inherited `fixtures/a1-titleblock.pdf`, aimed at
(2000, 320) — a point the check's own log line admits is *"on a single-run
line, so no multi-piece disclosure was owed"*, which is to say it cannot reach
the refusal the check exists for. Repair (1) is therefore load-bearing: without
`SW41177.pdf` this check has no subject.

## R7 — `MAX_MAX_ZOOM_PERCENT`'s doc comment describes a world that ended 21 days ago

**Moved.** This row is now `DOC_DRIFT.md` **S0**, because it turned out to be
the first instance of a family rather than a one-off, and the family needed a
register of its own. `DOC_DRIFT.md` **S9** is the second instance of the same
shape in `viewer/ceiling.rs`, and the two should be repaired in one commit
together with the gate that would have caught both.

**File, for convenience:** `crates/pdfcer-gui/src/app/prefs/mod.rs:168-190`.

---

## R8 — two zoom checks fail on a panic that IS the delivered fix, and the existing escape hatch asserts nothing

**Measured 2026-09-13 (chunk 101 of the 220-check sweep).** Three FAILs; all
three are harness defects; **zero app defects.** Two of them are this one.

### The failures

- `zooming_does_not_throw_away_where_the_operator_panned` — all eight stages
  passed, worst per-notch drift 29 % of tolerance, then FAIL on
  *"A THREAD OF THE APPLICATION PANICKED"*.
- `the_page_still_renders_at_every_decade_of_zoom` — six rungs drawn,
  `drawn=1` at every one, then the same FAIL.

Both panics are tiny-skia 0.11.4 `pipeline/mod.rs:188`,
`range start index … out of range for slice of length 1088737`.

### The trace says it is the fix working, in four consecutive lines

From `target/scratch/uv-full/zoom-gallery.trace.txt` around line 9607:

```text
render-spawn gen=37 page=0 scale=509704.38
thread '<unnamed>' (46212) panicked at tiny-skia-0.11.4/src/pipeline/mod.rs:188:9
raster-limit scale=509704.4 region=1 panic=range start index 356279522944 …
render-inline gen=37 ms=1 async=0
raster-ceiling-learned page=0 scale=382278.3 zoom=509704.38 to=382278.28 moved=true
ui-rect name=status-group:raster-stop rect=[[110.4 768.0] - [547.4 792.0]]
canvas … zoom=382278.2812 … drawn=1   /   canvas-coverage covered=1.000 sharp=1.000
```

The worker panicked, `pdfcer-render` caught it, the refusal arrived, the canvas
learned a ceiling and backed off x0.75, the operator got a sentence in
`status-group:raster-stop`, and the page kept drawing. That is
`reply_G002_deep_zoom_refuses_instead_of_panicking_SHIPPED.md` (`69d4d67`)
behaving exactly as delivered, and the reply says so in terms:

> The panic hook is deliberately NOT silenced. … Your thread-panic guard will
> therefore keep seeing the text if this ever fires — which is what you want.

`raster_wall.rs:1279` already declares `session.expect_thread_panic()` for this
same wall, with a long comment making this same argument. These two checks climb
into the same wall and do not declare it.

### Also measured: the engine's published floor is NOT falsified

I nearly filed that it was. The panic scale is **509,704x**, above
`MAX_GUARANTEED_REGION_SCALE = 250_000.0`. The trap was the units: the
zoom-gallery's last *reported rung* is 3,099,514 **%** (= 30,995x), which looks
an order of magnitude below the floor — but the climb continues past its last
reported rung before it panics, and the panic is at 50,970,438 %. **A rung list
records what the check finished, not where it got to.**

### The repair — and the part that is NOT just adding the call

Adding `session.expect_thread_panic()` to both checks makes them green, and
leaves three checks permanently blind to a *real* panic in the most
arithmetic-heavy code in the shell. That is an assertion both outcomes satisfy.

So: make the declaration **conditional on the conversion**. A panic seen while
the expectation is declared must be followed by a `raster-limit` line — the
refusal's own trace slot — or the check still fails. Shape:

```text
session.expect_thread_panic_converted_to("pdfcer-diag raster-limit")
```

- Do **not** match on the `panic=` text. The engine asked explicitly:
  *"it is third-party text and explicitly not a contract, so please do not
  match on it."* Match the presence and ordering of `raster-limit`.
- Strengthen `raster_wall.rs:1279` the same way in the same edit. Its current
  bare declaration has the same hole; it is only invisible there because that
  check's subject is the refusal, so its other assertions happen to cover it.
- Keep it per-session and per-call-site. Never a profile flag. The existing
  doc comment on `expect_thread_panic` already argues this and should be
  amended rather than replaced — add the conversion requirement and the second
  and third callers to its "if a second caller ever appears, read it
  sceptically" clause, with this trace as the reason each one passed scrutiny.

## R9 — `the_wheel_turns_pages_when_the_operator_asks_it_to` drives a ONE-PAGE document

**The third FAIL, and it is the inherited-fixture shape again.** The check's own
failure message is confident and wrong:

> ★★★ THE WHEEL DID NOT TURN A PAGE. … The most likely cause is the one this
> check was written for: `OpenDoc::prefs` is a SNAPSHOT adopted when the
> Settings window is applied…

The trace (`target/scratch/uv-full/wheel-flips-pages.trace.txt`) refutes every
clause of that:

```text
start argv1=Some("fixtures/a1-titleblock.pdf")
open ok pages=1 path="fixtures/a1-titleblock.pdf"
status page=0 pages=1 zoom=19.04 fit=Page wheel=scroll     (line 170)
ui-rect name=status-wheel-paging rect=[[914.2 768.0] - [978.3 792.0]]
status page=0 pages=1 zoom=19.04 fit=Page wheel=flip       (line 692)
```

**`pages=1`.** There is no page 2 to turn to. And `wheel=flip` reached the status
line on the very next status after the press, so the live push
(`app/frame.rs:863-875`, which copies `prefs.wheel_paging` into every open
document the moment it changes) is working and the snapshot defect the check
names is already fixed.

⇒ The check measures nothing, and has been reporting a fabricated mechanism in
every sweep it has appeared in.

**Repair:** pin a multi-page document. `fixtures/four-pages.pdf` — *this
repository's* copy, via `open_local_fixture`, four pages — not the engine's
same-named four identical Letter sheets. Give it its own `const FIXTURE` and
the `--pdf and --doc-point are IGNORED` note, same as the eight checks that
already do this, so a future shared-fixture change cannot reach it. Then the
check can finally assert the thing in its name: page index 0 before the wheel,
1 after, with the toggle on, and unchanged with the toggle off (it already has
that control and the control passed).

**Why it presented as an app defect rather than a SKIP:** the inherited fixture
*opened fine*. A wrong-fixture SKIP announces itself; a wrong-fixture FAIL
writes a diagnosis. See the standing lesson that a harness with a bad input
produces defects that do not exist — this is the fourth instance, and the first
where the fabricated diagnosis named a mechanism that had already been repaired.

---

## R10 — `pages_stay_drawn_when_you_scroll_back` skips on a reason its own trace falsifies BOTH halves of

**Measured 2026-09-13, in the full 220-check sweep, as the only new SKIP in it.**

The check is the regression test for the operator's 2026-08-19 report that
pages were constantly redrawing. It ran ALONE, with the eight-page document
`fixtures/synthetic-image-only-8pages.pdf` that the ALONE table was written to
give it — see R5 — and it skipped anyway, saying:

> no `strip-raster-requested` line after scrolling 40 notches. Either this
> document has too few pages for a strip, or the mode did not switch to a
> continuous display.

**Both named causes are false, and the check's own trace says so.** From
`target/scratch/uv-full-pages_stay_drawn_when_you_scroll_back/page_cache.trace.txt`:

```text
:15   pdfcer-diag page-display mode=continuous source=mode-default ribbon-mode=read
:231  pdfcer-diag canvas ... page=0 pages=8 off=[470.0 570.3] display=continuous visible=1 drawn=1
:236  pdfcer-diag strip-beyond-raster pages=0
:438  pdfcer-diag canvas ... page=3 pages=8 off=[470.0 2126.0] display=continuous visible=2 drawn=2
```

Eight pages. Continuous. Two pages visible and **both already rastered**.
`strip-beyond-raster pages=0` traced once and never again, so no visible page
was ever above the rasterizer's ceiling either. The document and the mode were
exactly what the check needed.

### What actually happened, and why the obvious repair is wrong

The obvious repair is to scroll further, or to pick a different document. Both
are wrong, because the subject is not a distance.

`render/settle.rs` emits `strip-raster-requested` at **one** place: the scan
that finds a visible page which is **not the current page** and has **no
raster**. Read the offsets in the trace — 2021.2, 2097.8, 2110.1, 2126.0,
2138.1, 2146.3, 2153.1, 2157.3, 2160.7, 2163.3, 2165.2, 2166.6, 2167.6,
2168.3 — that is a **smooth-scroll glide decaying to a stop**, not forty
discrete jumps. The view travelled about 1,600 points continuously, and at
1.4201 zoom a 396 pt sheet is 562 px, so it passed **through** pages 1, 2 and 3
in turn. Each of them became the **current** page on the way, and the
current-page path rastered it. By the time the glide stopped, every visible
page already had a raster, so the strip had nothing to order and correctly said
nothing.

★★★ **The check's gesture fills the cache by the very act of travelling
through it.** A slow continuous scroll can never make the strip order a
neighbour, because there is never a neighbour the operator has not already
been looking at. Scrolling *further* makes this more true, not less.

★ And the same fact disarms the check's **second** half. Part 4 asserts that
scrolling back does not re-request what was already drawn — but nothing was
evicted (`strip-raster-evicted` never traced either), so even if part 3 had
found a request, part 4 would have been asserting against a cache that was
never under pressure. **Two assertions, one unreachable precondition.**

### The repair

Reach the subject by a **discontinuity**, not by a longer glide:

1. **Jump**, do not scroll — set the page number, or press Ctrl+End, so a
   page the operator has never been on becomes visible as a *neighbour* of the
   new current page. That is the only state in which the strip path runs.
2. **Then** scroll away far enough to evict, and back, for part 4 — and assert
   on `strip-raster-evicted` explicitly rather than inferring eviction from the
   absence of a request, which is the mistake that made this SKIP readable as
   an answered question.
3. **Split the `||` in the skip message.** *"Either too few pages, or not
   continuous"* is an alternation between two conditions the check can each
   measure from the trace it already holds — `pages=` and `display=` are on
   every `canvas` line. A skip reason that names two causes and checks neither
   is how a false sentence survives a sweep: it sounded like a diagnosis.

### How to falsify the repair

Run it against a build with the strip-order path **deleted** (make the `next`
scan in `render/settle.rs` return `None` unconditionally). The repaired check
must go **red**. If it skips, the jump did not create an unrastered neighbour
and the repair has only moved the hole.

★ **Not an application defect.** The strip declined to order a page that was
already drawn, which is the whole purpose of the cache the check exists to
defend. The program behaved correctly and the check could not see it.

---

# Appendix A — the measurements R4 and R6 rest on

Folded in from the sweep's own findings file so this register is self-contained. These are traces and arithmetic, not conclusions: they are here because both rows above assert something counter-intuitive — that a drift the harness reported is the harness's own, and that a check reporting *"the shell built no plan"* was watching a shell that built one — and an assertion like that is worth nothing without the numbers underneath it.

**Do not delete this appendix when R4 and R6 are repaired.** Delete it when the repairs have been **driven green**, because until then it is the only record of what the correct behaviour looked like, and a repair that reintroduces the symptom is otherwise indistinguishable from one that fixes it.

## 3b. What the trace actually shows for the zoom drift — read before touching any code

**O24e is already fixed.** `canvas/geometry.rs:758` `zoom_anchor_offset` returns
**unclamped**, with a three-paragraph comment saying the clamp moved to
`strip_offset` because the pasteboard made the old range wrong. So the check's
first named cause is stale prose, exactly like the wheel check's. Do not go
looking for the clamp it describes; it is gone.

**Where the measurement is taken matters, and this one is on the boundary.**
The check panned so that the page point under the viewport centre is
`(1192.23, 1683.93)` — and the page is `2383.937 x 1683.780`. The anchor is
**0.15 pt past the bottom edge of the page**, i.e. on the pasteboard, on the
clamp's own boundary. By the failing notch it is at y = 1690.24, about 6.5 pt
out.

At the failure, `x` moved 0.0027 pt and `y` moved 0.8355 pt. **The drift is
almost entirely on the axis whose anchor is off the page.**

**The pasteboard is sized in viewports.** `content_extent(strip, v, over)` adds
a viewport of slack per side (O23), and `strip_offset` clamps to
`content_extent - viewport`. A slack expressed in screen units is a *shrinking*
slack in page units as zoom rises: at 4216 % one viewport is about 10.8 pt of
page; at 5150 % about 8.8 pt. So the reachable region beyond the page edge
changes with every notch, and an anchor sitting in it cannot be held by a solve
that does not know the extent moved underneath it.

⇒ Two candidate readings, and the next session must measure rather than pick:

- **App**: the anchor solve is correct in page space but the offset it returns
  is then clamped against an extent that changed with the zoom, so the held
  point slides whenever the anchor is outside the page. Fix belongs in the
  relationship between `zoom_anchor_offset` and `strip_offset`, not in either
  alone.
- **Check**: anchoring 0.15 pt past the page edge is choosing the one place the
  clamp lives. A probe placed well inside the sheet would answer a different
  question — and both questions are worth asking, so the likely outcome is
  **two** rungs, not a moved one.

★ The trace also shows the scroll offset is quantised: every `at=` x is a whole
number and every y carries the same fraction. That is roughly 0.024 pt at this
zoom — an order of magnitude too small to be this drift, so it is **not** the
cause. Recorded so it is not chased.

---

## 3c. SETTLED: the deep-zoom drift is a HARNESS defect. The probe is parked outside the page, at the pan clamp.

★ **This supersedes §3b.** §3b's pasteboard reading was directionally right
and mechanically wrong, and its "measure rather than pick" instruction has now
been carried out. Read this section; treat §3b as the working-out.

### What was measured

Two quantities, per frame, over the whole ladder (225 frames, 200 % to 5150 %),
from `uv-full/zoom-keeps-place.trace.txt`:

- `at_y` — the pan, i.e. the viewport's top-left minus the page's origin on
  screen, in screen pixels (`canvas/trace.rs:512-545`; it is **not** the
  viewport centre, which is the trap that cost the first hour);
- `(at_y + V/2) / z` — the page point under the viewport centre, which is
  exactly what the check computes at `checks/zoom_keeps_place.rs:240-241`.

The canvas viewport, read off the trace's own `ui-rect name=canvas-viewport`:
`[[288.0 165.7] - [758.0 744.0]]` ⇒ **470.0 x 578.3 px**, half
**(235.00, 289.15)**.

### The result

| zoom | `at_y` | `page_h * z` | `at_y - display` | held point |
|---|---|---|---|---|
| 4216.5 % | 70979.3 | 70996.2 | **-16.93** | 1690.2360 |
| 4494.0 % | 75669.3 | 75669.1 | **+0.24** | 1690.2194 |
| 4793.0 % | 80703.3 | 80703.7 | **-0.43** | 1689.8037 |
| 5150.0 % | 86715.3 | 86715.0 | **+0.31** | 1689.4005 |

From 4494 % upward `at_y` equals `display` to within half a pixel and stays
there. **The pan is on its clamp.** Below 4494 % it is inside the range and the
held point is flat at 1690.23 ± 0.01 pt across 200 frames — the check's
arithmetic is correct and the application holds the point essentially
perfectly.

The clamp is at `at = display` exactly, which is the pasteboard being one full
viewport per side (O23): the operator may pan until the page's bottom-right
corner reaches the viewport's **top-left** corner, and no further.

### Why the held point then slides, to three decimal places

Once clamped, the viewport's top edge is pinned to the page's bottom edge, so
the centre sits **half a viewport below the page bottom — a fixed distance in
SCREEN pixels**. Measured: 289.4 px at every rung above the clamp, against a
half-viewport of 289.15 px.

A fixed screen distance is a shrinking page distance. So the held point must
move by

```
(V/2) * (1/z0 - 1/z1)
```

Predicted, 4494 % to 5150 %: `289.15 * (1/44.94 - 1/51.50)` = **0.8197 pt**.
Measured: `1690.2194 - 1689.4005` = **0.8189 pt**.

Agreement to 0.0008 pt over a 0.18 pt tolerance. There is nothing else in it.

### Therefore

The check parked its probe **6.4 pt below the bottom edge of the sheet** — the
page is 2383.937 x 1683.780 and the held point is 1690.23. That point is
reachable up to about 4494 % and **unreachable above it**, because reaching it
would require panning past the clamp. The application slides by exactly the
amount by which the request exceeded what it can do, which is the only
geometrically available answer. **There is no application defect at this rung.**

⚠ **This does not close O186.** Ken's report — *"the cursor still jumps at
deep zoom"* — is about a cursor inside the drawing, not half a viewport off the
bottom of it. This measurement removes the only evidence we had, so O186 part 1
is now **unreproduced**, not diagnosed. The next step is a probe placed well
inside the sheet, on the `deep` tier as well as `scroll`, and driven at the
zooms he actually works at.

★★ It is also the second sighting of a lesson already on file: *a slack in
screen units shrinks in the units he cares about — any reachable area sized in
viewports has a zoom above which it reaches nothing, and it presents as the
anchor being broken.* It presented that way again, and it took a fit against
`A + C/z` to see it. Worth appending the recurrence to that RAG entry: **the
tell is that the fitted constant equals half the viewport.**

---

## 4. `text_edit_on_a_real_drawing` — **three harness faults in one check, and the application did everything right**

### What the check reported

> *the caret took keystrokes and NO commit was raised at all … no `edit-text`,
> `text-edit-commit` or `edit-text-refused` line followed — so the shell built
> no plan.*

### What the trace says

`uv-full/text_edit_real.trace.txt`, in order:

```
629  text-edit-became-add reason=no-run-under-the-click
630  text-edit-caret kind=Edit page=0 origin=1998.1,320.9 len=0
721  text-edit-typing … draft=true len=1
737  text-edit-typing … draft=true len=2
771  text-edit-enter means=Commit
772  text-edit-abandon
777  add-text page=0 n=1 epoch=1 disclosures=new run uses a bundled Standard-14 face …
778  page-objects-built page=0 objects=29     (it was 28)
797  page-text page=0 runs=29 chars=126 status=ok
```

**The text went in.** The click found no run, so the shell converted to Add —
which is a deliberate, documented, *operator-requested* behaviour
(`canvas/textedit/place.rs:105-145`, added 2026-08-19 after Ken wrote *"How do
I make new text when I click on the canvas and expect to edit there?"*) — took
two characters, committed on Enter, and published `add-text`. `text-edit-abandon`
is the draft being torn down after the hand-off, not a loss.

### The three faults

1. **Wrong fixture.** It ran on `fixtures/a1-titleblock.pdf` from `ctx.pdf`. The
   check is named *on a real drawing* and belongs on the pinned
   `D:/dev/pdfTests/SW41177/SW41177.pdf` with a SKIP-if-absent guard — the
   Tranche C repair already owed. At (2000, 320) on the title block there is no
   text at all, which is why `resolve_run` returned `NoRun`.

2. ★ **It narrates a fact the trace denies, forty lines earlier.**
   `text_edit_real.rs:403` prints *"this point is on a single-run line, so no
   multi-piece disclosure was owed"* — as the `else` of
   `trace.last("text-edit-shares-line").is_some()`. The absence of that event
   has two causes and the check assumes the flattering one. The real cause was
   `no-run-under-the-click`: **there was no run to share a line with.** This is
   the *unevidenced-excuse* class — a sentence explaining an absence it never
   measured reads as an answered question, so nobody investigates.

3. **Its completion oracle omits the path it logged itself onto.**
   `:575` accepts only `edit-text` or `text-edit-commit`. The shell publishes
   `add-text` for the Add path, and the check had already observed the fallback
   fire. An oracle that does not enumerate every branch of the mechanism under
   it reports the correct outcome as silence.

⇒ No application defect. Three harness repairs — see `sweep_repairs.md` R6.

---

# Appendix B — the SKIP inventory: sixteen checks that asserted nothing, and why

**The finding that outranks every row in it:** a SKIP is not red. A check can stop running — or never start — and the sweep's summary still reads well. Two of these fixtures were **purpose-built to end the SKIP** and then nothing routed the check to them, so the fixture shipped, the SKIP stayed, and the tally looked the same either way.

**Fourteen of the sixteen have one cause:** the shared `--pdf fixtures/a1-titleblock.pdf` that `sweep-full.sh` hands to every check. The repair is the same single change in fourteen places — a per-check `const FIXTURE: &str` plus `workspace_root().join(FIXTURE)`, which about thirty checks already do.

**The other two are worse, because they are not fixture defects.** `print_clip_claim_follows_the_preview` concedes in its own SKIP text that the skip is *the expected outcome on most machines* — so it has almost certainly never run its assertion on any sweep since it was written. And `the_print_window_opens_on_the_settings_you_last_used` skipped saying no click reached the ribbon.

★★★ **CORRECTED 2026-09-13 — the second half of that sentence used to blame "the File-tab activation route" and call it a suite-wide blocker. It was neither.** The check's own control launch deleted `userdata/preferences.txt` to reach the shipped print defaults, and that file is where the sandbox keeps `ask_default_app = false`. Every absent key takes its compiled-in default and that one's default is `true`, so the O173 *"Open PDFs with pdfcer"* offer opened as a real OS window and took the press. `reset_prefs` instead of `remove_file`, and the check **PASSES**: all twelve remembered print settings come back into the dialog and the job is planned with them. ⇒ **An absence reported by a check is first a question about the check**, and the trace said so in one line forty lines above the failure message.

★ **Two entries must NOT be "repaired".** `a_save_that_would_produce_blank_pages_is_refused` is reporting a **fixed engine** and should be deleted; `removing_embedded_fonts_reaches_the_document` is reporting a **correct refusal**. Counting either as a harness defect would inflate the list with work that makes the suite worse.

★ **Deliberately held back from the releases so far, and that is still the right call.** Not one of these repairs changes anything the operator runs; they change what the suite is able to observe. A release whose notes are half harness bookkeeping buries the things he actually asked for.

Zero FAILs in the first 60 checks. Every non-PASS so far is a SKIP, and
**every SKIP so far is the harness being pointed at the wrong document**, not
the application refusing anything.

⇒ The finding that matters more than any individual repair: **seven checks in
a 213-check roster have never once asserted anything**, and for two of them
the fixture was purpose-built on 2026-09-11 *specifically to end the SKIP* and
then nothing routed the check to it. A SKIP is not red; a check can stop
running — or never start — and the sweep's summary still reads well.

| check | why it skipped | repair |
|---|---|---|
| `layers_search_narrows_the_list` | Layers panel drew no rows; the `--pdf` has no optional content | own `fixtures/layered-drawing.pdf` — built 2026-09-11 **for this check**, six OCGs, one off by default, clears `MIN_LAYERS_FOR_SEARCH = 2` |
| `blend_space` | zoom crossed the 183% ceiling and no `raster-blend-space cmyk_buffer=true` ever appeared — the sheet has no transparency to composite | own `fixtures/transparency-cmyk.pdf` — built 2026-09-11 **for this check**; needs `--page-size 596x791` per its own SKIP text |
| `new_document_makes_a_page` | fixture opened with 1 page, and the check's whole oracle is the count *becoming* 1 | own `fixtures/four-pages.pdf` |
| `dragging_a_form_field_moves_it` | field is 27.8 px wide at zoom 0.1737; a quarter of that is grip, so no point presses the BODY | small page, or drive a zoom first. Also a real product finding: a field that small is one **he** cannot drag either |
| `a_resized_check_box_is_redrawn_not_stretched` | 50x35 px box, eight 8 px grips with 2 px slack — a corner grip cannot be aimed without touching its neighbour | same |
| `measure_calibrates_by_picking_two_points` | second point (2400, 320) is outside the crop box (0,0)-(2383.9, 1683.8) — the check adds 400 to the shared `--doc-point 0,2000,320` | derive **both** points from the crop box the app traces, never from `--doc-point` plus a constant |
| `measure_hover_shows_what_it_will_take` | the entity under the pointer is not a straight run, so there is no endpoint for a snap marker | own `fixtures/polyline-nodes.pdf`, aim at a straight segment |

## The mechanism the repair uses

A check may own its document outright — `const FIXTURE: &str` +
`workspace_root().join(FIXTURE)` — and launch with it, ignoring `--pdf`
entirely. `off_page_census`, `annot_delete_gate`, `field_delete_gate`,
`password_prompt` and about thirty others already do. The ALONE table in
`sweep-full.sh` is the *other* mechanism and is the weaker one, because it puts
the knowledge of what a check needs in a shell script a long way from the check.

⇒ **Prefer the check owning its fixture.** The ALONE table should hold only
the cases where the shared fixture is right and the *aim point* is not.

## ★★★ The operational rule this sweep taught

**Do not edit any `.rs` or `.toml` anywhere in the repository while a sweep is
running.** Two independent staleness guards fire:

* `main.rs::refuse_if_self_is_stale` compares the running `ui-verify.exe`
  against the newest `.rs`/`.toml` under `tools/ui-verify/`;
* `launch.rs::staleness_complaint` compares the driven `pdfcer-gui.exe`
  against its own sources.

Either one turns every remaining chunk into an `rc=2` usage dump, and
`sweep-full.sh` aborts the whole run on the first `rc=2`. A full sweep is about
ninety-five minutes of wall clock and the obvious use of it is source work.
That is exactly what it forbids. **Markdown, fixtures and new shell scripts are
safe; `sweep-full.sh` itself is not, because bash reads a running script
incrementally.**

---

## Chunk 61 onward — appended as the sweep ran

**Running tally at chunk 61: 48 PASS, 12 SKIP, 0 FAIL over the first 60
checks.** Against the previous sweep's `162 / 11 / 40` over 213, this is the
first sixty checks with **no failure at all** — the Tranche-A harness repairs
did what they were for.

★★ **Every SKIP message in this sweep names its own repair, with the measured
reason.** That is worth stating because it is the property that makes a SKIP
actionable rather than decorative: not one of them says "could not run".

| check | why it skipped, from its own message | repair |
|---|---|---|
| `page_ops_round_trip` | `a1-titleblock.pdf` **already contains one `/Rotate` entry**, so Phase H's evidence (finding `/Rotate 90` in the saved copy and not in the source) would be indistinguishable from the fixture's own furniture | own fixture with no page rotation — `fixtures/four-pages-unrotated.pdf`. ⚠ **NOT** the engine's `synthetic/pageops/four-pages.pdf`, which the check's own message suggests: that document's four sheets are all US Letter and this repo has a same-named file whose sheets differ. The message should be corrected at the same time. |
| `pages_drag_shows_where_it_lands` | the fixture shows **1 page tile** and the check needs at least 3 — on a shorter document every landing is the dragged block's own lip, where refusing is correct, so the run could not tell a working gesture from a dead one | own `fixtures/four-pages.pdf` |
| `a_save_that_would_produce_blank_pages_is_refused` | the guard walked a real four-level tree and found nothing wrong (`walked=true pages=11 declared=Some(11) levels=4 bad=0`) and a file **was** written | **delete the check**, close `request_delete_pages_leaves_ancestor_count_stale_on_a_nested_page_tree.md`, and delete the engine half of `app::save::tests::deleting_a_page_from_a_nested_document_is_caught_at_the_save`. ★ **The guard itself stays** — one tree walk at save time is all that stands between a regression and a file that opens in Acrobat with blank pages on the end. |
| `signature_trust_is_reported_as_its_own_fact` | **the Signatures panel never opened** — no `dock.tab.view.panel_signatures` region; the declared tabs were `markup.comments`, `view.panel_forms`, `file.document_properties` | the check must activate the panel before reading it. Long-standing, already on the list. |
| `removing_embedded_fonts_reaches_the_document` | `unembed-fonts-opened targets=0 blocked=6 shown=3` — **every** embedded font in this document is one pdfcer judges unsafe to unembed (identity-encoded or Type 3), so the greyed button is correct | own fixture with a font whose verdict is `removable`; `pdfcer list-fonts <file>` names the verdict per font |
| `blend_space` | the zoom crossed the 183 % ceiling and the renderer **never engaged the CMYK buffer**, because a page with no transparency composites nothing | own `fixtures/transparency-cmyk.pdf`, built 2026-09-11 for this check, with `--page-size 596x791` per its own SKIP text |

★ **Two of these are not harness defects at all and must not be "repaired":**
`a_save_that_would_produce_blank_pages_is_refused` is reporting a **fixed
engine** and should be deleted, and `removing_embedded_fonts_reaches_the_document`
is reporting a **correct refusal**. Counting either as a harness defect would
inflate the repair list with work that makes the suite worse.

### Chunks 81–100 — two more, same single cause

| check | why it skipped, from its own message | repair |
|---|---|---|
| `exporting_form_data_writes_a_file` | `export-form-data-declined reason=no-acroform` — the shared `--pdf` is a CAD title block and **carries no AcroForm at all**, so the decline is correct and the check's real subject (the command was registered, drawn and inert behind a stale blocker) was never reached | own fixture with an AcroForm; the forms checks already need one, so one fixture serves three |
| `a_drawing_dropped_on_the_thumbnails_becomes_pages` | `no panel-pages-tile.1 region … Tiles declared: panel-pages-tile.0` — a one-page `--pdf` has exactly one thumbnail, and the check aims **between** two of them to prove the drop point is honoured | own multi-page fixture. ⚠ Note this check already pins its *second* document (`fixtures/four-pages.pdf`, held back 20 s) — **pinning the second PDF and leaving the first shared is the trap**, because the check reads as though its inputs were under its own control |

★★ **Fourteen SKIPs, one cause.** Every one of the fourteen traces to the shared
`--pdf fixtures/a1-titleblock.pdf` that `sweep-full.sh` passes to every check.
The repair is the same single change in fourteen places — a per-check
`const FIXTURE: &str` plus `workspace_root().join(FIXTURE)`, which about thirty
checks in the suite already do. It is preferred over adding rows to the ALONE
table, because the ALONE table puts the knowledge in a shell script far from
the check that depends on it, where the next reader of the check cannot see it.

★ **Held back from this release, deliberately.** Not one of these fourteen
repairs changes anything Ken runs: they change what the suite is able to
observe. The previous release did the same, and the reason is the same — a
release whose notes are half harness bookkeeping buries the six things he asked
for.

### Chunks 101 onward — and the "one cause" claim above is now WRONG

**Tally at the end of chunk 81: 84 PASS, 16 SKIP, 0 FAIL over 100 checks.** The
two new SKIPs are **both print checks and neither is the shared fixture.** That
matters more than the rows do: the previous section's "fourteen SKIPs, one
cause" was true when it was written and became false two chunks later, which is
the ordinary fate of a cause claim made from a partial sweep.

| check | why it skipped, from its own message | cause class | repair |
|---|---|---|---|
| `print_clip_claim_follows_the_preview` | `clipped=Some(0) claim=none:0 overhang=fits` — the sheet fits, so the correction the check exists to verify was never exercised. ★ **The message says so itself: "that is the expected result on most machines"** — the scale mode defaults to Fit, which does not clip | **a starting state, not a fixture.** The check measures a default that cannot produce the condition | drive the scale mode off Fit (Actual size, or a custom percentage above 100) **before** reading the claim, then the overhang is produced rather than hoped for |
| ~~`the_print_window_opens_on_the_settings_you_last_used`~~ | ~~`the click on ribbon.tab.file produced no ribbon-tab-activated tab=file line, so no click reached the ribbon`~~ | ✓ **FIXED 2026-09-13. Not the ribbon — the check's own control launch deleted the preferences file, which is where the O173 suppression lives, so a real OS window took its click.** `sandbox::reset_prefs` instead of `std::fs::remove_file`; the check PASSES on all twelve settings | — |

★★★ **The row above is struck rather than deleted, against this document's own rule 1, and the reason is the cause column.** A row whose *measurement* was right gets deleted when its repair lands; this row's measurement was right and its **cause claim was wrong**, and it had already propagated into `RESUME.md`, `HANDOFF.md` and rule 3 of this file. Deleting it would have removed the only place the wrong attribution could be found and corrected, while leaving three documents repeating it. ⇒ **Correct a wrong cause in place; delete only a row whose cause was right.**

★★ **What the measurement actually was**, recorded here because the next person to meet a *"no click reached the ribbon"* SKIP will start where this started. `ribbon/tabs.rs` emits `ribbon-tab-activated` for **every** tab, unconditionally, on `response.clicked()` — there is no File-tab-specific path to be broken. The control launch's trace carried `dialog-owned title="Open PDFs with pdfcer" owned=true` and `dialog-focus — focused=Some(true)` forty lines ahead of the click. ⚠ The diagnosis had in fact been recorded a day earlier, in this session's own memory under *a fix that names its victims can still miss one* — and **three project documents went on naming the ribbon**, because the register is what a cold session reads first and nothing reconciles it against memory.

★★ **`print_clip_claim_follows_the_preview` is the more serious of the two, and
it is not a fixture defect — it is a check that cannot fail.** Its own SKIP text
concedes that the skip is *the expected outcome on most machines*, which means
the check has almost certainly never run its assertion, on any sweep, since it
was written. A check whose precondition is "the machine happens to be set up
differently from the default" is documentation. The repair is for the check to
**establish** the clipping condition itself rather than to wait for it.

⇒ Carry forward: **triage a SKIP by asking what would have to be true for the
assertion to run, and whether the check is in a position to make it true.** If
the answer is "the operator would have to have changed a setting", the check is
inert and the SKIP has been hiding that fact behind a sentence that reads like
diligence.
