# `panels::properties::markup::textbox` — restyling a text box

The `/FreeText` rows of the Properties panel's markup section, reached through
`textannot::Reach::TextAnnot` with `Face::TextBox`. Every row raises one
`AnnotAction::SetTextAnnotStyle` naming only its own property
(`textannot::untouched()` plus one field), so a change to the border never
touches the text and the reverse.

## Rows

| row | `TextAnnotStyle` field | engine effect |
|---|---|---|
| Border colour | `color` | `/C` |
| Border width | `border_width` | `/BS /W`; 0 removes the border, >0 on a borderless box adds a black one |
| Line style | `dash` | `/BS /S` and `/D`; drawn only while a border is |
| Background | `fill` | `/IC`-equivalent frame fill; Clear removes it |
| Text colour | `text_color` | the `/DA` fill |
| Font | `font` | the `/DA` face; the two symbolic Standard 14 faces are not offered (`std14_styled` answers `None` for exactly those, and the re-bake's text layout refuses them) |
| Text size | `font_size` | the `/DA` size; 0 is auto-size |

The colour rows use the shell's `swatch` control, which commits once when the
picker closes, so one gesture is one re-bake and one undo entry. The two
number fields commit on release or focus loss, never per drag pixel.

## Reading

`TextBox::of` reads the engine's `TextAnnotSpec::FreeText`, the same spec
`set_text_annot_style` re-bakes from. The text colour is shown only for grey
and RGB; a CMYK `/DA` colour gets a sentence saying a pick replaces it with
RGB. The border colour's CMYK narrowing uses the parent's `swatch_of`
disclosure.

## Refusals

The engine refuses a restyle of a box whose appearance another program drew
(`FreeTextAppearanceForeign`) or whose words carry rich text
(`FreeTextIsRichText`). Those are answered by the redraw offer — see
`anyannot` and `pdfcer-gui-base::redrawoffer` — not by these rows.

## Regions

`properties.markup.textbox` (the block), `.width`, `.size`, `.border`,
`.fill`, `.ink` (each swatch also publishes `{region}.picker` while open).

## Trace

`textbox-rows id= width= bordered= fill= text= font= size=` when the reading
changes.
