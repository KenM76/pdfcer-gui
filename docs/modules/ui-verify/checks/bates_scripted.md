# `ui-verify/checks/bates_scripted`

`bates_numbering_without_the_mouse`: Pages ▸ Stamp ▸ Bates numbering… on
`fixtures/four-pages.pdf`, with nothing picked in the page rail, numbers all
four pages as one edit. The window reopened afterwards continues the run. The
window is off the desktop and driven only through `ScriptedPointer`, so the
check runs under `--no-input`.

## Steps

1. The check clicks `ribbon.mode.review` (Pages is not a Read-mode tab), then
   `ribbon.tab.pages`. If the band is narrow it clicks the collapsed
   `ribbon.group.pages.stamp`, then `ribbon.item.pages.bates`, then
   `bates.stamp`.
2. A `bates-applied` line must carry `n=4 first_label=000001
   last_label=000004 next=5`. Four pages rather than one is the assertion
   that separates this command's scope rule from the shared current-sheet
   operand rule.
3. The edit funnel must write a `bates-stamped` line. That is the undoable
   edit landing in the session, not only the engine call.
4. The window is opened again, and its `bates-opened` line must carry
   `start=5`.

## Falsification

This was falsified by making the dialog's whole-document scope return page 0
only. Step 2 fails, naming `n=1 … last_label=000001 next=2`.

## What it does not cover

- Pixels. That the label draws upright at the chosen corner is the engine's
  (`bates::label_matrix`) and is covered by its tests.
- The saved bytes.
- The rail-pick scope.
- The refusal lines.
