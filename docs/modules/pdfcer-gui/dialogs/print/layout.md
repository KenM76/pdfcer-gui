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

## Item notes

### `fn splitter`

# Why a real splitter and not a `ui.separator()`

Operator request, 2026-09-03: *"the preview should be adjustable
size."* The preview column was a hard-coded 340 pt, so widening the
dialog widened the empty space and left the sheet postage-stamp sized —
which is the wrong way round, because the preview is the reason the
dialog exists.

# The affordance is a CURSOR, and nothing is drawn on the preview

Rule 4's pre-commit clause: a resize cursor over the divider and a
hover-lift on the divider itself are the *pointer*, which is welcome. No
grip dots on the sheet, no outline round the preview, nothing that
changes what a screenshot of the previewed page looks like.

`drag_delta()` rather than the pointer's absolute position, because
the two differ by wherever inside the divider the press landed —
absolute tracking makes the divider jump to centre itself under the
cursor on the first pixel of movement.

### `fn the_scrollbar_allowance_exceeds_the_scrollbar`

The body reserves [`SCROLLBAR_ALLOWANCE_PTS`] out of both its width and
its height before laying anything out. If that reservation were merely
*equal* to the bar's drawn width the content would land exactly on the
viewport, and egui reserves slightly more than the drawn width for a
solid bar — measured at 14 pt for a 10 pt bar — so equality is not even
the boundary, it is already over it.

This is the one thing in that arithmetic a constant CAN pin, and it is
pinned here so that a later change tuning the bar's width cannot quietly
make the allowance too small. It does not, and cannot, prove no bar
appears; see the retired test above for why.

### `fn the_default_preview_width_is_within_its_own_floor`

[`PREVIEW_DEFAULT_WIDTH_PTS`] is what the dialog opens at and what a
double-click on the splitter restores to. A default below the floor
would be silently clamped, so the restore gesture would not restore what
the operator saw on opening — the two would differ by however far out of
range the constant had drifted, and nothing would say so.

### `fn popping_the_preview_out_collapses_its_column_and_gives_the_room_away`

[`MIN_CONTENT_WIDTH_PTS`] is what the body refuses to lay out below,
and it must equal the two column floors plus the splitter — not
approximate them. A floor smaller than its parts would let a column be
squeezed under its own minimum with no scrollbar offered, which is the
*"content clipped and unreachable, no bar anywhere"* half of the
operator's report — the half a single screenshot at one size would have
missed entirely.
**With the preview popped out the column is GONE, not hidden** — O112
ask 2, and this is the assertion the whole of R9 rests on here.

Three separate claims, and all three have to hold or the operator gets
something this project's no-placeholders rule forbids:

1. `preview == 0.0` — no width is reserved for it;
2. `splitter == 0.0` — no divider is drawn beside a column that is not
   there, which would be a control that moves nothing;
3. `options == content` — **the room is taken**, which is the half a
   "hide the column" implementation gets wrong. A build that zeroed the
   preview and left the options at their old width would leave a 340 pt
   hole in the dialog, which is a placeholder made of nothing at all and
   is exactly as bad as a greyed rectangle.

Claim 3 is the one worth the test. Claims 1 and 2 are what anybody
would write; claim 3 is what makes the difference between *collapsing*
the column and merely *emptying* it, and it is invisible in a screenshot
of a wide dialog where the extra room is not obviously anybody's.

### `fn with_the_preview_in_place_both_columns_are_laid_out`

Asserted beside the one above rather than left implicit, because an
absence test alone passes on a build that has lost the preview
altogether. `ui-verify`'s driven check makes the same pairing through
the OS: the column's region is declared before the click and retired
after it.

### `fn a_narrow_dialog_with_the_preview_popped_out_still_fits_its_options`

Carrying [`MIN_CONTENT_WIDTH_PTS`] — both column floors plus the
splitter — into the one-column case would refuse to lay the body out
below 628 pt of content, so a dialog dragged to 560 pt would be told its
content is 628 pt wide and would raise a horizontal scrollbar. That is
the operator's original complaint, re-entering through the new feature.
