# `pdfcer-gui/app/prefs/offpage`

**Whether the canvas grows to show what sits off the sheet — remembered
separately for each ribbon mode.**

This module owns one preference and the whole of its reasoning: the type,
the per-mode default, the file keys, the parser and the writer. The parser
and the writer are two spellings of one vocabulary and must not live in
different files, which is why this group is a module rather than another
family of arms in `prefs::file`. [`printing`](super::printing) is grouped on
the same rule.

# What the preference is about

A CAD export frequently carries marks outside its own `/MediaBox`. When
pdfcer is allowed to show them, the canvas grows a band of pasteboard wide
enough to hold them, and that band is not free: it lengthens every scroll,
and in a continuous layout it opens a visible grey gap between one sheet
and the next exactly where the off-sheet material lives.

So the operator's answer is not *"can pdfcer do this"* — it always can —
but *"is it worth the band right now"*, and that depends entirely on what
they are doing. Reading a drawing, it is not: the page is the subject and
the band is in the way. Reviewing or editing one, it is: a title block
dragged off the sheet is a defect you must be able to see and grab.

# ★★★ Why the answer is stored per MODE rather than once


> *"by default, read doesn't show off page items, review and edit do show
> off page items. these settings can be changed by the user and their
> preference is remembered for each read review edit modes."*

That is three preferences wearing one name, and the reason it has to be is
that **the modes disagree about what the document is for**. A single global
flag would force the operator to re-answer the question every time they
switched modes, which is the same defect as not remembering it at all —
worse, because it would look like the toggle was forgetting.

The consequence worth stating plainly: **switching mode can change the page
layout**, because leaving Edit for Read removes the band. That is intended.
It is also why the re-seed happens at exactly one place (the mode-change
site in `crate::app::surfaces`), so the layout can never be one mode's
answer while the ribbon shows another's.

# What is NOT stored here

Nothing about the document. Two open drawings can disagree about off-page
display, because the flag lives in [`crate::viewer::ViewState`] alongside
zoom and the overlays; this module holds only the *opening* answer and the
operator's last word per mode. A per-document memory would be a different
feature and would fight this one — see the mode-change note above.
