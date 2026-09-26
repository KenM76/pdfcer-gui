# `ui-verify/checks/tab_reorder`

`document_tabs_can_be_rearranged` — dragging a tab along the strip moves
it, and does **not** change which document is on screen.

Requested 2026-08-20: *"can we make it so the tabs can be rearranged"*.

# Why this needs driving

`PdfcerApp::move_slot`'s arithmetic is swept in a unit test across all
eighty `(active, from, gap)` combinations on a four-tab strip. **Every one
of those passes on a build where a tab cannot be dragged at all**, because
the arithmetic is a pure function of two fields and the gesture is three
frame-level facts none of them can reach:

1. the tab's `Button` senses a **drag** and not only a click;
2. a drag begun on it survives to a release read from raw pointer input;
3. the boundary the caret marked and the boundary the release reports are
   the same decision.

(1) is the one that would have shipped. `egui::Button` senses clicks and
nothing else by default, and a strip built from plain buttons looks
completely correct until somebody tries to drag one.

# The second assertion is the one worth having

**The document on screen does not change.** Reordering tabs is tidying, not
navigation, and the failure — the active document following an *index*
rather than its own tab — produces a strip that is in the right order and a
canvas showing the wrong drawing. Nothing errors, nothing is lost, and an
operator who tidies their tabs while reading sheet 12 of a set finds
themselves reading somebody else's.

# What a passing run does NOT prove

That the caret was drawn where the tab landed. The trace carries the
boundary the release used and this check reads it; the caret is painted by
`egui_shell::tabstrip` and publishes no region of its own, so a caret drawn
at the wrong x would pass here. That is a real gap and it is stated rather
than hidden — the fix is a `ui_rect` on the caret, and it is worth doing the
day anything about the strip's geometry changes.
