---
name: a-slack-in-screen-units-shrinks-in-the-units-he-cares-about
description: Any "reachable area" expressed as a multiple of the viewport has a zoom above which it reaches nothing — and it presents as the anchor being broken when the anchor was right and the clamp ate its answer
metadata:
  type: feedback
---

**A slack measured in screen pixels is a slack that shrinks in the units the
operator cares about.** Any "reachable area", margin, pasteboard or overscroll
expressed as a multiple of the viewport has a magnification above which it
reaches nothing.

**Why:** O23 shipped an off-page pasteboard of `viewport × 1.0`. Ken could see
his off-page object, band it and drag it — and could not zoom in on it, which is
what editing means. Three weeks later: *"how do I view and edit objects that are
off of the page? we added this feature but I didn't see how to enable it."* The
sum: an object `k` points off the sheet is centrable only while
`zoom ≤ viewport / 2k` — about **235 %** on a 470 px canvas with 100 pt of
overhang.

**How to apply:**

- ★★★ **It presents as the wrong bug.** The zoom anchor solved the right offset;
  the offset *clamp* threw it away, because the clamp is
  `content_extent − viewport` and `content_extent` was built from the fixed
  slack. **Suspect the clamp before the solver** whenever something "walks off
  the screen while zooming toward it".
- The corrected form is
  `max(viewport × FRACTION, overhang × zoom + viewport / 2)`. The `+ viewport/2`
  is *the difference between reaching a point and looking at it* — without it he
  can drag to the object but never bring it to the middle of the screen.
- Compute it **once per frame** and publish it onto the document state. Eight
  geometry call sites recomputing it is eight chances to disagree, and a
  disagreement here lands an offset in a content rectangle that does not exist.
- Bound it against whatever constant the precision tier hands over at — a
  zoom-multiplied slack is unbounded, and the hand-over would be late and
  **silent**.
- ★★ **Widen the signature; never default the argument.** Ten `E0061`s made
  every call site state what it passes. See
  [[a-compile-error-is-an-invitation-to-read-the-reply]] — a keep-old-behaviour
  default is how a call site silently declines a feature.
- **A driven check for this must calibrate against the measured viewport**, not
  a hard-coded percentage: `target = 1.5 × viewport / off_pts`. A constant that
  is a real test on a laptop is vacuous on a wide monitor, and it is green
  either way. Related: [[a-check-that-cannot-fail-is-not-evidence]].

Full write-up:
`D:/dev/rag/egui/a_scroll_slack_measured_in_screen_pixels_reaches_less_of_the_document_at_every_zoom.md`.
