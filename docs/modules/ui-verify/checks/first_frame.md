# `ui-verify/checks/first_frame`

`the_first_frame_names_the_armed_tool` — **zero clicks**, and the operator
is told what they are holding.

# What this is for

Every other check in this suite drives a gesture. This one drives
**nothing** — it opens a document, enters Edit, and asks what an operator
sees before they have touched anything.

The defect shape it exists to break, which this project has walked
through end to end: a tool is registered, drawn, chord-bound and covered by
passing driven checks, and the operator reports it MISSING. Diagnosing that
as a discoverability defect and shipping a panel that NAMES the tool does
not fix it, because **naming a four-step route is not removing it**. And a
panel that lists tools is itself a list that can go stale — naming the tools
that were there when it was written and not the one added since.

# The question survives its answer changing

`OPERATOR_REQUESTS.md` **O123** dissolved the Tool panel: *"The Tool panel
becomes a one-line tool status (name, one sentence, 'Put this tool down');
its buttons duplicate the ribbon and go."*

⚠ **This check is written against the QUESTION, not against the surface that
answers it.** What is under test is not *"is there a list"* — it is *"does
the first frame, with no clicks, tell the operator something true about what
they can do."* The surface answering it now is `crate::app::toolstatus`: a
strip the right dock reserves above its columns, permanent, uncloseable,
present at frame one. A check retired along with the surface it happened
to name takes the guard with it, and the replacement surface then ships with
none — so a check is rewritten onto the new surface, never deleted with the
old one.

# The two regions, and why BOTH are asserted

| region | published by | the claim |
|---|---|---|
| `dock.right.banner` | `egui_shell::dock::banner`, through the application's `ui_rect_visible` sink | **the strip's compartment is on screen** |
| `toolstatus` | `crate::app::toolstatus::banner`, through `ui_rect_visible` | **the application drew something into it** |

A build whose handler returned early — no document, an unregistered command,
a `None` from the menu host — keeps the first and loses the second. A build
whose dock stopped reserving the strip loses both. Asserting only the dock's
would pass on an empty strip; asserting only the application's would pass on
a strip drawn outside the side it belongs to.

# And a pixel, because a rectangle cannot say that anything was painted

**The banner has a constant height and publishes a rect whether or not
it painted anything**, so a check asserting only the region's presence is
vacuous. The strip's rectangle is therefore sampled with
[`crate::pixels::region_not_uniform`]: a strip that laid out correctly and
painted nothing is a flat block of panel colour, and that is precisely what
`is_uniform` reports.

This is the two-channel discipline `read_mode_chrome.rs` states: *"the rect
is exact and cheap and would be satisfied by a build that moved the canvas
without repainting anything; the pixels cannot be faked by an arithmetic
error."*

# Why "inside the client area" is still a real assertion

Because a control drawn **below the fold** is a shipped failure mode here: a
window tall enough to push its own title bar off the desktop is one the
operator has reported. A strip that draws at y = 1400 in a 900-pixel
window is drawn, publishes a healthy rect, and is invisible — so the rect
must be checked against the CLIENT AREA and not merely for existence.
