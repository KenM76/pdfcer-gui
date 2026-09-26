# `pdfcer-gui/app/dispatch/exchange`

`app::dispatch::exchange` — the File ▸ Export band's commands.

Every command here moves content **between this document and a file on
disk**: exports that write a derivative of the page's own content, and the
round-trip halves beside them that read one back in.

| command | direction | window |
|---|---|---|
| `file.export_dxf` | out | [`crate::dialogs::export_dxf`] |
| `file.export_image` | out | [`crate::dialogs::export_image`] |
| `file.export_text` | out | [`crate::dialogs::export_text`] |
| `file.import_text` | **in** | picker, then [`crate::dialogs::import_text`] |
| `file.export_form_data` | out | a save picker, no window |
| `file.import_form_data` | **in** | a picker, no window |
| `file.stamp_collection` | out | [`crate::dialogs::stamp_collection`] |

## ★★ Why this is a module and not a run of arms in [`super`]

**R2.** These commands are one subject, and [`super`] is a file about
several dozen. The ceiling on a file's size is not a budget to spend down
to; it is a signal that a file has stopped being one subject, and the seam
to cut on is the subject, never the line count.

## ★★ Why the band's SUBJECT is "exchange" and not "export"

Because half of it imports. `RIBBON_IA.md` calls the band Export and that is
the right *label* — the operator meets exporting first and far more often —
but a module named for it would have to explain, on the day somebody adds
another import, why `import_form_data` lives in `export.rs`. The subject is
**content crossing the boundary of this document in either direction**,
which is what every verb here does and the only thing they all do.

★ `file.save_as`, `file.save_copy` and `file.save_compacted` are NOT in this
band and are not here. They write the document *itself*, not a derivative of
it, and they stay in [`super`] beside `file.save` where an operator's mental
model puts them.
