# `app::status::notes` — the narrator, demoted behind a disclosure triangle


> The left half carries four things, and only the first is the narrator.
> The others look similar and are governed by different rules.

- Everything left in the parent answers *"how is the bar laid out, and what
  does each group show?"* — a fixed row, two rule-4 disclosure lines, a
  worded decline, and two clusters of stateless controls.
- Everything here answers *"what did the renderer have to compromise on,
  and how is that said in one line?"* — which is a different question with
  its own widget state (an open/closed flag in `egui::Memory`), its own
  pure decision function ([`notes_line`]), and its own editorial rule about
  which of the renderer's counters an operator can act on.

## ★ Why this line is *narration* and the three below it are not

`DEFECTS.md`'s "Not defects" table records the old shell opening with a
substitute-glyph census:

> The first thing a user reads is the app talking about itself. Excellent
> information, wrong prominence — put it behind the disclosure triangle
> that is already there.

So the report is complete, still here, and **closed by default**. The words
"Render notes" stay visible so it is discoverable; only the report itself
is one click away.

That prominence argument is exactly what does **not** apply to the lines
beside it. A rule-4 disclosure and a worded decline are facts about the
operator's own document and their own gesture, and a disclosure the
operator has to *open something* to find is a disclosure that did not
happen — the opposite failure. Hence: this one is demoted, those are not.

## ★ Opening it does not make the bar taller (R128)

The parent's header carries the measurement — a content-driven status
panel takes space from the central panel, an active `FitMode` recomputes
its zoom from the canvas viewport every frame, and the result on pdfcer was
a page that shrank 230 % → 224 % → 215 % across three frames with no zoom
input. This module is the half of that defence that lives in the widget:
the line is drawn **beside** the triangle, inside the parent's single
allocated row, elided at [`super::NOTES_WIDTH_FRACTION`] of the bar with
the whole text on hover.

It is also why there is no [`egui::CollapsingHeader`] anywhere in this
file. Changing its own height is that widget's entire behaviour, which is
the one thing this surface may not do.
[`super::tests::the_bar_is_exactly_as_tall_open_as_closed`] pins it, from
the parent, where the whole bar can be measured at once.
