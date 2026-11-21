# `pdfcer-gui-base/text/sign_picture`

The words of the Sign window's *Picture in the box* row, and the signing
receipt's line about it.

`receipt(requested, drawn)` returns `None` when no picture was asked for, so a
plain signature's receipt is unchanged. It says the picture was drawn, or that
it was not and the box shows its text only. The second case is the engine's
`appearance_image_drawn == false`, which the receipt reports instead of
claiming success.
