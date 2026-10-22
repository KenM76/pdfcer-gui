# `formlookband` — Forms ▸ Style

Draws Fill, Outline and Outline width for the **next** form field placed,
editing `formdraft::Remembered` through `set_look`. Reads `Remembered::look`,
which is the remembered look or a fresh text field's when nothing has been
placed or set.

Regions: `forms.style.fill`, `forms.style.outline`, `forms.style.width`.
Fill offers *no colour*; Outline does not, because width 0 is how a field
has no outline.

Trace on every change: `form-look width=<pt> bg=<c> bc=<c>`, where `<c>` is
`unset`, `none`, `g:<v>`, `rgb:<r>,<g>,<b>` or `cmyk:<c>,<m>,<y>,<k>`.

Session-only, like the markup pen: `Remembered` is not persisted.
