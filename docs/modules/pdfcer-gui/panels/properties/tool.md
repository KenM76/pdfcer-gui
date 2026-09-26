# `panels::properties::tool` — the armed tool's own settings, where a
property belongs

`OPERATOR_REQUESTS.md` **O123**, and this module is the whole of his
argument made real:

> *"I never understood why there is a tool dock when everything can be in
> object and properties."*

## Why the armed tool's options live in Properties

A one-line tool strip cannot hold them — a 28 pt row has no room for a font
picker, a size, a swatch and a disclosure note. But that is an argument
about a strip, not about where these controls belong. They are not the
tool's; they are **properties of what is about to be drawn**, which is the
same category of thing as the properties of what is already drawn, and this
panel is the surface that owns that category.

| control | block |
|---|---|
| text pen **font** picker | [`text_pen`] |
| text pen **size** | [`text_pen`] |
| text pen **colour** swatch | [`text_pen`] |
| the pen's disclosure note | [`text_pen`] |
| the circular measure's **pick list**, one removable row per point | [`measure_points`] |
| *Scale line weight* | [`scale_switches`] |
| *Keep the inner margins* | [`scale_switches`] |
| *Allow the artwork to distort* | [`scale_switches`] |
| the switches' note | [`scale_switches`] |

Every string is [`crate::text::tool`]'s; every store is the canvas's
(`canvas::textedit::pen`, `canvas::measure`, `canvas::scaling`). This module
draws them and owns none of them.

## Where it sits in the panel, and why it is TWO places

[`Slot`] decides, per block, and the two answers have different reasons.

**[`Slot::AboveTheSelection`]** — the text pen and the circular measure's
pick list — for the two reasons this section has always given:

1. **It is where the operator's eye already goes** — the top-right corner
   of the window, which is where the armed tool's settings have always been
   found.
2. **An armed tool is the more immediate subject.** When somebody has armed
   the text pen, the question they are about to ask is *what size?*, not
   *what is that path's line width?*

**[`Slot::BelowTheSelection`]** — the three resize switches — because
reason 2 is false for them. They *are* a statement about the next gesture,
but Select is armed nearly always, so placing them above the description of
the CURRENT selection puts a hypothetical ahead of the thing on screen.
`Slot`'s own doc carries the measurement. `OPERATOR_REQUESTS.md` O198.

## It is deliberately NOT part of `something_drew`

`OPERATOR_REQUESTS.md` **O75** collapses the *This document* section
whenever a selection-scoped section has spoken. This section is **not**
selection-scoped: [`scale_switches`] draws whenever the Select tool is armed,
which is most of the time and has nothing to do with what is selected.
Folding it into that predicate would collapse the document section for ever
and suppress *"nothing is selected"* for ever, which is O75 answered
backwards.

## The reachability rule this module inherits, and why it is written twice

`panels::tool`'s own header recorded it: the three scale switches were first
written into a branch `CanvasTool::Select` **cannot reach**, and *"an option
row added there is dead code that compiles, reads correctly, and draws
nothing … Every unit test in the chain passed. Nothing tested that the
control is on screen."* The check that caught it drove the real binary.

⇒ So every region here publishes through [`crate::diag::ui_rect_visible`]
rather than `ui_rect`, and one per **switch** rather than one per block. A
rect proves layout; only a rect measured against the clip in force proves
the operator could reach it — and this panel is a `ScrollArea`, so a control
scrolled past the fold has a perfectly healthy rectangle.
