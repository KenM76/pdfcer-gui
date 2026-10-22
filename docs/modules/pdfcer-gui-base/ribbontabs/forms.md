# `pdfcer-gui-base/ribbontabs/forms`

The **Forms** tab — *what will someone filling this in be asked?*

`RIBBON_IA.md` §5.5a. Three groups: Fields, Style, Manage. Edit mode only,
because placing a field changes the document's design.

# Style sets the next field

The Style group is a custom item, `FORM_LOOK`, drawn by
`pdfcer_gui_base::formlookband` (`formlookband.md`). It edits the same
`formdraft::Remembered` the placement dialog opens from, so the dialog and
the band cannot disagree: the band changes what the dialog will open on,
and accepting the dialog changes what the band shows.

It never touches a placed field. Restyling one is Properties; copying one's
look into Style is **Use as default look** (`edit.form_use_as_default`),
which is on this tab and on the field's right-click menu.

# The ids keep `edit.form_`

The commands' ids carry the `edit.form_` prefix rather than `forms.`. A
keymap or saved layout names them by id, so the id is not renamed to follow
the tab.
