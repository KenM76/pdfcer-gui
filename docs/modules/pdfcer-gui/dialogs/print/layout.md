# `pdfcer-gui/dialogs/print/layout`

`dialogs::print::layout` — where the print dialog's two columns go.

# Why this is its own file


# THE ONE RULE THIS FILE EXISTS TO HOLD

> **Every width and height is derived from the space OUTSIDE the scroll area
> and from constants. Nothing is measured from inside it.**

Breaking that rule is what produced the operator's report of 2026-09-03 —
*"I have two scroll bars in the pop up window that won't go away no matter
how"* — and it was broken in three different ways in succession, each of
which read as obviously correct:

1. the content was forced to `ui.available_width()` measured **outside** the
   scroll area, which is one scrollbar wider than the viewport the content
   is actually laid into;
2. `auto_shrink([false, false])` *defines* the content to be at least the
   pre-bar viewport, so the content was again always at least one bar too
   wide — by construction, at every window size;
3. the two `item_spacing` gaps that `horizontal_top` inserts between three
   children were not in the arithmetic, and the preview's control strip was
   laid out **40 pt wider than its own column** and pushed the total out.

Each of those raised a horizontal bar; a horizontal bar consumes height;
that raised a vertical bar; the vertical bar consumed width, which kept the
horizontal one. **The two bars were each other's cause**, which is why no
amount of resizing dismissed them.

And the failure was INVERTED, which walking the size series found and a
single screenshot would not have: bars at 1000x760 and 1300x900 where
nothing needed scrolling, and **no bar at all** at 700x520 where the Paper
section was clipped and unreachable.

# The oracle

`ui-verify`'s `print_dialog_body_does_not_deadlock_its_scrollbars`, which
reads egui's own `content_size` and `inner_rect` out of a running frame via
the `print-body` trace line. It was falsified by planting cause 1 back in.
Nothing in a unit test can produce those numbers, and no screenshot can say
which of the three causes is the live one — the unit tests in `mod.rs` pin
only the relationships that are genuinely between our own constants.
