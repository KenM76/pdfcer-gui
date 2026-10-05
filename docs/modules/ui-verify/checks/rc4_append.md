# `ui-verify/checks/rc4_append`

`rc4_edits_wait_for_the_operator_and_say_what_they_cost` — an edit to an
RC4-encrypted file is refused with a sentence and a button until the operator
allows it; allowed, the edit lands, Ctrl+S saves under the file's own
encryption, and the save says how many changed parts reuse a key stream.

# What it drives

A scratch copy of `fixtures/encrypted-rc4-128.pdf` (ignores `--pdf`), with
`PDFCER_DIAG_INVOKE=pages.rotate_right`, under `--no-input` with a
`ScriptedPointer`.

1. `rc4-disclosed` (`app::rc4::on_open`, through `app::opennotes::disclose`).
2. `rotate-pages-refused`: the invoked rotation, refused by the engine.
3. `ui-rect status-group:decline.remedy` declared: the decline's button, drawn
   because `Declined::Rc4Refused` names `file.allow_rc4_edits`. Clicked.
4. `rc4-append policy=preserve` (`app::rc4::toggle`).
5. The rail's `rail.rotate.pages.rotate_right` button gives a new
   `rotate-pages` line: the edit now lands. Not `]`: the chord is not offered
   in Read mode.
6. Ctrl+S gives `save-in-place outcome=ok` and `rc4-keystream-reused n=`.
7. The saved copy is longer than the original and still names `/Encrypt`: a
   revision appended under the old encryption, not a decrypted rewrite.

# Why the remedy button and not the ribbon

The decline's button is the route an operator meets first, at the moment of
refusal. The ribbon item is the same command; `reach` covers its routing.

# Falsified

- Dropping `record_rc4_refused` from the funnel's error arm leaves the generic
  decline with no button and fails step 3.
- Without the status bar's cap on its left half (`statusfitting::floor_width`)
  the button is drawn at x 867..1003 under the zoom group (928..1068); the
  click lands on the zoom and step 4 fails.
