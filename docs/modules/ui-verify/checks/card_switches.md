# `ui-verify/checks/card_switches`

`every_card_switch_is_in_one_window_beside_its_status` — **Settings ▸
Display opens one window holding every graphics-card switch beside what it
did on this machine, and its switches write the settings their first homes
write.**

## What it drives

One launch on `fixtures/a1-titleblock.pdf` with
`draw_pages_on_graphics_card = true` planted in the sandbox's preferences
and `PDFCER_DIAG_INVOKE=file.settings`, through the scripted pointer so it
runs under `--no-input`. It opens the Display page, rolls until
`settings.display.card_switches` is declared, presses it, then presses the
pages switch, Display's overprint `pdfx` choice (anything that differs from
the default, so the draft is dirty and Save is enabled), Settings' Save, and (in a build with the 3D viewer) the 3D
picture switch. The preferences are reset afterwards.

## What it asserts, in order

| # | assertion | what its failure means |
|---|---|---|
| 1 | `settings.heading.display` and `settings.display.card_switches` are declared | the window cannot be reached from Settings |
| 2 | pressing it traces `card-switches-shown` and declares `dialog:card-switches` | the button does not open the window |
| 3 | one `cards.switch.<path>` and one `cards.status.<path>` per row: `rows=` of them with the 3D viewer, the pages row alone without | a row has no switch or no status beside it |
| 4 | the last `card-status path=pages` outcome is `in-use`, `processor` or `limit` | a page was drawn with the switch on and its status is not recorded (`untried` and `off` are both wrong here) |
| 5 | pressing `cards.switch.pages` traces `card-switch path=pages on=false` and the file says `draw_pages_on_graphics_card = false` | the window's switch is not the Display page's setting |
| 6 | after Settings' Save the file still says `false` | the Settings draft, opened before the change, puts the old value back |
| 7 | pressing `cards.switch.picture` traces `on=false` and the file says `model3d_card_picture = 0` | the 3D switch does not reach the viewer's saved defaults |

The page row's outcome in 4 is what the machine did: a card drew the page
(`in-use`), it could not and the processor did (`processor`), or the page
was too large (`limit`). The check accepts each; it rejects only an outcome
that says nothing was attempted.
