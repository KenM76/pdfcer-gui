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

### `fn tab_label`

The file name, prefixed with [`UNSAVED_MARKER`] when the document has edits
that no save has taken. See this module's header for why the marker leads.

A path with no file-name component — which `Path::new("")` and a bare root
both are — falls back to the whole path rather than to an empty tab. An
empty tab is indistinguishable from a rendering failure.

### `fn tab_tooltip_created`

It says the document has never been written, which the path cannot: a
created document's path is a *name*, so showing it as a location would
assert that a file exists at `Untitled 2.pdf` in whatever the operator reads
as the current directory.

### `fn tab_tooltip_unopened`

The reason travels because the tab itself has room only for a name, and a
tab that says a file's name with no indication of why it is unreadable is a
tab the operator will click repeatedly.

### `fn tab_reason_needs_password`

Its own sentence rather than the `Failed` message, because §7.6 makes this a
third state and not a failure — pdfcer *can* read this document and has not
been told how.

### `fn window_title`

Three forms, and the third is the one this function exists for:

| open | title |
|---|---|
| nothing | `pdfcer` |
| one | `SW41177.pdf — pdfcer` |
| several | `SW41177.pdf — 3 documents open — pdfcer` |

The count is there because the window title is the **only** place a
tabbed application reaches an operator who is not looking at it. Alt-Tab,
the taskbar and a screen-reader's window list all read this string and none
of them can see the tab strip; an operator who left three drawings marked
up and went to answer an email is entitled to be told so by the thing they
are about to close.

The active document leads in every form, because that is what every
application in the class does and what a truncated taskbar button keeps.


The operator's report:

> *"I didn't see a way to get back out of read mode. if there is a shortcut
> for this it should have a note what the key combo is in the top bar that
> holds the window controls."*

This application draws no custom title bar — the window controls beside it
are the operating system's — but **the title is ours**, and it is literally
the text in the strip he pointed at. `read_mode` (the `Option<&str>`
parameter) is the exit statement when the mode is on and `None` the rest of
the time, so the hint exists for exactly as long as the state it explains.

It goes **first**, ahead of the file name, and that overrides the
paragraph above for one state only. The argument is this module's own, from
the unsaved marker three functions up: *a trailing marker is the first thing
the ellipsis eats.* A taskbar button showing `SW41177.pdf — pdfcer — 2…` has
already discarded everything after the name, and an operator who has just
hidden their whole chrome is not the operator with a roomy title bar. The
document name is still there and still ahead of everything else that is not
this.

The build stamp stays **last**, untouched, because
`ui-verify`'s `the_title_bar_carries_the_build_time` finds it by splitting
the title from the right. Prefixing costs that check nothing; appending
would have silently re-aimed it at this sentence and left the stamp
unguarded.

The chord inside the statement is resolved by
[`crate::app::window::exit_chord`] from the live keymap, never spelled here
— see [`crate::text::window`].

### `fn drag_landing_here`

Kept identical in shape to `crate::text::pages::drag_landing`, whose docs
carry the argument for saying it in page numbers as well as drawing a
caret: *"a hairline between two near-identical drawing sheets is precise
and not checkable"*.

### `fn drag_landing_other`

It says **copy**, and saying so is the whole point of the sentence.

Dragging a page from one open document into another does not remove it from
the one it came from, and an operator who assumed a move would find out by
discovering their source drawing intact tomorrow — or, worse, by assuming it
was not and deleting the wrong copy.

The reason it is a copy rather than a move is not squeamishness. A move is
two edits in two documents, and this application has one undo stack per
document: Ctrl+Z after a cross-document move would put the page back in the
source and leave the copy in the target, or take the copy out and leave the
source short, depending which document had focus. There is no ordering of
those two edits that makes one Ctrl+Z mean "undo what I just did". Windows
Explorer reaches the same conclusion for the same reason and copies between
volumes by default.

It names the **source**, not the target. The operator is looking at the
target — it is the panel or the page view the pointer is inside — so the
document that is not on screen is the one the sentence has to supply.

### `fn drag_landing_move`

Shift, because that is what Shift does on this desktop. Windows has bound
the drag modifiers the same way since the mid-nineties and every operator on
it has the reflex already:

| held | what a drag does |
|---|---|
| nothing | move within a volume, copy across one |
| **Ctrl** | copy |
| **Shift** | **move** |
| Ctrl+Shift, or Alt | make a shortcut — no analogue here, and pdfcer offers none |

Two documents are two volumes by that analogy — they are separate files with
separate undo stacks — so the unmodified drag copies and Shift is what asks
for the sheets to be taken out of where they came from.

## The sentence says the source will lose them, in those words

A copy that turns out to have been a move is discovered a day later, on the
drawing you did not have open. So the caption names the source document and
says *removed*, and it is on screen for as long as Shift is held, before the
button is released.

### `fn drag_landing_copy_with_hint`

One function rather than two joined at the call site, because the joined
result is what the operator reads and `R1` puts *that* in the catalogue. A
`format!("{} {}", a, b)` in a caller is a composition decision — how the two
sentences meet, whether with a space, a dash or a newline — made somewhere
nobody looking for the operator's words would think to look. The gate
caught it.

The hint rides with the copy sentence and only with it: an operator
already holding Shift does not need to be told Shift is available, and a
hint that is always on screen is furniture nobody reads.

### `fn moved_out_of`

It states the undo consequence, and that is the part that cannot be left
out. A cross-document move is **two** edits in two documents, each with its
own undo stack, so one Ctrl+Z reverses one half of it. There is no ordering
of the two commands that makes a single undo mean *"put it back how it
was"*, and an operator who assumes otherwise will undo the insert, see the
pages vanish, and believe the source still has them.

This is why the drag defaults to a copy and the move is the modified
gesture rather than the other way round.

### `fn move_left_the_source_alone`

Its own sentence rather than silence, and rather than the engine's raw
refusal, because the operator is now looking at a state neither of the two
things they asked for: the pages are in both documents. Saying which half
happened is the only way they can finish the job by hand.

It names the **remedy**, because the operator's next act is not guessable
from the refusal: the sheets they wanted moved are sitting in the document
they came from and have to be deleted there, in that document, by hand.

### `fn drag_over_nothing`

Distinct from "it would change nothing", which
`crate::text::pages::drag_lands_nowhere` already says: this one means the
pointer is not over a page list or a page view at all.

### `fn drag_refused_self_copy`

**Refused rather than performed.** It is almost always a mis-drag, the
result is a document with every sheet twice, and undoing it is one keystroke
the operator has to know to reach for. Acrobat's Insert Pages will do it if
you ask in the dialog; nothing does it on a drag.

### `fn drag_target_refused`

One sentence for the three engine refusals a caller cannot do anything
about at drop time — a certified document, an encrypted one, a page tree
that will not walk. The engine's own reason follows it, because it is the
only part that says which.
