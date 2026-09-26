# `app::persistence` — the dock layout, on disk

`egui-shell` can already read and write a [`LayoutDocument`]; what it
deliberately does **not** do is decide *where* the file lives or *when*
it is written. Its own header says so in as many words:

> It does not decide *when* to save, and it does not choose a path. An
> application saves on `DockFrameReport::layout_changed` and picks its
> own location — which for this project means a named partition of the
> distribution folder rather than a platform app-data directory.

This module is pdfcer's answer to both questions, and nothing else. It
owns one type, [`LayoutStore`]: a path, a document, the report from the
load that produced it, and a small amount of write-scheduling.

## Why a rearrangeable layout that forgets is worse than a fixed one

`MODES_AND_PANELS.md` Part 2's build order opens with the argument, and
it is the whole reason this module exists before anything else in the
dock is made more flexible:

> **(f) persistence** — cheapest, highest value, unblocks everything.
> *A rearrangeable layout that forgets itself each restart is worse than
> a fixed one.*

Worse, not merely less good. A fixed layout costs an operator nothing
after the first day; a rearrangeable one that forgets charges them the
rearrangement every single session, and teaches them not to bother.

## Where the file lives, and why it is not this module's decision

`<the settings directory>/layout.ron` — **beside `settings.txt`**, and
the directory is resolved by asking `pdfcer-core` rather than by
computing it here:

```text
pdfcer_core::settings::resolve_store()      →  StoreLocation { path, kind }
                       .directory()        →  <exe dir>/userdata      (Portable)
                                           or the platform config dir (PlatformFallback)
                                           or None                    (None)
```

Three properties follow from deferring to that call, and every one of
them would be lost by a second implementation here:

1. **One location decision, not two.** `ARCHITECTURE.md` §6's
   single-folder-portable posture is enforced by the *ordering* inside
   `resolve_store` — portable first, platform config only as a fallback
   — and `pdfcer-core`'s own docs end that paragraph with *"do not
   reverse it"*. A layout file that resolved its own directory could
   reverse it by accident and nothing would notice until an operator's
   settings and layout ended up in different folders.
2. **The writability probe comes with it.** `resolve_store` does not
   assume the executable's folder is writable, it creates and removes a
   temporary file to find out — because a program that assumes it can
   write beside itself *"works perfectly on the developer's machine and
   fails the first time someone installs it under `Program Files`"*.
   That probe is exactly as necessary for a layout file as for a
   settings file, and it is not worth writing twice.
3. **[`StoreKind::None`] stays a usable state.** No writable location
   at all is not a failure to start: the defaults load, the session
   runs, the dock is arrangeable, and only saving is impossible. See
   [`LayoutStore::can_save`].

What is deliberately **not** shared with the settings file is the file
itself. `settings.txt` is a flat `key = value` grammar written by hand;
a layout is a tree, and RON is what `egui-shell` serializes it as.
Two files, one directory.

## When it is written

On change — [`egui_shell::dock::DockFrameReport::layout_changed`] —
**debounced**, and with a ceiling on the debounce. Three requirements
pull in different directions and the schedule satisfies all three:

| Requirement | What it rules out |
|---|---|
| not every frame | a splitter drag reports a change on every frame of the gesture; writing per frame is one file write per frame of a drag |
| not only at exit | *"a crash must not cost the arrangement"* — and the benchmarked application's file being *"rewritten on every exit"* is precisely what makes its community's copy-the-file-aside workaround race |
| eventually, unconditionally | a debounce that is re-armed by every change can be starved forever by a slow continuous drag |

So: a change arms a deadline at `last change + `[`SAVE_SETTLE`], capped
at `first unsaved change + `[`SAVE_MAX_DEFER`]. [`LayoutStore::tick`]
writes when the deadline passes and otherwise reports how long is left,
so the caller can ask `egui` for a repaint then — without which an idle
window would sit on an unsaved change until the operator happened to
move the mouse. That is the same shape, for the same reason, as the
zoom debounce in [`crate::app::state`], which schedules its own wake-up
with `request_repaint_after`.

## Fail-soft, and disclosed — but not popped up

Reading **never fails**: `LayoutDocument::from_ron` drops what it cannot
use, item by item, and returns a [`LoadReport`] saying what went. This
module carries that report rather than consuming it, because the surface
that should say so is the status bar and this is not it. Two rules the
caller inherits:

- **A missing file is a first run and is not news.**
  `LoadReport::is_noteworthy` already excludes it, and
  [`LayoutStore::is_noteworthy`] forwards to that rather than
  re-deciding. An application that announced every skip would tell every
  operator, on the first launch of a fresh profile, that their layout
  could not be restored — from a profile that never had one.
- **Never a dialog.** A layout is not worth interrupting anybody for.

## A dropped panel is never written back

`SHELL_FRAMEWORK.md` §5b: *a capability's presence is expressed by
registering it, and by nothing else.* A build compiled without some
capability registers no panel for it, so a saved layout naming that
panel loses **that tab** on load, with a
[`egui_shell::layout::LayoutSkipReason::UnknownPanel`] disclosing it,
and keeps everything else.

The half that belongs to *this* module is the write path: what is saved
is [`LayoutStore::document`], which is the **sanitized** document — the
one the loader already pruned — updated from a live
[`egui_shell::dock::DockState`] that by construction cannot contain a
panel the dock never drew. There is no path by which an id that was
dropped on load can reappear in the file, and
`a_dropped_panel_is_not_written_back_into_the_file` is the test that
says so.

The honest consequence, stated rather than discovered: **the entry is
then gone for good.** Running a reduced build once and rearranging
anything costs the full build's tab for that capability, permanently.
The alternative — preserving unknown ids and re-emitting them — was
rejected because it makes the file accumulate entries nothing can ever
validate, and because a stale id that survives a *rename* would then
outlive every migration.
