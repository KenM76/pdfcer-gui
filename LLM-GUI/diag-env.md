# Diagnostic environment variables

pdfcer-gui reads these at start-up. Use them only on a private copy (hidden-driving.md).

**Gated:** these act only while `PDFCER_DIAG` is non-empty.
**Ungated:** these act whenever set. An empty `*_PATH` value answers that
picker with "Cancelled".

## Gated by PDFCER_DIAG

| Variable | Value | Effect |
|---|---|---|
| `PDFCER_DIAG` | `1` | Writes the trace to stderr; each line starts `pdfcer-diag `, including the ui-rect lines |
| `PDFCER_DIAG_INVOKE` | `id,id,...` | Runs these command ids in order, one per frame |
| `PDFCER_DIAG_KEYS` | `chord,chord,...` | Presses keymap-grammar chords (`Ctrl+S`, `Ctrl++`), one every 20 frames. Traces `diag-keys index=N chord=... spelled=yes\|no` |
| `PDFCER_DIAG_POINTER` | step-file path | Pointer, keys, text and screenshots (pointer-script.md) |
| `PDFCER_DIAG_SELECT_FIELD` | field name | Selects that form field once |
| `PDFCER_DIAG_FORM_ACCEPT` | non-empty, not `0` | The form-field dialog accepts itself |

## Ungated: windowing and input

| Variable | Value | Effect |
|---|---|---|
| `PDFCER_DIAG_VIEWPORT` | `x,y,w,h` | Window position and inner size, opened unfocused. `-4200,-4200,1400,900` is off every monitor |
| `EGUI_SHELL_DIAG` | `1` | Adds `egui-shell-diag ...` ribbon lines (for example `ribbon-command-invoked`) |
| `PDFCER_DIAG_TYPE` | text | Seeds the next text-edit draft with this text (`text-edit-seeded len=N`) |
| `PDFCER_DIAG_DROP_PATH` | path | Simulates dropping this file on the window once (`dropped source=env`) |
| `PDFCER_DIAG_DROP_AFTER_MS` | ms | Delays that drop |
| `PDFCER_DIAG_PASTE_CHORDS` | `new_field_first` \| `acrobat` | Overrides the paste-chord preference |
| `PDFCER_DIAG_FONT_DIR` | `dir;dir` | Extra donor-font folders for Embed and Unembed fonts |

## Ungated: answers to file pickers (no OS dialog appears)

| Variable | Answers |
|---|---|
| `PDFCER_DIAG_OPEN_PATH` | Open |
| `PDFCER_DIAG_SAVE_PATH` | Save as, Save a copy, Extract and similar save pickers |
| `PDFCER_DIAG_INSERT_PATH` | Insert pages from file |
| `PDFCER_DIAG_MERGE_SOURCES` | Merge: `;`-separated paths |
| `PDFCER_DIAG_IMAGE_PATH` | Insert image |
| `PDFCER_DIAG_MODEL_PATH` | Insert 3D model |
| `PDFCER_DIAG_MESH_SAVE_PATH` | 3D mesh save |
| `PDFCER_DIAG_ATTACH_PATH` | Attach a file |
| `PDFCER_DIAG_ATTACHMENT_SAVE_PATH` | Save an attachment out |
| `PDFCER_DIAG_FORM_DATA_PATH` | Import or export form data |
| `PDFCER_DIAG_TEXT_IMPORT_PATH` | Import text as pages |
| `PDFCER_DIAG_FONT_FILE_PATH` | A font file |
| `PDFCER_DIAG_FONT_FOLDER` | A font folder |
| `PDFCER_DIAG_ACROBAT_PATH` | The Acrobat executable |
| `PDFCER_DIAG_TRUST_STORE_PATH` | An Acrobat trust store |
| `PDFCER_DIAG_CERTIFICATE_PATH` | Signing certificate (`.pfx`/`.p12`) |
| `PDFCER_DIAG_EVIDENCE_FILES` | Validation evidence: `;`-separated paths |

A pointer step that opens a picker whose variable is unset shows a real OS
dialog, which can appear on the user's screen. Set the variable before launch.
Pickers trace `<kind>-picked source=env|native` (for example `save-picked`,
`merge-picked`). `source=native` means a real dialog was shown.
