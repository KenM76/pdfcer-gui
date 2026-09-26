# `ui-verify/checks/preview_popout`

`the_print_preview_pops_into_its_own_window` — **and the print dialog's own
preview column goes away when it does.**

# ⚠ THIS CHECK HAS NEVER BEEN RUN


# The request


> *"also the preview should be adjustable size, and even better if it has
> the option to pop out into its own resizeable window - closing the window
> pops it back into place on the print window."*

Ask 1 shipped the same day. This is ask 2.

# ★★★ WHY THE OBVIOUS CHECK IS WORTHLESS, and what this does instead

The obvious check is *"press Pop out, and assert a second window appeared"*.
It passes on a build that opens the pop-out window **and goes on drawing the
preview column too** — two previews of one sheet, a 340 pt duplicate the
operator did not ask for, and precisely the shape R9 forbids. A presence
assertion cannot see that, because everything it looks for is present.

So the load-bearing assertion here is an **absence**: after the click, the
`print.preview.column` region must be **retired**. `diag::end_ui_frame`
publishes `ui-rect-gone name=…` for every region drawn last frame and not
this one, which is exactly the event that says a surface stopped existing.

## ★★ And an absence assertion is vacuous unless the run is DRIVEN into the
## state where the absence is the claim

A check that merely asserted *"`print.preview.column` is not declared"*
would pass against a build with no print dialog at all, against a launch
where the dialog failed to open, and against a fixture with no printers. All
three are absences, and none of them is the feature working.

Hence the pairing, which is the whole design:

| position | asserted |
|---|---|
| before the click | the column IS declared, and `print-body … popped=false` |
| after the click | the column is RETIRED, the popped window's body is declared, and `print-body … popped=true preview_w=0.0` |

Neither half alone is worth anything. The first proves the run reached the
state the second is about; the second proves the state changed.

## ★ The width is asserted as a RELATIONSHIP, never a value

`options_w == content_w` while popped — the options take the whole room —
rather than "the options are N points wide". Every width in this dialog
depends on the theme preset's font and button padding, so a constant here
would be a claim that decays, which this project has spent six corrections
on. `preview_w` is compared against zero, which is not a measurement but the
literal the code writes.

# The return trip

*"Closing the window pops it back"* is `Frame::closed`, which G4 makes the
OS close button **and** Escape together. This check presses Escape at the
popped window and asserts the preview comes home: `print-preview-popped
state=in`, and the column's region declared again.

★★ That half **degrades to a skip rather than a failure** when the popped
window did not have the keyboard. Focus is a window-manager question this
harness has been wrong about before — nine checks skipped in one sweep on a
stray `OpenWith.exe` holding the foreground — and reporting *"closing the
window does not put the preview back"* because a toast stole focus would be
a confident, wrong failure about working code. The message says which of the
two it is.

# What this does NOT establish

* **That it looks right.** No pixels are read. The sheet could be drawn at
  the wrong scale in the popped window and every assertion here would pass.
  The only oracle for that is a rendered frame, and there is not one.
* **That the window is resizable, or opens at a sensible size.** Those are
  `dialogs::host`'s, shared with thirteen other dialogs.
* **That the operator can find it again.** `with_taskbar(true)` is the
  host's and is asserted nowhere.

# What a first run will probably teach it

Two guesses, written down now so that a red is read against them rather than
against nothing:

1. **The click may need the strip in view.** The Pop-out button is the last
   control in a `horizontal_wrapped` row, so on a narrow dialog it wraps to
   the second line. `REGION_POP_OUT` is published with the
   visibility-gated publisher for exactly this reason, so a button off the
   clip rectangle declares no region and this check will say so rather than
   clicking nothing — which is the failure `dialogs::formfield`'s rotation
   row records.
2. **Escape may reach the print dialog instead.** Both windows are children
   of the same process and the harness aims at a `WindowHandle`. If the
   return half skips for that reason, the fix is to aim the key at the
   popped window's own handle, not to weaken the assertion.
