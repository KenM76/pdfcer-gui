# `text::panels` — every string the dock's panels show

One area of the catalog described in [`crate::text`]'s header. It covers
the panel bodies in [`crate::panels`]. The three document-structure
panels are in this file; Comments, Fonts, Objects and Properties each have
their own module, and Forms and Pages have their own areas one level up.

| Module | Panel |
|---|---|
| `mod.rs` (this file) | the three **document-structure** panels — Bookmarks, Layers, Signatures — plus [`byte_size`], which several areas share |
| [`attachments`] | the Attachments panel — the files a document carries inside itself, and the four verbs opposite them |
| [`comments`] | the Comments panel — every annotation on the document, what each one is, and the five disclosures a row can carry |
| [`face`] | the **face chooser**, drawn on two surfaces from one module, and the standard-14 disclosure it owes |
| [`fonts`] | the Fonts panel's inventory report |
| [`objects`] | the Objects panel, and the wording of every [`crate::panels::objects::summary::ObjectSummary`] fact |
| [`properties`] | the Properties panel |

## Almost every sentence here is salvaged verbatim, and that is the point

These strings came across from the old shell's `ui_text.rs` (7,912 lines,
1,193 entries) **with their doc comments**, because the doc comment is
usually the record of a defect the wording was changed to fix. Three
examples, all of which are below:

- [`signature_leaves_tail`] is worded as *under-protection* rather than
  as damage, because ISO 32000-1 §12.8.1 makes whole-file coverage a
  `should` — the document is conforming, and an operator told "invalid"
  about a legal file has been misled just as surely as one told nothing.
- [`layers_session_only_note`] exists because a panel of tickboxes over a
  document is, by every other application's convention, an editor — and
  this one is not. Its doc comment carries the reasoning the wording holds.
- [`fonts::font_verdict_removable`] and its four siblings are **two
  words each**, because a full sentence there clips at the dock's edge and
  takes the byte size with it.

⚠ **Rewriting any of those from fresh words re-derives a decision already
paid for**, and the rewrite has no access to the screenshot or the defect
that bought the current wording. Amend a string only with a reason, and
write the reason into its doc comment.

## The three panels in this file share a posture

**Each leads with the shape of what it is about to tell you**, because a
panel headed "Signatures" listing byte counts is the single likeliest
place in this application for an operator to take away more than was said.
The Signatures panel's opener is [`crate::text::trust::panel_intro`],
beside the rest of the trust copy. The Layers panel opens by saying that a
toggle changes what you see and not the document, and that nothing it does
is saved. The Bookmarks panel says when its own reader gave up.

That ordering is not stylistic. A caveat below a list arrives after the
operator has already drawn a conclusion.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the thing and what the operator can do about it.**
- **Never state a capability the build does not have** — and never state
  the ABSENCE of one either, which is the half that rots. A denial is a
  claim about the engine with a shelf life of hours; see the note above
  [`signatures_none`].

## Item notes

### `fn each_structure_panel_leads_with_its_own_limitation`

Each of the three structure panels leads with a limitation, and the
value of doing so is entirely in the limitation being specific. Three
near-identical hedges would satisfy the convention and teach an
operator to skip the first line of every panel, which is worse than
having none.

### `fn the_layers_note_says_a_toggle_changes_the_view_and_not_the_document`

This test replaces `the_layers_note_states_that_switching_is_unavailable`,
which asserted the S3 truth — that the panel had no visibility control
— by pinning the words "not available". S4 gave the panel its control
back (`crate::app::actions::Action::SetLayerVisible`), so that clause
became a lie and came out **in the same commit as the checkbox**.

The test is rewritten rather than deleted because the *thing it
guards* did not go away, it inverted. Two directions now:

1. **The claim that must be present.** A panel of tickboxes over a
   document reads as an editor. "not the document" is the clause that
   says it is not, and it is the single most load-bearing phrase in
   this panel.
2. **The claim that must be absent.** If a later change reverts to the
   S3 wording *without* removing the control, the panel is back to
   describing a program that does not exist — the failure the old
   shell's module header records happening twice, to two different doc
   comments, in this exact file. A control that says of itself that it
   is unavailable is not a copy-edit defect; it is the operator
   concluding the application is broken.

Asserted on clauses rather than on the whole string: a copy edit
should be free, and a capability claim should not.

### `fn the_reset_control_names_what_it_returns_to`

`Action::ResetLayers` restores *the document's own default*, which on
a file that declares a "Confidential" watermark off by default is
emphatically not "show everything". Those are two different acts and
only one of them is a disclosure event, so the control that performs
the safe one must not be readable as the other.

The label alone cannot carry that — "Reset" in a layers panel is
genuinely ambiguous — so the tooltip is where the distinction lives
and this is what keeps it there.

### `fn the_overridden_count_agrees_with_itself_about_number`

Same reasoning as [`layers_count`]: cheap to get wrong, immediately
visible, and it sits directly beside the Reset control where an
operator is deciding whether to click.

### `fn an_overridden_layer_says_which_way_the_document_asked`

The point of [`layer_overridden_tooltip`] is that an operator can see
what they are diverging from without resetting to find out. If both
arms read alike, the row that says "you are showing content this
document hides" — the one with a disclosure consequence — is
indistinguishable from its harmless twin.

### `fn the_delete_promise_and_the_delete_report_name_the_same_quantity`

This is the one piece of arithmetic in the bookmark wording and it has a
genuine off-by-one waiting in it:

* `bookmark_delete_takes_subtree` is handed the shell's **exclusive**
  count (how many are filed *under* the row), because that is what the
  panel can see and what `tree::descendants` returns;
* `bookmark_deleted` is handed the engine's **inclusive** count, because
  `EditSession::delete_outline_item` returns how many items went
  *including* the one that was clicked.

So the second must subtract one to name the same set the first named. If
it did not, an operator promised *"also removes the 11"* would be told
*"along with the 12"*, and the only conclusion available to them is that
pdfcer removed a bookmark it was not asked to.

The fixture is deliberately chosen so the right and wrong answers are
different strings — the discipline the engine's `Pass 156.0` note asks
for after its own delete test survived every sabotage: *"when you assert
that A and B differ, check your fixture can tell them apart."*

### `fn a_leaf_removal_says_nothing_about_a_subtree`

`delete_outline_item` returns `1` for a childless bookmark, and
*"along with the 0 filed under it"* is the shape of sentence that makes
a program look like it is filling in a template. The two branches must
therefore be genuinely different sentences rather than the same one with
a zero in it.

### `fn the_removal_disclosure_names_the_way_back`

`panels::bookmarks::edit`'s header carries the choice: delete is
*undoable* rather than *confirmed*, on the grounds that one press is one
engine command so `Ctrl+Z` restores the whole subtree. A disclosure that
reported the loss without naming the remedy would be the worst of both —
no question beforehand and no way out afterwards that the operator has
been told about.

### `fn the_subtree_warning_agrees_in_number`

It is read beside a button the operator is deciding whether to press, so
*"the 1 bookmarks filed under it"* is exactly the sort of seam that
makes a warning read as boilerplate and stop being read at all.

### `fn the_pages_line_names_the_pages`

The operator's reasonable fear beside a control that takes several
things at once is that the pages are what is going. An outline is a
document-level structure (§12.3.3) and removing a bookmark removes a way
of *reaching* a page, so the sentence is a fact rather than reassurance —
and it must actually mention the pages to do its job.

### `fn the_three_bookmark_states_are_distinguishable`

"Points at a page", "is a heading" and "pdfcer could not follow it"
are one good outcome, one normal outcome and one problem. Collapsing
any two would send an operator looking for a fault in a document that
has none, or reassure them about one that does.

### `fn full_coverage_and_a_tail_read_as_different_answers`

One is reassurance and one is a warning, and both are about a
conforming file. An operator who cannot tell them apart at a glance
gets no value from the panel at all.

### `mod annotgeometry`

A module of its own rather than four more functions in [`properties`], and
its header argues the seam: the heading, the units note, the four labels and
the Apply button are **shared with the page-content half and stay in
`properties`**, because the units note states the coordinate convention and
a second copy of a convention is how one panel ends up measuring Y from the
top in one half and from the bottom in the other. What is in the new module
is only what the annotation subject adds — the file's own refusal — plus the
argument for why the *other* refusal it could reach gets no string at all.

### `mod attachments`

Its own header carries the three sentences that are **not optional** on this
feature, each one an obligation `pdfcer-core` states in its own doc comment:
that removing an attachment does not erase its bytes under an incremental
save, that an embedded file may be encrypted inside an otherwise unencrypted
document, and that the name pdfcer writes to disk is not always the name the
document shows.

### `mod bookmarks`

The Bookmarks panel's other strings stay in this file, with Layers and
Signatures; these went to a module of their own under R2 when the two verbs
arrived, and the subject boundary is *the words for the verbs that change a
bookmark's PLACE rather than its name*.

Its header carries the two rules a reader must not have to re-derive: why
**two** numbers describe one move (the engine's `visible_items` counts what
was on screen, and a collapsed branch reports `1` however large it is), and
why neither of its decline sentences names the bookmark it is about.

### `mod dimension`

Its own header carries the one rule this catalog must not break: **it never
builds a label**. A limit tolerance suppresses the nominal rather than
printing beside it, and a panel previewing the two by concatenation
disagrees with the bytes in the page.

### `mod docprops`

A module of its own because the surface is a panel of its own, and because
`properties.rs` sits against its 1,500-line R2 ceiling. Nothing here
carries a `properties_` prefix: the module path already says it.

### `mod face`

Its own header carries the obligation that made it a module rather than a
block inside [`properties`]: since `Pass 162.0` the chooser offers faces the
document does **not** contain, pdfcer authors those without embedding
anything, and the text is then drawn with the reader's own copy of the face
— an inference the operator cannot see on this screen and can see on
somebody else's, which is exactly the case rule 4 requires a sentence for.

### `mod textannotstyle`

Its own module rather than more of [`properties`], on exactly
[`textobject`]'s precedent and the same measurement: that file stood at
**1,487 of its 1,500-line R2 ceiling** when `Pass 253.2` landed. The seam is
the one the code takes one directory over — `properties::markup` draws what
one verb reaches, `properties::markup::textannot` draws what the other does
— so the copy follows the code rather than the file it started in.

### `mod textobject`

Its own module rather than more of [`properties`], and the reason is
measured rather than stylistic: that file stood at 1,446 of its 1,500-line
R2 ceiling on the day this was written. Its header carries the seam as well
as the measurement — [`properties`] says *what is selected*, this says
*what a route the operator could not guess will and will not touch*.

### `fn byte_size`

Base-1024 arithmetic with the colloquial "KB"/"MB" labels rather than the
pedantically correct "KiB"/"MiB": this is read by operators comparing the
figure against what their file manager tells them, and matching that is
worth more here than matching IEC. The exact count is always shown beside
it (see [`fonts::font_size_line`]), so nothing is lost to the rounding.

Deliberately different from the byte counts in the Signatures panel, which
are printed raw. Those exist to be compared against a file's own length —
an exactness task. These exist to be ranked across up to a couple of
hundred rows — a magnitude task. Different purpose, different format.

### `fn panel_no_document`

**One sentence for every panel**, deliberately. A panel is never
blanked — a blank region is indistinguishable from a broken one — and a
bespoke "open a document to…" sentence per panel would be nine chances for one of
them to drift into a different voice, for no gain: at the moment nothing
is open, which panel the operator is looking at does not change the
answer.

### `fn panel_unknown`

Reachable exactly one way: a **saved layout** — or a named workspace —
naming a panel whose capability is not compiled into this binary
(`SHELL_FRAMEWORK.md` §5b). The dock's loader drops such entries and
reports them, so in practice this is the belt to that loader's braces.

It says what happened rather than apologising, because the operator's
next question is whether their layout is broken. It is not: the rest of
it loaded, and re-saving will forget this entry.

**Not a placeholder.** The no-placeholders rule forbids a control that
looks available and is not; it does not forbid explaining a pane the
operator's own saved layout asked for. A blank pane here would be
indistinguishable from a panel that had nothing to say.

### `fn signatures_file_unreadable`

Distinct from "no signatures": pdfcer could not read the file's length
from disk, so it has nothing to compare a byte range against. Saying
"no signatures" here would be a claim about the document made from an
inability to look.

### `fn signature_leaves_tail`

Worded as under-protection rather than as damage. ISO 32000-1 §12.8.1
makes whole-file coverage a `should`, so this document is CONFORMING —
and an operator told "invalid" about a legal file has been misled just as
surely as one told nothing.

### `fn signatures_measured_on_disk`

**New at salvage, and not decoration.** The old panel's doc comment
carried this fact and the panel never said it out loud:

> Unsaved edits are not counted, and cannot be: they are not in the file
> yet. The panel says which state it is describing rather than leaving an
> operator to assume.

The second sentence was a promise the code did not keep — the panel
stated the numbers and left the reader to work out that they are about
bytes on disk. `/ByteRange` is a claim about bytes, so it can only be
checked against bytes, and the length used is the file **on disk right
now**. Once this shell can edit, "does the signature cover the file as it
currently exists" and "does it cover what I am looking at" are different
questions with different answers, and only one of them is being answered.

### `fn layers_session_only_note`

## Wording history, because this string has been wrong twice

It has now had three lives, and the record matters more than any one of
them: **nothing compiles a doc comment against the behaviour it
describes**, so the only defence is that the sentence and the control
change in the same commit, and that the reason is written down here when
it does.

| When | What it said | Was it true? |
|---|---|---|
| old shell, pre-checkbox | *"Switching a layer changes what you see, not the document. Nothing here is saved."* | **No** — for three commits it described a checkbox the panel did not yet have, and the error was found by a person reading the file. |
| old shell, post-checkbox | the same sentence | yes |
| this build, S3 | *"…Switching a layer on or off is not available in this build…"* | yes — the panel genuinely had no control |
| this build, S4 (**now**) | the sentence below | yes — the control is back |

## What changed at S4, and when

S4 completed the third and last of the preconditions
`crate::panels::layers`' header tracks. `crate::app::actions::Action`
gained `SetLayerVisible`, `ResetLayers` and `ToggleAnnotations`, each
implemented in `PdfcerApp::apply`; the render worker's `RenderKey` had
already gained `layers_generation`, and `crate::app::state::OpenDoc` had
already gained the override and its mutators. So the panel has a control
again, and the S3 clause — *"is not available in this build"* — became
false the moment it did. It is removed **in the same commit that added
the control**, which is the whole of the discipline this table exists to
record.

## Why the first sentence is the salvaged one, verbatim

*"Switching a layer changes what you see, not the document"* is the thing
an operator most needs to know and the hardest for them to discover: a
panel of tickboxes over a document is, by every other application's
convention, an editor. Restating it in fresh words re-derives a decision
already paid for, and the restatement has none of the evidence that bought
this wording — so it stands verbatim.

## Why a clause was ADDED, and what it is for

The old sentence's second half — *"Nothing here is saved"* — is true and
incomplete in a way that reads as a pdfcer limitation. It is not one:
§8.11.2.1 puts a group's live ON/OFF state **outside the document
entirely**, so there is nowhere in the file for a save to put it. An
operator who reads "not saved" as "pdfcer cannot save this yet" will wait
for a version that can, and no version can. So the sentence now names the
consequence they will actually meet — the states come back on reopen —
and attributes it to the format rather than to the build.

That is also why it does **not** promise the states survive anywhere
else: they do not survive a reopen, a second window, or an export.

### `fn layers_overridden`

Salvaged verbatim from the old shell. Shown beside the Reset control, and
only when the number is non-zero, so the pair reads as one statement:
*this many things differ, and here is the way back*.

**"Differ from the document", not "you changed"**, because the panel
computes it by comparing the effective hidden set against
`pdfcer_core::annot::optional_content_default_off` rather than by counting
clicks. A layer switched off and on again is back to agreeing with the
document and is not counted — which is the answer the operator would give
if asked, and the one a click-counter gets wrong.

### `fn layers_reset_tooltip`

Says what it returns **to**, because "reset" in a layers panel could
equally be read as "turn everything on" — and those are different acts on
a document that declares a "Confidential" watermark off by default.
Revealing such a layer is a disclosure event; returning to the document's
own configuration is the opposite of one.

The second sentence exists to make the difference concrete rather than
leaving it in the word "specifies".

### `fn layer_toggle_tooltip`

Salvaged verbatim. Repeats the boundary that
[`layers_session_only_note`] states above the list, deliberately: the note
is read once when the panel opens and the tooltip is read at the moment of
the click, which is when the question *"am I editing this file?"* is
actually being asked.

### `fn layer_overridden_tooltip`

Names the document's own state, so the operator can always see what they
are diverging FROM without resetting to find out. Salvaged verbatim.

Both arms are needed and they are not symmetric in consequence: *"You have
shown this layer. The document hides it."* is the one that matters, since
a layer the document hides may be hidden for a reason.

### `fn layers_auto_managed`

Says what the list IS rather than apologising for what it is not: the
states shown are the document's opening states, and for these layers the
page may legitimately disagree at the current zoom.

### `fn layer_design_intent_tooltip`

Explains why a layer the document lists as off is shown anyway —
otherwise the only available reading is "pdfcer got it wrong".

## A second sentence was added at S4, and it is not a stylistic one

The first sentence alone became **actively misleading** the moment the
visibility control returned, because it invites the reading *"so there is
no point switching this row"*. The opposite is true, and it is a property
of the engine rather than of this panel:

`pdfcer_render`'s interpreter resolves a group's state from
`oc_off_set()`, and when an operator override is in force that function
returns **the override verbatim** — no `/Intent` filtering, no `/AS`
usage application (`interpret.rs`, and `annot.rs` for annotation `/OC`).
Intent filtering happens only inside
`pdfcer_core::annot::optional_content_default_off`, which is what builds
the *document's* answer. So:

| state | is a design-intent group's `/OFF` membership honoured? |
|---|---|
| no override (the document's own configuration) | **no** — §8.11.2.3 filters it out, and the group draws |
| any override in force | **yes, for every group in the set**, this one included |

That asymmetry is the engine's documented replace-not-merge contract
(core API trap T-12.9) doing exactly what it says. It is disclosed rather
than papered over, per rule 4: pdfcer inferred something and the inference
changes the page.

### `fn layer_unnamed`

`/Name` is Required (Table 98), so its absence is a real malformation.
The placeholder says so rather than inventing "Layer 3", which would
disguise a defect as data from the file.

### `fn layer_locked_tooltip`

States what the lock actually is: the specification's own table blesses
JavaScript and `/AS` bypass, so calling it "cannot be changed" would
overstate it.

At S4 this became the tooltip on a **disabled** control rather than on a
bare row, which is why `crate::panels::layers` attaches it with
`on_disabled_hover_text` as well as `on_hover_text` — egui does not show
the ordinary hover text of a disabled widget, and this is the one row
whose explanation is the whole reason it looks broken.

### `fn layer_radio_tooltip`

Table 101's `/RBGroups` are "radio button" groups: at most one member
visible at a time.

**The wording did not change at S4, and that is worth recording**: it was
already written as a plain statement of what a switch does, so restoring
the control turned a description of the document into a description of the
panel without a word moving. The fact was worth knowing either way — a CAD
drawing with two mutually exclusive title blocks is a different document
from one with two independent ones.

Note what it does **not** say: that switching this layer *off* switches a
sibling on. "At most one" permits none, and choosing a replacement would
be pdfcer deciding which alternate the operator meant.

### `fn layer_radio_locked_sibling_tooltip`

## This is pdfcer's answer to a question the standard leaves open

`pdfcer_core::layers`' own module docs name it `DA-A8` and hand it here
verbatim: *"a locked group's state 'cannot be changed through the user
interface', while a sibling being turned ON means 'all others **shall** be
turned OFF'. Reported, not resolved — resolving it is the toggling
surface's decision to make and to disclose."*

**pdfcer lets the lock win.** Turning on a radio member leaves a locked
sibling exactly as it was, so the panel can end up showing two members of
a mutually exclusive group at once.

The reasoning, since the choice is not obvious and the losing option is
respectable: both rules are addressed to the *user interface*, so the
tie-break has to be about which failure an operator can see and act on.
Turning off a locked layer as a side effect of clicking a **different**
row is a lock bypass through a side door — invisible at the moment it
happens, and it is exactly the "Confidential watermark quietly switched
off" shape that `/Locked` exists to prevent. Two title blocks painted over
each other is wrong too, but it is wrong *on the screen*, where the
operator is already looking. Between an invisible violation and a visible
one, take the visible one — and then say so, which is what this string is.

### `fn bookmark_add_destination`

Stated rather than chosen, and stated by **page number**, for the reason the
Insert-from-file dialog gives for its own destination: the panel is beside a
document the operator may have scrolled, and the number is what makes the
choice checkable.

### `fn bookmark_add_under_collapsed`

The engine called it *"not a footnote … the entire difficulty of the
feature"*: a bookmark added under a **collapsed** ancestor does not change
the document's visible total, because it is not visible — §12.3.3 defines no
`/Open` key, so the sign on `/Count` is the only carrier of open-or-closed.

Getting the count right is the low bar. The operator's actual problem is
that they will add a bookmark, look at the panel, and **not see it** — and
the panel will be correct. So this is said **before** the press, which is
the same posture the ce-dimension group window takes about re-measuring on
a move.

Worded as a fact about the parent, not as a warning about the action: the
add will work perfectly.

### `fn bookmark_add_needs_a_title`

Greyed **with** an explanation rather than absent, unlike the Rename
button in the groups window — and the difference is which control it is.
That one is an alternative to a field that already shows the name; this one
is the whole of the feature, and a row that vanished until you typed would
leave an operator looking for where bookmarks are added.

### `fn bookmark_edit_heading`

Names both verbs, because the block is only drawn when a row is selected
and an operator who has clicked a bookmark to jump somewhere needs to know
why two new controls appeared under it.

### `fn bookmark_edit_selected`

The selected row is named rather than merely highlighted, for the reason
the ce-dimension group window names its group: this block sits **above** an
unbounded scroll area, so the row it acts on may be scrolled out of sight by
the time the operator presses a button. A highlight nobody can see is not a
selection indicator.

### `fn bookmark_copy_takes_subtree`

*"Remove"*, not *"Delete"*: the operator-facing distinction this application
keeps is that removing a bookmark takes it out of the navigation and leaves
every page exactly where it was. A button reading "Delete" beside a document
How many bookmarks a copy will take with it.

The same shape as `bookmark_delete_takes_subtree`, deliberately: an
operator who has read one has read the other, and a copy and a delete take
exactly the same set. Two wordings for one fact is how a panel comes to
describe two different operations that are in fact identical in scope.

### `fn bookmark_paste_heading`

It counts the WHOLE subtree, not the roots, because that is what will
arrive — and an operator who copied one chapter heading and sees *"12
bookmarks"* has learned something true that the tree did not show them.

### `fn bookmark_paste_destinations_dropped`

A pasted bookmark whose destination names a page this document does not have
is **dropped, not clamped** — it arrives, shows, keeps its title, and does
nothing when clicked. Nothing on screen distinguishes it from one that
works, which is why this is the only pre-press warning in the panel.

# Why it names both numbers

*"needs 14 pages, this one has 6"* is a fact the operator can act on: add
the sheets first, or accept the loss. *"Some destinations will be dropped"*
is a warning they can only obey or ignore.

It does **not** say how many will drop, and that is honest rather than
lazy: the clip knows its deepest destination, not the distribution of the
rest, so any count here would be a guess. The engine reports the real number
after the paste, which is where an exact figure belongs.

### `fn bookmark_paste_dropped`

The panel predicted this before the press and this is what happened, and
the two are not duplicates: a prediction is a guess nobody confirmed, and a
report alone arrives too late to choose differently. The operator gets the
choice *and* the outcome.

### `fn bookmark_delete_takes_subtree`

The engine's rule, and the reason it is Acrobat's too:

> *"promoting orphaned children to the deleted item's parent silently
> reorganises a document's navigation, and an operator who deleted one
> chapter heading would find its ten sections spliced into the top level."*

So the subtree goes, which is the predictable act — and it is also an act
whose size the operator **cannot see** when the row is collapsed, because
§12.3.3 gives a closed item's ancestors a `/Count` contribution of exactly
one however large its subtree is.

⇒ Stated as a fact about what the button will do, before it is pressed, in
the same posture `bookmark_add_under_collapsed` takes about the add. The
count is the shell's own read of the tree it drew; the count reported
afterwards is the engine's, and `bookmark_deleted` explains why they are
allowed to differ.

### `fn bookmark_delete_keeps_pages`

An outline is a document-level structure reached from the catalogue's
`/Outlines` (§12.3.3), not from any page. Removing a bookmark removes a way
of *reaching* a page and changes nothing drawn on one. Worth saying beside a
control that removes several things at once, because the operator's
reasonable fear at that moment is that the pages are what is being removed.

### `fn bookmark_deleted`

`EditSession::delete_outline_item` returns how many items went, the clicked
one included, and that number is the answer to the question this verb raises
and cannot answer any other way: the subtree went too, and on a collapsed
parent the operator could not see how large it was.

**It may disagree with the number promised before the press, and that is
why both are said.** `read_outline` gives up part-way on a cycle, on
excessive depth, or on exhausting its item budget — the panel draws a
truncation notice when it does — so the pre-press count is a count of *what
pdfcer could read* and this one is a count of *what it removed*. On any
ordinary document they agree.

One is not spelled as none: *"including its 0 bookmarks beneath it"* is the
shape of sentence that makes a program look like it is reading from a
template, so the leaf case gets its own words.

### `fn bookmark_rename_needs_a_title`

Absent rather than greyed, unlike the Add button one block up, and the
asymmetry is deliberate in both directions. The Add button is the **whole**
of its feature and a row that vanished until you typed would leave an
operator hunting for where bookmarks are added; the Rename button sits
beside a field that already shows the bookmark's current name, so the field
alone reads as *"this is what it is called"*, which is true. This sentence
is the hover text on the field for the one case worth explaining — a name
typed down to nothing.

### `fn bookmarks_truncated`

A truncated tree looks exactly like a short one from the outside, so
silence here would let an operator conclude the document simply has few
bookmarks. Stated as what it is: pdfcer stopped, the document did not end.

### `fn bookmark_untitled`

Its row still has to exist: a bookmark's children hang off it, and
omitting an untitled parent would show them at the wrong depth, silently
misrepresenting the document's structure.

### `fn bookmark_row_tooltip`

The destination page is stated rather than left to be discovered by
clicking: an operator scanning a long outline for "where is the parts
list" should not have to jump to find out.

### `fn bookmark_row_unresolved_tooltip`

Distinct from a bookmark with no destination at all, which is a heading
and perfectly normal. This one MEANT to point somewhere and pdfcer could
not work out where — the operator should know the difference before
concluding the document is broken.
