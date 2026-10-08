# `pdfcer-gui/app/siblings`

Moving a document between pdfcer-gui windows. View ▸ Window carries *Move to
new window* and *Move to other window*; a document tab's menu carries both for
that tab.

## What moves

The file, not the session. The receiving window opens the path from disk, so a
move is offered only when the screen equals the disk: the document was opened
from a file (`Origin::Opened`), its status is `Open`, it has no unsaved edits,
and the path still names a file. Anything else is refused with a sentence
saying to save first, and nothing is sent.

The tooltips and the refusal say that the other window opens the saved file.
What follows from that, and is not said on screen:

- The receiving window shows the file from its first page; view state does not
  travel.
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
- A request is `open\t<absolute path>\n`, answered `ok\n` or
  `no\t<reason>\n`. `ok` means the path reached the receiver's frame; the open
  then reports its own failures there.

The server thread hands each path to the frame through a channel and requests a
repaint, so a window that is idle still opens what it is sent.

## Sender's sequence

1. *Move to other window*: no other window — refused; one — sent to it; more
   than one — a small picker window lists each by its documents' names, with
   Cancel.
2. The send runs on a background thread; the frame never waits on a pipe.
3. On `ok`, the tab closes only if it is still movable (an edit made while the
   send was in flight keeps it). A window left with no documents closes itself.
4. *Move to new window* starts the same executable on the path, with every
   `PDFCER_DIAG_*` variable but the placement removed and its output discarded,
   so a harness-placed parent's child is also off the desktop and never writes
   into the parent's trace. The parent then closes the tab.

Both commands grey (temporarily unavailable, R9) through two conditions set per
tab: `docs.tear_off` needs more than one document in the window, and
`docs.move_to_window` needs another window to exist.

## Trace

`window-pipe`, `window-peers n= peers=`, `window-move-sending`,
`window-move-sent`, `window-move-refused`, `window-move-received`,
`window-emptied`, `window-torn-off pid= path=`. The picker declares its buttons
as `window-pick.<pid>` and `window-pick.cancel`.

## Not covered

Dragging a tab out of the strip onto another window or the desktop. The two
commands are the route; a drag would call the same sends.
