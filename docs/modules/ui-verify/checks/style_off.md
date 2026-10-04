# `ctrl_b_again_takes_bold_off`

**Defect it guards:** Bold or Italic pressed on text that already has it is
declined or does nothing, so a style can be put on and never taken off.

## What it drives

Off-screen, scripted pointer, `mode.edit,edit.text`, on
`fixtures/paragraph.pdf` (Helvetica 12 pt). A click at (112, 703) opens a
caret inside `drawing`; the Format tab is raised so the toggles report their
pressed state. Then Ctrl+B, Ctrl+B, Ctrl+I, Ctrl+I.

## Oracles, per press

| press | requires |
|---|---|
| every one | a `text-span-style-applied` with `applied` not `0` after the press, and a `text-style-ladder` line after it |
| first of a pair | `requested=<axis>` |
| second of a pair | `removed=<axis> rung=page-face bound=Helvetica`: the run's own regular face, which the page carries; and the shell's last `ribbon-item-selected` for the toggle after the press is `selected=0` |

The face comes from the engine's ladder report, not from the shell's
toggle, so a press that asked "on" again reads `rung=already-styled
requested=bold` and fails.

## Falsification

Making `app::dispatch::textformat::toggle` ask bold on whatever the text
carries fails the second press with *"did not take bold off onto the page's
own Helvetica"*.
