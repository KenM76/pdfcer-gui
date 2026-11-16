# `pdfcer-gui/app/conditions/armed`

## Item notes

### `fn armed_conditions`

Called from [`super::PdfcerApp::conditions`] with the set it is building,
so there is one `ConditionSet` per frame and this adds to it rather than
returning a second one to be merged — a merge being a place two answers
about one control could both be present.

### The paired text controls

`edit.add_text` / `markup.add_text` press while the caret tool is armed in
Add mode; `edit.paragraphs` / `markup.paragraphs` press while paragraphs
open whole. Each pair is one tool or one switch on two tabs, so they are set
from one predicate and cannot disagree.
