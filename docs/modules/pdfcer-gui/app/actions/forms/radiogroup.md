# `pdfcer-gui/app/actions/forms/radiogroup`

Puts a radio button into a radio group, by two routes: `Ctrl+Shift+V` over a
copied radio (`add_member`) and Properties ▸ Radio group or a rename onto a
group's name (`move_to`, from `FieldAction::JoinRadioGroup`).

Both author through `EditSession::add_radio_button`, which joins the field
of that name. `paste_field`'s `AdditionalWidget` is not used for radios: it
keeps the copied export value, and the engine refuses that as
`RadioExportValueTaken` because two members sharing an on-state select
together. The shell picks a free export with `free_export` (the copied
value, else its stem plus the lowest free number). This is a workaround,
filed to the engine as a request.

`move_to` adds to the target, then `delete_widget`s the source, then folds
the two into one undo step with `coalesce_last(2, AddFormField)`. When the
fold does not happen the status says two undos are needed. A failed delete
undoes the add.

`exports` lists every on-state of every member; `formdraft::next_in` uses it
so a third consecutively placed radio does not reuse the first's export.
