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

# What `output_preview_coverage_warnings` drives

One session on `fixtures/output-preview.pdf`. Its four patches, PDF points:
400% CMYK at (72,500)-(272,650), rich black (C 60%, K 100%) at
(340,500)-(540,650), plain K 100% at (340,300)-(540,450), 40% cyan at
(72,300)-(272,450).

1. The switch, as above, then a click on `output-preview.tac` (300%).
   `coverage-mask` owes `tac_box` within 2 pt of the 400% patch in canvas
   points (72,142,272,292) and `rich=-`. A screenshot owes the overlay green
   (0,220,0) over at least 90% of a 40 pt square at the 400% patch's centre
   and none on the rich black.
2. A click on `output-preview.rich-black` (95%). `rich_box` owes both top
   patches (72,142,540,292); the panel owes `found_tac=1 found_rich=1`, and
   `coverage-overlay` is declared. A screenshot owes orange (255,140,0) on
   both top patches, since rich black is painted over coverage, and none on
   plain black, which has no other ink.
3. `50` typed into `output-preview.warn-opacity`; the panel owes `opacity=50`,
   and the 400% patch owes no pixel of solid orange and at least 90% orange
   mixed with the page (red over green over blue, red between 80 and 240).

The 40% cyan patch is under both rules, and plain black is the rich-black
rule's control: K at 100% with nothing mixed in.

## Falsified

- The rich black rule checked as total area coverage at 95% fails at step 2:
  `rich_box` spans the page, plain black included.
- The overlay traced and not painted fails the step 1 screenshot (0.000 of
  green).
- The opacity ignored fails step 3 (1.000 of solid orange).

# Driven off-screen

Scripted pointer, window at `-4200,-4200`, `spec.place = false`; it runs under
`--no-input`.
