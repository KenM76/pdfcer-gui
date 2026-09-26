# `pdfcer-gui/app/status/fit`

## Item notes

### `fn group`

**Two of the three are toggles and one is a button, and that asymmetry
is honest rather than sloppy.** `FitMode::Page` and `FitMode::Width` are
*modes*: they persist, they re-fit on every window resize, and a control
that shows whether you are in one is telling the truth. `FitMode::None`
is the absence of a mode, so a "selected" Actual size would light up at
any pinned zoom — including 73 % — which is the module docs' defect
rendered on screen instead of merely wired. A plain button makes no claim
about state.

Called *last* of the three groups because the layout runs right-to-left;
see [`show`].
