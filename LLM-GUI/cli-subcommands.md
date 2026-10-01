# pdfcer.exe subcommands, grouped (202 including `help`)

There is one line per subcommand in `pdfcer.exe --help`, and full detail in
`pdfcer.exe <name> --help`. Glosses are given only where a name is ambiguous.

**Inspect and read:** inspect, render-page, export-image, find-text, extract-text,
extract-tags (structure tree), extract-layout (infers blocks on untagged pages), extract-tables,
list-objects, object-list, list-fonts, font-preflight (which fonts `format-text --set-font` accepts),
list-layers, list-links, list-scripts, tab-order, page-labels, dump-object, dump-structure,
export-structure, import-structure, scan-offpage (content outside the sheet), list-standards.

**Pages:** merge, merge-document (into this one, preserving the session), split,
extract-pages, insert-pages, delete-pages, reorder-pages, rotate, rotate-page,
set-page-size, set-crop-box, scale-pages (scale content onto a new sheet), page-copy, page-paste,
copy-page (to the OS clipboard as vectors), set-page-labels, clear-page-labels, set-page-tabs,
bates-stamp, bates-remove, redact-offpage.

**Vector objects:** object-list, object-move, object-move-each, object-transform,
object-transform-each, object-copy, object-paste, object-delete, subpath-move,
subpath-delete, node-move, nodes-move, node-delete, handle-move, unshare-form
(give one page its own copy of a shared form XObject), add-image, set-object-layer.

**Text:** edit-text, format-text, reflow, add-text, place-text, run-repertoire
(which characters a run can take), text-run-delete, text-run-move, text-run-width, text-run-merge,
text-object-split.

**Annotations and markup:** list-annotations, annotate, set-markup-note,
set-markup-style, set-text-annot-style, set-review-state, add-reply,
set-annotation-open, set-annotation-flags, move-annotation, resize-annotation,
rotate-annotation, reorder-annotations, delete-annotation, annotation-vertex,
ink-edit, add-caret, add-screen, add-sound, attach-file-annotation,
set-annotation-layer, flatten-annotations, place-stamp, stamp-list, stamp-pack.

**ce dimensions:** dimension-add, dimension-list, dimension-label, dimension-style,
dimension-rotate, dimension-vertex, dimension-offset, dimension-display,
dimension-extension-gap, dimension-delete, dimension-group, group-add,
group-rename, group-delete, group-set-scale, group-set-standard, group-style.

**Layers:** list-layers, layer-add, layer-edit, layer-toggle, layer-move,
layer-delete, layer-merge, layer-flatten, layer-folder-add, layer-folder-rename,
layer-folder-delete.

**Forms:** list-fields, fill-field, reset-form, recompute (calculation scripts, without
running JavaScript), export-data, import-data, flatten, regenerate-appearances,
add-text-field, add-check-box, add-radio-button, add-push-button,
add-choice-field, copy-field, paste-field, inspect-field-clip, edit-field,
edit-widget, move-widget, rotate-widget, delete-widget, delete-field,
delete-field-group, rename-field, set-field-script, set-button-action,
adopt-widget (register a widget annotation as a field), promote-dr-fonts, password-values,
purge-password-values.

**Bookmarks and links:** list-outline, add-bookmark, rename-bookmark,
move-bookmark, delete-bookmark, set-bookmark-open, bookmark-copy,
bookmark-paste, add-named-dest.

**Attachments and media:** list-attachments, extract-attachment, attach-file,
detach-file, 3d-list, 3d-extract, 3d-mesh, 3d-render, 3d-embed.

**Security and signatures:** encrypt, set-permissions, remove-encryption,
redact-mark, list-redactions, redact-apply, sign, list-signatures,
verify-signatures, add-ltv (offline revocation evidence), timestamp (document time-stamp),
create-digital-id (self-signed), trust-store-list.

**Fonts:** embed-font, unembed-font (dry runs unless `--apply`), list-fonts, font-preflight.

**Metadata:** set-info.

**Export:** export-dxf, export-docx, export-xlsx, export-ods, export-image,
extract-tables.

**OCR:** fetch-ocr-models (verified download), ocr.

**Print:** list-printers, printer-properties, list-paper-sizes, print-preview,
print (dry run unless `--send`).

**Verification:** round-trip (save and check byte, reload and raster identity).

**Not implemented:** to-pdfa, validate-pdfa.
