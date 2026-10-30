# `ui-verify/checks/pagestamps_scripted`

`header_footer_and_printer_marks`: on `fixtures/four-pages.pdf`, with nothing
picked in the page rail, Pages ▸ Stamp ▸ Header & footer… and File ▸ Print ▸
Printer's marks… each reach all four pages as one edit. The window is off the
desktop and driven only through `ScriptedPointer`, so the check runs under
`--no-input`.

## Steps

1. The check clicks `ribbon.mode.review`, `ribbon.tab.pages`, then
   `ribbon.item.pages.header_footer` (through the collapsed Stamp group if the
   band is narrow), then `running.add` with the window's defaults.
2. A `running-applied` line must carry `n=4 kind=Footer
   first_label=Page_1_of_4` (spaces in the label are traced as `_`). Four
   pages rather than one separates the Bates scope rule from the shared
   current-sheet rule; the label proves the `{page}` and `{pages}` tokens
   reached the engine.
3. The edit funnel must write a `running-stamped` line: the undoable edit
   landed in the session.
4. It clicks `ribbon.tab.file`, `ribbon.item.file.printer_marks`, then
   `printmarks.add` with the defaults (every mark, 6 pt gap, 18 pt marks).
5. A `printmarks-applied` line must carry `n=4` and `grown_pt` of at least
   24, the default gap plus mark length; the funnel must write a
   `printer-marks` line.

## Falsification

Making either dialog's whole-document scope return page 0 only fails step 2
or step 5 naming `n=1`. Leaving either command unregistered fails at the
ribbon click, naming the regions declared under the group.
