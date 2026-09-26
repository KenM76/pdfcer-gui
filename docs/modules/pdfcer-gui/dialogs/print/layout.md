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

### `const OPTIONS_COLUMN_MIN_WIDTH_PTS`

A floor, not a width. It used to be a fixed 400 pt whose stated reason was
that *"a fixed width is what gives the horizontal scrollbar something stable
to measure"* — and that reasoning is what produced the scrollbar deadlock
[`PrintDialog::body`] documents. The scrollbar does not need a stable number
to measure; it needs to be told the truth about how wide the content is.

Sized to hold the longest radio label in the three tabs without wrapping,
which is what makes it a floor worth having: below this the options start
reflowing and the tab strip wraps, and scrolling is the better answer.

### `const PREVIEW_MIN_WIDTH_PTS`

Below this the sheet is too small to judge a margin or a clipped edge by,
which is the only reason to look at a print preview. The operator can still
collapse the dialog itself; what they cannot do is drag the preview into a
sliver by accident and be left with a control that has no visible grab.

### `const SPLITTER_WIDTH_PTS`

Wider than the line it draws, on purpose: the line is 1-2 pt and a 1 pt drag
target is not a drag target. 8 pt is the smallest band that can be hit
reliably without hunting, and it is what the dock's own splitter uses.

### `const SCROLLBAR_ALLOWANCE_PTS`

# This constant is the fix for the operator's "two scroll bars"

Wider than [`SCROLLBAR_WIDTH_PTS`] on purpose. egui reserves slightly more
than the bar's drawn width for a solid bar — measured at **14 pt** for a
10 pt bar, by reading `available_width` inside the scroll area against the
width handed to it. A reservation equal to the drawn width would leave the
content one or two points wider than the viewport, which is a scrollbar just
as surely as a hundred points would be.

16 is that measurement rounded up, and the rounding is the point: this
number's job is to be **comfortably more** than whatever egui takes, so that
the common case has strictly less content than viewport and neither bar is
drawn at all. Being a few points generous costs a few unused points at the
window edge; being a point mean costs a scrollbar that cannot be dismissed.

### `const MIN_CONTENT_WIDTH_PTS`

Below this the columns would have to reflow into each other, and horizontal
scrolling is the better answer — the operator can see a whole column at a
time rather than two half ones. The window's own 520 pt floor is below this,
deliberately: a dialog dragged to its minimum should scroll, not shred.

### `const SCROLLBAR_WIDTH_PTS`

Named rather than inlined because it is used **twice for one reason**: it
is what the bar is drawn at, and it is what the body reserves out of the
column height so that a horizontal bar appearing cannot raise a vertical one
as a side effect. Those two uses must agree or the deadlock in
[`PrintDialog::body`] returns, so they read one constant.

### `const MIN_BODY_HEIGHT_PTS`

The window has its own 380 pt minimum, so this is reached only transiently —
during a resize, or on the first frame before the viewport reports its real
size. It exists so that arithmetic on `available_height` can never hand a
negative or absurd height to a column that will allocate it.

### `const REGION_SPLITTER`

A drag target's position cannot be computed from outside the process, and
this one exists *because* the operator asked for it — so a check that it is
present, has area, and moves the split needs somewhere to aim.

### `const REGION_PREVIEW_COLUMN`

# It exists to be able to GO AWAY

Every other region in this shell is published so that something can be
aimed at, measured or clicked. This one is published so that
`diag::end_ui_frame` can emit `ui-rect-gone name=print.preview.column` on
the frame the operator pops the preview out — which is the only evidence
from outside the process that the column **collapsed** rather than merely
gaining a sibling in another window.

Without it, the honest-looking check *"a second window appeared"* passes
on a build that opens the pop-out **and keeps drawing the column too** —
two previews of one sheet, which is precisely the shape O112 asked against.
A presence assertion cannot see that; an absence assertion can, and an
absence assertion is only worth anything when the run has first been driven
into the state where the absence is the claim. See `ui-verify`'s
`the_print_preview_pops_into_its_own_window`, which asserts the region is
there before the click and gone after it.

### `struct Columns`

# Why a type rather than four `let`s

Because there are now **two** layouts — preview beside options, and options
alone with the preview in its own window (O112 ask 2) — and every number in
the second differs from the first: the number of `item_spacing` gaps egui
inserts, the floor the content may not go below, and both column widths.
Four `let`s with an `if` threaded through them is how the two cases come to
disagree about one of the four, and the failure of a width in this file has
twice been a scrollbar the operator could not dismiss.

# It is PURE, and that is what makes the collapse falsifiable

The one thing a unit test genuinely can assert about this dialog's layout is
a relationship between numbers we own — `layout::tests`' own header says so,
and says why the *presence of a scrollbar* is not such a relationship. The
collapse **is** one: *"when the preview is popped out, the preview column is
zero wide, there is no splitter, and the options column is the whole
content."* That is an assertion about arithmetic, it needs no window, and a
build that popped the preview out and left the column standing fails it.

⇒ Which matters here more than usual, because a layout change's only true
oracle is a rendered frame and this session could not render one. Pushing
as much of the claim as possible into arithmetic is what is left.

### `fn split`

`gap` is the live `item_spacing.x` — read from the style rather than
assumed, because this shell's `Metrics::gutter` differs per theme preset
and a hard-coded gap is right in one preset and reintroduces a scrollbar
in another.

`preference` is [`PrintDialog::preview_width`] — the width the operator
dragged the splitter to. It is **clamped for layout and never written
back**, which is why this function returns a value instead of taking
`&mut`: writing the clamp back destroyed the operator's chosen width the
first time a narrow window clamped it, and the fix was to keep the
preference and the layout apart. See [`PrintDialog::body`].

### `fn laid_out`

The columns **plus** the gaps egui inserts between them — never the sum
of the columns alone. Reporting the sum was the mistake the whole
two-scrollbar defect was made of, and it is not made twice because there
is one function that knows how many gaps there are.

### `const FOOTER_HEIGHT_PTS`

The footer is drawn AFTER the scroll area, so the scroll area must be told
not to eat the whole window. Reserved as a constant for the same reason
[`preview`]'s strip height is: the commit button's position must not depend
on how much the body happens to contain this frame.

### `fn footer`

# The commit button is ABSENT, not greyed, when there is nothing to print

The no-placeholders rule's own distinction: greying is for
*temporarily* unavailable, and there are two genuinely different
reasons this button might not act.

- **No device, or no pages selected** — the job does not exist. The
  button is not drawn. Something else on screen already says why (the
  preview column's own sentence), so a disabled button would be a
  second, quieter statement of a fact already made loudly.
- There is no third case. A job that exists can always be sent; whether
  it *should* be is the operator's call, and the clip count in the
  label is how they make it.

# The label's count is corrected by what the preview has seen

Operator request O113, 2026-09-04. It used to be [`Job::clipped`] —
a geometric count of page boxes exceeding the printable rectangle —
which on a 1:1 CAD sheet read *"Print — 1 sheet will be clipped"* over
a preview showing nothing hatched and saying the overhang was blank.

It is now the geometric count **minus the sheets the preview has
examined and found blank**, with every sheet nobody has looked at still
counted. [`super::verdicts::ClipClaim`] carries both the number and how well
it is known, and picks the sentence that number can support; this
function does not choose wording, so the button and the preview's own
caption cannot come to say different things about one job.

Drawn AFTER the body, which is what makes the sheet on screen count
as examined on the same frame it is drawn. The alternative — the
footer reading a cache the preview has not written yet — would make the
button lag the picture beside it by exactly one frame, which is a
contradiction that flickers rather than one that persists, and is
therefore harder to notice and worse.
