# `added_text_keeps_its_cmyk_and_opacity`

**Detects:** a text pen whose picker offers no CMYK or no opacity, or an
added run written in another colour space or fully opaque.

## Drive

1. Launch on a copy of `fixtures/layer-assign.pdf` with Edit ▸ Add text armed,
   window off the desktop, scripted pointer.
2. Raise Properties if needed, open the pen's swatch
   (`properties.tool.text_pen_colour`), press its CMYK tab and its opacity
   row, then Escape. Closing the picker is the commit.
3. Click blank paper, type `QKMYW`, click away, Ctrl+S.

## Oracle

Two, in order:

- The last `text-pen` trace line must read `ink=cmyk:..` and an opacity
  below 1. A press at the opacity row's centre lands on the slider, so any
  value under full opacity is the pen taking the control.
- The saved content stream showing `(QKMYW)` must carry `k` after the run's
  `BT` and a `gs` before the show, and the file must hold a `/ca` below 1.
  The engine writes the fill in the space the pen named, and the opacity as
  an `ExtGState` set under an outer `q`, ahead of the run's own `q`
  (`AddTextRequest::fill_alpha`). An opaque pen writes no `gs`, so its
  presence is the slider reaching the file.

Escape on the popup is assumed not to disarm Add text; the caret check after
the paper click reports it if it does.
