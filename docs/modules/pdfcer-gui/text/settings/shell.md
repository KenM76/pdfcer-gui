# `text::settings::shell` — the settings that change pdfcer's own window

The fourth row of the blast-radius taxonomy in [`super::look`]'s header.
Every setting here stops at what the operator sees or does: none of them
writes a byte, none of them changes a page, and each one's `_radius` line
says so, because an operator changing how Tab behaves or how big the
buttons are drawn has every reason to wonder whether they are also changing
the document.

## Why this is a module and not a section of [`super::look`]

The window's opening paragraph promises that everything in it exists
*because the standard declines to have an opinion*. That promise is true of
the first three modules and false of this one, and the difference is not
cosmetic: a `_silence` line has to answer *"what does the standard leave
open?"*, and for a setting the standard has never heard of the honest
answer is **"nothing — this is not a standards question"**. Those lines say
exactly that rather than inventing a clause to sit under, and keeping them
in one place is what stops the next one being written to match its
neighbours instead of the truth.

Two settings here answer a real ambiguity whose blast radius still stops at
a keystroke — the Tab-order pair, where the standard describes the second
pass twice and the two descriptions disagree. They are filed by radius like
everything else in this window, so they are here.

## The theme and UI scale are twins and must stay together

They are the only two settings in the whole window that take effect **before
Save** — both apply the moment they are picked so the operator can see them,
and both are put back by Cancel. That is an exception to the window's
draft-until-Save contract, and it is stated in two `_radius` lines that have
to keep saying the same thing. Split across files, one of them drifts.
