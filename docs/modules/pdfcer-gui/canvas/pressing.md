# `canvas::pressing` — **what a press would land on, and what it would mean**

## Why this is its own file

Everything here answers one question — *if the primary button went down at
this point, right now, what would happen?* — and nothing here changes
anything. `canvas::interact`'s remaining sections advance a gesture, route a
click and paint; this one only looks.

## ★★ The precedence, in one place

Four different things can be under the pointer at once, and the order they
are asked in is the whole behaviour. **The most specific thing under the
pointer wins, and specificity is depth down the selection ladder:**

1. a **Bézier handle** of a selected anchor — it sits *inside* the
   selection box, so anything asked before it swallows every press on one;
2. an **anchor**, reached through the inflated move box — an anchor sitting
   on the bounding box's edge is half outside it, so the box is inflated
   rather than the grips suppressed a second time;
3. a **resize grip**, Object rung only — it scales the whole object, which
   is the wrong subject at an inner rung, and left unconfined it covers the
   corner **anchors**, making the end point of every path undraggable while
   the middle ones work;
4. the **selection body** → move — the least specific claim, so it answers
   last.

⚠ Every one of those failures is silent. `grip_at` answering `Move` for a
press on a **handle** moves the whole object instead of shaping the curve,
and that is entirely plausible from a chair, because the object *did* move.

## ★ Everything here reads `press_origin`, not the current pointer

`egui` does not call an interaction a drag until the pointer has travelled a
threshold, so by the frame it says so the pointer is **already that far from
where it went down** — measured at 94 PDF points of error on an A1 sheet at
0.21× zoom. A grip is an 8 pt square and a handle mark is 7 pt across.
Reading the current position misses both, and the miss is silent: the
gesture becomes a marquee, which *clears the selection the operator was
trying to resize*.
