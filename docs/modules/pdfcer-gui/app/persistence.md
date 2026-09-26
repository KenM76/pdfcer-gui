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

## Item notes

### `struct Pending`

Two instants rather than one, because the deadline is the *earlier* of
two independent promises: quiet for [`SAVE_SETTLE`], and never later
than [`SAVE_MAX_DEFER`] after the first unsaved change.

### `fn default`

Exists for one concrete reason rather than for symmetry:
[`crate::app::PdfcerApp`] derives [`Default`], and a field whose type
has no `Default` would break that derive — turning "hold the layout
store on the application" into a refactor of every place an app is
constructed.

It is deliberately the same state as [`StoreKind::None`]: an empty
document, no path, and saving that quietly does nothing. That is the
honest meaning of "a store nobody has loaded", and it is a state the
rest of the module already handles — see
`a_store_with_nowhere_to_write_still_loads_and_runs`. What it must
**not** be is a store pointing at the real layout file with an empty
document, which would overwrite the operator's arrangement the first
time anything armed a write.

### `fn write`

**A failure clears the pending state too**, deliberately. Retrying
every frame against a path that cannot be written — a read-only
share, a full disk — burns the write attempt sixty times a second
to produce the same error, and the operator has already been told.
The next actual change re-arms it, which is the same posture
[`crate::app::PdfcerApp::settle_and_rasterize`] takes towards a page
that would not render.

### `fn the_layout_file_lives_beside_the_settings_file`

The location rule, asserted against `pdfcer-core`'s own resolution
rather than against a path spelled out here — a second spelling is
exactly how a layout file ends up in a different folder from the
settings file it was supposed to sit next to.

### `fn a_first_run_loads_the_fallback_and_says_nothing`

A missing file yields the fallback, records the fact, and reports
nothing worth telling anybody — an application that announced this
would tell every operator on every fresh profile that their layout
could not be restored.

### `fn broken_syntax_falls_back_and_is_reported`

The one genuinely wholesale case — a parser cannot say which half of
a broken file was meant — and it must still be a *disclosure*, never
a dialog and never a silent reset.

### `fn a_dropped_panel_is_not_written_back_into_the_file`

`SHELL_FRAMEWORK.md` §5b from both sides. The loader drops the
unregistered mount and discloses it; the write path emits the
sanitized document, so there is no route by which the dropped id
reappears in the file. Asserted on the *text on disk*, because an
assertion on the in-memory document would pass even if the writer
re-emitted something it had kept aside.

### `fn a_continuous_drag_costs_one_write_rather_than_one_per_frame`

A splitter drag reports a change every frame. Sixty ticks inside the
settle window must produce no writes at all; the write happens once,
after the gesture stops.

### `fn an_endless_gesture_still_gets_written_within_the_ceiling`

A continuous rearrangement re-arms the settle window on every frame,
which without a ceiling would starve the write for as long as the
operator keeps dragging — reintroducing the "only saved at exit"
failure through the mechanism meant to prevent it.

### `fn a_store_with_nowhere_to_write_still_loads_and_runs`

`StoreKind::None` is what `pdfcer-core` returns when neither the
portable directory nor the platform one can be written. Everything
loads from defaults, the dock is arrangeable, and only saving is
impossible — and a caller can find that out before promising the
operator anything.

### `fn a_default_store_points_nowhere_and_can_erase_nothing`

`Default` exists so `PdfcerApp` can keep deriving it. The hazard that
buys is a store that looks loaded, holds an empty document, and
points at the real file — which would erase the operator's
arrangement the first time anything armed a write. It points nowhere
instead.

### `const LAYOUT_FILE`

`.ron` because that is what `egui-shell` serializes a
[`LayoutDocument`] as, and because an operator may open it: RON keeps
real enum names, comments and trailing commas, which a layout file
benefits from for the same reason the shell manifest does.

### `const SAVE_SETTLE`

A splitter drag reports a change on every frame of the gesture, so this
is what turns one gesture into one write. It is short enough that
letting go of a splitter and pulling the power cord a second later
still keeps the arrangement, and long enough that a two-second drag
costs one write rather than a hundred and twenty.

### `const SAVE_MAX_DEFER`

Without a ceiling, [`SAVE_SETTLE`] is re-armed by every change and a
slow, continuous rearrangement can starve the write indefinitely — the
exact failure the "not only at exit" requirement exists to prevent,
reintroduced by the mechanism that was supposed to prevent it.

### `struct LayoutStore`

Held by the application for the whole session. Cheap to construct once
and never again — [`Self::load`] performs the writability probe and one
file read, and nothing after that touches the filesystem except a save.

### `fn load`

**Never fails.** A missing file, an unreadable one, broken syntax
or a newer schema all yield `fallback` with a reason recorded in
[`Self::report`]; a panel `catalog` does not recognise loses its tab
and nothing else. See the module header.

`fallback` is the arrangement a fresh profile starts with — for
pdfcer, [`crate::app::modes::layout_for_build`] of the mode the
application opens in. `catalog` must be the **real** panel registry:
passing [`egui_shell::dock::AnyPanel`] would disable the check that
turns a mount for a compiled-out capability into a disclosed skip
rather than an empty compartment.

### `fn load_in`

The twin of `pdfcer_core::settings::store_in`, and it exists for the
same two reasons: tests, and a future `--user-data-dir` override.
It reports [`StoreKind::Portable`] because an explicitly named
directory is portable by definition — it travels with whatever the
operator pointed at.

### `fn default_path`

Exists so the location convention is assertable: it is derived from
the same `pdfcer_core::settings::resolve_store()` call that decides
where `settings.txt` goes, so the two cannot drift.

### `fn can_save`

`false` means no writable location was found — a state in which
everything else works. A status surface may want to say so **once**,
on the first change the operator makes, rather than at start-up:
nobody cares that their layout cannot be saved until they have
arranged something.

### `fn report`

Returned rather than rendered: the shell has no business deciding
how the application words a note to its operator, and a surface that
wants to offer "remove this stale entry" needs the structured skip
rather than a sentence containing it. Every [`egui_shell::layout::LayoutSkip`]
also implements `Display` for the diagnostic case.

### `fn is_noteworthy`

Forwards to `LoadReport::is_noteworthy`, which excludes "there was
no file" — a first run is not a failure. Deliberately a forward
rather than a re-implementation: one definition of "worth saying".

### `fn document_mut`

**Arms a write.** Handing out `&mut` means this module cannot see
what was changed, so it assumes something was; a caller that takes
the borrow and changes nothing costs one file write. That is the
right way round: a missed write loses an operator's arrangement, and
a spurious one costs a few kilobytes.

### `fn active_mode`

`None` covers three real cases and the caller must treat them alike —
see [`egui_shell::layout::LayoutDocument::active_mode`]. So must an id
this build's manifest no longer declares, which is why the caller checks
`Modes::is_known` rather than trusting what it reads here.

### `fn record_active_mode`

[`Self::record_active`]'s twin, and the equality check is there for the
same reason: `Modes::on_mode_changed` may be driven from the ribbon's
state every frame, so re-recording the mode already recorded must cost
nothing. Without the check, every frame would arm the debounce and the
ceiling in [`SAVE_MAX_DEFER`] would turn an idle application into one
that writes its layout file every five seconds forever.

Returns whether anything changed.

### `fn record_active`

Called when the dock reports
[`egui_shell::dock::DockFrameReport::layout_changed`]. The equality
check is what keeps a caller honest: a frame that reports a change
which nets out to nothing — a splitter dragged one way and back
within the frame, a menu that closed without acting — does not cost
a write.

Returns whether anything changed.

### `fn record_active_at`

The debounce is a **schedule**, and a schedule is only assertable
against a clock the test controls: the alternative is a suite that
sleeps, which is slow, flaky, and still cannot reach
[`SAVE_MAX_DEFER`] without taking five real seconds. Both halves of
the schedule therefore take their instant from the caller — this and
[`Self::tick`] — and a caller that mixes the wall clock into one and
a synthetic instant into the other gets nonsense, which is exactly
why they are spelled differently.

Public rather than test-only because a harness driving the
application frame by frame is a real second caller, and a
`#[cfg(test)]` seam is one such harness cannot use.

### `fn due_at`

The schedule, exposed rather than inferred: a diagnostic surface can
say *"unsaved, writing in 0.4 s"* and a test can assert the deadline
itself instead of guessing at it from the outside.

### `fn tick`

Call once per frame with `Instant::now()`. A `Some(remaining)`
answer is a request for another frame at about that time —
`ctx.request_repaint_after(remaining)` — because nothing else will
wake `egui` on an idle window and the change would otherwise sit
unwritten until the operator moved the mouse. `None` means there is
nothing outstanding.

Takes `now` rather than reading the clock so the schedule is
testable without sleeping.

### `fn flush`

For an exit path, which must not lose the last change to a debounce
that had not yet expired. Returns whether a write was attempted;
[`Self::save_error`] says whether it succeeded.

### `fn save_error`

A real `Result` on the way in, because a failed save is something
the operator must be told about — they are about to close an
application believing their arrangement is safe. Cleared by the next
successful write.
