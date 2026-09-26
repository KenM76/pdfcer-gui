# `pdfcer-gui-base/ocr/job`

## Item notes

### `struct Job`

Held by `pdfcer-gui`'s `dialogs::ocr` for exactly as long as one job takes. See
the module header for why this is a thread and why it carries neither a
cancellation token nor a staleness key.

### `struct Reporter`

One type rather than two arguments, because they are one relationship: the
worker reports upward and is told downward, and a function that took only
the sender could not honour a Stop.

Sending is deliberately allowed to fail and is ignored. A dropped receiver
means the dialog is gone; the run then finishes or is abandoned on its own
terms and nobody is listening either way. Treating it as an error would turn
"the operator closed the window" into a reported fault.

### `fn spawn`

The thread is detached rather than joined: nothing the UI does depends
on it finishing, and if the dialog is closed first the channel's
receiver drops, the send fails harmlessly, and the thread exits when
the work it was already doing completes. The alternative — joining on
close — would freeze the window for exactly as long as the operation
this thread exists to keep off the window.

### `fn poll`

Non-blocking, and idempotent after the answer has been taken: `done`
stops a second call reading a disconnected channel and reporting the
disconnection as a refusal.
