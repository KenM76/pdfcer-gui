# pdfcer.exe: the engine's command line

When `pdfcer.exe` sits beside `pdfcer-gui.exe`, it is the same engine as a
scriptable CLI, with its own README.md, LICENSE and BUILD-INFO.txt. It edits
files on disk, so read file-safety.md before writing a file the user has open.
If it is absent, use the live link or a hidden GUI (hidden-driving.md).

## Discover before guessing

```
pdfcer.exe --help                 every subcommand, one line each
pdfcer.exe <subcommand> --help    options, units, refusals, output format, exit codes
```

Each subcommand's `--help` is authoritative and detailed. Read it before first use.
Grouped names are in cli-subcommands.md.

## Conventions

- `<INPUT>` is positional. Write the result with `-o/--output <path>`, or
  `--in-place` (cannot be combined with `--output`). Writing-subcommands need one
  of the two (missing → exit 2). `--output` may name the input.
  All writes go to a temporary file, then rename: a failure leaves the target intact.
- `--page N` is 1-based (default 1). Object indices (`--object`, `--objects`,
  `--index`) are 0-based, as printed by the matching list command.
- Geometry is PDF user space: points (1/72 in), origin at the page's lower-left,
  Y up. The page box may not start at 0,0 (some CAD sheets are centred on the
  origin), so read the bboxes the list commands print; never derive them from
  screen pixels.
- Rects are `x0,y0,x1,y1`; points `x,y`; colours `RRGGBB`.
- Dry runs: `embed-font` and `unembed-font` need `--apply`; `print` needs `--send`.
  Many edits accept `--dry-run` or `--preview`.
- Not implemented: `to-pdfa`, `validate-pdfa`.
- Global options: `--open-password-file <PATH|->` (prefer it; `--open-password`
  shows in process lists), and `--on-malformed keep-last|keep-first|refuse`
  (default `keep-last`: a malformed file opens).

## Save modes (`--mode`, on about 125 writing subcommands)

- `incremental` (default): appends a revision, leaving every earlier byte intact.
  This is the only mode that keeps existing digital signatures. Each step grows
  the file, so a long chain of edits can make it many times larger.
- `full`: a single fresh revision. Smaller, drops superseded revisions, and
  **destroys every signature**. Refused for a hybrid-reference file. A file that
  opened only through xref recovery forces a full rewrite.
- For a long edit chain, give only the last step `--mode full`, and only when
  `list-signatures` reports none.

## Signed documents

Run `list-signatures` first: it shows what each signature's byte range covers.
`verify-signatures` checks validity. Any edit to a signed file may invalidate the
signature, and a full save always removes it. Tell the user before writing.

## Redaction is two steps

1. `redact-mark` (`--rect` on `--page`, or `--search TEXT`, or `--pattern ###-##`)
   adds reviewable /Redact marks, drawn as a red outline. Nothing is removed.
   Check with `list-redactions`.
2. `redact-apply` **truly removes** the covered text, vector paths and image
   pixels, scrubs metadata, and forces a full rewrite. This is irreversible.
   Use it only when the user asked for content to be removed.
   `--residual-scope marked-only|hidden-carriers|whole-document`.
   Exit 10 means residuals were disclosed but not scrubbed.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | runtime error |
| 2 | bad usage (missing or invalid arguments) |
| 3 | I/O error, or a bad `--open-password-file` |
| 4 | not a PDF |
| 5 / 6 / 7 | `round-trip`: not byte-identical / reload failed / raster differs |
| 8 | save refused by name (for example, a full save of a hybrid file) |
| 9 | edit refused by name (encrypted, certified, page or index out of range...); the reason is printed |
| 10 | redaction applied, residuals disclosed, no `--acknowledge-residuals` |
| 11 | opened only via cross-reference recovery |
| 12 / 13 | `verify-signatures`: a signature failed / could not be verified |
| 64 | not implemented |

## Task → subcommands

| Task | Start with |
|---|---|
| Look | `inspect`, `render-page --scale S [--region x0,y0,x1,y1]` (scale = dpi/72), `extract-text`, `find-text` |
| Vector objects | `object-list --page N [--hit X,Y]` → `object-move`, `object-transform`, `object-copy`/`object-paste`, `object-delete`; points: `node-move`, `nodes-move`, `handle-move` |
| Draw new geometry | `annotate --type square\|circle\|line\|polyline\|polygon\|ink --as-content` (straight into page content) |
| Existing text | `find-text` → `edit-text`, `format-text`, `reflow`; text runs: `text-run-*`; new text: `add-text`, `place-text` |
| Markup and comments | `list-annotations`, `annotate`, `set-markup-note`, `add-reply`, `move-annotation`, `delete-annotation`, `flatten-annotations [--index I]` |
| Pages | `extract-pages`, `insert-pages`, `delete-pages`, `reorder-pages`, `rotate`, `rotate-page`, `set-page-size`, `set-crop-box`, `merge`, `split` |
| Forms | `list-fields`, `fill-field`, `reset-form`, `export-data`, `import-data`, `flatten` |
| ce dimensions | `dimension-list`, `dimension-add`, `dimension-label`, `group-set-scale` |
| Layers | `list-layers`, `layer-toggle`, `layer-edit`, `set-object-layer` |
| Bookmarks | `list-outline`, `add-bookmark`, `rename-bookmark`, `move-bookmark`, `delete-bookmark` |
| Security | `encrypt`, `remove-encryption`, `redact-mark` → `redact-apply`, `sign`, `list-signatures`, `verify-signatures` |
| Export | `export-dxf`, `export-docx`, `export-xlsx`, `export-ods`, `export-image`, `extract-tables` |
| OCR | `fetch-ocr-models`, `ocr` |

## Notes

- `object-list` indices are exactly what `object-move`/`object-delete`/`node-move`
  take as `--object`. Form XObjects are not candidates: their contents appear as
  `leaf=` rows, not editable through `index=`. `--hit X,Y` answers which object
  a GUI click there would select.
- After a delete, later indices shift down. Delete last, highest index first.
- `object-copy` writes a self-contained clip file; `object-paste` places it (with
  no transform, exactly where it was copied from; it works across documents).
  Re-run `object-list` to find the new objects.
- `flatten-annotations` without `--index` burns **every** burnable annotation on
  the page, the user's own markup included.
- Each edit reads and writes the whole file. Chain the steps through intermediate
  files. Never run two edits on the same file in parallel.
- Drawings can hold tens of thousands of objects. Filter `object-list` output by
  bbox before showing it to anyone.
- ce dimensions are pdfcer's measurement annotations. A pdf dimension is CAD
  vectors and text on the page, so the `dimension-*` commands do not see it.
