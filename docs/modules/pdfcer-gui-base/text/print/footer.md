# `pdfcer-gui-base/text/print/footer`

## Item notes

### `fn cancel`

# It said "Close" until O185, and the old reasoning was right about the old window

The argument it carried: *nothing has started, so there is nothing to
cancel, and a Cancel button next to a Print button invites the reading that
a job is in flight and this stops it.* That was sound while the window held
nothing but a job description. It stopped being sound the day the window
started owning persistent state, because from that day there IS something
to cancel and it is not the job -- it is every setting changed since the
window opened.

The misreading the old comment feared is still worth avoiding and still is
avoided, by [`cancel_hover`], which says in words what the button puts back
and does not mention jobs at all.

# Why this word and not "Discard" or "Revert"

Because it sits on the route the window chrome means. `dialogs.md` G4 makes
the OS close button, Escape and this button deliberately indistinguishable,
and the operator learned what the X does from every other window on the
machine. A button labelled with a verb the chrome cannot be labelled with
would be claiming a meaning the other two routes then quietly share without
saying so.

### `fn cancel_hover`

Names all three routes, because they are one route with three doorways and
an operator who learns it from the button should not have to re-learn it
from the X. It says *"opened with"* rather than *"saved"*: the window may
have written settings already -- a Print that reached a printer which then
refused the job saves before it spools -- and "unsaved changes" would be a
false description of what is being put back.

### `fn keep_and_close`

# The fourth route, and why it needs a label rather than a chord

Operator request O185: *"I set the printer up, close the window to go check
something, and it's all gone."* The three routes that existed -- Print, and
the two spellings of Close -- could express "print and keep" and "leave and
lose", and had no way at all to say *"keep what I set, but do not print
yet"*.

It cannot be folded into the chrome. G4 reserves the chrome for one
meaning, and putting the KEEPING one there would mean an operator who shuts
the window on a mess of three hundred copies and the wrong tray inherits
that mess on their next print, having done nothing to ask for it. The safe
meaning goes on the chrome; the positively-chosen one gets a button with a
verb on it.

**Two words, both of them promises.** *Keep* says what happens to the
settings and *close* says what happens to the window, and an operator who
reads only the first word still gets the half that distinguishes this
button from the one beside it.

### `fn keep_and_close_hover`

*"without printing"* is the clause that earns the tooltip. The label
already says the settings are kept; what an operator hesitating over an
unfamiliar button needs to know is that pressing it does **not** put paper
through the machine.

### `fn commit_hover`

The behaviour is O166's and predates O185 by four days; the sentence is
new, because until there was a button beside it that ONLY keeps, nothing
had to distinguish the two.

### `fn commit_with_clipping`

The dialog is the confirmation and there is no second gate, so the
uncertainty is stated in the disclosure rather than implied by a confirm
step existing. Putting it *in the label* rather than beside the button is
the difference between a warning the operator has to have read and one
they can have looked past — it is on the control their hand is already on.

### `fn commit_losing_content`

# Why a second sentence rather than a reworded [`commit_with_clipping`]

The two say different things and both are needed.

`commit_with_clipping` says *"N sheets will be **clipped**"* — a geometric
fact about page boxes and the printable rectangle, taken at planning time
with no raster in hand. It is exactly true and it is what the button says
when nothing better is known.

This one says *"N sheets will **lose content**"*, and it may only be shown
when every clipped sheet in the job has been rendered by the preview and
its overhang tested for ink. `N` is then the number that really will lose
something, which is smaller than the geometric count whenever the operator
prints a 1:1 CAD sheet whose border is empty paper — *"the area that isn't
printed is just empty border."*

**Reusing the old sentence for the corrected number would have been the
defect.** With two of five clipped sheets known blank, *"Print — 3 sheets
will be clipped"* is plainly false: five are clipped. The count changed
what it counts, so the sentence has to say what it now counts. That is a
correction, and it is the opposite of softening a true statement to match a
better one.

### `fn commit_may_lose_content`

Shown when some clipped sheets have been examined and found blank and
others have not been looked at. The number is `known_inked + unexamined`:
the most sheets that could possibly lose something, with every sheet nobody
has looked at still counted, because a claim about an unexamined sheet
would be invented.

# The two words carrying the whole difference

*"up to"* and *"may"*. They are here because the number is a bound rather
than a count, and they are **absent** from [`commit_losing_content`] and
from [`commit_with_clipping`] because those two report numbers that were
measured — one by the ink test, one by the geometry. A hedge that appeared
on all three would say nothing at all; appearing on exactly the one bounded
number is what makes it informative.

# The singular has no "up to", and that is not an inconsistency

*"Up to 1 sheet"* reads as a quantity discount. *"1 sheet **may** lose
content"* carries the same uncertainty in the word that is doing the work,
which is `may` in both forms.

### `fn failed`

`detail` is `pdfcer-print`'s own error `Display`, passed through rather than
rewritten — for the same reason [`crate::text::canvas_render_failed`] does
it: those errors are structured, specific diagnostics, and replacing one
with "an error occurred" throws away the only part of the sentence that
helps.

**Says nothing came out.** A failed spool can leave an operator wondering
whether half a job reached the tray, and the first line of the answer
belongs in the message.

### `fn settings_synthesised`

Shown beside [`sent`] after a job whose `SettingsSource` came back
`Synthesised`, and after no other.

# Why this is a disclosure and not an error

The job printed. Paper came out. Nothing failed, and if the operator was
changing nothing but orientation they may not be able to tell the
difference.

What was lost is everything the driver holds that pdfcer does not model:
media type, print quality, colour handling, output bin, stapling, and the
whole vendor-private half of a `DEVMODE` — which on the printers measured
while this was built was between 920 and 7,972 bytes, up to 97 % of the
structure. A synthesised `DEVMODE` has no private tail to carry any of it.

So the failure mode is a print that is *subtly* wrong — plain where glossy
was configured, draft where best was — and it looks like a printer problem
rather than a pdfcer one. That is precisely the class of thing rule 4
exists for: pdfcer chose something the operator did not ask for, and it
says so.

# Why it names the remedy

Because there is one, and it is one button away: opening the driver's own
properties dialog produces a real `DEVMODE`, which the next job carries.

### `fn scale_custom_disabled`

`dialogs::print::tabs` has always argued that greying is the correct side
of R9 here *because* the field is only **temporarily** unavailable: one
click on the radio beside it makes it live. R9's other half is that the
argument has to reach the operator, and it never did — the control was
greyed with no hover explanation of any kind, so the reasoning existed only
in a source comment.

It names the remedy and where the remedy is. *"Choose Custom"* alone
would be true and would still leave him looking for what to choose it on;
the radio is immediately to the left and saying so costs three words.
