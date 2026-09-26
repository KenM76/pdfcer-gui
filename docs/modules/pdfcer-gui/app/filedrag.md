# `app::filedrag` — **where on the window a file was dropped**

## What this closes


> *"I should be able to drag and drop documents into the thumbnails section
> of another pdf to import the pages."*

[`crate::app::dropped`] already reads dropped files and opens them. What it
cannot do — and says so in its own header — is tell *where* the file
landed: *"`egui` reports drops on the **`Context`**, not on a widget."* A
drop on the Pages panel and a drop on the ribbon are the same event, so
*"drop it onto the thumbnails"* was not expressible.

This module supplies the missing coordinate and the protocol that lets a
surface claim a drop that landed on it.

## The coordinate does not exist in the toolkit, at all

Not "is awkward to get" — is **discarded**, twice, on the way up:

```text
// winit 0.30.13, platform_impl/windows/drop_handler.rs
pub unsafe extern "system" fn DragOver(this, _grfKeyState, _pt: *const POINTL, …)
pub unsafe extern "system" fn Drop    (this, pDataObj, _grfKeyState, _pt: *const POINTL, …)
```

The OLE drop point arrives in `_pt` and is thrown away in both. And the
usual fallback is not available either: **during an OLE drag the window
receives no mouse-move messages**, so `egui`'s `pointer.latest_pos()` is
stale from before the drag started — typically wherever the operator last
clicked, which is exactly the kind of plausible-but-wrong coordinate this
project has been bitten by three times.

⇒ So the position is asked of the operating system directly, once per
frame, through [`native_window::cursor_position`]. That is the same
argument `native-window` was created for: the toolkit will not say
something the platform knows and the operator can see.

## The protocol: a surface CLAIMS a drop; the fallback runs last

A dropped file is recorded at the top of the frame with the point it landed
on, and then sits there:

| when | who | what |
|---|---|---|
| top of frame | [`poll`] | record the drop and the point; keep the hover alive |
| during the frame | any surface | [`aim`] to see if the point is over it, [`claim`] to take it |
| end of frame | [`crate::app::frame`] | [`unclaimed`] — whatever is left means what it always meant |

The **fallback is unconditional**, and that is the property worth
protecting. A drop that no surface claims still opens the document, or
inserts the image, or explains the refusal — exactly as before this module
existed. So a surface forgetting to claim costs a *feature*, never the
drop: the failure mode is "it opened in a new tab instead of inserting",
which an operator can see and undo, rather than a file that vanished.

## One accessor for both phases, deliberately

[`aim`] answers *"where is the file-drag pointer?"* whether the file is
still hovering or has just landed. That is what lets a surface draw its
caret and resolve its drop with **one** piece of geometry code — and the
alternative, two accessors, is a place for the preview and the outcome to
disagree about where the file was going. The caret an operator watched
would then be a promise the drop did not keep.

## Rule 4

Everything drawn from this is a **cursor**: a caret in a gap while a button
(or in this case a file) is held, gone the instant it lands. Nothing here
marks content, tints a page, or draws a second rendering path. The words
half of the disclosure is the status note the insert itself records.

## Item notes

### `const DIAG_DROP_PATH`

Moved here from `app::dropped` unchanged, because the simulated drop now
needs the same *position* the real one has, and the position is this
module's subject.

### `const DIAG_DROP_AFTER_MS`

# Why a delay is the difference between drivable and not

A real drop carries a position, and this feature is entirely *about* the
position — so a check must be able to say **where** the simulated file
lands. It cannot pass a coordinate: the interesting points are tile
rectangles inside a scrolling panel, which do not exist until the
application has laid itself out, long after the environment was read.

So the check moves the **real cursor** over the tile it means — which it
can do, because that is the one thing `ui-verify` is good at — and this
delay gives it time to do so before the drop fires. The position is then
genuinely read from the operating system by the same line of code a real
drop uses.

⇒ The only synthetic part left is the OLE payload, which cannot be
synthesised by moving a mouse and is precisely why the seam exists at all.

### `static FIRST_FRAME`

Set on the first [`poll`] rather than at `main`, so the delay is
measured from the first frame — which is what the check is waiting for
too. Measuring from process start would count the time `eframe` spends
creating a window against a budget meant for the harness's pointer.

### `fn the_fallback_gets_a_drop_no_surface_took`

Written as a sequence rather than as two assertions about one function,
because the property is about the ORDER: a surface claims during the
frame, the fallback runs at the end, and exactly one of them acts.

### `fn a_landing_keeps_every_path`

`app::dropped` decides that only the first is acted on and says so to
the operator. This module must not make that decision early, or the
claiming surface loses the ability to say how many arrived.
