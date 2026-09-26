# `ui-verify/checks/print_layout`

`print_dialog_body_does_not_deadlock_its_scrollbars` — the regression test
for the operator's report of 2026-09-03.

# The defect

> *"I have two scroll bars in the pop up window that won't go away no matter
> how, and it doesn't close after I hit the print button that is so far off
> in the corner it is touching the edge the window, and it looks greyed out
> as though it doesn't do anything even when I hit print - but it is
> working, so after many clicks I checked the printer and of course there
> was a dozen jobs there because the button just looks greyed out and
> broken."*

Four causes. This check owns the first; the other three are asserted
elsewhere and named here so a reader can find them:

| symptom | where it is asserted |
|---|---|
| two scrollbars that never go away | **here** |
| the window is not its own OS window | `dialog_windows`, which now lists Print |
| the button touches the window edge | `dialogs::host::Host::BODY_MARGIN_PTS`, and the margin is visible in any capture |
| the button looks disabled | `egui_shell::Theme::accent_pair` and its unit tests |

# Why this is a check and not a unit test

Because the quantity that decides whether a scrollbar appears is
**egui's**, not ours, and it exists only in a laid-out frame. The previous
version of `PrintDialog::body` was wrong three times in a row about what
that quantity was, and each wrong answer looked completely reasonable in the
source:

1. it forced the content to `available_width`, measured **outside** the
   scroll area — one scrollbar narrower than the viewport the content was
   actually being laid into;
2. corrected to measure inside, it still used `auto_shrink([false, false])`,
   which *defines* the content to be at least the pre-bar viewport — so the
   content was again always at least one bar too wide, by construction;
3. corrected again, it did not account for the two `item_spacing` gaps
   `horizontal_top` inserts between three children, nor for the preview's
   control strip being **40 pt wider than its own column**.

Every one of those was found by reading `egui`'s own `content_size` and
`inner_rect` out of a running frame. No unit test could have produced either
number, and no screenshot could have said which of the three was wrong.

# The oracle

The `print-body` trace line, which reports both of egui's numbers, and the
`print-strip` line, which reports the inner overflow that fed them. The
assertion is an **inequality**, never a value: the widths depend on the
theme preset's font and button padding, so any constant would be a claim
that decays — which this project has spent six corrections on.

It is driven at **several window sizes**, not one. The defect was
*inverted* — bars present at 1000x760 and 1300x900 where nothing needed
scrolling, and **absent** at 700x520 where the Paper section was clipped and
unreachable. Two samples either side of that would have looked like no
defect at all.
