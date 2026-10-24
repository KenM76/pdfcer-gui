# `ui-verify/checks/paragraph_on_drawing`

`a_paragraph_on_his_drawing_opens_as_one` — O307, driven off the desktop
through the scripted pointer, on a drawing the operator names.

Takes `--pdf` and `--doc-point PAGE,X,Y` (a point inside the paragraph). His
drawings are not rights-cleared, so they are never fixtures; the check copies
the file to its output folder before opening it.

It launches twice on the same point:

1. Paragraphs at its default (on), the Text tool armed, a click at the point.
2. `edit.paragraphs` invoked (off; `open-paragraphs enabled=false` must be
   traced), the same click.

The second launch answers how long the clicked line is. The first must open a
`block=` caret holding at least twice that many characters; a block that is
the line alone, or a `run=` caret, fails. Comparing against the line under
the same point needs no expected length per drawing.

A click that opens no line with Paragraphs off is a SKIP: the point is not on
text.

## Driven

| drawing, point | verdict |
|---|---|
| `Code 4 - Final.pdf` page 4 alone, `0,309,275` | PASS: block of 749 characters against a 105-character line |
| `SW41177.pdf`, `0,1140,713` | FAIL: `run=120 len=40` both ways; the engine refuses the block (`SharedTextObject`, G226) |

The SW41177 failure is the falsification: the check reddens on a paragraph
that opens line by line.
