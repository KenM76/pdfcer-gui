# Pointer step file (PDFCER_DIAG_POINTER)

This works only with `PDFCER_DIAG` set, on a private copy (hidden-driving.md).
Append one step per line. The app polls the file every 50 ms. Events go through
egui's own hit-testing, so they behave like real input in any viewport. The OS
mouse and keyboard are never used.

## Grammar

```
<seq> move X Y [vp=V]
<seq> click X Y [btn=l|r|m] [mods=ctrl+shift+alt] [vp=V]
<seq> dclick X Y [mods=...] [vp=V]
<seq> tclick X Y [mods=...] [vp=V]
<seq> down X Y [btn=...] [mods=...] [vp=V]
<seq> up X Y [btn=...] [mods=...] [vp=V]
<seq> drag X0 Y0 X1 Y1 [steps=N] [btn=...] [mods=...] [vp=V]    steps default 6
<seq> wheel X Y DY [mods=...] [vp=V]                              DY in wheel lines; egui sign (negative = down)
<seq> gone [vp=V]                                                 pointer leaves the window
<seq> shot [vp=V]                                                 screenshot of the viewport
<seq> key NAME [mods=...] [vp=V]                                  egui key name: A, Enter, Tab, Escape, Backspace, Delete...
<seq> type [vp=V] TEXT                                            rest of the line, spaces included, to the focused field
```

- `seq` is any integer. It is echoed back. Increasing by one is only a convention.
- `mods` tokens: `ctrl`, `shift`, `alt`, joined by `+`.
- X, Y are egui logical points of the viewport, the same space as `ui-rect`
  lines (not screen pixels). `vp=` is `root` (the default) or a `ui-rect`
  line's `viewport=` value, copied verbatim, for a pop-up window.
- `type` goes to whatever has keyboard focus: click the field first.
  `key A mods=ctrl` selects all in a focused field.

## Replies in the trace

```
pdfcer-diag diag-pointer-armed path=<step file>
pdfcer-diag diag-pointer seq=3 verb=click vp=root frames=3
pdfcer-diag diag-pointer seq=8 verb=shot ... path=<step file>.shot-8.ppm w=1400 h=900
pdfcer-diag diag-pointer-refused seq=9 line=<the line>
```

- Wait for a step's `diag-pointer seq=N` before writing the next one.
- `shot` writes a binary PPM (P6) beside the step file. It is the only way to
  see an off-screen window, because an OS capture shows whatever is on screen there.
  Convert it to PNG to view it.
- An acknowledgement means the events were delivered. Whether they did anything
  must be read from the feature's own trace lines or a `shot`.

## Example

```
1 click 640 44
2 click 300 400 btn=r
3 dclick 300 400
4 drag 100 200 400 260 steps=12
5 wheel 600 400 -1
6 key A mods=ctrl
7 type 12.5
8 shot
9 gone
```
