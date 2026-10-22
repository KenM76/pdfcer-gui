# `forms_style_sets_the_next_field`

**Defect it guards:** Forms ▸ Style is drawn but does not reach the next
placed field, or *Use as default look* on a field's right-click menu does not
read that field's look back.

**Drive** (off-screen, scripted pointer, `layer-assign.pdf`):

1. Edit mode; click the Forms tab; find `forms.style.width`.
2. Drag the width 40 px right. `form-look width=` must exceed 1.5 — a new
   field starts at 1 pt.
3. Arm Text field from the same tab and click empty paper; the dialog accepts
   itself (`PDFCER_DIAG_FORM_ACCEPT`) and the field is left selected.
4. Right-click the field's `form-box` centre and press
   `menu.item.canvas.field.edit.form_use_as_default`.

**Oracle:** `form-look-taken width=` equals the band's width within 0.05 pt.
That width is read from the document by `parse_acroform`, so it proves the
band, the remembered draft, the authoring path and the menu row in one
chain. A field placed at the old 1 pt reads back 1, which no passing band
value can produce.
