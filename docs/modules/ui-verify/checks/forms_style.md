# `forms_style_sets_the_next_field`

**Defect it guards:** Forms ▸ Style is drawn but does not reach the next
placed field, or *Use as default look* on a field's right-click menu does not
read that field's look back.

**Drive** (off-screen, scripted pointer, `layer-assign.pdf`):

1. Edit mode; click the Forms tab; find `forms.style.width`.
2. Drag the width 40 px right. `form-look width=` must exceed 1.5 — a new
   field starts at 1 pt.
3. Open `forms.style.border_style` and press row `.1`; `form-look style=`
   must read `dashed`.
4. Open `forms.style.text_font`, press row `.4` (Times-Roman), and drag
   `forms.style.text_size` 40 px right; `form-look` must read
   `font=TimesRoman` and a size above 0.
5. Arm Text field from the same tab and click empty paper; the dialog accepts
   itself (`PDFCER_DIAG_FORM_ACCEPT`) and the field is left selected.
6. Right-click the field's `form-box` centre and press
   `menu.item.canvas.field.edit.form_use_as_default`.
7. End the process; its sandbox's `userdata/preferences.txt` must hold
   `form_style_border_style = dashed` and `form_style_text_font = TimesRoman`.
   Launch again over the same sandbox; its first `form-style` line must read
   `style=dashed font=TimesRoman`. With the startup load removed it reads
   `set=0`, which is how the step was falsified.

**Oracle:** `form-look-taken width=` equals the band's width within 0.05 pt,
`form-look-taken style=` reads `dashed` — a fresh field is solid — and
`font=` / `text-size=` read the band's face and size back from the field's
`/DA`. A field placed without the text look reads `Helvetica` / `0`.
That width is read from the document by `parse_acroform`, so it proves the
band, the remembered draft, the authoring path and the menu row in one
chain. A field placed at the old 1 pt reads back 1, which no passing band
value can produce.
