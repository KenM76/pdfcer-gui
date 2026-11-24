# `centre_line` — a centre line between two lines

`a_centre_line_is_drawn_between_two_lines` opens `two-lines.pdf` (two parallel
200 pt lines at y 100 and y 180), arms Measure ▸ Centre line in Review, and
clicks once on each line.

1. After the first click, no `add-dimension` may be traced: one line is not a
   pair.
2. After the second, `centre-line` must say `relation=parallel overlap=200.00`.
   The overlap is the fixture's full line length, so the pair picked is the
   fixture's pair and not one line twice.
3. An `add-dimension` must follow it: the centre line reached the document,
   with no placing click, because centre geometry has no label.

A `centre-line-refused` line fails the check with the refusal it names.
