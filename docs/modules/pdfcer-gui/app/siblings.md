# `pdfcer-gui/app/siblings`

Moving a document between pdfcer-gui windows. View ▸ Window carries *Move to
new window* and *Move to other window*; a document tab's menu carries both for
that tab; and a tab dragged off the strip does one or the other by where it is
dropped (`drag`). A selection dragged off the canvas onto another window is
pasted there (`objdrop`).

## What moves (`parcel`)

The document as it is on screen. A move is offered when the document was
opened from a file (`Origin::Opened`), its status is `Open`, the path still
names a file, and no redaction is staged. A document with no file says to
save first; a staged redaction says to save or cancel it first, because the
content it removes must not leave the window. Nothing is sent in either case.

What travels:

- The path and the page on screen; zoom, scroll within the page and selection
  do not.
- With unsaved edits, a copy written through the save funnel
  (`save::carry_copy`) into `carry/` in the discovery directory, and the count
  of unsaved edits. The receiver reads the copy in place of the file
  (`open_arrival`), binds the document to the original path, deletes the copy
  and holds the count in `OpenDoc::carried_edits`, which `save::has_unsaved_edits`
  and `save::unsaved_edit_count` include until the next save. Save then writes
  the carried revision over the original.
- The password the document was opened with (`OpenDoc::opened_with`), so the
  receiver does not ask for it. It crosses only the pipe.

The undo history stays behind: the engine cannot hand a session's steps to
another process (G184). The receiver says so on the status line, and the
commands' hover text says so before the move.

Carry files are deleted by the receiver once read, by the sender when the send
fails, and by any window starting up once older than ten minutes.

## Transport (`wire`)

One pipe and one discovery file per window:

- The window listens on `pdfcer-gui-window-<pid>` through `native-pipe`, which
  restricts it to the current user.
- It names the pipe and its open documents in `<settings dir>/windows/<pid>.txt`
  (written to a temporary name, then renamed). Windows of one installation see
  each other; a copy in another folder (the ui-verify sandbox) sees none of
  them.
- Files older than the prune age whose pipe no longer exists are removed by
  whichever window reads the directory next.
- The discovery file also carries the window's client area in desktop pixels,
  `rect=<left>,<top>,<right>,<bottom>`, rewritten when it moves or resizes.
- A request is
  `open\t<absolute path>[\t<page index>|-[\t<name>=<value>]...]\n`, answered
  `ok\n` or `no\t<reason>\n`. The page is zero-based, `-` when none is sent but
  named fields follow. The named fields are `carry=<absolute path>`,
  `edits=<n>` and `key=<hex of the password's UTF-8>`; an unknown name, a
  relative carry, a bad number or a bad hex refuses the request. A key is
  never traced. `ok` is answered only once the receiving frame has the
  document open; a failure there is answered `no` with its reason, and no
  answer within 30 s is answered `no` by the receiver's own server thread.
- A paste request is `paste\t<x>\t<y>\n`, a desktop pixel; coordinates that
  are not finite numbers refuse it. `ok` means the request reached the frame.

The server thread hands each request to the frame through a channel and
requests a repaint, so a window that is idle still opens what it is sent; for
an open it then waits for the frame's outcome before answering.

## Sender's sequence

1. *Move to other window*: no other window — refused; one — sent to it; more
   than one — a small picker window lists each by its documents' names, with
   Cancel. The document is packed (carry written) when the send starts.
2. The send runs on a background thread; the frame never waits on a pipe.
3. On `ok`, the tab closes only if its edit epoch is the one packed: an edit
   made while the send was in flight keeps it, because the other window has the
   document without that edit. A window left with no documents closes itself.
   On `no`, the reason is said here and the carry file is deleted.
4. *Move to new window* on a document with neither edits nor a password starts
   the same executable on the path and `--page <n>` (one-based) and closes the
   tab. With either, it starts the executable with no document, waits up to
   30 s for the child's pipe, and sends the parcel as in 1–3. Either way every
   `PDFCER_DIAG_*` variable but the placement is removed and the child's output
   discarded, so a harness-placed parent's child is also off the desktop and
   never writes into the parent's trace.

Both commands grey (temporarily unavailable, R9) through two conditions set per
tab: `docs.tear_off` needs more than one document in the window, and
`docs.move_to_window` needs another window to exist.

## Dragging a tab out (`drag`)

The tab strip (`egui-shell`) raises `DragOut` instead of a reorder when a tab
is released more than one strip-height above or below the strip, or beyond its
ends; while the pointer is out there it draws the tab's label at the pointer.
The release point is in this window's points and may lie outside the window,
because the platform keeps reporting the pointer to the window that holds the
press.

- Released outside this window's client area and inside another window's
  published `rect` — the document is sent there, as *Move to other window*
  does, with the same refusal for an unmovable document.
- Released anywhere else, including on this window's own canvas — it tears off
  into a new window, as *Move to new window* does, if this window has another
  document. The window's only document stays.

Conversion to desktop pixels multiplies by `pixels_per_point`, which includes
the operator's interface scale.

## Dropping a selection on another window (`objdrop`)

A move drag of the selection released outside this window's client area and
inside another window's published `rect` is not a move. On the release frame
the canvas skips the commit and records the drop; the next frame copies the
selection to the clipboard, as Copy does, with the same partial-copy
disclosure, and sends `paste` with the release point. The receiver pastes as
Paste does, with the release point standing in for the pointer: the same mode
gate, the same adoption of the other window's clip, and the viewport's centre
when the point is not over its canvas.

- A drag copies; Shift held at the release moves. A move cuts the selection
  here once the receiver has answered `ok`, and only if the document is the
  same and unedited since the copy.
- A refusal from the receiver is said here; nothing changes here.
- The peers' rectangles reach the canvas through the context, written each
  frame by `siblings_poll`, because the canvas cannot see the window state.

## Arriving at a page

A page named at open (`show_page`, from the request or from `--page`) sets the
view's page before the canvas has laid out. Two rules in the canvas keep it:

- The page-change scroll waits for the seed frame (`canvas::offset`), which it
  outranks. Spent on frame 0, it would be overwritten a frame later by the
  open-seed placement, which centres the page instead.
- Scroll tracking (`canvas::strip::track_current_page`) leaves the page alone
  while a navigation is unspent (`page_index != tracked_page`), so the view's
  position before it moves does not overwrite the page it is moving to.

## Trace

`window-pipe`, `window-peers n= peers=`,
`window-move-sending to= path= page= carried= key=`, `window-move-sent`,
`window-move-refused`, `window-move-received pid= path= page= carried= key=`,
`window-move-opened pid= ok= unsaved= why=`, `window-emptied`,
`window-torn-off pid= path= page= carried=`, `window-rect pid= rect=`,
`window-tab-dropped slot= at= desktop= onto=` (`onto=0` when no other window
was under the drop), `window-selection-dropped onto= desktop= shift=`,
`window-paste-sent to=`, `window-paste-refused to= why=`,
`window-paste-received pid= desktop=`. `page=` is zero-based, `-1` when none was sent. The picker
declares its buttons as `window-pick.<pid>` and `window-pick.cancel`.

## Not covered

Dropping onto another window whose client area is partly covered by a third
window: the match is by published rectangle, not by what is on top.
