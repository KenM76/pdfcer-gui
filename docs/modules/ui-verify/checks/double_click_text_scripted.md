# `ui-verify/checks/double_click_text_scripted`

`double_click_text_without_the_mouse` — the proof that the scripted-pointer
seam carries every class of pointer gesture a canvas check needs, with no OS
input at all.

The window is placed off the desktop (`PDFCER_DIAG_VIEWPORT`) and driven
only through `ScriptedPointer`, so the check runs under `--no-input` and
while the operator is using the machine.

## Steps

1. **A ribbon click.** The centre of `ribbon.mode.edit` is clicked, and a new
   `mode=edit` line in the shell trace must follow. This proves the point
   space: a click that lands elsewhere selects nothing.
2. **A canvas click.** The fixture's pinned text point
   (`fixture::text_point_target`), mapped through `CanvasMapping`, is
   clicked. `canvas-selection first=` must name something. The OS-driven
   twin `double_clicking_a_text_box_edits_the_text` selects text at the same
   point, so a miss here indicts the seam and not the fixture.
3. **Click counting.** A double-click at that point must write a
   `text-edit-caret` line. This is the step that needs egui's own click
   counter to see two presses inside its window.

## Falsification

Aiming step 3's double-click at the ribbon instead of the text fails the
check at step 3, with the delivered acknowledgement quoted in the reason.

## What it does not cover

Whether the caret's keyboard focus survives OS activation. The window is
never focused off-screen, so that stays with the OS-driven twin.
