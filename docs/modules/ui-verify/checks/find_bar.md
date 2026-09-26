# `ui-verify/checks/find_bar`

`find_opens_and_finds` — Ctrl+F reaches Find, and Find finds something.

# What this is for

Find is the command that decides whether pdfcer can replace a PDF reader,
and its chord is the most reflexive one in any application. It was built
with 38 unit tests and driven once by hand; this is the part that runs
every time.

It could not be written when Find landed. `Driver::press` sends a bare
virtual key with no modifiers, so a command bound to `Ctrl+F` was simply
unreachable from this harness — the check was filed rather than written,
and `Driver::press_chord` was added to unblock it.

# What it asserts, in order, and why each step is separate

1. **The chord dispatches the command.** `chord-command chord=Ctrl+F
   id=edit.find`. This is the one that would have caught the defect the
   Open work found in `Ctrl+O`: a chord printed in a tooltip, present in
   the keymap, and bound to nothing, because the key table could not spell
   a letter chord. That state is invisible in every unit test — the keymap
   was right, the command was right, and the two were never introduced.
2. **The bar opens.** `find-toggled open=true`. Separate from step 1
   because a command that dispatches and does nothing is exactly the
   `command-unimplemented` shape this project keeps finding, and merging
   the two assertions would let a dispatch with no effect pass as a
   working Find.
3. **The bar is on screen.** A `find-bar` region with a non-zero area.
   Separate again: the Find state can be open while the widget is clipped
   to nothing, which is a real failure mode this project has hit — three
   panels shipped with a body, a rail entry and no control anyone could
   click, and every verification passed for the whole of their shipped
   life.
4. **A search runs and reports hits.** `find needle=… hits=N`. With
   `hits=0` the check still PASSES but says so loudly in a note: whether a
   given PDF contains a given word is a property of the fixture, not of
   the application, and failing on it would make this check fail whenever
   somebody pointed it at a drawing instead of a report.

# Why the needle is typed rather than injected

There is no `PDFCER_DIAG_FIND` seam and this check deliberately does not
ask for one. The native-file-dialog seam exists because a native dialog is
**outside egui's event loop** and cannot be driven at all; the find field
is an ordinary egui text field inside the window, so typing into it
exercises the focus handling, the text field, the Enter binding and the
search in one gesture. Substituting the answer there would skip the parts
most likely to be wrong.

# It types into a real window

Every keystroke goes to the foreground window, so this check needs
`--no-input` to be off and refuses rather than degrades otherwise. See
`Driver::press_chord`, which will not send a chord at all without a target
window — a bare keystroke into the operator's editor types a character,
but a chord runs a command.
