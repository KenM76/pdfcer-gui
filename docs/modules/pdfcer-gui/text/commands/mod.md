# text::commands — the label and tooltip of every ribbon command

One function per command, each returning a [`CommandText`]. The ribbon's
*structural* strings — tab labels, tab questions, group captions, mode
labels — live next door in [`crate::text::ribbon`].

## Why a pair rather than two functions

Every command needs both a label and a tooltip, and they are written
together: the tooltip's job is to say what the label cannot fit, so
reviewing one without the other is reviewing half a sentence. Two
functions per command would also double a file that is already the
longest in the catalog, for no gain — nothing ever wants one without
being able to reach the other.

## Every command has a tooltip. That is a rule, not an accident.

`RIBBON_IA.md` P3 reserves greying for *temporarily* unavailable — no
document open, undo stack empty — and requires that it *"is always
explained on hover."* A command with no tooltip cannot honour that, so
[`CommandText`] has no way to express "no tooltip" and a test below
asserts none is empty.

The salvage source got this wrong in exactly one place and it is
instructive: the four Measure buttons (`Linear Dimension`,
`Radius / Diameter Dimension`, `Set Group Scale…`, `Manage Dimension
Groups…`) were rendered as text-only selectables **with no tooltip at
all** — the four controls on the tab most likely to be used by someone
who has never used a PDF measuring tool.

## Voice, carried across from the salvage source deliberately

pdfcer's tooltips are unusually long and unusually specific, and that is
a deliberate quality of the product rather than an accident of who
wrote them. They say what a command *changes* ("This changes the
document, not just the view"), what it *cannot* do, and what is
*irreversible*. Where the salvage source's wording said something worth
keeping, it is kept close to verbatim.

**DO NOT QUOTE A LIVE TOOLTIP HERE AS AN EXEMPLAR.** A header that
holds one up as a model makes a SECOND COPY of that string's claim, in a
file nobody opens when the claim expires. Two such exemplars stood in this
paragraph and both were false before anyone noticed — one denying
verification the engine had grown, one calling an apply irreversible after
it began staging into the next save.

⇒ Name the **shape** of the good sentence, not its text. The examples
above survive because each describes a permanent property of a tooltip
rather than a measurement of the build.

Two things are trimmed:

1. **Tooltips that enumerate the alternatives.** The old `Add Text`
   tooltip explained itself by contrast with three other commands over
   four sentences. One contrast is a clarification; three is a menu.
2. **Tooltips that describe a defect.** `"click-to-place editing on the
   canvas is coming"` is a roadmap entry, not a tooltip.

## Labels: three renames that `RIBBON_IA.md` §5.4 requires

`Aa`, `I⁺ Aa` and `Obj` become **Edit text**, **Add text** and **Edit
objects**. They are the primary content-editing tools and were the
least legible controls in the application — and the first two returned
the *same literal*, `"Aa"`, distinguished only by icon and tooltip.

## Item notes

### `mod view`

Glob-re-exported, so a caller sees no seam at all: every call site still
writes `crate::text::commands::view_zoom_in()`. See that module's header
for why the cut is there and nowhere else.

### `struct CommandText`

A plain pair of `&'static str` rather than owned `String`s because
every one of them is a literal in this file: the catalog is the
definition site, so there is nothing to allocate and a command's text
can be read in a `const` context.

### `fn file_new`

**`New`, with no ellipsis**, and that is the label carrying a promise: an
ellipsis says *this will ask you something*, and this does not. Two of the
three reference applications create a document immediately from a default
(Acrobat from a locale default, Inkscape from its default template) and
only SolidWorks asks — and what it asks is which *kind* of document, a
question pdfcer has no analogue for. See `crate::app::blank` §3.

The tooltip states the page size **because the command does not ask for
it**. A default that is never mentioned is a default an operator discovers
by measuring the page they just made, and A4 is a real choice — argued from
what the three reference applications do, and from this operator's own
A-series drawings — rather than an accident to be hidden.

**The last sentence is about the SHAPE of saving, not about what this
build cannot do**, and the difference is the whole reason it is worded the
way it is. It once read *"this build cannot yet write a document to disk"*
— accurate while `file.save_copy` had no dispatch arm, and a lie to the
operator's face the day it got one, with a control two groups away keeping
the document the tooltip said could not be kept.

New is still the command where the shape of saving bites first: `Save a
copy…` asks for a destination every time and never adopts it, so a created
document keeps its `Untitled` name however often it is saved — Inkscape's
behaviour for the same verb. See `crate::app::save` §3.4.

⇒ ⚠ **A tooltip that denies a capability is a claim about the rest of the
program, and nothing re-reads it when the rest of the program changes.**
The only defence that works is catching it at the site of the change that
invalidated it, which means whoever wires a command reads the tooltips that
mention its absence.

### `fn file_new_from_template`

# The label follows `RIBBON_IA.md` and the tooltip corrects for it

§5.1 specifies the row as `New from template… (page size)`, following
Inkscape's `Ctrl+Alt+N`. What this shell offers is page sizes and not a
template gallery, so the word "template" over-promises — and the IA is
settled and reviewed, so a session may propose an amendment and may not
make one.

The tooltip is therefore doing real work rather than restating the label:
it says **page size** in its first four words, so an operator hovering
before they click learns what the window offers without opening it. See
`crate::dialogs::new_document`'s header for the full argument.

### `fn file_close`

**This tooltip is a SPECIFICATION that sat on the ribbon for weeks
unmet.** *"You are asked what to do about unsaved edits first"* — and
nothing asked: `Action::Close` consulted `save_pending`, permanently
`false` by design, then dropped the `EditSession`. Every edit since the file
was opened went with it, silently, with no prompt and no undo.

The sentence is **unchanged**, because it was never wrong about what pdfcer
should do. `crate::dialogs::unsaved` is the surface that now meets it.

⇒ ⚠ **An operator-visible string that describes behaviour is a claim, and
nothing in this project checks a claim of that shape.** The ui-strings gate
asserts the string lives in `text/`; the catalog tests assert it is a
sentence and that no two labels collide; no gate can ask whether it is
*true*. Only a reader holding the tooltip and the code side by side can,
and that is why these doc comments name the route rather than the promise.

### `fn file_recent`

**The control this text belongs to is not a button.** `file.recent` is
drawn by the `recent_files` custom item in File ▸ File — a menu of the
documents the operator had open — so this label and tooltip are what the
*menu button* says, and the rows inside it are file names from
[`crate::text::files`]. See `crate::shell::manifest::CUSTOM_BACKED`.

The tooltip names the two behaviours an operator would otherwise have to
discover: the cap, and the fact that a document on a drive which is not
connected right now is hidden rather than forgotten.

### `fn file_save`

**Save. In place. On the operator's instruction:** *"can I please have
a save button like every other program in existence has? We're on week two
of this and just have a save as button."*

# Why the argument against it does not hold here

[`file_save_copy`]'s doc comment said, and still says of itself:

> *"A button labelled `Save` would promise in-place saving, which cannot
> ship before autosave and crash recovery exist."*

That was a real position rather than an oversight, and it is **weaker than
it looks, for a reason specific to this application**: pdfcer writes an
INCREMENTAL UPDATE. The new revision is appended; the previous one stays in
the file, byte for byte, reachable through its own cross-reference table.
An in-place save here does not overwrite the operator's document in the
sense the objection assumed — **the format is the crash recovery**, and it
was already shipping.

What remained genuinely unsafe was the WRITE, not the save: `fs::write`
truncates and then streams, so a crash mid-write leaves a partial file where
a whole one was. That is a solved problem, and `save::save_in_place` solves
it — materialise the replacement in a temporary beside the target, then
rename, which either happens or does not.

So the honest account is not *"the operator overruled a safety rule"*. It is
that the rule was aimed at the wrong hazard, and the right hazard has a
three-line answer that had not been written because nobody was asking the
question. That is the same shape as `Ctrl+P` never being bound.

# The description names the incremental behaviour on purpose

Because an operator who has been told for a fortnight that pdfcer *never*
overwrites deserves to know exactly what changed, and because "the previous
version stays inside the file" is the fact that makes pressing this button
comfortable.

### `fn file_save_copy`

The label is `Save a copy…`, not `Save`, and that is load-bearing:
pdfcer writes the edits as an incremental update to a file you name, and
never overwrites the original unless you pick it. A button labelled
`Save` would promise in-place saving, which cannot ship before autosave
and crash recovery exist.

### `fn file_save_compacted`

**The name is the disclosure**, and it is the first line of defence
against a press nobody meant. `OPERATOR_REQUESTS.md` **O48** asked for it
*"named so it cannot be pressed by accident"*, and *Save a compacted copy…*
is a phrase nobody reaches for while looking for Save.

The tooltip names **both losses before the gain**. That inverts the usual
order and it is deliberate: an operator scanning tooltips reads the first
clause, and the first clause of this one has to be the part they cannot see.
A smaller file needs no advocacy — it is why they are hovering.

### `fn edit_reflow_block`

**The label says what it does to a paragraph, and the tooltip says the
one thing that will otherwise surprise.** `OPERATOR_REQUESTS.md` **O54**.

A reflow is planned against the document *as opened* — it needs position
information the in-session staging buffer does not carry — so it refuses a
page this session has already changed. One typed character is enough. That
is a correctness property rather than a limitation (the alternative is
splicing offsets into a stream that has moved), and the remedy is specific,
so both are in the tooltip where an operator meets them before the refusal
rather than after it.

## Why it leads with the preconditions — `OPERATOR_REQUESTS.md` **O127**

> *"I also haven't seen the reflow option actually work with anything when I
> press it."*

The sentence above **presupposed a caret**: *"the paragraph the caret is
in"* tells somebody who already has one what will happen, and tells somebody
who has not that they are missing something without naming it. And it named
only one of the two preconditions.

⇒ The tooltip now **leads with what must be true before the press**, in the
order the operator has to satisfy them, using the words on the buttons they
have to press (*Edit text*, not "place the caret"). R9's obligation is that a
control which requires something says so before it is pressed; this is that
sentence, and the `⊗` decline is the one after.

The third fact — that it only works on prose, not on a title-block cell or
an isolated label — is deliberately here too. It is the most likely refusal
on the drawings this program is for, and an operator who reads it here stops
pressing the control on a dimension label and wondering.

### `fn file_stamp_collection`

**The label and the tooltip are NOT written here.** They are
[`crate::text::stamps::command_label`] and
[`crate::text::stamps::command_tooltip`], and this function only pairs them
into the shape the registry takes.

The reason is the one `crate::text::protect::group_file_security` records
for a caption living beside its feature: **when a feature's copy is one
subject and one module, splitting it across two modules gives it two places
to drift.** Every other sentence this feature says — the window title, the
per-page rows, the five adjustment disclosures, the eleven Document
Properties strings — is in `crate::text::stamps`, and its vocabulary note
(*stamp* vs *collection* vs *category* vs *display name*) is the reason
those sentences agree with each other. A twelfth string over here would be
outside the one header that keeps them consistent.

### `fn file_import_form_data`

The tooltip says **what it overwrites**, because that is the fact an
operator needs before pressing rather than after. An import sets values in
the document they have open; a data file that names a field they have
already filled replaces what is in it. One `Ctrl+Z` takes the whole import
back — the engine makes it a single command however many fields it sets —
and saying so is what makes the press safe to try.

### `fn file_copy_page_text`

**It lives under FILE, not EDIT, because copying is not authoring** and
the File tab is the one tab every mode shows. ⚠ This file is ordered by
tab, and a command's text sitting under the wrong heading is how the next
reader concludes the command is somewhere it is not — so moving a command
between tabs moves its entry here too.

**A re-parenting does not rewrite the wording.** Nothing about what the
command does changes when its tab does, and a tooltip rewritten during a
move is a change nobody asked for arriving inside one they did.

The chord is `Ctrl+Shift+C`, bound to this id in
`crate::shell::manifest`'s keymap. The sentence about inferred word and
line breaks is the thing an operator cannot guess: a PDF is under no
obligation to record where a word ends, so pdfcer infers it from letter
positions and says how much of the copy that was.

### `fn file_copy_document_text`

Under File for [`file_copy_page_text`]'s reason. The warning about the
window not responding is the honest description of a synchronous extraction
over every page, and is exactly the kind of sentence a re-parenting must not
quietly lose.

### `fn file_properties`

## ONE COMMAND, ONE SUBJECT — and this tooltip names only one panel

It once read *"The document's own title, author, subject and keywords, AND
the properties of whatever is selected on the page"* — two subjects in one
sentence, and the panel drew both, the document half permanently, at the
foot of a surface whose subject is the selection. The operator: *"the
document properties are still always visible in the properties tab. it
needs to get out of there and be in its own document properties tab."*

The second subject is [`file_document_properties`]. ⚠ **A tooltip that
conjoins two subjects survives the split of the surfaces**, and then
promises on this control what a different control does — which is why the
wording was replaced rather than shortened.

It names the three kinds of thing that can be selected, because the panel
is empty until one of them is and an operator hovering an empty panel's
control deserves to know what would fill it.

### `fn file_document_properties`

**The operator's own words for the surface**: *"it needs to get out of
there and be in its own **document properties** tab."* The label
is what names the dock tab — `PdfcerApp::new` builds every `PanelInfo` from
its command's label — so this string is the tab he asked for, spelled the
way he asked for it.

*"Document properties"* rather than *"This document"* (the panel's own
heading) or *"Metadata"* (the format's word). A tab has to be recognisable
in a strip of five and legible out of context; a heading sits under a tab
that has already said which document. The two are deliberately different
strings, and `text::panels::docprops`' own test asserts they stay different.

The tooltip names the four fields, because they are the reason an operator
opens this, and then the facts, because they are what they get for free.

### `fn file_fonts`

Moved here from View ▸ Panels. The Fonts panel answers "what is inside
this file", not "what is on my screen", so it belongs beside Properties
as document-level inspection.

### `fn file_about`

**No ellipsis, deliberately.** This catalog's `…` means *you will be
asked something before anything happens* — the reading `view_reset_layout`
had its ellipsis taken away for getting wrong. About asks nothing; it
shows. Its neighbour `file_shortcuts` is the same kind of window and is
spelled the same way, and all three reference applications agree: Acrobat,
Inkscape and SolidWorks all write "About <product>" plain.

The tooltip names **all three** things the window carries rather than just
the version, because the version is the least of them. The reason this
command exists is the attribution surface — see [`crate::text::about`] —
and an operator looking for licence terms has to be able to tell from the
hover that this is where they live.

### `fn file_ocr`

**The tooltip states the uncertainty, and that is not optional here.**
OCR is the single largest inference pdfcer makes — `pdfcer-core`'s own
`ocr::layer` header says *"every word here is a guess"* — and rule 4 asks
that an inherently uncertain inference say so rather than imply otherwise.
A hover is the first place an operator meets this command, and a tooltip
that described only the benefit would be the sentence they remember.

It also states what does **not** change, because that is the question a
scanned document raises: nothing visible is added and the image is never
re-encoded, so a scan that is the record of something stays exactly the
bytes it was. That is `ocr::layer`'s own guarantee rather than a claim this
catalog is making on its behalf.

The dialog's fuller disclosure lives in [`crate::text::ocr`]; this is the
one-line version, and the two must not drift apart in what they promise.

### `fn pages_delete`

**`Delete pages`, not `Delete`.** `RIBBON_IA.md` §5.3 writes the row as
`Delete`, which is unambiguous *in its band* — it sits under a tab
called Pages, in a group called Organise, beside Extract and Move.
It is not unambiguous against the contextual Format tab's `Delete`,
which removes the selected object and can appear over any tab at any
time. Two controls reading `Delete`, one of which removes a sheet from
a drawing set, is a collision worth two extra characters.

### `fn pages_resize`

**The tooltip says what the command does NOT do**, and that is the
whole reason it is worded this way. Every other "page size" control an
operator has met — Word, LibreOffice, a print dialog's Fit-to-page —
reflows or scales, and this one changes the paper and leaves the drawing
exactly where it is. A tooltip reading *"change the page size"* would be
true, useless, and would confirm the wrong belief.

The window itself says it again, at greater length, with the overhang
measured — see `crate::text::page_size`. Saying it twice is deliberate: the
tooltip is what an operator reads *before deciding whether to open the
window at all*.

### `fn edit_attachments`

**The tooltip says what the panel SHOWS as well as what it does**, which
is [`view_panel_bookmarks`]' shape and is the right one here for a reason of
its own: an operator has no way to discover that a PDF can carry whole files
inside it, because nothing on the page ever shows one. A tooltip reading
only *"Manage attachments"* would name a capability to somebody who does not
know the capability exists.

It does **not** promise a description edit. `attach_file` takes a
description at attach time and `pdfcer-core` has no verb that changes one
afterwards, and `view_reset_layout`'s recorded defect is exactly this — a
tooltip that promised a choice the build did not offer. The panel discloses
the limit where an operator meets it.

### `fn edit_form_radio_button`

The tooltip names the grouping rule, because it is the only one of the
five whose behaviour depends on another field: two radios sharing a name are
one control. An operator who does not know that places two buttons that both
stay on and reasonably calls it a bug.

### `fn edit_select_all`

The tooltip names the RECOVERY rather than the mechanism, because that is
what sends an operator looking for it — *"I sometimes drop objects there,
and when I do I can't get them back."*

"Select all" is the label because it is the phrase every hand already
knows and a ribbon group competes for width. The off-the-sheet half — the
thing that makes this a rescue rather than a convenience — is in the
tooltip, where there is room to say it properly.

### `fn edit_form_push_button_unavailable`

R9 permits greying only for a **temporarily** unavailable capability
that is **always explained on hover**, and this is the explanation. It draws
the distinction that matters: pdfcer can *place* a button perfectly well —
what it cannot do is *run* what the button would do, because it executes no
PDF actions. Placing one would give the operator a control that looks
finished and does nothing, which is worse than not offering it.

It says what is missing rather than apologising, so an operator can judge
whether it matters to them and can ask for it if it does.

### `fn edit_form_manage_fields`

**The tooltip must not offer to "retype" a field**, because that is a
promise nothing can keep. Acrobat has offered no field-type conversion
since Acrobat 6, and
`pdfcer-core` models the same limit by making the request **unrepresentable**
rather than by accepting it and returning an error — so there is not even a
control to grey. A tooltip is a contract, and this clause had been offering
an operator something no route in either crate provides.

The label keeps *"Manage fields"* and the command now opens the **Forms
panel**, which is where listing, renaming and removing already live.

### `fn edit_find`

**The one command in this catalog whose only control is on the status
bar.** `RIBBON_IA.md` §6 puts the Find toggle there rather than on the
ribbon, so this label and tooltip are what that toggle's *command* says —
reachable from a keymap, from a customized quick-access toolbar, and from
the shortcut list — while `crate::text::find` holds the copy the bar's own
controls own. The two are not duplicates: this one is keyed by command id
and consumed by `crate::shell::commands`, that one is keyed by control and
consumed by a widget.

The tooltip names `Ctrl+F` because the chord genuinely works: the manifest
keymap binds it AND `crate::app::keyboard::parse_chord` can spell it. Both
halves are required — `Ctrl+O` was in the keymap and printed in a tooltip
for the whole of the ribbon's first life while pressing it did nothing,
because the spelling table held only digits.

It also names the two limits an operator has no way to guess and that
account for almost every surprising empty result: the search is over the
text **drawn on the pages**, and it matches within one text run at a time.

### `fn edit_redact`

**The closing clause must NOT read *"Marking is reversible; applying is
not."*** On the default destination, applying arms the next save rather
than rewriting at the click: the undo log survives, the page does not
change, and a Cancel disarms it.

What stays exactly as emphatic is that **once a file with the marks applied
has been written, nothing brings the content back.** The distinction is
*when*, never *whether*.

⚠ This tooltip and `crate::text::redact::panel_intro` describe the same
staging to the same operator, and are worded to match on purpose — **a
change to one is a change to both**, or an operator comparing them is
comparing two accounts. A change to the panel's catalog that leaves this
one alone is precisely how they last came apart.

### `fn edit_redact_apply`

**It must not say *"This cannot be undone."*** — see [`edit_redact`]
above for the whole account. On the default destination the click stages
the removal into the next save
(`crate::redact::stage_into_session`), which is undoable and cancellable;
the irreversible moment is the write, and both ordinary save routes refuse
by name while a redaction is armed rather than writing a half-redacted
file.

### `fn edit_undo`

# Why this does NOT name the operation, and what it would take to

The engine supplies everything a named one would need:
`EditSession::undo_kind` answers *what would be undone* without undoing it,
over 66 `CommandKind` variants. Writing *"Undo add annotation (Ctrl+Z)"* is
therefore catalog work and nothing more — a `CommandKind → &'static str`
mapping in this file, with a fallback for the kinds this shell cannot
author.

**The blocker is the registry, not the catalog.** `egui_shell`'s
`Command::tooltip` is a `String` fixed at registration;
`CommandRegistry` exposes `get`, `iter` and `register` and **no mutable
accessor and no removal**, and `PdfcerApp::commands` is built once in
`PdfcerApp::new` and handed to the ribbon by shared reference every frame.
So a tooltip that changes with the log needs one of two things:

1. a `get_mut` (or a `tooltip` closure) on `CommandRegistry` — which is a
   change to `crates/egui-shell`, the crate `check-shell-purity.sh` keeps
   application-agnostic and which this work is not permitted to touch; or
2. rebuilding the whole registry every frame so one string can differ —
   which pays an allocation per command per frame for one tooltip, and
   changes the **accessible name** of an icon-only control under the
   operator's pointer, since `egui_shell::ribbon::a11y` promotes the
   tooltip to the name when there is no visible label.

Half-doing it — naming the operation in the status bar instead, say — would
put the answer somewhere the operator is not looking when they hover the
control that asks the question. So the plain label ships, deliberately, and
the operation *is* named on the diagnostic channel (`undo kind=…`), which is
where it is currently readable. The right fix is (1), by whoever next has
cause to open `egui-shell`'s command registry.

The chord is still printed, and that is the part P3 actually requires: this
command is greyed whenever the log is empty, and a greyed control must
explain itself on hover. `egui_shell::ribbon::qat` uses
`on_disabled_hover_text`, so this sentence is read in exactly the state it
most needs to be.
