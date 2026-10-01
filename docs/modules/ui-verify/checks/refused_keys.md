# `ui-verify/checks/refused_keys`

`every_refused_key_is_named_with_a_face_that_takes_them` — keys typed into
existing text that its font cannot take are all named beside the edit, and one
click changes the text to a face that has them and types them back in.

The fixture is `fixtures/subset-font-floor.pdf`: one run, `ABC`, in a subset
font with only those three glyphs. The window is placed off the desktop and
driven only through `ScriptedPointer`, with
`PDFCER_DIAG_INVOKE=mode.edit,edit.text` arming the Edit Text tool, so the
check runs under `--no-input`.

## Steps

1. **Every key named.** A click on the run, `End`, then `qz` as one burst. The
   last `text-edit-refused-keys` line must name `U+0071,U+007A`, and the
   region `textedit.refused-keys` must have been drawn.
2. **A face offered.** That line must read `state=offer`, and the button
   `textedit.refused-keys.use-face` must have been drawn.
3. **One click.** Clicking the button must produce `text-style-applied` (the
   font-change path the Properties panel uses) and move the notice to
   `state=retyped`, and the canvas's own `text-edit-typing` line must report
   the draft at `len=5` (`ABC` plus both keys): the held keys went back in.

## Falsification

With the sieve reporting only the first refused character, step 1 fails on
`characters=U+0071`. With the button not drawn, step 2 fails. With the
put-back removed from `keys`, step 3 fails on the draft length.
