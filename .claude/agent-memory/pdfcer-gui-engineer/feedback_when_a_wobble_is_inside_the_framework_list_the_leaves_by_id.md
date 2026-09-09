---
name: when-a-wobble-is-inside-the-framework-list-the-leaves-by-id
description: Two days and five instruments stalled at "inside egui's Panel::show"; one instrument that listed every REGISTERED widget crossing the edge, by egui id, named the culprit on its first read. A number that decays across frames is an animation.
metadata:
  type: feedback
---

**When every rect you publish is constant and the framework's rect still
moves, stop instrumenting containers and list the leaves.** egui registers
every allocation in `ctx.viewport(|v| v.this_pass.widgets)`; filter that list
by the geometric predicate that defines the defect (here: "crosses the window
edge") and publish each survivor **by id**. The id is a hash, but it is the
same hash every frame, so two frames can be intersected.

**Why:** 2026-09-08/09. The central-panel width jitter (0.1–0.5 pt on isolated
frames) had five instruments aimed at it across two sessions — window rect,
`pixels_per_point`, parent `available_rect`, the panel's allocated frame, the
body's `min_rect` — and each one narrowed the hunt by one container and
stopped at the next: "it is inside egui's `Panel::show`". The RESUME's own
instruction was *"bisect from there … do not theorise"*, and a bisect by hand
would have meant guessing which body widget to name next. The leaf list
returned five rects, four of them containers and one a **10-pt strip on the
pane's edge, decaying `1400.4 → 1400.1 → 1400.0`**. Ten points at that edge
said *scroll bar*; a decay said *animation*; `scroll_area.rs` then took ten
minutes and a 60-line headless repro confirmed it to the digit.

**How to apply:**

- The predicate is the defect's geometry, not a widget name — you do not know
  the name; that is the point.
- Read the *shape* of the survivors before their identity: width, position
  relative to an edge, and **whether the number changes monotonically across
  consecutive frames**. A decay is an animation; a 1/32-grid step is
  rounding; a constant offset is a margin.
- Then reproduce it **headlessly with nothing but the framework** — a
  `Context`, a fixed clock, the same container nesting. If it reproduces, the
  fix is a workaround in your own layer plus a test that goes red when the
  framework fixes it; if it does not, the application is doing something the
  repro is not, and that difference is the next bisect.
- The instrument that found it costs one comparison per frame and publishes
  nothing when nothing is wrong. Keep it as a tripwire; do not delete it
  because the hunt is over.

Related: [[feedback_a_disproof_is_a_measurement_too_and_the_dead_hypothesis_was_the_truth]] —
same hunt, previous day; [[feedback_a_guard_that_stops_repetition_does_not_stop_creep]] —
the floor that made the jitter harmless while it was still unexplained, and
which stays after the cause is gone because it is the right contract anyway.
