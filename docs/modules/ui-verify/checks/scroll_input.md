# `ui-verify/checks/scroll_input`

`scrolling_far_keeps_the_canvas_its_pointer_input` — the experiment that
decides whether `O23` is a feature problem or a defect the operator already
meets.

# Why this exists


```text
.scroll_offset(vec2(100, 100))   → canvas keeps its pointer input
.scroll_offset(vec2(484, 492))   → canvas receives NOTHING
```

No pasteboard arithmetic is involved in either. The page's rect settles at
one stable value wholly inside the viewport, a click computed from it lands
comfortably inside both, and no `canvas-pointer` event is ever emitted.

That leaves one question, and everything else waits on it:

> **Does a scroll offset the OPERATOR reaches — with the wheel — break input
> the same way?**

| if | then |
|---|---|
| input dies | this is a **pre-existing defect in today's shell**, met whenever he scrolls a long way down a drawing. It outranks O23 entirely, and O23 has been getting the blame |
| input survives | the difference is that the offset was **forced on the frame the content was first laid out**, and the fix is to force it one frame later |

★★★ **ANSWERED 2026-09-05: input SURVIVES.** Driven on `four-pages.pdf`, at
a wheel-reached offset of **1,182 pt** with a page still under the pointer,
the canvas answers every movement. Driven on `a1-titleblock.pdf` at 832 pt,
the same. So the second row is the true one, the pasteboard is cleared, and
what remains of O23 is the forced-offset-on-the-first-layout-frame question
— a smaller and much more specific thing than *"the canvas loses the
mouse"*. ⚠ It took a repair to this check to establish that; see the section
at the foot of this header before quoting the answer.

# ★★ Why the assertion is `canvas-pointer` events and not a selection

Because the symptom is the *absence of input*, not a bad hit test. Asserting
a selection would need an object under a point that survives an arbitrary
scroll — a fact about the fixture — and would fail for reasons that have
nothing to do with the question. `canvas-pointer` is emitted whenever the
pointer is over the page, needs no object, and is exactly the line that went
to zero.

# ★ The control comes first

It moves the pointer and counts events **before** scrolling. Without that,
"no events after the scroll" is indistinguishable from "this build never
emits them", "the window was not focused", and "the pointer never reached
the canvas" — three things that look identical in a trace and none of which
is the defect.

Read mode, deliberately: it is the default, it defaults to continuous
scrolling, and the question has nothing to do with editing. One fewer click
is one fewer thing that can go wrong before the measurement.


The first full driven sweep filed it as application defect **A3**: *"the
canvas stops seeing the pointer after a long scroll"*, one pointer event
before the wheel and one after it, reproducing on two fixtures. The failure
message it printed contained this sentence:

> *The page is still drawn and its rect is still published — only the input
> is gone.*

**The trace of that very run says the opposite, three lines from the end:**

```text
canvas rect=[[296.0 296.2] - [764.0 626.8]] zoom=0.1963 page=0 pages=1 off=[484.0 1102.3] display=single
canvas-unavailable reason=nothing-visible
ui-rect-gone name=canvas-viewport
ui-rect-gone name=page
```

Forty notches of wheel on a **one-page** document in `display=single` had
scrolled the sheet clean off the top of the viewport. There was no page
under the pointer, so of course no `canvas-pointer` line followed: that line
is emitted when the pointer is **over a page**, which the module header
above says in its own words and which the assertion then ignored.

⇒ **The check asserted a precondition it never measured.** Its message
stated the page was still drawn; nothing in the run had asked. That is the
ordinary shape of a false red — an absence assertion that holds because the
run left the state it was supposed to be measuring in — and it cost a
filed defect and a row in the sweep report.

## What it does now

The wheel is turned in **steps**, and after each one the trace is asked
whether a page is still on screen. The measurement is taken at the largest
offset the document actually has, with a page still under the pointer:

| after the wheel | verdict |
|---|---|
| a page is still drawn, the offset grew by at least [`OFFSET_GAIN`] | measure — this is the subject |
| a page is still drawn, the offset barely moved | **SKIP** — the fixture is too short to scroll |
| no page is drawn | back off to the last step that had one; if none, **SKIP** |

★ And the pointer is aimed at the **page's own rect after the scroll**,
not at a fixed fraction of the viewport. The viewport does not move when
the document scrolls and the page does, so a fixed aim point drifts off the
sheet as the very thing under test happens — which is the same mistake in a
smaller form.
