# `every_tab_draws_its_glyph`

**Defect it guards.** O298: a ribbon tab or a dock panel tab is drawn as text
alone, its glyph falls outside the tab, its glyph's rect is blank, or the
painter is handed a key it has no icon for and draws the missing mark.

**Fixture.** `fixtures/cropped-sheets.pdf`, opened off the desktop through the
scripted pointer, in Read mode.

**Steps.**

1. Collect every live `ribbon.tab.<id>` and `dock.tab.<panel>` region. Fewer
   than two in either family is a harness SKIP.
2. Each tab must publish `tabicon.ribbon.<id>` or `tabicon.dock.<panel>`,
   contained in the tab within half a point.
3. The glyph's rect in the screenshot must not be one uniform colour.
4. The trace must hold no `icon-unknown-key` line: the missing mark has ink
   and would pass step 3.

**Falsified** against the v0.82.0 build, which draws tabs as text alone: every
tab in both families fails step 2.

**What it does not prove.** Only Read mode's tabs, and only the panel tabs
visible in the bar, are judged. A tab in the ribbon's or the dock's overflow
menu, and tabs on floating panels, are not driven. Steps 3 and 4 have not been
falsified by a planted blank or unknown key. Whether a glyph *suits* its tab
is a judgement made from the screenshot, not an assertion.
