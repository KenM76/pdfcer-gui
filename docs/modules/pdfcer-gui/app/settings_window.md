# `pdfcer-gui/app/settings_window`

## Item notes

### `fn save_settings`

Four steps, in this order, and the order carries two of the decisions.

# 1. Adopt FIRST, then persist

`self.settings = draft.working` happens before the write is attempted,
so a disk that refuses does not cost the operator the choice they just
made. The session behaves as they asked **while telling them it will not
survive a restart**.

Adopting only on a successful write is the obvious alternative and is
worse: it silently ignores a deliberate choice, and the operator's only
evidence would be that nothing changed — which reads as the setting not
working rather than as the file not being writable.

# 2. Save closes the window

`take()` rather than a borrow. There is no *Apply*: a window with Apply
has a third state — saved, unsaved, and saved-but-still-open-with-more-
edits — and this one is short enough that Save-and-close costs nothing.

# 3. Every cached raster is invalidated

This is the step whose absence would be a defect, and a confusing one.
Several settings change how a page **renders** — `cmyk_intent`,
`mask_resample`, `image_minify`, `cmyk_jpeg_polarity` and `missing_as`
among them — and the canvas texture and the thumbnail rail were both
produced under the old values. Without an invalidation the operator
changes how black is drawn, presses Save, and **nothing on screen
moves** until something else happens to dirty the cache.

That reads as the setting not working. The rule that prevents it: go
through the established funnel rather than reaching into the two caches
by hand, because a third cache added later joins the funnel and does not
join a pair of hand-written clears.

# 4. A failure is reported in the status bar, never in a dialog

A save that **failed** says so, because the operator asked for something
to be remembered and is owed the truth if it was not. Not modally: a
configuration problem may not interrupt the document they are working
on, and it may certainly not stop them opening a file.

A save that **worked** says nothing, which is this shell's standing
convention rather than an omission here. The operator pressed a button
in a window they were looking at and the window closed; narrating that
back to them trains them to stop reading a bar whose only other job is
to carry the failures.

### `fn opening_a_document_adopts_the_operators_settings`

The regression test for the one hole this design leaves open.
`OpenDoc::assemble` starts every document on the *shipped defaults* —
it cannot reach `PdfcerApp` — so the snapshot is only ever correct
because `adopt` calls `adopt_settings`. A fourth open path added later
that forgot to would produce a document rendered under pdfcer's answers
while the settings window showed the operator's, correctly, which is a
control that reads back what you set and does not do it.

The assertion is on a **non-default** value, deliberately. Asserting
that a defaulted app produces a defaulted document would pass with the
call deleted.

### `fn adopting_settings_drops_the_text_derived_under_the_old_ones`

The other half of `adopt_settings`, and the half whose absence would be
invisible: the snapshot updates, the caches do not, and the operator
changes how black is drawn and sees nothing move.

Asserted on the page-text cache rather than on a raster, because it is
the one this test can fill and observe without a GPU context — and
because it is the more dangerous of the two. Three settings change what
an extraction produces and one of them can make a whole run vanish, so
a stale extraction is not differently spaced, it can be missing content
that a find or a redaction-by-pattern would then fail to see.

### `fn settings_window`

# The ✕ is a Cancel, and that is a contract rather than a convenience

`egui::Window::open` gives the title bar a close button that this code
does not otherwise see. If closing by ✕ did anything different from
closing by Cancel — kept the draft, half-applied it, saved it — the
window would have two exits with two meanings and no way to tell which
one an operator took. Both paths drop the draft.

### `fn adopt_settings`

One function, two acts, and they are together on purpose. This is the
only place in the application that writes `OpenDoc::settings`, and it is
also the only place that clears the caches keyed to it — so there is no
state in which a page texture, a strip entry or a page-text cache
disagrees with the snapshot beside them.

Splitting them is the obvious mistake and would be invisible: update the
snapshot without clearing, and the operator changes how black is drawn,
presses Save, and **nothing on screen moves** until something else
happens to dirty the cache — which reads as the setting not working.
Clear without updating, and every cache immediately refills under the
old configuration, which reads the same way and is harder to find.

# Called from a settings Save, and from the one place a document opens

The first is obvious. The second is the one that is easy to forget:
`OpenDoc::assemble` starts every document on the *shipped defaults* — it
cannot reach `PdfcerApp` — so a document opened by an operator who has
configured anything would otherwise render under pdfcer's answers rather
than theirs.

`opening_a_document_adopts_the_operators_settings` is the test that
stops another open path being added without this call.

# Why the caches are cleared wholesale

Working out which cached sheets a given setting change affects would be
a second statement of what each setting does, in a function that has no
business knowing. They refill from the visible set on the next frame —
the same argument `app::actions::pages` makes when a page permutation
invalidates the strip.
