# `dialogs::bates` — Bates-number a document

`pages.bates`, Pages ▸ Stamp ▸ Bates numbering…. Opened by
`app::dispatch::pages` with the page rail's pick and the page count. Commits
`PageAction::StampBates`, whose body is `app::actions::pagesize::bates`
(`EditSession::stamp_bates`, one `CommandKind::StampBates` undo entry).

## Contract

| Item | Contract |
|---|---|
| Scope | Every page by default. A rail pick of two or more pages (and fewer than all) is the default scope, with *All N pages* offered beside it. A single picked page is treated as "the page being looked at" and does not narrow the stamp. This deliberately differs from the shared `page_operands` rule, which falls back to the current sheet. |
| Numbering | Prefix, digits (1–`MAX_DIGITS`), suffix, start number. Labels run in document order from the start number. |
| Placement | One of the six `BatesPosition`s; margin in millimetres (converted to points); text size in points. |
| Preview | The first and last labels, computed with `BatesNumbering::label`. If the engine would refuse, the preview line is replaced by the reason (`text::bates::problem`, one sentence per `BatesError` variant), and the Stamp button is absent (R9). |
| Receipt | `text::bates::receipt`: pages stamped, first and last label, and the next number. Shown through the edit funnel's disclosure line. |
| Batch | `BatesForm` outlives the window, in `DialogsState::bates_form`. After a stamp, `remembered` advances the start number by the number of pages stamped, so the next document's window continues the run. The form survives a document closing; the window does not. |

## Why the engine's error text is not shown

`BatesError`'s `Display` names `WinAnsiEncoding` and quotes margins in points.
The dialog checks the same conditions before committing and words each one in
the operator's units. The engine's text appears only for a variant added after
this was written (the `other` arm).

## Signatures and certification

The engine refuses a certified document, and an encrypted document, before
anything is written. Those refusals reach the operator through the funnel's
refusal line. A stamp on a signed-but-not-certified document invalidates the
signature; the save-time question (`dialogs::signature`) asks about that, as it
does for every page edit.

## Trace and regions

- `bates-opened picked of only_picked start digits`
- `bates-commit n first labels position margin_pt size`
- from the action: `bates-applied n first_label last_label next`
- from the funnel: `bates-stamped`

Regions: `bates.body`, `bates.prefix`, `bates.start`,
`bates.position.{0..5}` (index in `BatesPosition::ALL`),
`bates.scope.{all,picked}`, `bates.stamp`.
