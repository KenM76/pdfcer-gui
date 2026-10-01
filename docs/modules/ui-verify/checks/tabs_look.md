# `tabs_read_as_tabs`

**Request:** `OPERATOR_REQUESTS.md` O268. Tabs looked like buttons.

**Defect it catches:** a selected ribbon or panel tab drawn as one solid
accent plate.

## What it drives

Nothing is clicked. Launched off-screen (`-4200,-4200,1400,900`, scripted
pointer, `--no-input` safe) on `fixtures/cropped-sheets.pdf`; the window's own
frame is captured through the scripted pointer's `shot`. An OS capture of an
off-desktop window returns whatever is on screen there, so it is never used.

## Oracle

For every `ribbon.tab.<id>` and `dock.tab.<panel>`, sample a 2 pt strip along
the top edge (inset 8 pt past the corners) and the body below the label line.
A tab is *ruled* when the two differ by at least `MIN_PRESSED_DELTA`.

- At least one tab per family is ruled. A solid plate has the same colour at
  its top as in its body, so the old look fails here.
- Not every tab is ruled.
- Exactly one ribbon tab is ruled.

Falsified by restoring the plate-drawing ribbon and dock tab code: `ruled:
none`, FAIL.

## Not covered

Document tabs and the Align sub-tabs (not in the default layout), and the
pixel look under the dark preset.
