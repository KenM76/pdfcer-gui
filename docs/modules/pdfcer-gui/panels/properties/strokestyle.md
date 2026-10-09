# `panels::properties::strokestyle` — the "Line and opacity" section

Drawn for a selection of page objects (never an annotation) that holds at
least one path or picture, or, when no page object is selected, for the
selected parts of one placed drawing (`selected` returns the indices and
which list they index). Opacity on a part needs the drawing to own its
`/Resources`; the engine refuses otherwise and the refusal is shown. It raises `Action::SetObjectStrokeStyle` with only
the indices each row applies to.

## Rows

| Row | Shown for | Sends |
|---|---|---|
| Width | paths | `width`, points, 0–144 |
| Style | paths | `dash`: the four `LineStyle` entries, points, phase 0 |
| Line opacity | paths | `stroke_alpha` |
| Fill opacity | paths | `fill_alpha` |
| Picture opacity | pictures | both alphas — a placed drawing reads both, an image only the fill |

Values are read in points: width × the path's scale, and the dash array × the
scale rounded to `1e-3` before `LineStyle::of_pattern` names it, so a pattern
drawn at an awkward scale still reads as the entry that wrote it.

A field commits once, when a drag stops or the field loses focus, and only if
the value changed. Otherwise every pixel of a drag would be a content rewrite
and an undo entry.

## Mixed selections

A row shows the first member's value. When members disagree it adds a note
that a new value sets them all. The dash chooser reads `DashReading::Mixed`
and selects no entry.

## Regions and trace

`properties.stroke`, `.width`, `.dash` (entries `.dash.<i>` in
`LineStyle::ALL` order), `.line-opacity`, `.fill-opacity`, `.picture-opacity`.

`stroke-style-shown leaves= paths= pictures= width_pt= dash= line_alpha= fill_alpha=
picture_alpha=` is written on change. Values print to three decimals, or as
`mixed` or `none`.
