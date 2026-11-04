# `pdfcer-gui::app::conditions::offered`

Publishes the selection-dependent right-click rows' conditions: the
radius / diameter and area / perimeter switches of a ce dimension, text-run
merge and line split (offered, then allowed by the engine's preflight), and
`LAYER_ASSIGNABLE`. Called from `PdfcerApp::conditions` inside an open
document.

## Contract

Each name is asked on the same test the row's dispatcher arm routes on, so a
row is drawn exactly when its command can act. An `*_OFFERED` name draws the
row; its `*_ALLOWED` partner enables it, and the gap is the greyed row (R9).
