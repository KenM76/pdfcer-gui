# `ui-verify/checks/ocr_colour_setting`

`ocr_colour_setting` — **the recognised text is drawn in the colour the
preferences file names, and Settings ▸ Display's Reset writes the default
back.**

## What it guards

`Prefs::ocr_layer_colour` is read from `preferences.txt`, handed to the
painter through `canvas::ocrlayer::sync` each frame, and edited in Settings ▸
Display, where Reset is drawn only while the colour differs from
`DEFAULT_COLOUR` (`#CC0099`). It also guards the Settings window's footer:
Save must lie inside the window, and the host must never report
`dialog-fit-runaway` for it.

## How it drives

Three launches of the check's sandboxed binary, all against its own
`userdata`:

1. It plants `ocr_layer_colour = #00B000` and opens `fixtures/ocr-layers.pdf`
   with `PDFCER_DIAG_INVOKE=mode.read,view.ocr_layer`. It classifies the
   pixels in the recognised line's band (`ocr_layer_view::OCR_BAND`) as
   green-leaning or magenta-leaning, and requires green and no magenta. No
   coloured pixel at all is SKIPPED: `ocr_layer_view` owns whether the layer
   paints.
2. It opens Settings with no document (`file.settings`), clicks
   `settings.heading.display`, and rolls the page body until
   `settings.display.ocr_colour.reset` is declared. It presses Reset and
   requires it gone. It then requires no `dialog-fit-runaway` line and
   `dialog:settings.save` inside `dialog:settings`, presses Save, waits for
   `prefs-saved`, and requires the file's line to read `#CC0099`.
3. It relaunches as in step 1 and requires magenta and no green.

## Not covered

- Choosing a colour with the swatch's picker; the colour is planted in the
  file instead.
- That the colour never reaches a save, export or print (`FEATURES.md` still
  owes that).

## Falsification

- Passing `DEFAULT_COLOUR` to `ocrlayer::sync` in place of the preference
  fails step 1: 0 green, 116 magenta.
- Removing the row cap in `settings::nav::show` fails step 2 on
  `dialog-fit-runaway … at=860x770 wanted=860x808`.
