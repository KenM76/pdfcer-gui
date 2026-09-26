# `pdfcer-gui-base/ocrlayerpref`

## Item notes

### `fn a_value_it_cannot_read_is_refused_rather_than_guessed`

`"#CC00999"` is the one worth having: it is seven hex digits, so a
parser that read the first six and stopped would accept it and draw a
colour nobody typed.

### `fn parse`

Accepts `#RRGGBB` and `RRGGBB`, and the three-digit shorthand `#RGB` where
each digit is doubled — the notation a CSS-literate operator will reach for
first. Case does not matter. Anything else is `None`, which
[`super::file`] turns into a `BadValue` note.

The leading `#` is optional on the way **in** and always written on the
way **out**. A parser that insisted on it would reject the value a
spreadsheet or a colour picker hands out, and a writer that omitted it
would leave the file looking like it held a number.
