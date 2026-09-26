# `pdfcer-gui/text/doctabs`

## Item notes

### `const UNSAVED_MARKER`

An asterisk rather than a bullet, a dot or a coloured label: it is ASCII, so
no font in any fallback chain can fail to draw it (this project has been
bitten by a codepoint that rendered as a substitution box in a sentence
whose whole job was to give directions), and it is the oldest and most
widely understood "there are unsaved changes here" marker in desktop
software.

### `fn build_day`

The operator spent part of a morning reporting a defect that had been fixed,
against a build he did not know was old:

> *"Just realized windows wasn't opening the latest version. … I had linked
> the default pdf opener to a different location. I thought I had relinked
> it to the new one but it didn't take."*

The cause was a file association he believed he had repointed and which had
not taken — his, not the packager's, and it does not matter. **Nothing on
screen could have told either of us which build was running**, and that is
the part that is fixable here.

⇒ The failure mode is *a report about the wrong build*, and it costs the
same however the wrong build got launched: an operator describes a defect
that was fixed, and the engineer investigates a version nobody is running.
The stamp existed the whole time — `build.rs` sets it, `dialogs::about`
shows it — two clicks behind a menu nobody opens while they are working.

⇒ The title is the one surface that is legible **without doing anything**:
it is in the taskbar, in Alt-Tab, in a screenshot, and in the accessibility
window list. If a report can be about the wrong build, the build has to be
on the outside of the window.

**The day AND the local time** — 2026-09-02, on the operator's ask:
*"add the local compilation time to the top bar at the end of the date you
added."*

It was the date alone, on the reasoning that a title is read at a glance and
the question is *"is this today's?"*. That reasoning was incomplete, and the
record says so: **two reports have now been closed by "you were running an
old build"** (O85 and O87), and on a day when several builds are published
the date cannot separate them. A date answers *is this today's*; a date and a
time answer *is this the one I just installed*, which is the question that
was actually being got wrong.

# The zone is shown when it is NOT local, and that is the whole subtlety

`PDFCER_BUILD_TIME` has two producers and they disagree about zone:


A packaged build's time is local, so the offset adds nothing to somebody
standing in that zone and is dropped. A dev build's is UTC, and showing
`06:25` bare would invite reading an hour that is not the wall clock — so
`UTC` is kept. **A stamp that says the wrong hour is worse than one that says
a true hour in a named zone**, which is `build.rs`'s own sentence about why
the fallback labels itself.

Still derived by truncation from the one value with one producer, so it
cannot disagree with what About shows. A second stamp computed elsewhere
eventually would.

### `fn stamp_for_title`

Every unrecognised shape falls through to the whole string rather than to a
placeholder: the failure this guards against is a title with **no build in
it**, and something datelike is always better than nothing.

### `fn a_packaged_stamp_shows_the_local_time_without_its_offset`

The operator's ask (2026-09-02) and the common case: `package-portable`
stamps local time with a numeric offset, and the offset is noise to
somebody standing in that zone. What matters is that **the minutes are
there** — two of his reports have been closed by *"you were running an
old build"*, and on a day with several publishes the date alone cannot
separate them.

### `fn an_unlocalised_stamp_keeps_its_zone`

`build.rs`'s fallback computes UTC because it has no date crate and
cannot know the machine's offset. Showing `06:25` bare would invite
reading an hour that is not the wall clock — and `build.rs`'s own
sentence is that *a stamp that says the wrong hour is worse than one that
says a true hour in a named zone*.

This is the assertion that would fail against the obvious simpler
implementation — truncate to sixteen characters and stop — which is why
it is here.

### `fn an_unexpected_shape_is_shown_whole`

The failure this guards against is a title with **no build in it**. A
stamp in a shape this function does not know is still information; a
placeholder is not.

### `fn an_ordinary_title_carries_no_hint`

A hint that were always present would be furniture nobody reads, and it
would be a *false statement* for every minute the mode is off — the
window title being the one surface that reaches an operator who is not
looking at the application at all.

### `fn read_mode_puts_the_way_out_first`

First, not last, and the argument is this module's own about the unsaved
marker: a taskbar button truncates from the right, so a trailing hint is
the first thing the ellipsis eats — and the operator who has just hidden
all their chrome is not the one with a roomy title bar.

### `fn the_build_stamp_is_still_the_last_field`

`ui-verify`'s `the_title_bar_carries_the_build_time` finds the stamp by
splitting the title from the right. Prefixing costs that check nothing;
appending would have silently re-aimed it at this sentence and left the
stamp — which has already closed two "you were running an old build"
reports — unguarded.

### `fn closing_the_last_document_does_not_lose_the_hint`

Read mode is per window, not per document, so an operator can close
their last file while in it. That is the form a four-branch
implementation drops, and it is the state with the least on screen.
