# `pdfcer-gui/dialogs/print/dismissal`

**Why the print window is closing, and what that does to the settings** —
`OPERATOR_REQUESTS.md` **O185**, the operator's words of 2026-09-14: *"I set
the printer up, close the window to go check something, and it's all
gone."*

# The problem this file exists to hold

Until O166 the print window had no state worth keeping: it described one
job, the job went to a spooler, and closing the window threw away a
description that had already been consumed. Leaving was not a decision.

O166 gave it a **persistent** subset — [`crate::app::prefs::PrintPrefs`],
the settings that describe the *operator* rather than the document — and
from that moment leaving the window was a decision about that subset, taken
every single time, with no way to say which decision was meant. The one
button called *Close* had to stand for both *"keep what I set up"* and
*"forget what I was doing"*, and it picked one.

# Why it is a file rather than four lines in [`super`]

Because the argument is long and it is not an argument about printing. It
is about `dialogs.md` **G4** — the rule that makes the OS close button,
Escape and the cancel button deliberately indistinguishable — meeting a
fourth route that G4 never contemplated, and about which of the two possible
meanings is safe enough to put behind a gesture people make without
deciding anything. None of that belongs in the middle of a function whose
subject is laying out a print job, and `super`'s own header says so about
its other split-out modules for the same reason.

R2 forced the timing — `mod.rs` passed 1,500 lines the moment this work
landed — but the seam was already here. A file that grows past the limit is
usually a file that acquired a second subject, and the limit is only how
anybody notices.
