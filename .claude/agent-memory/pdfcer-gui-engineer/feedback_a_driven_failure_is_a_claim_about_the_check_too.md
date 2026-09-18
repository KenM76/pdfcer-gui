---
name: a-driven-failure-is-a-claim-about-the-check-too
description: A [FAIL] or [SKIP] from ui-verify is not yet an application defect. Triage which side is wrong before writing a fix — a stale check's failures are confident and about the wrong subject.
metadata:
  type: feedback
---

**Before treating a driven result as an application defect, establish which side
is wrong.** A stale or blind check fails *confidently* and *about the wrong
subject*, which is the most expensive kind of wrong.

## The evidence, 2026-09-05

The first full run of all 175 driven checks produced **7 harness defects** — each
of which had made a check permanently unrunnable — and a list of six apparent
application defects. When the first of those six was investigated properly,
**all three of its symptoms were the check**:

- **The check was not hermetic.** Its four sections each relaunched the binary,
  and the application persists its dock layout to `userdata/layout.ron`. Section
  B inherited what section A had saved; the failure even crossed **whole runs**,
  so a fresh sweep began in the state a previous sweep had left. The headline
  number `docked=0` was **honest** — nothing was floating, because an earlier
  section had closed it.
- **The oracle was blind.** It asserted that some region carried a `viewport=`
  tag. Nothing published one and nothing could. **It could not have passed
  against a working build.**
- **The fixture could not exercise the subject.** The panel under test draws its
  measurable control only with ≥2 optional-content groups, and **no fixture in
  the repository has any** — all 26 grepped.

★★ Three independent reasons, and any one alone would have produced the same
red line.

## The tells

| tell | what it usually means |
|---|---|
| a check that **relaunches** the binary more than once | it inherits its own previous launch's persisted state — assert a reset, and **assert the reset landed** |
| a number that reads as failure (`0`, `false`, `moved=false`) | ask whether the precondition ever happened. `docked=0` after nothing floated is a correct answer |
| a retry that does not help | **the aim, not the wait.** One check called an 800-px miss "a harness timing question" for days; it was clicking a child-viewport dialog against the main window's coordinates |
| an assertion nothing in the codebase emits | grep for the key. If no site produces it, the check has never passed and never could |
| a check that fails on a subject the fixture lacks | grep the corpus for the feature before believing the verdict |

## How to apply

- **Read the trace, in launch order, before reading the source.** The 2026-09-05
  diagnosis came entirely from four traces read in sequence; the answer was in
  them plainly.
- **Ask what the check SAMPLED** before asking what is broken. That question is
  already recorded here twice for gates; it applies identically to driven checks.
- When you fix a check, **falsify the repair**: plant the defect it now claims to
  detect and require its own `[FAIL]`.
- ⚠ And do not over-correct: the same sweep found **real** application defects
  too — `edit.paste` withheld in Review while `edit.copy` is offered, found by
  two independent checks agreeing. *Triage* means deciding, not disbelieving.

★ One more from the same investigation, small and vicious: `egui-shell` pins egui
with `default-features = false` *"so this crate does not silently acquire
fonts"*, so **every galley in its tests measures 0 × 0** and a test written with
`ui.label("x").rect` fails against a perfectly working dock. Use
`allocate_space` there; keep label-based measurement in `pdfcer-gui`, which has
fonts.

Related: [[feedback_a_measurement_of_the_wrong_surface_looks_exactly_like_a_broken_one]],
[[feedback_a_measurement_of_the_wrong_surface_looks_exactly_like_a_broken_one]] —
this is the same lesson in the driven harness rather than in a gate, and the
2026-09-05 sweep is its largest single instance.

## ★★★ A CHECK'S REPORTED RUNGS ARE WHAT IT FINISHED, NOT WHERE IT GOT TO — 2026-09-13

Two of three FAILs in one sweep chunk were a tiny-skia worker panic at extreme
zoom. The panic is **the delivered fix working**: `pdfcer-render` catches it and
hands back `RasterizerLimit`, the canvas learns a ceiling and backs off, the
operator gets a sentence. The engine's reply said outright that the panic text
would keep appearing and that this is what we want. One check already declares
`session.expect_thread_panic()` for the same wall; these two climb into it and
do not.

★★ **The near-miss is the lesson.** The engine publishes
`MAX_GUARANTEED_REGION_SCALE = 250_000.0` as a measured floor. The check's last
reported rung was **3,099,514 %** — which is 30,995x, an order of magnitude
*below* the floor — so I had a write-up started saying the floor was falsified.
The panic scale in the trace was **509,704x**: eight times *above* the floor.
Two errors stacked: percent read as scale, and **the rung list read as the
extent of the climb when it is only the list of rungs that completed.** The
check was mid-climb toward the next decade when it died, and that rung never
printed.

★ Both errors pointed the same way — toward a finding against someone else's
published constant. Two independent mistakes agreeing is not corroboration; it
is the shape of a conclusion arrived at first.

**How to apply:**

- Before quoting any number out of a check's *report*, find it in the check's
  *trace*. The report is a narrative of completed steps; the trace is what
  happened. They diverge precisely at the failure.
- When a number is about to contradict another project's measurement, convert
  units **out loud**, in writing, with both forms side by side, before drafting
  a word of the request.
- Never let `expect_thread_panic()` stand alone: require the conversion it
  claims — a panic followed by the refusal's own trace line — or you have an
  assertion both outcomes satisfy. See
  [[an-assertion-both-outcomes-satisfy-is-not-a-measurement-of-which-one-shipped]].

## ★★★ AND THE SECTION ABOVE WAS WRITTEN INSTEAD OF THE FIX — 2026-09-15

Read the 2026-09-13 section again. It names *these two checks*, says they climb
into the wall and do not declare it, and explains why that is wrong. Then the
next full sweep, two days later, produced the identical two red lines, and I
spent an hour re-deriving the same conclusion from the same two traces.

⇒ **A lesson filed in memory in place of a fix is a defect with a subscription.**
Memory is for the generalisation. The instance goes in the code, in the same
session, or it comes back at full price. The tell that this has happened: you
are reading a memory entry that describes the failure in front of you in the
present tense.

**And the fix the old entry prescribed was not the one I first drafted.** Its
last bullet says `expect_thread_panic()` must never stand alone — declare the
panic and you have silenced the detector for the whole session, so a worker
that *genuinely died* now passes too. The repair therefore has two halves: the
declaration, plus `panic_was_converted`, which requires one `raster-limit`
line carrying a `panic=` field per panic in the capture. Measured 1:1 across
three traces before the rule was written.

## ★★ A check's failure paragraph is authored BEFORE the run — 2026-09-15

`the_wheel_turns_pages_when_the_operator_asks_it_to` failed with three
confident paragraphs naming `OpenDoc::prefs` as a snapshot that ignores the
preference. **Its own trace disproves that on the same page**: `status page=0
pages=1 ... wheel=flip`. The preference arrived. The document had **one page**.
There was nowhere to go.

⇒ A `defect()` string and a failure paragraph are written when the check is
written. They are the author's *prior*, and they cannot have been informed by
what the run measured. Read the artifact before the accusation — and the first
thing to read is whether the fixture could contain the subject at all.

The repair is the pair, not either half: **pin** the document
(`fixtures/four-pages.pdf`, following `page_display_recentres.rs`) **and
assert** the page count from the status line. Pinning alone is a claim about
what is on disk; asserting alone leaves the sweep with no coverage.

★ The audit this opened is in `target/scratch/sweep-triage-20260915.md`: twenty
checks take `--pdf`, pin nothing, and reason about a page index. The grep
over-reports, because `sweep-full.sh`'s `ALONE` table is a second way to get a
different document and is invisible to a file-level search — `page_cache.rs`
is fed an 8-page scan that way. Two mechanisms, one of them unfindable from the
check's own source: worth knowing before trusting any audit of fixtures.
