# `pdfcer-gui-base/text/commands/view`

## Item notes

### `fn view_page_continuous`

The operator's instruction of 2026-08-12 is what this control is, and the
tooltip carries its reasoning rather than a feature description: *"continuous
scroll should be an option under the view tab as the way I move around a
page is great when working with drafting drawings."* So the words say what
it is **for** — a document you read through — and leave single page
standing as the right answer for a sheet set, which it is.

The second sentence states the per-document persistence, because it is
behaviour the operator cannot see until it surprises them: choosing this on
a report and then opening a drawing set must not carry the setting across,
and a control that silently remembers something should say so.

### `fn view_panel_pages`

The one panel toggle whose tooltip has to say what the panel is *for*
rather than what it contains: a grid of thumbnails is self-explanatory to
look at and not to read about, so the sentence spends its words on the two
verbs — go to a page, and act on several at once — that are not obvious
from the picture.

### `fn view_zoom_region`

The tooltip says *"drag"* because arming this command does not zoom
anything — it changes what the next drag on the page means. A control
that arms rather than acts has to say so, or its first press reads as
broken.

### `fn edit_copy`

The tooltip names **what it copies**, not what it does, because the honest
scope is narrower than the word "copy" promises: `EditSession` has no verb
that puts page content back, so a copied path could never be pasted. Saying
*"comment or markup"* is the difference between a control that under-promises
and one an operator discovers is a lie the first time they try it on a line.

### `fn edit_paste`

The tooltip was rewritten on 2026-08-29 and the old wording is worth
recording, because it had quietly become false twice: it said *"the copied
comment or markup"* after `Pass 120.0` made page content pasteable and again
after form fields joined the clipboard. A tooltip that names a NARROWER scope
than the command has is the same defect class as one that names a wider one —
the operator never tries the thing that would have worked.

### `fn edit_paste_duplicate`

**Ken, 2026-08-29:** *"ctrl shift v for paste as duplicate."*

The tooltip leads with the CONSEQUENCE — *"typing in one fills both"* —
rather than with the mechanism (*"a second widget of the same field"*), which
is a sentence about PDF structure. The consequence is the thing that will
surprise him at the printer, and it is the ONLY disclosure there is: two
linked boxes and two independent boxes are pixel-identical on the page.

It says what happens over a non-field clipboard too, because the command is
not withheld there — it falls through to an ordinary paste, and a control that
silently means something else is worse than one that says so.

### `fn edit_duplicate`

The tooltip's first clause is *"without using the clipboard"*, and that
is the whole reason the command exists rather than a nicety of phrasing.
`Ctrl+C` then `Ctrl+V` already produces a second comment; what it also does
is throw away whatever the operator was carrying — a part number, a
title-block string, a cell from a spreadsheet — and they find that out the
next time they press `Ctrl+V` somewhere else. An operator placing a row of
identical revision marks pays that price once per mark.

⇒ So the disclosure and the selling point are the same clause. Leading with
the mechanism (*"copies the selected annotation and plants a second one"*)
would describe what they can already see happening and omit the only thing
they cannot.

It names the OFFSET, because a duplicate that landed exactly on its
original would look like a command that did nothing — the invisible-paste
problem `canvas::clipboard`'s header names — and because the offset is what
makes pressing it repeatedly walk a row, which is the gesture it is for.

*"Comment"*, not *"annotation"*: `crate::text`'s standing rule that a
label is the operator's vocabulary, and the same word the Comments panel and
the whole of `crate::text::markup` use.

### `fn edit_copy_as_vector`

**Ken, 2026-09-03** (`OPERATOR_REQUESTS.md` **O120**): *"Also I'd like to be
able to copy and paste anything to other software - like copy and paste
vector graphics into word or inkscape for example if possible."*

The tooltip leads with **what arrives at the other end**, because that is
the only thing that distinguishes this from the Copy beside it. Ordinary Copy
puts an internal clip on the clipboard plus a picture; this puts the
*geometry*, so what lands in the next drawing is still line-work rather than
a photograph of line-work. An operator cannot tell those apart by looking at
the paste — they find out when they try to recolour it — so the difference
has to be stated before the press rather than discovered after it.

It names Word and Inkscape by name, which this catalogue does sparingly,
because they are the two applications the operator named and because the
promise is *specifically* about them: the format order was measured against a
real Word paste, and Inkscape's own preference list is where the SVG's
position came from. A vaguer "other programs" would be a weaker claim than
the one that was actually tested.

It says what the operand is — selection if there is one, page otherwise —
because that is the one thing about this command that is not visible on the
button, and getting a whole sheet when three parts were selected is the
surprise worth spending a clause on.

### `fn pages_copy`

The tooltip names the OPERAND RULE, because it is the one thing an
operator cannot see: with sheets picked it copies those, with none picked it
copies the one they are looking at. Every `pages.*` verb works that way and
none of them says so anywhere else.

### `fn edit_redact_selection`

The tooltip's second sentence is the whole control. *"Nothing is removed
until you apply"* is the fact that decides whether an operator trusts this
button or is frightened of it, and it is also the fact that stops them
believing the job is done. A redaction that was marked and never applied is
a document that still contains every word.

The first sentence names what the search box cannot reach, because that is
why this exists: on a CAD drawing a title-block value is often vector
strokes and a stamp is often an image, and neither is findable by typing.

### `fn edit_offpage`

**The label says what the operator is looking for, not what the program
does.** *"Check for content off the sheet"* over *"Off-page census"* or
*"Scan document"*: the first is a sentence a draughtsman would say out loud
about a drawing they are about to send, and the other two are a description
of a mechanism nobody asked about.

The tooltip leads with the CONSEQUENCE, and it must. Every other control
in the Protect group is about content the operator can see and has decided to
remove; this one is about content they **cannot see and do not know is
there** — it does not render, it does not print, and no amount of looking at
the drawing discloses it. An operator with no reason to believe their file
contains anything hidden will not press a button that offers to look.

The third sentence names the examples, because the abstraction is the
part that fails to land: "objects outside the page boundary" means nothing,
and "a revision note dragged off the sheet instead of deleted" is a thing
every draughtsman has done.

It does **not** promise removal, for `edit.redact_selection`'s reason
turned around: that one has to say *nothing is removed until you apply*
because its name sounds destructive. This one has to avoid implying removal
at all, because its name sounds like a report — and it is one. What it offers
is a mark, and the mark's own button says so.

### `fn view_tool_node`

Named *Points*, not *Node*, and not *Direct selection*. "Node" is this
program's internal word (`SelectionLevel::Node`); a draughtsman says
*point*, and `text::commands`' standing rule is that a label is the
operator's vocabulary and an id is the format's. Illustrator's own name for
it — "Direct Selection" — describes the mechanism rather than the subject
and has confused people for thirty years.

### `fn view_tool_text`

The tooltip says *"drag"* and names what the tool **replaces**, for the
reason `view.zoom_region`'s does: this command arms rather than acts, so its
first press changes nothing an operator can see except the pointer, and a
control that arms has to say so or it reads as broken.

It names the marquee explicitly because that is the trade an editor is
making — the primary drag stops selecting objects for as long as this is
pressed — and because in Read and Review the tool is redundant (the select
tool already sweeps text there), so the sentence has to be true in a mode
where it changes nothing as well as in the one where it changes the most.

### `fn view_zoom_fit_height`

Deliberately the same sentence as its two siblings with one word
changed. The three fit modes differ in exactly one respect and the copy
says so; three separately-worded tooltips would invite the reader to hunt
for a distinction that is not there.

### `fn view_smart_select`

The tooltip states what a **click** does and what a **double-click** does,
because those are two halves of one rule and an operator told only the first
concludes the second is not offered — the failure
`panels::tool::armed`'s form-field instruction records.

### `fn view_text_chunks`

The label is **Text chunks**, not *Parts* and not a third *Points*.
`RIBBON_IA.md` §8 item 5 already carries two controls labelled *Points* —
`view.tool_node` and `view.show_points` — and a third would make the word
mean nothing. *Chunk* is also the word the operator used for the thing.

The tooltip says what the boxes are FOR rather than what they are, because
an operator who reads only that they are boxes has no reason to want them.

### `fn view_ocr_layer`

The label says **OCR text**, not *Show invisible text* and not *X-ray*.
An operator who has scanned a drawing knows the letters OCR; the word
*invisible* describes the mechanism, and the mechanism is the one thing
about this feature they do not have to know.

The tooltip has to carry an unusual amount, because the control's whole
subject is something that is on the page and cannot be seen: it says the
text is already there, that this only draws it, and that nothing about the
document changes. Without the last clause the honest reading of a button
that makes writing appear on a scan is *"it wrote on my drawing"*.

### `fn view_rulers`

The tooltip states the unit, because the unit is the thing an operator
cannot see until they have already trusted a number. `crate::canvas::rulers`'
header §1 carries the decision in full: points is what the file itself
says, and a document that has been given a measurement scale reads in that
scale's units instead — through the same formatter a dimension label uses,
so the ruler and a dimension across the same span agree to the digit.

The second sentence also does the discoverability work for the guides:
dragging out of a ruler is how a guide is made, and that is not something
an operator finds by looking at the guides button.

### `fn view_grid`

"Over the page" rather than "over the canvas", deliberately: the grid is
anchored to each sheet and scrolls with it, which is the whole of the
decision in `crate::canvas::rulers`' header §2, and the wording is what
tells an operator that before they scroll and find out.

### `fn view_guides`

Names all three verbs — place, move, remove — because a guide is the one
control in this group that the operator *does something to*, and a toggle
whose tooltip only says "show or hide" would leave them with lines they
cannot get rid of. The persistence is stated for the same reason
`view.page_continuous`' tooltip states its own: it is behaviour the
operator cannot see until it surprises them.

### `fn view_line_weights`

**`OPERATOR_REQUESTS.md` O137, asked for by name** — *"the button to
show all lines without their thickness — thin lines or something like cad
has. The button never worked but I do want that display option!"*

# Every clause of this tooltip is doing a job, and three of them are
defences against a specific misreading

**The label is "Line weights", not "Thin lines" or "Hairlines".** It names
the thing the switch governs, so its two positions read as *on* and *off* —
which is Acrobat's spelling (**View ▸ Line Weights**, checked by default)
and AutoCAD's (`LWDISPLAY`). A label naming the *off* state would render
pressed when line weights were being shown, which is backwards. It is also
the phrase a draughtsman already has: he asked for "lines without their
thickness", and thickness on a drawing is a *weight*.

**"Turn this off"** leads, because turning it off is the whole feature. On
is what pdfcer already did.

**"one pixel wide"**, not "thin" or "hairline". Thin is relative and invites
the opposite reading — Acrobat's *enhance thin lines*, which makes thin
things THICKER — and this is the other convention entirely. One device
pixel is what it actually does, and it is checkable.

**"however wide the file says they are"** says the ceiling applies to
everything, so an operator does not expect only the fat ones to change.

**"Filled shapes and hatching are not affected"** is not padding. Only
stroked paths reach the engine's stroke width; a region built out of thin
*fills* keeps every pixel. Without the sentence, an operator whose hatch is
fill-based would turn this on, see the hatch unchanged, and conclude the
control is half-broken.

**"Printing, exporting and the print preview always use the real
widths"** is the constraint the whole feature was built around, and it is
stated *here* rather than only in the status bar because this is where he
decides whether to press it. The status line says the same thing while it is
on; a disclosure the operator meets only after acting is half a disclosure.

# Why there is no Settings entry for it (a decision, not an omission)

The 2026-08-17 sweep moved the two surviving `view.*` settings into
Settings ▸ Drawing the page on the argument that *"a value set once and
forgotten is not an activity"*. This is the other case: it is a **reading
aid he flips several times while reading one sheet** — turn it off to see
whether two lines are coincident, turn it back on to check a drafting
weight — so it is an activity, and P2 says a ribbon tab picks those. It is
also per **document** ([`crate::viewer::ViewState`]), so two open drawings
can disagree, which is what comparing a sheet against its neighbour needs.

A *persisted* default would additionally mean pdfcer opening every drawing
for the rest of time showing something the file does not say, because of a
switch set weeks earlier — one step removed from the failure O137 forbids
outright. If he asks for it to stick, it belongs in
`crate::app::prefs::opening::PageChrome` beside the other three view
toggles, seeded by `Prefs::seed_view`, and that is the whole of the work.

### `fn view_off_page`

> *"in our view ribbon area we need an option to show the stuff that is
> off page or not (and when not showing the stuff that is off page there
> shouldn't be a gap between pages where the stuff is, so it just goes
> back to looking before we added the view things that are off the page
> feature). by default, read doesn't show off page items, review and edit
> do show off page items."*

# Why the label names the CONTENT and not the band

**"Off-page content"**, on [`view_line_weights`]'s reasoning and for the
same structural reason: a toggle's label must name the thing it governs,
so that *pressed* reads as *that thing is on*. A label naming the side
effect — *Pasteboard*, *Margin* — would render pressed when the operator
was looking at extra grey, which describes the cost rather than the
feature. It is also the operator's own phrase: *"the stuff that is off
page"*.

# Every clause of the tooltip, and the job it does

**"Some drawings carry marks outside the sheet"** establishes that this
is about the file rather than about pdfcer, because an operator meeting
the control on a document with nothing out there will press it, see no
change, and otherwise conclude it is broken.

**"click them"** is not padding: this switch governs REACH as well as
sight (`crate::canvas::tier`), and a reader who thought it only affected
drawing would be surprised to find a visible object unselectable.

**"the page is shown on its own, with no extra space around it"** is the
parenthesis in the request, stated as the *off* state's promise. It is
what he will check first.

**"Read starts with this off; Review and Edit start with it on"** — the
per-mode default, disclosed at the control. Without it, an operator who
switches to Read and watches the layout change has met an unexplained
event; with it, he has met a documented one. See
[`crate::app::prefs::offpage`].

**"pdfcer remembers your answer for each"** is the last clause of the
request and the reason there is no Settings twin: the answer is not a
value set once, it is three answers the program keeps for him. A
Settings page would offer a fourth, global one that could only
contradict them.

⚠ The tooltip deliberately does NOT say *nothing is hidden from
printing or export*. It is true — this is a view switch and reaches no
renderer but the canvas — but saying so raises the possibility that a
view setting might silently change an exported file, which is a fear
this shell should not plant in order to allay.

### `fn view_panel_layers`

## Reworded at S4, because the old tooltip undersold a capability

It read:

> Show the document's layers and which of them a reader draws by default.

which was a claim about the document with **no verb in it** — accurate for
the S3 panel, which was a report. S4 restored the visibility control
(`crate::app::actions::Action::SetLayerVisible`), and a tooltip that
describes a panel as read-only when it is not costs the operator the
capability: they read it, conclude there is nothing to click, and never
open the panel.

The new wording follows [`view_panel_bookmarks`]'s shape — *what it shows,
then what you can do in it* — because that is the shape of the only other
panel in this build with a verb, and two panels that answer the same
question should answer it the same way.

The third clause is not padding. It is the same boundary
`crate::text::panels::layers_session_only_note` states inside the panel,
and it is repeated here because the ribbon tooltip is read **before** the
panel opens — which is the moment an operator decides whether clicking
this is a safe thing to do to someone else's file.

### `fn view_panel_signatures`

**CORRECTED 2026-09-05.** The second sentence read *"pdfcer does not
check whether they are valid."* It became false when
`signature::verify_all_with_trust` was wired (`crate::trust::examine`,
engine `pdfcer-core` v0.38.0 at `b01964f`), and it was **missed by the
sweep that corrected the panel's own explainer** — `crate::text::panels`
deleted the near-identical sentence the same day and left this one
standing, because the feature work opened that file and never opened this
one. The tooltip is the surface read *before* the panel opens, so it was
the worse of the two to leave wrong.

The replacement states the three facts the panel actually draws rather
than a verdict, because the panel never folds them into one.

### `fn view_panel_forms`

The taxonomy's argument for keeping form-filling out of Read is recorded
in [`crate::app::modes`]; the operator's answer is that Acrobat Reader
fills forms in its default view and replacing it is the stated goal. A
command lives on exactly one tab (P1), and Read is shown `file` and
`view` alone — so the fill verb had to move to a tab Read has, and this
is that move.

**The label stays a verb while the id became a panel toggle**, which is
the one thing about this that looks inconsistent and is not. The id
names what the command *does to the shell* — it shows the Forms panel,
exactly as its five siblings in View ▸ Panels do, and it inherits their
mechanism rather than inventing a second one. The label names what the
operator *came to do*, and nobody opens this panel to look at a list of
field names. Renaming it "Forms" to match its neighbours would file the
capability under a noun the person looking for it is not searching for.

### `fn view_next_document`

The tooltip names the chord because that is how this command will
actually be used. Ctrl+Tab is the gesture; the button is the thing that
tells an operator the gesture exists, which is why the button is on the
ribbon at all when nobody will click it twice.

### `fn view_close_other_documents`

The label says which one **survives**, not how many go. *"Close others"*
is what every browser and every editor calls it, and an operator reading it
in a menu opened on a specific tab already knows which that is — which is
what makes the short wording safe rather than merely terse.

The tooltip has to say **which** *this one* is, because the command has
two routes with two operands: from a tab's context menu it keeps the tab
that was right-clicked, and from the ribbon it keeps the one on screen. So
it says *"the one you opened this on"* rather than naming either, which is
true from both and misleading from neither.

### `fn view_panel_float`

**This entry lost an ellipsis and a promise, and both losses are the
same correction.** It used to read *"Reset layout…"* / *"Put the panels
back where they started. You choose which ones — the left panel, the
right panel, or just whether they are open."*

The choice is real and specified — `RIBBON_IA.md`: *"an operator who only
wanted the right dock back must not lose their left one"* — and
`egui_shell::layout::ResetScope` implements all three scopes. What does
not exist is anywhere to **ask**: this build has no modal, no popup and no
split-button item kind, so the command was wired to `ResetScope::All` (see
`crate::app::PdfcerApp::dispatch_command`, whose arm records what a chooser
would take). A tooltip offering a choice the operator is never given is
exactly the "never state a capability the build does not have" failure
this catalog's header forbids, and the trailing `…` made the same promise
in punctuation.


# The words, and the two that were rejected

*"Float"* rather than *"Undock"* or *"Tear out"*.

**"Undock"** names the thing that stops happening rather than the thing
that starts. An operator reading a menu is looking for what they will
get, and what they get is a window — the fact that it left the dock is
how, not what.

**"Tear out"** is the gesture's name in every product that implements
this as a drag, and this one is not a drag: `MODES_AND_PANELS.md`
specifies *"a stationary Float this panel… command rather than
drag-to-tear"*, and a menu row named after a gesture that does not
exist here would teach the wrong thing about the interface.

**No ellipsis.** It acts; it does not ask. The convention this
catalog follows is that an ellipsis means a dialog is coming.

### `fn view_panel_dock`

The tooltip promises *where it came from* rather than *back in the
dock*, because that is the property the implementation actually holds
and the one an operator would otherwise have to test to find out. See
`egui_shell::dock::float`'s header for why putting it back "somewhere
sensible" was rejected.

### `fn view_panel_close`

The tooltip names the way back, and that is not padding. Closing is
the only one of the three verbs that leaves no visible trace of the
panel anywhere, so it is the only one where an operator can be left
wondering whether they have lost something. Naming the View tab costs
a clause and removes the whole question.

### `fn view_dock_all_panels`

# Why this exists as a command of its own

A floating panel lives in an OS window at a remembered desktop
position. Unplug the monitor that position was on and the window is
still open, still in the layout, and **unreachable**: it cannot be
dragged, it cannot be closed, and it cannot be floated again because it
already is.

Every other route out of that state needs the operator to act *on the
window*. This one acts on all of them at once, from the application
window, which is the one surface guaranteed to be on a monitor that
exists.

Reset layout is the other route and it is stronger — it also
restores the arrangement. This one is the *cheap* route: it costs the
operator nothing they arranged. Offering both is the two-tier shape
`MODES_AND_PANELS.md` singles out as the thing the best product in its
benchmark table got right.

Greyed rather than hidden when nothing is floating, and that is R9
applied rather than R9 broken: this is *temporarily* unavailable —
there is simply nothing to dock this second — and the hover says so.
Hiding it would make the remedy invisible exactly until the operator
needs it, which is the wrong half of the cycle to be visible in.

### `fn view_ribbon_auto_hide`

The wording says what he will SEE, not what the setting is called: the tab
names stay, the buttons go until the pointer arrives, and the drawing does
not move. The last clause is there because it is the property that makes the
setting usable rather than nauseating, and an operator deciding whether to
turn it on cannot know it otherwise.
