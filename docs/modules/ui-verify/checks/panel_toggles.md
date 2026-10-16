# `a_dock_right_click_hides_and_shows_any_panel`

**Defect it guards.** O306: right-clicking a dock tab, or the empty part of a
dock tab bar, lists every registered panel with a tick for whether it is on
screen. A list that is missing, short, or that does not close and reopen the
picked panel fails.

**Fixture.** `fixtures/a1-titleblock.pdf`, after `view.reset_layout`.

**Steps.**

1. Right-click the last `dock.tab.*` on screen; at least two
   `dock.menu.panel.*` rows must be drawn after it, one of them that tab's.
2. Press that tab's row: `panel-closed id=<panel>` must follow.
3. Right-click the first live `dock.*.tabbar.blank`; the hidden panel's row
   must be drawn.
4. Press it: `panel-shown id=<panel>` must follow.

**Falsified** by returning no rows from `PanelToggles::census`: step 1 fails
with an empty row list.

**What it does not prove.** That the ticks match what is on screen — the row
is pressed by name, and its checked state is not read.
