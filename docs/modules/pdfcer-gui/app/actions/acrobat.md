# `app::actions::acrobat` — the two halves of handing the document over

`OPERATOR_REQUESTS.md` **O122**. Two functions and one ordering.

| | Runs when | Does |
|---|---|---|
| [`PdfcerApp::apply_open_in_acrobat`] | the operator presses the control | asks the right question, and **nothing else** |
| [`PdfcerApp::resume_after_open_in_acrobat`] | the operator answered *proceed* | saves if needed, launches, then closes |

## The ordering in the second half, which is the whole of the risk

**Save → launch → close.** Not save → close → launch, and the difference is
everything.

`Command::spawn` can fail after every check has passed: the executable was
removed since discovery, the operator lacks permission, the process table is
full. If the document were closed first, that failure would leave the
operator with **no document on screen and no Acrobat**, having pressed one
button — a state indistinguishable from the application losing their work,
and reached by a code path that had already done everything right.

Launching first means a failure leaves the document exactly where it was,
open, saved, with a sentence on the bar saying Acrobat would not start and
where the path is set. See [`crate::text::acrobat::launch_failed`].

The cost of that order is a window in which both programs have the file:
between `spawn` returning and `close_document` running, which is one
function call and no frame. Acrobat cannot have opened, read and written
the file in that interval — and even if it somehow had, pdfcer's close
writes nothing. The alternative's failure mode is losing the operator's
document off their screen; this one's is theoretical.

## The save is in-place and its failure stops everything

`crate::app::save::save_in_place` is the same verb `file.save` uses, so the
bytes written here are the bytes that command writes — and it materialises
the whole replacement in a temporary beside the target and renames, so a
failure has touched nothing the operator owns.

A save that did not happen must **never** be a route to the thing it was
supposed to make safe. So a failed save stops the sequence dead: no launch,
no close, and a sentence saying so. That is the rule
[`crate::dialogs::unsaved`] states about its own resume — *"a save that did
not happen must never be a route to discarding the work it was supposed to
preserve"* — applied to a sequence where the discarding would be done by
another program.

## Item notes

### `fn apply_open_in_acrobat`

# Why every refusal here is silent except the one the operator can act
on

Two of the states this declines in are ones the ribbon already
prevents: no document open (the command carries
`enabled_when("doc.open")`) and no Acrobat (the item carries
`visible_when("acrobat.available")`, so the control is not drawn). They
are handled anyway, because a customized keymap can reach any command
from any state and an arm that assumed otherwise would be a panic
waiting for an operator to find — but they are traced rather than
worded, because there is no surface the operator could have pressed and
therefore nothing to explain.

A document that has never been saved **is** worded, in its own window,
because the operator did press a control that was there and enabled,
and a control that does nothing on press is the defect this project
keeps finding.

### `fn resume_after_open_in_acrobat`

Called from the frame loop immediately after the dialogs draw, beside
[`PdfcerApp::resume_after_unsaved`] and for that function's reason: it
is not a command but a frame-level observation that a window has been
answered, and the acts it authorises — a write, a close and a process
launch — belong to the application rather than to a dialog.

See this module's header for why the order is save → launch → close.
