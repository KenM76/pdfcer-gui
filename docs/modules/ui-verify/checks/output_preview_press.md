# `ui-verify/checks/output_preview_press`

`output_preview_overprint` — Output Preview says whether the page uses
overprint, and tells a page where it changes the print from one where it
changes nothing.

`output_preview_soft_proof` — the *Simulate* list offers the document's output
intent, and choosing it, then *Simulate paper colour*, each redraws the canvas.

# What `output_preview_overprint` drives

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

# What `output_preview_soft_proof` drives

One session on `fixtures/output-preview.pdf`, whose output intent is
`fixtures/output-preview.icc`.

1. The switch, as above; `output-preview-proof source=none` follows.
2. A click on `output-preview.proof`; `output-preview.proof:intent` must be
   declared in the open list, then it is clicked.
3. `output-preview-proof source=intent paper=0 black=0` follows, and a
   `render-spawn` after the click: the canvas was redrawn, not only the label.
4. A click on `output-preview.paper`; `source=intent paper=1 black=1` (paper
   implies black ink) and another `render-spawn` follow.
5. `output-preview.black-ink` is declared.

The fixture profile has no black point compensation, so `bpc=0` is the
engine's answer and the panel shows its no-BPC note.

## Falsified

A render key that ignores the soft proof fails at step 3 (`no render-spawn
followed`).

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
