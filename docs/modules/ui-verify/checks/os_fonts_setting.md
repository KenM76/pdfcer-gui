# `ui-verify/checks/os_fonts_setting`

`font_folders_lands_on_the_fonts_setting` — **Tools ▸ Font folders opens
the window AT the folders, with the checkbox reachable.**

# What this is for

`OPERATOR_REQUESTS.md` **O50**. He asked for a font-folder setting that had
shipped the day before, and the interesting half of that is not that he
missed a row:

> the route built for exactly his question — a command called *Font folders*
> — dropped him at the top of ten **collapsed** headings and left the finding
> to him.

⇒ **A route that exists because of one setting must land on that setting.**
Opening the right window is not the same as answering the question the
command's own name asks.

## ★★★ Why this check is possible at all, and what it replaces

The Fonts group is inside a `ScrollArea`, below the fold, inside a
`CollapsingHeader` that is **closed by default**. A control in that state
publishes no `ui_rect_visible` region — deliberately, because
`settings_headings_legible` once measured three headings that were laid out
and clipped and reported the drawing behind the dialog as illegible text.

An earlier session tried to drive the Settings scroll to reach a group,
fixed four separate causes, still failed, and reverted. **This check does not
scroll.** It invokes the command whose job is to land there, and asserts that
the landing happened — which is both a smaller mechanism and a truer test,
because scrolling is not what an operator does either: they press the thing
named after what they want.

## ★★ The oracle is a VISIBLE region, and the distinction is the point

`settings.fonts.use_os` is published through `ui_rect_visible`, which needs
60 % of the control inside the clip rect. So the assertion *"this region was
declared"* means the checkbox is **on screen and pressable**, not merely
constructed. A build that forced the group open and did not scroll to it
would lay the checkbox out below the fold and publish nothing — and would
pass any check that asked whether the group existed.

## What this does NOT cover

**Ticking it.** The click and its effect on an embed are a second gesture and
a second document; `embedding_works_with_no_font_folder_at_all` already
drives the *resolver* half from the other end. What is asserted here is the
half O50 is actually about: that an operator looking for this can find it.
