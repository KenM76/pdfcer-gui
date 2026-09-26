# `ui-verify/checks/dialog_windows`

`dialogs_open_in_their_own_window` — **every dialog is an OS window**, not
only the one the operator complained about.

# What this is for


> *"Print dialogue box doesn't pop up in its own movable window. It is
> locked within the boundaries of the program's window. Like, I just assume
> you've been trained on a million lines of code and software that pops it
> up in its own window."*


# ★★ Why one process per dialog

`PDFCER_DIAG_INVOKE` fires **once** per process, by design: an environment
variable is not an event, and the latch that turns it into one is consumed
the first time it fires. So a check that wants eight dialogs launches eight
processes. That is slower and it is the honest shape — a dialog opened
*after* another dialog is a different state from a dialog opened first, and
this check is about the plain case.

# ★★★ Why it needs no pointer, and why that matters

Every dialog here is reachable by a command id, so the whole check runs
through the diagnostic invoke seam: no clicks, no keystrokes, nothing that
takes the operator's cursor. It is therefore one of the few checks that is
**safe to run on a machine somebody is using**, and it does not skip under
`--no-input`.

★ The price is stated rather than hidden: five dialogs in this directory are
reachable only by a gesture — Insert image (needs a chosen file), Insert
pages, Set scale, the text-annotation editor, and the unsaved-changes
question. They are **not covered here**, and a regression in any of them
would not fail this check. `OPERATOR_REQUESTS.md` records them as
NOT VERIFIED rather than letting a green run imply otherwise.

# The oracle

`viewport-inner`, and there is no other. A screenshot cannot answer this: a
dialog in its own window is **absent** from a capture of the application
window, and an in-viewport panel that regressed would look like a perfectly
good dialog in that same capture. *"Is this a separate OS window"* is a fact
about the window manager, and the only thing in the process that knows it is
the viewport egui created.
