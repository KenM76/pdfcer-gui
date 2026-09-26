# `app::recent` — the documents this operator had open, on disk

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
