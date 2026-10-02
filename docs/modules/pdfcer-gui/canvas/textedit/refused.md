# `canvas::textedit::refused`

When a key typed into existing text is one the run's font cannot take, one
of two things happens:

- **A face nearby has it** (`canvas::textedit::reface::plan`): the key goes
  into the draft and the notice reads `Planned` — the keys will be set in that
  face at the commit, with the rest of the line untouched
  (`app::actions::reface`). The button then offers the whole line in that
  face instead (`Whole`).
- **No planned face**: the sieve in `keys` leaves the key out of the draft,
  and the rest of this page applies.

Either way it is never silent and never a dead end:

- **Every refused key is named.** [`note`] collects each distinct refused
  character for the open draft; the notice's first line names them all
  (`text::refusedkeys::named`). The status bar's sentence still names the
  latest one, through `TextAction::KeyRefused`.
- **The notice sits under the editor box**, in a popup `egui::Area` on the
  foreground layer. If there is no room below, it sits above.
- **One click changes the run to a face that takes them all.** The faces are
  the same pre-flight the Properties face chooser reads
  (`pin::font_preflight` with every refused character as the candidate, then
  `face::choices`). `editmodel::nearface::nearest` picks the one nearest the
  run's own face: family class first, then weight and slant, then a face the
  page already carries. The button raises `Action::TextStyle` with
  `StyleChange::Face`, the same route the Properties panel and the ribbon use.
  When the pick is a standard-14 face pdfcer would add, the rule-4 disclosure
  (`face_addable_disclosure`) is drawn above the button.
- **The held keys go back in.** If the draft is exactly as it was after the
  last refusal (same text, caret and selection), [`resume`] sieves the held
  keys against the new face's repertoire and `keys` inserts them at the caret.
  If the operator moved on in between, the notice says to type them again.
- **No face takes them all** (a character outside every face the page can
  use): the notice says so and draws no button (R9).

## Stages

| Stage | Shown | Ends |
|---|---|---|
| `Planned` | the keys went in, the face they will be set in, the whole-line button | the draft closes, or the button is clicked |
| `Whole` | the whole line took the face; the keys stayed | the draft changes |
| `Offer` | the named keys, the disclosure if any, the button | the draft closes, or the button is clicked |
| `Asked` | "Changing this text to …" | the revision moves (the face landed), or two frames pass without it (refused) |
| `Retyped` | the face, and the keys that went in | the draft changes |
| `TypeAgain` | the face, and the keys to type again | the draft changes |
| `SwapRefused` | the face could not be applied; the status bar says why | the draft changes |

A new refusal in any stage starts a fresh `Offer`. The notice is forgotten
when the draft is abandoned or committed (`textedit::abandon`).

## Why a popup beside the edit does not break R8b

R8b puts disclosures off-canvas and never relative to the document. The
notice is about keys that are **not** in the document: they were refused
before anything was applied, while the draft is still the cursor's. It marks
no applied content, it is drawn on its own layer rather than with the page
painter, and a screenshot of the saved and reopened document differs from the
editing canvas only by the open draft and its caret, as it would without the
notice. It is a pre-commit affordance of the same kind as the caret.

## Trace

`text-edit-refused-keys page= run= characters=U+0071,U+007A font= faces= face= state=`,
written when any field changes. `state` is `reading`, `offer`, `no-face`,
`planned`, `whole`, `asked`, `retyped`, `type-again` or `swap-refused`. `face` is the selector the
button would send. Regions: `textedit.refused-keys` (the notice) and
`textedit.refused-keys.use-face` (the button).
