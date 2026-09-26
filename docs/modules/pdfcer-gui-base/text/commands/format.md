# `pdfcer-gui-base/text/commands/format`

## Item notes

### `fn format_select_text_line`

**The words that make a rung findable.** `OPERATOR_REQUESTS.md` O188(A).

The operator asked to *"move the individual text blocks"* inside a grouped
block of text *"and have the ability to delete them"*. One line of a text
object is a *run*, and `EditSession::delete_text_run` removes one — but the
verb is reachable only once the **Part rung** is entered, and the other
entrance to that rung is arming the Points tool with the `A` chord and then
clicking, which no surface announces. A capability nobody is told about has
not shipped, so this label and this tooltip are the feature as much as the
verb behind them is.

# Every clause, and what it answers

**"Select this line of text"** — *this* line, because the operand is the one
under the pointer and the row would be a lie on any other. *Line*, because
that is what the operator sees and called it; *run* is the file's word and
`text run` is the panel's, and a third vocabulary invented here would be one
more thing to learn. **Not "Select part"**, which is the ladder's word and
means nothing to a drafter, and not "Select the text", which is what the
whole block already is.

⚠ **Never "dimension"** (R8b rule 15). On his drawings these runs usually
ARE labels of pdf dimensions — CAD-exported page content pdfcer reads and
must not silently alter — and the word would collide head-on with the ce
dimensions this application authors.

**"instead of the whole block"** is the contrast that makes the row
legible. Right-clicking text already selects something; without this clause
the row reads as a no-op, because the operator cannot see that what they
have is one rung up from what they want.

**"Delete then removes just that line"** is the reason to press it, and
it is the clause that discharges the report. Delete at this rung is the
thing he asked for; before this row, he could only discover it by arming a
tool he had not heard of. A row that descends without saying what descending
buys is a row nobody presses.

**"Press Escape to go back to the whole block"** is the way out, stated
before it is needed. R83 — a control that changes what the operator is
standing on owes them the way back, in the same breath.

# What this tooltip does NOT say

It names only the delete, not the **move**, which is the half of O188 the
operator asked for first. `EditSession::move_text_run` now exists and
`crate::canvas::moving` drives it, so the omission is no longer a statement
of what the program cannot do — it is a clause this tooltip still owes.

⇒ The rule the clause has to satisfy when it is written: a surface states
the remedy it **has**. A tooltip that promised a gesture the rung refused
would put the disappointment before the gesture rather than after it.

### `fn format_select_form`

**The deliberate second act that pays for the deep hit test.**

A click reaches *inside* a form XObject and selects what is drawn there,
and the form itself is excluded from the hit test outright — a `/BBox` is a
clipping extent (§8.10.1), not a claim about ink, so a page-sized form
otherwise wins every click at every point. That is the operator's report:
*"when I click on one of the objects all I get is the page selected."*

But a form is a perfectly good thing to want. It is one page object with an
ordinary paint-order index, and moving a title block or deleting a stamp is
*the form*, not the two hundred objects inside it. So the reach gained
inside forms must not cost the reach to the form, and this command is how
that is paid: reachable on purpose, never by default.

# Every clause of the tooltip, and what it answers

**"the form that contains it"** names the structure, which is the fact the
operator can act on — it explains the page-sized outline they used to get,
and it is the word they need if they go looking in another tool.

**"one object you can move, delete or copy"** is the reason to press it.
The thing selected before pressing is none of those, and this sentence is
the only place that trade is stated in the operator's own vocabulary.

**"Everything drawn inside it moves with it"** is the consequence they must
know *before* pressing, not after. It is also the honest warning that a form
may be shared: `pdfcer-core`'s decision 076 rules editing inside a shared
form as edit-in-place, and a page invoking a form twice draws it twice.

# Why "the form" and not "the container" or "the group"

Because *form XObject* is what the file calls it, what `pdfcer
object-list` prints, and what the Objects panel row says. A friendlier word
invented here would be a fourth vocabulary for one thing, and the operator
reads all four.

### `fn format_unshare_form`

**The "option" half of `pdfcer-core`'s decision 076, and the remedy the
SHARED CONTENT disclosure needs somewhere to point at.**

This shell can edit text **inside a form XObject**. ISO 32000-1 §8.10.1
names a CAD system's standard component as the *purpose* of that construct,
and no clause in either edition binds a form to a page — a confirmed
permanent negative in pdfcer's spec corpus (`FX-N1`). So one title block is
one stream object invoked from thirty-six sheets, and an operator fixing a
typo on sheet 12 changes all thirty-six.

pdfcer cannot prevent that structurally: there is exactly one stream to
write. Decision 076 rules edit-in-place-and-disclose the **default**, and
`R206` requires that two defensible behaviours ship as two options with a
chosen default. **This is the second option.**

# Every clause of the label and the tooltip, and what it answers

**"Give this page its own copy"** is the label, and it is a *sentence in the
imperative*, which no other command on this tab is. That is deliberate. The
alternatives were all worse in the same way: **"Unshare"** is the engine's
word and means nothing to a drafter; **"Detach"** implies the drawing is
removed from the page; **"Make unique"** is Inkscape's phrasing for a
different act and invites the reading *"make it look different"*. What the
operator wants is stated exactly by what happens: this page gets a copy.

**"if it is also drawn on other pages"** is a CONDITION, and it must stay
one. The unconditional form — *"This drawing is drawn on other pages
too."* — is a claim about the operator's own file made before anything in
the command's chain has asked how many times the form is invoked. On an
ordinary one-page CAD sheet wrapped in a single form, the shape of this
operator's own SolidWorks exports, it is simply **false**; and being
identical either way, it tells the operator who genuinely *does* have a
thirty-six-sheet title block nothing, because an unconditional sentence
carries no information about the case it is unconditional over.

⇒ **A control cannot assert a fact about a document it has not measured.**
The tooltip's job is to say what the command *does* and under what condition
it helps; the measurement is a whole-document page walk, it belongs on the
press, and what it found is disclosed afterwards. See
`crate::app::actions::xobject::fanout` for the walk and R9 for why it is not
in a per-frame condition.

**"changes here will not affect them"** is the reason to press it — the
promise, in the operator's terms, and the only clause that distinguishes
this command from `format.select_form` one row above.

**"pdfcer checks when you press, and says what it found"** carries the
measurement, and it does two jobs. It tells the operator where the answer
to *"is it actually shared?"* comes from, which nothing else on screen
states — the Objects panel does not say a form is
shared, the canvas cannot, and the SHARED CONTENT disclosure says it only
*after* an edit has already fanned out. And it sets the expectation that
pressing this may **decline**: on a drawing nothing else draws,
`crate::text::unshare::UnshareRefusal::NotShared` says so and changes
nothing, which is a service rather than a failure and should not arrive as a
surprise.

**"Everything looks exactly the same afterwards"** is the clause an operator
would otherwise report as a bug. The copy is byte-identical to the original
until it is edited, so a successful unshare renders pixel-for-pixel as
before. A tooltip that promised a visible result would make every success
look like a failure.

# Why the tooltip does not say "before you edit it"

It is the true instruction — unsharing *after* an edit copies the
already-edited stream and leaves every other page changed as well — but a
tooltip is read while deciding whether to press, and a sequencing
instruction there competes with the four clauses above for the one line an
operator actually reads. The sequence belongs where it is acted on, which is
the disclosure that fires the moment an edit *has* fanned out:
`crate::text::unshare::shared_content_remedy` states it in order, and its
doc comment carries the argument.

### `fn format_properties`

**A second route to `file.properties`, not a second implementation of
it.** Its dispatch arm raises `Action::Command("file.properties")`, which is
the mechanism that exists so exactly this cannot become two ways of opening
one panel with two sets of guards — the Find bar's OCR offer is the
precedent.

It is registered as its own id rather than listing `file.properties` twice
because the shell enforces **one command, one tab**, and the two placements
answer different questions: File ▸ Document is *"tell me about this file"*
and Format is *"tell me about the thing I just clicked"*.

The tooltip names the ce dimension case explicitly — the style cascade, the
tolerance and the radius/diameter switch — because it is the one an operator
has no other way to discover: a selected ce dimension looks exactly like an
unselected one apart from its outline.

### `fn format_font`

Drawn by an `Item::Custom`, not by this command's button, because a face
chooser has to ask *which* of the page's fonts and a button cannot. The
label and tooltip are still registered here and still used: the shell reads
them for the a11y name and for `shell::commands::reach`'s reachability
check, and the custom renderer draws the label beside its combo.

### `fn format_bold`

The tooltip names the **fallback**, exactly as the Properties panel's
does, because the fallback is what an operator would otherwise meet as a
surprise: a page carrying a real bold cut gets the real face, and one that
does not gets thickened letters and a sentence in the status bar saying so.
Rule 4 — the thickened text renders exactly as the saved file will render
it, and the disclosure is off-canvas.

### `fn format_font_colour`

`format.colour` belongs to the markup property editor in the same tab, and
the two are genuinely different subjects — one is an annotation's ink, the
other is a page-content fill. Word calls this one
*Font Color*, which settles the name in the operator's own vocabulary
rather than by disambiguation.

The tooltip states the **refusal** as well as the act, because the
refusal is common on exactly the documents this program is for: a run
painted in DeviceCMYK or a spot colour has no faithful sRGB, so the swatch
is replaced by a sentence rather than showing a nearest-match that the next
press would write back — converting a drawing's ink on its way to a printer
that cares.
