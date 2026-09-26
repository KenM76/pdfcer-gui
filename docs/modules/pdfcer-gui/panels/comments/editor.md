# `panels::comments::editor` — everything on a row that WRITES


## ★★ Why this is the seam, and not "split the rows from the strip"

R2's rule is *find the seam*, and the file genuinely comes apart here.
[`super::body`], [`super::row`], [`super::delete_control`] and
[`super::filter_strip`] are all **the list**: they decide what is shown and
in what order, and every one of them is a pure function of the document plus
a filter. What is in this file is the only part of the panel that holds the
operator's own half-finished work — a [`super::note::NoteDraft`] — and the
only part whose output is a **verb**.

⇒ Which is why the split survives the next feature rather than needing to be
redone: a control that writes will land here, a caption that describes will
land next door, and the question *"which one is this?"* has an answer that
does not depend on how many lines are left in either file.

★ The same seam `super::tests` took the day before, one step further along:
that file is *what is asserted about the panel*, this one is *what the panel
can change*, and [`super`] is left as the list itself.

## ★★★ TWO destinations, ONE box — the thing this file exists to keep true

A note edit and a reply are the same text box pointed at different engine
verbs: `set_markup_note` edits a dictionary that already exists, `add_reply`
**creates an annotation**. Everything they share — the box, its trace
region, the Escape route, the stale-draft rule — is written once in
[`editor`], and [`super::note::DraftTarget`] is asked exactly once, at the
point where the two genuinely differ.

⚠ The failure this shape is defending against is specific and silent: a box
captioned *Post reply* whose commit writes `/Contents`, which puts a
reviewer's answer **over the comment they were answering** and looks, on
screen, exactly like the reply having worked. Nothing but reading the file
afterwards distinguishes the two, which is why the destination lives on the
draft's stamp and is compared rather than inferred.

## What is deliberately NOT here

**Delete.** `super::delete_control` writes to the document too, and it stays
next door because it is drawn on the row's *navigation* line beside *Go to*
rather than in the editor block, and because it holds no draft. The line
this file draws is not "does it change the document" — it is "does it hold
the operator's unfinished words".
