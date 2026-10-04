# `ui-verify/checks/installed_letter`

`a_letter_no_page_font_has_is_typed_in_an_installed_face` — a letter that no
page font and no standard-14 face can take is refused at the keystroke, and
the Properties panel offers to type it in an installed font; the press commits
it in a face the engine's replacement-face ladder picked from the font
folders.

The fixture is `fixtures/retype-seam.pdf`: `Hello world` at y=700, with `Hel`
in Helvetica and `lo` in Times-Roman, neither embedded, both WinAnsi. Greek
capital omega is in no WinAnsi font. Because the word is drawn in two fonts,
the exact surgery cannot take the edit and the commit also exercises the
Retype workaround's use of the ladder.

The launch is `workaround_offer::launch` with `PDFCER_DIAG_FONT_DIR` set to
`C:\Windows\Fonts`, so the folders do not depend on the operator's
preferences. The drive is off-screen, `ScriptedPointer` only, with Properties
raised.

## Steps

1. **Refuse.** A click inside `Hello`, `End`, `Ω`, then no `Escape`; the
   draft must stay open. `text-edit-key-declined` and
   `installed-letter-offer` must be traced, the button region
   `properties.refusedchar.installed` declared, and no
   `edit-text-workaround` applied yet.
2. **Press.** `edit-text-workaround` must read `used=retype` with a `face=`
   other than `none`, and a new `edit-text-left-edge` must read
   `committed=yes`.

Which face wins is the engine's ranking and is not asserted; on this computer
it is a coverage-rung tie broken by file order (engine request G110).

## Falsification

With `canvas::textedit::installed::letter_commit` building the commit with
`workarounds: false`, the commit carries neither the retype nor the ladder,
and the check fails at step 2.
