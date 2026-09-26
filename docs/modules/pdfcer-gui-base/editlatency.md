# `editlatency` — **which half of an edit is the one you can feel?**

An instrument, not a feature. It exists to answer one question, and the
whole design of `OPERATOR_REQUESTS.md` **O63** turns on the answer.

## The question

**Ken:** *"we need to make it so we have a live preview as we
drag and move and resize and rotate … the live preview should remain while
the update to the pdf structure runs in the background"*, clarified minutes
later to *"live preview request is for everything we do."*

He is describing a delay he can feel between acting and seeing. An edit that
reaches the screen is two things in series:

| half | what it is | crate |
|---|---|---|
| **(a) the commit** | `EditSession`'s verb — locate, plan, rewrite the content stream, stage the objects | `pdfcer-core` |
| **(b) the raster** | redrawing the page at the current zoom because the edit epoch moved | `pdfcer-render` |

**They need completely different fixes.** If (a) dominates, his description
is right as written and the shell needs an optimistic edit model with a queue
and backpressure — the screen showing a document state the engine has not
reached yet, with everything that implies about a refusal arriving for an
edit already shown. If (b) dominates, the fix is far smaller and carries no
such risk: keep displaying the last good frame while the next renders behind
it. The document is never ahead of the screen, because the edit really did
happen before the frame was asked for.

## Why this is a `#[test]` and not a reasoned paragraph

`BENCHMARK.md` is this project's standing answer to a performance question,
and it is written from measurement because a weakness reasoned **from
architecture** did not survive one: pdfcer's whole-page raster was declared
to need a tile cache, and the operator's contrary report ("it feels faster
and more pleasant than the tiled competitor") was the correct one.

The same trap is open here, and the prior is strong enough to be dangerous: a
129,758-object CAD sheet takes roughly a second to rasterise at scale 1, so
*"it is obviously the raster"* is the comfortable answer. Comfortable answers
reasoned from architecture are exactly what this file exists to refuse.

## Running it

```text
cargo test -p pdfcer-gui --release edit_latency -- --ignored --nocapture
```

`#[ignore]` because it wants a multi-megabyte drawing that is not in either
fixture corpus, and `--release` because a debug-build measurement of a
release-build question is not a measurement of anything. It **skips with a
printed reason** rather than failing when the drawing is absent: a machine
without it has not found a defect.

## What it deliberately does NOT measure

The raster. `pdfcer-render`'s cost is already measured, already traced by the
running program (`render-inline ms=`, `render-async-done ms=`) and already
written up in `BENCHMARK.md`. Measuring it again here would produce a second
number for one fact, and two numbers for one fact drift. This file measures
**only the commit**, because the commit is the half nobody has a number for.

## Item notes

### `const DRAWING`

Not in `fixtures/` and not in the engine's corpus — it is 5.6 MB of the
operator's real work, and it lives outside both repositories. Named by
absolute path here rather than copied in, because a benchmark corpus that
grows by copying is a repository that grows without bound.

The location is a `const` here rather than a sentence in a document for a
reason worth keeping: a path in prose goes stale silently, while a path in a
test that skips with its own printed reason tells the next reader exactly
where it looked and what it did not find.

### `fn median_ms`

The **median**, not the mean and not the best. The mean is dragged by a
single scheduler hiccup on a machine that is also running an editor and a
browser; the best-of is a number the operator will never experience. The
median is what a press feels like.

### `fn edit_latency_the_commit_half`

Prints a table. Asserts nothing about the numbers, and that is deliberate:
a threshold asserted here would be a number nobody chose, on hardware nobody
specified, and the first slow machine would turn a measurement into a red
suite. What it asserts is that the measurement **happened** — a run that
silently measured nothing is the failure this whole harness exists to remove.
