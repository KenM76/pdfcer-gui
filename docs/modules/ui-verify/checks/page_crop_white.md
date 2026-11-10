# `remove_white_margins_crops_to_the_ink`

**Defect caught:** Remove white margins does nothing, crops to the wrong box,
or crops without going through the edit funnel.

**Drive.** `blank-overhang.pdf`, the Crop pages window raised with
`PDFCER_DIAG_INVOKE=pages.crop`, the `page-crop.white` button pressed with the
default margin of 0 mm.

**Oracle — independent of the GUI.** The box the engine's own CLI writes for
the same file:

    pdfcer.exe set-crop-box blank-overhang.pdf --remove-white-margins -o out.pdf

gives `[58.50 398.50 501.50 741.50]`. The check requires
`white-margins-measured … cropped=1` with every corner within 0.5 pt of that,
then `page-crop-applied n=1`, so a measured box that never reached the
document fails. A screenshot is kept.
