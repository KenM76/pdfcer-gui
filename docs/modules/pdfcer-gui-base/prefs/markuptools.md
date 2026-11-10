# `prefs::markuptools` — the markup tools' remembered choices

`Prefs::markup_tools: MarkupToolPrefs`.

| Field | File key | Tokens | Default |
|---|---|---|---|
| `highlight_shape: HighlightShape` | `highlight_shape` | `text` \| `area` \| `ellipse` \| `freehand` \| `polygon` | `text` |
| `region_touching: bool` | `region_delete` | `inside` \| `touching` | `inside` |

`HighlightShape` lives here, not in the canvas, so the file can name it; the
canvas re-exports it as `highlight::Shape`. `ALL` is Tool Options' order and
the index its regions carry. An unknown token keeps the default.

Kept in step with the GUI by [`app::prefs::markuptools`](../../pdfcer-gui/app/prefs/markuptools.md).
