# `ui-verify/checks/output_preview_press`

`output_preview_overprint` — Output Preview says whether the page uses
overprint, and tells a page where it changes the print from one where it
changes nothing.

# What it drives

Two sessions, each the other's control:

| Fixture | Built by | Owed |
|---|---|---|
| `fixtures/output-preview.pdf` | `fixtures/output-preview.PROVENANCE.py` | `set=1 effect=0 whole=1` — overprint is on a patch whose four inks are all non-zero, so nothing shows through |
| `fixtures/print-shop.pdf` | `fixtures/print-shop.PROVENANCE.py` | `set=1 effect=1 whole=1` — magenta overprints cyan |

1. Scripted clicks on `ribbon.tab.view` and `ribbon.item.view.output_preview`,
   skipped when `output-preview.on` is already declared: both sessions share
   one profile directory, so the second opens with the panel the first left
   up, and pressing the item again closes it.
2. A click on `output-preview.on`.
3. The newest `output-preview-overprint` line after the switch carries the
   owed fields.
4. `output-preview.overprint` is declared: the answer was drawn, not only
   traced.

# Falsified

- Forcing `takes_effect = true` in the panel fails on `output-preview.pdf`
  (`owes effect=0`).
- Forcing `takes_effect = false` fails on `print-shop.pdf` (`owes effect=1`).

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
