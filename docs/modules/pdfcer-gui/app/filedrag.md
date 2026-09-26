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

## ★★★ The coordinate does not exist in the toolkit, at all

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

## ★★ The protocol: a surface CLAIMS a drop; the fallback runs last

A dropped file is recorded at the top of the frame with the point it landed
on, and then sits there:

| when | who | what |
|---|---|---|
| top of frame | [`poll`] | record the drop and the point; keep the hover alive |
| during the frame | any surface | [`aim`] to see if the point is over it, [`claim`] to take it |
| end of frame | [`crate::app::frame`] | [`unclaimed`] — whatever is left means what it always meant |

★ The **fallback is unconditional**, and that is the property worth
protecting. A drop that no surface claims still opens the document, or
inserts the image, or explains the refusal — exactly as before this module
existed. So a surface forgetting to claim costs a *feature*, never the
drop: the failure mode is "it opened in a new tab instead of inserting",
which an operator can see and undo, rather than a file that vanished.

## ★ One accessor for both phases, deliberately

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
