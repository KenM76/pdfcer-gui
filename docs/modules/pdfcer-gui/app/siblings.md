# `pdfcer-gui/app/siblings`

Moving a document between pdfcer-gui windows. View ▸ Window carries *Move to
new window* and *Move to other window*; a document tab's menu carries both for
that tab; and a tab dragged off the strip does one or the other by where it is
dropped (`drag`). A selection dragged off the canvas onto another window is
pasted there (`objdrop`).

## What moves

The file, not the session. The receiving window opens the path from disk, so a
move is offered only when the screen equals the disk: the document was opened
from a file (`Origin::Opened`), its status is `Open`, it has no unsaved edits,
and the path still names a file. Anything else is refused with a sentence
saying to save first, and nothing is sent.

The tooltips and the refusal say that the other window opens the saved file.
What follows from that, and is not said on screen:

- The page on screen travels; zoom, scroll within the page and selection do
  not.
- A password-protected file asks for its password again in the new window.
- Undo history stays behind with the window it was made in.

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
- A request is `open\t<absolute path>[\t<page index>]\n`, answered `ok\n` or
  `no\t<reason>\n`. The page is zero-based; a page field that is not a number
  refuses the request. `ok` means the path reached the receiver's frame; the
  open then reports its own failures there.
- A paste request is `paste\t<x>\t<y>\n`, a desktop pixel; coordinates that
  are not finite numbers refuse it. `ok` means the request reached the frame.

The server thread hands each path to the frame through a channel and requests a
repaint, so a window that is idle still opens what it is sent.

## Sender's sequence

1. *Move to other window*: no other window — refused; one — sent to it; more
   than one — a small picker window lists each by its documents' names, with
   Cancel.
2. The send runs on a background thread; the frame never waits on a pipe.
3. On `ok`, the tab closes only if it is still movable (an edit made while the
   send was in flight keeps it). A window left with no documents closes itself.
4. *Move to new window* starts the same executable on the path and
   `--page <n>` (one-based), with every `PDFCER_DIAG_*` variable but the
   placement removed and its output discarded, so a harness-placed parent's
   child is also off the desktop and never writes into the parent's trace. The parent then closes the tab.

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

`window-pipe`, `window-peers n= peers=`, `window-move-sending`,
`window-move-sent`, `window-move-refused`, `window-move-received`,
`window-emptied`, `window-torn-off pid= path= page=`, `window-rect pid= rect=`,
`window-tab-dropped slot= at= desktop= onto=` (`onto=0` when no other window
was under the drop), `window-selection-dropped onto= desktop= shift=`,
`window-paste-sent to=`, `window-paste-refused to= why=`,
`window-paste-received pid= desktop=`. `page=` is zero-based, `-1` when none was sent. The picker
declares its buttons as `window-pick.<pid>` and `window-pick.cancel`.

## Not covered

Dropping onto another window whose client area is partly covered by a third
window: the match is by published rectangle, not by what is on top.
