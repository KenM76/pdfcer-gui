# `dialogs::print::direct` — a job sent to the printer as a PDF

Source: `crates/pdfcer-gui/src/dialogs/print/direct.rs` and its child
`direct/compose.rs`. Words: `pdfcer_gui_base::text::print::direct`. The switch
is `prefs::print_route::VectorRoute::pdf_direct`, off by default.

## Route

At Print, with the switch on, `PrintDialog::commit_direct` decides in order:

1. **Driver-only options** (`PrintDialog::direct_blocker`): print as image,
   separations, black only, a colour profile, moved pages, fixed line
   weights, a header or footer, a non-default annotation scope on an
   annotated document, finishing. Each is `FallbackReason::DriverOnlyOption`.
2. **The printer**, asked on a worker since the switch went on: a port with no
   network address is `NoNetworkAddress`, a printer that answered without PDF
   is `NoPdf`, nothing answering is `Unreachable`.
3. **The layout** (`compose::Layout`). The engine's `Shape::needs_composing`
   (n-up, booklet, poster, a chosen paper, a percentage scale), or reverse
   order, means composing. The print-ready copy's bytes are opened as a
   scratch `EditSession` and handed to `pdfcer_print::compose::compose`. A
   compose error is `CannotCompose`.
4. **Send.** Composed sheets go with `print-scaling` `none`, no
   `orientation-requested` and no `page-ranges`. The document as it is goes
   with `page-ranges`, `print-scaling` from the scale (Shrink is `auto-fit`)
   and a forced orientation. Sides come from the device settings the driver
   route plans with, so a booklet's forced short-edge duplex travels.

Any fallback is traced (`print-direct route=driver fallback=<keyword> why=…`)
and kept in `Direct::fallback`. The dialog appends it to the receipt
(`Direct::take_fallback`).

## Why not `Shape::for_job`

`Shape::for_job` fixes the n-up page order to horizontal, turns auto-rotate
on and drops the booklet's sheet range. The dialog has all three as controls,
and the preview shows them. So the shape is built from
`impose::State::nup_spec` / `booklet_spec` and the poster options, which are
the specs the driver route imposes with.

## Disclosure (off-canvas)

- Before Print, the Pages-tab line says the sheets will be laid out for the
  planned paper in millimetres and that it must be loaded
  (`text::print::direct::ready_composed`).
- The receipt carries `sent`, then `composed` (sheet sides and paper) or
  `printer_lays_out`, then every `Composed::notes` string and the print-ready
  copy's disclosures. A fallback carries `fell_back(reason, why)`.

## Trace

| Line | When |
|---|---|
| `print-direct-found queue= uri= from=` / `uri=none fallback= why=` | the printer's answer |
| `print-direct-compose shape= sheets= paper=WxH notes=` | after composing; `paper` in points |
| `print-direct-note i= text=` | one per `Composed::notes` entry |
| `print-direct route=direct uri= bytes= pages= ranges= copies= composed= dry_run=` | the send; `pages` is the sheet sides sent |
| `print-direct route=driver fallback= why=` | a fallback |

Checks: `print_sends_the_pdf_itself` (the document as it is) and
`print_composes_sheets_for_the_pdf` (two per sheet, composed).

## Limits

- A batch print's receipts do not carry the fallback sentence. The trace line
  is still written.
- The scale and orientation mappings into `pdfcer_print` types are repeated
  from `pdfcer_gui_base::printspooler`, where they are private.
