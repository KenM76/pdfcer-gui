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

## Item notes

### `fn dismiss`

Returns whether the window stays open, which is what
[`PrintDialog::show`] returns in turn — so a `false` from here is the
window going away this frame.

# The one place that knows WHY

Every route sets [`PrintDialog::dismissal`] except the window chrome,
which arrives as egui's own `frame.closed` and is mapped to
[`Dismissal::Revert`] below. Collecting all four here rather than
writing the settings at each button is what makes the decision
auditable: there is one `match`, it is total, and a fifth route added
later stops the crate building rather than silently doing nothing.

# The arguments, and why two of them are not fields

- `closed` is egui's report that the viewport was dismissed, which is
  only true for the frame it happened in and is therefore not state this
  struct could hold.
- `saved_on_commit` is whether the Print press earlier in the same frame
  left the settings persisted. It is a local of that block and is handed
  across rather than stored, because storing it would make it readable
  on frames where no press happened — and a field that is only
  meaningful for part of a frame is a field somebody will read in the
  other part.

An explicit button wins over `closed`. They cannot both be produced by
the same gesture, but a frame carrying a viewport close AND a footer
press should honour the press: it carries a decision and the other does
not.
