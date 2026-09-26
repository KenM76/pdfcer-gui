# `render::worker::key` — **what a render is OF**, as one comparable value


## ★ Why this is the seam, and not "move the tests out"

The obvious way to get a file under the ceiling is to move its `#[cfg(test)]`
modules to a sibling, and it would have worked here — there are 375 lines of
them. It was rejected because it answers the gate without answering the
rule: R2 exists so that a reader can hold a file's subject in their head,
and a file whose tests live elsewhere has exactly the same subject it had
before, only harder to read.

[`super`] has two subjects and they change for different reasons:

* **the worker** — a thread, a channel, a cancellation token, one in-flight
  slot, and the mapping from `pdfcer-render`'s result to an [`super::Outcome`].
  It changes when the rasterisation contract does, which this week meant a
  new refusal variant.
* **the key** — which inputs make two pictures different. It changes when a
  new *control* is added: a layer override, a stroke-display mode, a region.

Nothing in this file mentions a thread, and nothing in [`super`]'s worker
half decides what makes a picture stale. Two subjects, two rates of change,
which is this project's test for a seam.

## What did NOT move

[`RenderKey`]'s tests. They stay in [`super`]'s test module beside the
worker's, because several of them assert the **pairing** — that the key the
worker spawns with is the key the texture is stamped with — and splitting
an assertion from one of its two subjects is how the pairing stops being
tested by either.
