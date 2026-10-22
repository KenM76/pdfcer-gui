# `formlookband` — Forms ▸ Style

Draws Fill, Outline, Outline width, Border style and the text's font, size
and colour for the **next** form field placed,
editing `formdraft::Remembered` through `set_look`. Reads `Remembered::look`,
which is the remembered look or a fresh text field's when nothing has been
placed or set.

Regions: `forms.style.fill`, `forms.style.outline`, `forms.style.width`, `forms.style.border_style`
(its drop-down rows are `forms.style.border_style.<i>` in `STYLES` order:
solid, dashed, beveled, inset, underline). `style_picker` is shared with the
placement dialog, whose picker is `dialog.form_field.border_style`.
Text: `forms.style.text_font` (rows `.<i>` in `Std14::ALL` order),
`forms.style.text_size` (0 shows *Auto*), `forms.style.text_colour`. The text
look applies to text and choice fields; the placement dialog carries it
through unchanged and does not yet show it.
Fill offers *no colour*; Outline does not, because width 0 is how a field
has no outline.

Trace on every change: `form-look width=<pt> style=<token> font=<Std14> size=<pt> bg=<c> bc=<c>`, where `<c>` is
`unset`, `none`, `g:<v>`, `rgb:<r>,<g>,<b>` or `cmyk:<c>,<m>,<y>,<k>`.

Session-only, like the markup pen: `Remembered` is not persisted.
