# `panels::properties::installedletter`

One button under the refused-letter block: *Type "Ω" in an installed font*.

## When it is drawn

Only when all hold (`canvas::textedit::installed::letter_commit`):

- the letter was refused at the keystroke (`RefusedCharacter::typed` is
  `None`); a commit-time refusal has no live draft to type it into;
- a text draft is still open on that page's run;
- no face is already planned for the run (`canvas::textedit::reface`); that
  route needs no installed font;
- there is at least one font folder (`app::dispatch::fonts::augment_folders`).

Otherwise nothing is drawn, not a disabled button.

## What the press does

It inserts the letter at the draft's caret, commits the draft as
`Action::CommitTextEdit { workarounds: true, reface: None }` and abandons the
draft. The commit applies the engine's workarounds and its replacement-face
ladder over the font folders (`editmodel::replacementfaces`); the engine keeps
the run's font for every letter it encodes and embeds a subset of the picked
face for the rest. The status line names the face and the rung; the canvas
shows the saved result and nothing else. Undo takes the edit back.

## Why a press and not the keystroke

The keystroke query cannot afford to read every installed font, and the
preview must draw what the commit will save, so a letter only an installed
font has is never taken silently by typing. See
`docs/modules/pdfcer-gui-base/editmodel/replacementfaces.md`.

## Trace

`installed-letter-offer page= run= character=` on every frame the button is
drawn; the region `properties.refusedchar.installed`. The commit traces
`edit-text-workaround … face= rung=`.

Checked by `a_letter_no_page_font_has_is_typed_in_an_installed_face`.
