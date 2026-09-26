# `recentfiles` — the documents this operator had open, on disk

`GUI_ROADMAP.md` Phase 3 asks for a recent-files list. Nothing in the
salvaged shell wrote one — `grep -i recent` over the old crate returns
nothing at all — so this is new rather than carried across, and every
decision below is therefore made here for the first time and written
down.

One type, [`RecentFiles`]: a capped list of absolute paths, newest
first, persisted beside the layout and the settings, plus the small
amount of caching that keeps *"is this file still there?"* from costing
a blocking stat per frame.

## Where the file lives, and why that is not this module's decision

`<the settings directory>/recent.txt` — **beside `settings.txt` and
`layout.ron`**, and the directory is resolved by asking `pdfcer-core`,
exactly as [`crate::app::persistence::LayoutStore`] resolves it:

```text
pdfcer_core::settings::resolve_store()      →  StoreLocation { path, kind }
                       .directory()        →  <exe dir>/userdata      (Portable)
                                           or the platform config dir (PlatformFallback)
                                           or None                    (None)
```

There is deliberately no second resolution here, and
[`tests::the_recent_file_lives_beside_the_layout_file`] asserts it against
`LayoutStore`'s own answer rather than against a path spelled out twice.
The three properties that come with deferring are the ones
`persistence`'s header argues in full: one location decision rather than
two, the writability probe comes with it, and *no writable location at
all* stays a working session in which only saving is impossible (see
[`RecentFiles::can_save`]).

## Why a flat text file rather than RON

Because this crate **cannot serialize**. `serde` and `ron` are workspace
dependencies of `egui-shell`, not of `pdfcer-gui`, and `Cargo.toml` is not
this work's to edit. `LayoutStore` gets RON for free because
`egui_shell::layout::LayoutDocument` does its own serialization behind
the crate boundary; a list of paths has no such type and inventing one in
`egui-shell` would teach the reusable shell about a pdfcer feature, which
`tools/gates/check-shell-purity.sh` exists to prevent.

So the format is the one `settings.txt` already uses in spirit: **one
path per line, newest first, UTF-8, no header and no comments.** Every
non-empty line is a path; nothing else is legal, so nothing else has to
be parsed. It is readable, hand-editable, diffable, and impossible to
half-parse — a corrupt file degrades into a shorter list rather than into
an error the operator has to dismiss.

The one thing the format cannot carry is a path that is not valid
Unicode. Such a path is **dropped at save time**, and that is the single
place in this module where dropping at save is right: it is not a
judgement about whether the file still exists, it is the format saying it
cannot spell the name. See [`RecentFiles::render`].

## Missing files are dropped at DISPLAY time, never at save time

This is the rule the whole presence cache exists to serve, and it is a
statement about how drafting offices actually work:

> A network drive that is temporarily absent is not a file the operator
> wants forgotten.

A laptop off the VPN, a NAS still spinning up, a share that reconnects
when the operator logs in — every one of those makes `Path::exists`
return `false` about a file that is perfectly real and will be back in a
minute. Pruning the list on that answer destroys information the operator
cannot get back, permanently, in exchange for a tidier file nobody reads.

So [`RecentFiles::entries`] is the whole list, [`RecentFiles::render`]
writes the whole list, and only [`RecentFiles::present_at`] — what the
menu draws — filters. Reconnect the drive and the entry comes back by
itself.

## …and why the presence check is cached

`Path::exists` on a **dead** network path is not fast. It is a blocking
call that can take seconds while SMB waits out its own timeout, and the
UI thread is the thread that would be waiting. Ten entries checked on
every frame of a 60 Hz repaint is not a performance question, it is a
frozen application.

Two defences, and both are needed:

1. **The check only runs while the menu is open.** The caller is
   [`crate::app::PdfcerApp::ribbon_band`]'s custom-item renderer, which
   calls this from inside the popup body — a closure `egui` runs only
   while the popup is showing.
2. **And then at most once per [`PRESENCE_TTL`].** A popup held open for
   three seconds is two sweeps, not a hundred and eighty.

The residual cost is honest and stated rather than hidden: opening the
menu with a dead network entry in the list can block for as long as the
filesystem takes to answer, once every two seconds. Fixing that properly
means checking on a worker thread, which is a real answer and a larger
one than this feature is worth today.

## What is deliberately NOT here

- **No "clear recent" command.** It would be one more registered command,
  one more manifest entry and one more surface, and the file is one line
  per entry in a folder the operator already owns. It is listed in
  `crate::shell::manifest::PLANNED` when somebody wants it.
- **No pinning.** Same argument, and pinning needs a second field in the
  file format, which is the point at which "one path per line" stops
  being the right format.
- **No per-document view state** (last page, last zoom). That is a
  different feature with a different lifetime and a different file; a
  recent list that quietly became a session store would be the thing
  nobody could later separate.

## Item notes

### `fn parse`

Blank lines are skipped and the result is capped and de-duplicated,
so a hand-edited file cannot produce a list this module would not
itself have written. Only the line ending is trimmed — a leading or
trailing space is a legal part of a file name on some platforms, and
trimming it would silently rename the operator's file.

### `fn render`

A path that is not valid Unicode is **dropped here**, which is the one
place in this module where dropping at save time is correct: it is not
a judgement about whether the file still exists, it is the format
admitting it cannot spell the name. The alternative — a lossy
conversion — would write a path that names a *different* file, or no
file, and would look exactly like a real entry when it came back.

### `fn write`

**A failure clears nothing and retries nothing.** The next `remember`
tries again, which is the same posture
[`crate::app::persistence::LayoutStore`] takes towards a path
that cannot be written: retrying a read-only share on a schedule
produces the same error at a cost, and the operator has already lost
nothing they can see.

### `fn the_recent_file_lives_beside_the_layout_file`

Asserted against `LayoutStore`'s own resolution rather than against a
path spelled out here — a second spelling is exactly how two files
that were supposed to share a directory end up in different ones, and
`persistence`'s header explains why the directory decision belongs to
`pdfcer-core` and to nothing else.

### `fn the_newest_document_is_first_and_a_repeat_open_moves_it`

The property that makes this a most-recently-used list rather than a
log of opens. Without the move-to-front, opening the same drawing
twice fills the menu with one file.

### `fn a_missing_file_leaves_the_menu_and_stays_in_the_file`

The rule the module header argues: a network drive that is temporarily
absent is not a file the operator wants forgotten. Asserted from both
ends — the menu does not offer it, and the *file on disk* still holds
it — because an assertion on the in-memory list alone would pass even
if the save path had quietly pruned it.

### `fn the_presence_answer_is_reused_until_the_window_passes`

`Path::is_file` on a dead network path blocks for as long as the
filesystem takes to give up, on the UI thread. Without the throttle
the menu would take that answer on every frame it is open, which is
not slowness but a frozen application.

Asserted by removing a file and watching the answer *not* change until
the window has passed — the only way to prove a cache is a cache.

### `fn a_default_store_points_nowhere_and_writes_nothing`

`Default` exists so `PdfcerApp` keeps deriving it, and because
`PdfcerApp::new` deliberately uses it under `cfg(test)` so a unit test
that opens a fixture cannot scribble fixture paths into the operator's
own list. The hazard it must not have is a store that looks loaded and
points at the real file.

### `fn opening_a_document_records_it_and_a_failed_open_does_not`

The recording lives in [`crate::app::PdfcerApp::open_path`] — the one
function that opens documents, and the one `argv` reaches without an
action, so the first document of a session is recorded too.

The negative half is the decision worth pinning: a file that would not
open is not a *document* the operator had, and offering it from a menu
whose whole promise is "this worked before" invites the same failure
from the one surface that should be reliable.

### `fn the_recent_command_opens_the_parked_choice_or_the_newest_reachable`

Two routes into one command, which is the whole reason the menu is a
custom *item* rather than a command of its own: the item asks which,
the command acts. The fallback is not a guess — it is the defined
answer for an invocation that carries no operand, which an operator
reaches by binding a chord or adding the command to their quick-access
toolbar, neither of which draws a menu.

### `const RECENT_FILE`

`.txt` because the format is one path per line and an operator who opens
it should find exactly what they expect. `.ron` would promise a structure
that is not there and a parser this crate does not have.

### `const CAP`

Ten is what a File menu can show without becoming a scroll view, and it
is comfortably more than the two or three an operator actually reaches
for. A cap exists at all because the list is drawn: an uncapped list
eventually costs a stat sweep per open and a menu taller than the window.

### `const PRESENCE_TTL`

Short enough that a drive that comes back appears in the menu on the next
look, long enough that holding the menu open does not re-stat a dead
network path on every frame. See the module header.

### `struct RecentFiles`

Held by [`crate::app::PdfcerApp`] for the whole session. Cheap to
construct once — [`Self::load`] performs `pdfcer-core`'s writability probe
and one file read — and nothing after that touches the filesystem except
a save on [`Self::remember`] and the throttled presence sweep in
[`Self::present_at`].

### `fn load`

**Never fails.** A missing file is a first run, an unreadable one is
an empty list, and a line that is not a usable path is skipped. There
is nothing in a recent-files list worth interrupting an operator for,
and there is nothing in it that cannot be rebuilt by opening a
document.

### `fn load_in`

The twin of `pdfcer_core::settings::store_in` and of
[`crate::app::persistence::LayoutStore::load_in`], and it exists for
the same two reasons: tests, and a future `--user-data-dir` override.

### `fn default_path`

Exists so the location convention is assertable against
`LayoutStore`'s, which is what stops the two files from drifting into
different folders.

### `fn entries`

This is what a save writes and what a count reports. What a menu draws
is [`Self::present_at`]; the difference between the two is the whole
of the module header's display-time rule.

### `fn is_empty`

Answered without touching the filesystem, deliberately: it is what
decides whether the Recent control is enabled, and that decision is
made on every frame the File tab is drawn. A list whose every entry is
currently unreachable still enables the control, and the menu then
says so — which is a better answer than a control that greys out
because a drive is slow.

### `fn present_at`

What a menu draws, and the only method that touches the filesystem
after load. Throttled to one sweep per [`PRESENCE_TTL`]; the caller
supplies `now` so the throttle is assertable without sleeping, exactly
as [`crate::app::persistence::LayoutStore::tick`]'s deadline is.

Returns owned paths rather than a borrow because the caller is a
drawing closure that also needs `&mut self` for the sweep, and because
the vector is at most [`CAP`] entries long.

### `fn newest_present`

What `file.recent` opens when it is invoked without the menu having
chosen an entry — from a rebound chord, or from a quick-access
toolbar an operator has customized. See
[`crate::app::PdfcerApp::dispatch_command`].

### `fn menu`

Called from [`crate::app::PdfcerApp`]'s custom-item renderer for the
`recent_files` item in File ▸ File — the extension point
`egui_shell::manifest::Item::Custom` exists for, whose own doc names *"a
split button with a gallery"* as the case. The shell reserves the space
and hands back a `Ui`; what goes in it is the application's business,
which is exactly the seam that keeps a reusable shell from growing a
`RecentFiles` variant.

# It reports, and changes nothing

Returning the path rather than opening it is the actions-not-mutations
invariant reaching one control further out: this is a widget, it is being
drawn mid-frame, and `crate::app::actions`' rule is that **no code path
runs from a widget to a document**. The caller parks the answer, returns
the `file.recent` handler token, and the command goes through
`PdfcerApp::dispatch_command` — the same choke point a ribbon click, a
chord and a context-menu row all reach. The menu is to `file.recent` what
the file dialog is to `file.open`: the operand picker, not the verb.

# Where the words come from

The button's label and hover text are **the command's**, read from
[`crate::text::commands::file_recent`], for the reason
`crate::shell::menus`' header gives about menu rows: a second copy of a
command's words is a second copy that can drift. Only the things the
command's text cannot express — the rows, which are file names, and the
empty state — come from [`crate::text::files`].

# Greying, and what it is allowed to mean

`RIBBON_IA.md` P3 reserves greying for *temporarily* unavailable, always
explained on hover. An operator who has never opened a document is exactly
that, so the button is disabled and both hover texts are attached — plain
and disabled — because `egui` shows a different one in each state and
attaching only the first is how a greyed control becomes unexplainable.

A list that is non-empty but every entry of which is currently unreachable
is **not** greyed: the answer would depend on how a network share happens
to be feeling, and a control that greys itself out because a drive is slow
teaches the operator it is broken. It opens, and says so in a row.


`catalog::file`'s registration for `file.recent` carries `.with_icon("recent")`
and, beside it, a paragraph saying in as many words what was true until
today:

> *"this command's ribbon control is not a `Button`, it is the
> `recent_files` custom item, and `app::recent::menu` draws it with
> `ui.menu_button(text.label, …)` — application code that reads the
> command's LABEL and never consults its icon key. … what the operator sees
> in File ▸ File does not change until that custom item is taught to paint
> it — which is a change in `app::recent`, not here."*

This is that change. It was found by driving rather than by reading: the
approved mockup draws the control as `['Recent','recent',{menu:1}]` — an
icon **and** a word — and `tools/compare-mockup-ribbon.py` could not see the
difference, because a `Custom` item carries no command id for it to resolve
an icon key from. What made it visible was resolving both sides to the
**asset** each one draws and giving the custom kinds a declared glyph.

⇒ **A comment that names the file where the rest of the work lives is
the best available substitute for a mechanism, and it is still not one.**
That sentence was correct, precise, and sat unactioned; what moved it was an
instrument that compares the two pictures.

[`egui::Ui::menu_image_text_button`] rather than a hand-built
`Button::image_and_text`: the menu behaviour — the popup, the close
semantics, the submenu arrow when nested — is `egui`'s and must not be
re-implemented for the sake of an icon.
