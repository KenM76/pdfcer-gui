# `pdfcer-gui/canvas/handles/name`

## Item notes

### `fn name`

# Why a hand-written function rather than the derive

It goes into a `key=value` line that a driven check parses, and a
`Debug` spelling in a parsed field is banned in this tree. That is not
a style rule: `{:?}` on a domain value has already produced **two**
false failure reports here, one of which reported the opposite of the
truth while quoting the truth in its own message.

# The abbreviations

Compass points are abbreviated and the two non-compass grips are not.
`NorthWest` earns nothing over `NW` on a line carrying eleven other
fields — but `M` and `R` beside eight compass points would read as two
more directions, so [`Grip::Move`] and [`Grip::Rotate`] keep their
words.

⚠ Exhaustive with no wildcard, so a tenth grip is a compile error here
rather than a press that traces as something it is not.
