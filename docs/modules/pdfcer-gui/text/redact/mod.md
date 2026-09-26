# `text::redact` — every word the redaction surface says

Consumed by [`crate::panels::redact`] (mark and review) and
[`crate::dialogs::redact`] (the apply transaction and its report).


> The wording rules here are stricter than anywhere else in this catalog,
> because **this is the one feature where a comfortable sentence is a
> security defect.** Three of them, and binding on anyone editing these
> strings:
>
> 1. **Never say "removed" without qualification when anything was left.**
>    A residual is named in the SAME sentence as the success, never in a
>    footnote the operator can miss.
> 2. **Never say "verified" unless a verification step actually ran.** One
>    does — [`crate::redact::prepare_redaction_apply`] greps the finished
>    bytes — so the word is earned; but [`verified_line`] is the only place
>    it may appear, and only from a clean
>    [`crate::redact::AbsenceVerification`].
> 3. **Never put the word "Undo" near a post-apply state.** Every OTHER edit
>    in pdfcer teaches the operator that undo is available until save; this
>    is the one moment that learned expectation is wrong, so the copy
>    corrects it on screen instead of leaving it to be assumed.


Rule 3 was written for a world in which an apply was a write, and it survived
`Pass 250.1`'s collapsing verb because that verb destroyed the undo log
outright. `Pass 250.2` makes it false as stated: the deferred route
**preserves undo completely**, and a staged removal can be undone — by
stepping back over the marks, or by calling it off with
[`cancel_button_staged`]. A rule forbidding the word "Undo" near that state
would forbid the true and useful sentence.

What the rule was actually protecting is the state where undo genuinely does
not help, and that state is now precisely nameable:

> **3′. Never suggest that Undo can recover content that has reached a
> file.** Before the save, undo works and the copy may say so. After it, no
> sentence may offer undo as a way back — not on the write-now
> destinations, not on the deferred one after
> [`saved_applying_redaction`], not anywhere.

The distinction is the whole of it: undo reaches the **arming**, and never
reaches the **removal**. `tests::no_post_apply_sentence_mentions_undo_as_a_way_back`
enforces 3′ over the sentences the removal has happened in, and the sweep's
membership list is now the load-bearing half — a sentence about a
*pre*-save state belongs out of it, and one about a post-save state belongs
in it.

## The one distinction every string here has to keep alive

`crate::text::commands::edit_redact`'s shipped tooltip states it in four
words — ***"Marking is reversible; applying is not"*** — and
`crate::shell::manifest::edit` explains why the two commands sit together in
that order: *"the asymmetry between them is the dangerous part."*

The single most-cited real-world redaction failure is an operator who
believes marking **is** redacting and ships the marked file. So the marking
copy never says "removed", the review count says the content is *still
there*, and the apply copy leads with permanence rather than burying it.

## A departure from the source, and it is about this shell rather than
about copy

The old shell's permanence statement already deviated from its own ui-spec,
because apply there wrote a **new file** and left the open document alone.
That was true here too until 2026-09-04, when the operator asked for the
choice every other edit in this shell gives him — *"why can't it just wait on
saving until I choose to save over the existing file or save as a new
file?"* — so the permanence statement now has **three** forms, one per
destination: [`permanence_statement`]`(false)` for a new file,
[`permanence_statement`]`(true)` for replacing the open file, and
[`permanence_statement_deferred`] for the destination that lands in the open
document and writes nothing, which is the default and the one he actually
asked for. The clause that does **not** change between the three is the full
rewrite and the impossibility of getting the content back; what changes is
what happens to the file he opened. What is new is that this shell's *ordinary*
save is incremental
and promises so on `file.save_copy`'s tooltip — which makes
[`single_revision_note`] carry more weight here than it did there: it is the
one place an operator is told that this write does **not** behave like the
save they already know.

## Conventions

[`crate::text`]'s, unchanged: sentence case and no trailing period on a
label, full sentences with punctuation for prose, an ellipsis on a control
that asks a question before acting.

One addition of this module's own: **`⚠` is the residual mark and the only
non-ASCII character used here.** It is measured drawable — `DEFECTS.md` D12
records the correction that established it — and
`crate::icons::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph`
sweeps this file with the rest of the catalog, so a decorative glyph added
later that the bundled fonts cannot draw fails a test rather than shipping
as a box. That matters more on this surface than on any other: a residual
line whose first character is a broken box reads as a rendering failure, and
an operator who has decided a surface is broken stops reading it.

## Item notes

### `fn panel_intro`

States the whole two-phase model in one line, because an operator who
believes marking IS redacting is the single most-cited real-world redaction
failure. Carried verbatim from the old shell's `redact_panel_intro`.

### `fn mark_whole_page_disabled`

The control's only gate is `page_count > 0`, so there is exactly one reason
and it can be stated flatly. It is a document with no pages, which is legal
PDF (`/Count 0`) and which pdfcer opens rather than refusing — so this is a
real state an operator can be in, not a defensive branch.

It names the cause rather than the remedy, and that is the right way
round here: there is no action he can take inside this panel to give the
document a page, so *"add a page first"* would be advice about somewhere
else. Where a remedy exists — the search button beside it — the sentence
names the remedy instead.

### `fn match_pattern_tooltip`

Leads with the example rather than the syntax: the operator's actual thought
is *"redact every social security number"*, not *"I would like a wildcard
language."*

### `fn search_hint`

The scanned-page caveat is mandatory rather than decorative, and it is
carried in **both** modes deliberately. A silent zero-match result on a
scanned page is a named real-world failure: an operator reads *"no matches"*
as *"nothing sensitive here"* rather than as *"nothing SEARCHABLE here"*,
and dropping the warning from one of two hints is how it stops being read.

The pattern form states the whole syntax, because it is two characters long
and an operator who has to go looking for it will type a literal instead and
get nothing.

### `fn marks_count`

Zero is a distinct sentence rather than "0 marks", because *"no marks"* is a
state an operator reads as an answer while *"0 pending redaction mark(s)"*
is one they read as a counter.

The non-zero form is the load-bearing one, and the shouted clause is
deliberate: this is the sentence standing between a marked document and an
operator who is about to email it.

### `fn mark_row`

The size is shown because two marks on one page are otherwise
indistinguishable in a list, and *"which one is the one I mis-marked?"* is
the question the list exists to answer.

### `fn mark_remove`

**A word rather than a `✕` glyph.** The old shell's note is carried
because the measurement behind it is: a decorative Unicode glyph outside the
bundled font chain ships as a tofu box, and U+2715 is one of the codepoints
`DEFECTS.md` D12's corrected table lists as having **no supporting face** at
all. `crate::icons` exists for controls that need a mark; a list row does
not.

### `fn permanence_statement`

The middle clause — the full rewrite, and that nothing brings the content
back — is true either way and is worded identically in both, deliberately:
it is the part the operator must not have to read twice to compare.

### `fn removal_summary`

These are measurements, not predictions:
[`crate::redact::prepare_redaction_apply`] performs the whole removal in
memory before this dialog can show anything, so every number here describes
what actually happened to the bytes that will be written on confirm.

"character(s)", never "glyphs": the engine counts character codes removed
from content streams, and *glyph* is a typesetting word an operator has no
reason to know.

### `fn annotations_removed`

Overstating collateral damage is a smaller sin than understating it, and
still a lie — and this is the one feature whose entire value is that its
report can be believed. The overlap fact is still disclosed, because an
operator whose highlight silently vanished is owed the reason before it
happens rather than after.

### `fn single_revision_note`

It carries more weight in this shell than it did in the one it came from.
`file.save_copy`'s shipped tooltip promises that an ordinary save *"appends
the edits as an update so the previous version stays intact inside the
file"* — which is exactly the property a redaction must not have, and which
an operator has by then been taught to expect. This is the one sentence that
tells them this write is different.

### `fn verified_line`

Rule 2 of the module header. It is shown only from a clean
[`crate::redact::AbsenceVerification`], and what licenses it is that a real
search ran over the real output bytes: `crate::redact::proof` re-parses the
finished document, decodes every stream in it, and greps both the decoded
content and the raw buffer.

The three clauses at the end are the three places it looked, named
individually rather than summarised, because *"we checked"* is a claim and
*"we looked in the page content, in every other stream, and in the raw
bytes"* is a description someone could go and repeat.

### `fn verification_limit_line`

[`crate::redact::proof::MIN_VERIFIABLE_LEN`] is the four this names. A proof
that quietly skipped these would be claiming a completeness it does not
have.


It read: *"Some producers draw text one letter at a time; on such a file
every removed piece is one character and this proof cannot see it at all."*
That was **measured, true, and operator-visible** when it shipped this
morning — his 24-page Ghostscript drawing reported `["3", ".", "5", " ",
"T", "Y", "P"]` for one mark over `3.5 TYP`, seven needles all under the
floor, so the proof genuinely saw nothing.

It became false at engine `369d4de` (`Pass 286.0`, the same day), which is
the rev this repository is pinned to. `RedactionReport::redacted_text` is
now **one entry per `/Redact` mark**, carrying the concatenation of what
that mark removed — `["3.5 TYP"]` — so the per-glyph producer produces
ordinary words and clears the floor like any other file. The engine
volunteered that this *also* repaired `carrier_info` and the residual sweep
on such files, for the same reason.


**The first clause stays, and it is not a leftover.** A mark that covers
a genuinely short string — a single dimension `3`, an initial, a room
number — still yields a genuinely short needle, and no grouping in the
engine changes that. The engine said so in the same breath: *"a mark
covering a single character still yields a single character, and no
grouping changes that. Belt and braces is the right posture on the one
operation where a false 'clean' is an incident."*

### `fn mark_covers_image`

# This sentence was the exact opposite of the truth for nine hours

It read: *"pdfcer cannot yet remove image pixels, so applying redactions to
this document will be refused until no marked region touches an image."*
That was accurate when it was written on 2026-09-03 — it is the operator's
own report, reproduced with `pdfcer` — and `pdfcer-core` **v0.26.0**
(`Pass 245.0`, the same day) made it false: a region covering image samples
now clears those samples, and a region covering a whole image removes it.

⇒ The class this belongs to is the one this project keeps paying for: **a
sentence describing an external limitation is a dated citation, not a
verdict.** Nothing compiles differently when the limitation lifts, no test
goes red, and the gates stay green *precisely because* the code around the
sentence is unchanged. The engine's own reply said to re-word this, by name.
It is worth noticing that the engine had to tell us — see
`tools/gates/check-stale-blockers.sh`, which is aimed at exactly this and
could not have caught a claim phrased as a UI string.

# Why it is still said at MARK time, and why it is not a warning

The disclosure changed subject rather than going away. It used to say *"this
will be refused"*; it now says *"this will be destroyed"*, and that is the
more important of the two. A raster redaction is irreversible in a way a
text one is not: the samples are overwritten, the image is re-encoded, and
what the operator gets back is a black block where their logo was. Seeing
that fact while the rectangle is being drawn — rather than discovering it in
the saved file — is the same argument the original sentence made, applied to
the opposite outcome.

It offers no remedy, deliberately, for the reason it always did: telling
him to move the rectangle is advice we cannot check, because on a title
block the value and the logo may genuinely overlap.

The mark is still authored. This blocks nothing, and now nothing further
down the line blocks either.

### `fn residual_heading`

It states the **consequence** and never a limit of pdfcer, because the
section below it collects three different causes and only one of them is a
limit: content pdfcer could not reach, a byte run it could not rule out,
and — once the redaction reach is narrowed — copies it found, can remove,
and was told to leave. A heading reading "pdfcer could not" would make that
third kind read as a failure, which is the one thing the engine keeps two
separate verdicts to prevent. What every line under it has in common is
what the heading says: it survives the save.

### `fn raw_residual_line`

Worded so it claims exactly what pdfcer knows and nothing more: the byte run
is there, in *this* kind of place; whether it is a real leftover copy or an
unrelated coincidence is not something pdfcer can decide. See
`crate::redact::proof`'s table for why this is disclosed rather than refused.


This sentence used to end at *"somewhere in the saved file"*, and the
operator's report of that day is what a warning of that shape produces:

> *"it always finds text that wasn't redacted, and it always … counts
> everything I selected as unredactable."*

He was right on the facts and the sentence gave him nothing to do about
them. The commonest cause by far — an embedded font program whose `name`
table happens to spell an ordinary English word — is one an operator can
dismiss in a second **if they are told that is where it is**, and cannot
evaluate at all if they are not. A warning nobody can act on is a warning
everybody learns to click past, which then costs the real one its force.

It still refuses to judge. It says where the bytes are, in the operator's
vocabulary, and then repeats that pdfcer cannot tell a coincidence from a
carrier. Naming the place is more information, not a verdict — the catalog's
rule 1 is *"never say removed without qualification when anything was left"*,
and nothing here says removed.

### `fn images_destroyed`

# Why this is stated even though it is a SUCCESS


That is exactly what they asked for and it is still worth saying out loud —
the same argument the mark-time sentence makes, restated where the numbers
are. A report that lists glyphs removed and says nothing about a destroyed
photograph is a report that has quietly picked which irreversible act is
worth mentioning.

`over_covered` is separate and is a disclosure rather than a count: a
rotated or skewed image placement is cleared by its bounding rectangle **in
image space**, so more is destroyed than was marked. Never less — the
engine's own guarantee — but "more than you drew" is a fact about the
operator's file and it is theirs to know.

### `fn images_shared_copied`

Not a residual and not a warning: the *unmarked* placements were not
marked, so leaving them intact is correct. What the operator needs to know is
that the same picture still exists elsewhere in the document, because "I
redacted the logo" and "the logo is gone from this file" are different
claims and the second one is false here.

### `fn marks_retained_line`

# The one number that must be read before the word "redacted" is used

The engine says so by name: a retained mark is a region where **nothing was
removed**. It happens when a region touches an image whose samples pdfcer
cannot decode — a codec feature it lacks, a corrupt codestream — and the
engine's choice is to apply every other mark and leave that one standing
rather than refuse the document. That is the right choice and it makes a
half-redacted file that looks finished.

So this is a residual, in the strongest sense in this module: the content the
operator asked to be removed is still there, under a rectangle that says it
is not. It goes in the acknowledgement list, and the mark itself is still
visible in the output so a second pass can find it.

### `fn vector_paths_residual_line`

`RedactionReport::vector_paths_intersecting`. Since `pdfcer-core` v0.27.0 the
engine **cuts** paths at the region boundary, so this counts only the ones it
could not rewrite as a unit — a malformed path object — and reads zero on
every well-formed page. A non-zero value is therefore rare and is a real
residual rather than the ordinary case.

# This paragraph has been wrong twice in one morning, in both directions

Written first from `D:\Dev\pdfcer`'s **working tree**, which described
cutting the engine had not committed. Corrected to say cutting does not
exist, citing the pinned revision — and within the hour the engine shipped
v0.27.0 and the correction became the false half.

⇒ The rule is not *"do not read the engine's source"*: reading it is right,
and the second version was right about the revision it named. The rule is
that **a sentence about what the engine cannot do is a dated citation with a
shelf life measured in hours**, because that session runs in parallel and
answers within the hour. Where the claim can be spelled as an **assertion**,
spell it as one — `redact::tests`'s image test went red the moment the
engine changed underneath it, which is exactly the behaviour a paragraph
cannot have.

On a CAD sheet this is the residual that matters most and the one nobody
asks about. A title-block border or a view's geometry running through a
redacted rectangle is a shape, and a shape can be as identifying as the text
it surrounded.

### `fn vector_clips_kept_line`

`RedactionReport::vector_clips_kept`. ISO 32000-1 §8.5.4 applies a clipping
path *after* painting, so an object marked `W`/`W*` sets the window every
later object on the page draws through. The engine cuts its paint and keeps
its original geometry as the clip, because shrinking the clip would hide
later, **unmarked** content.

Kept geometry is not painted content — nothing of it is visible — and it is
still a shape in the file. Rule 1: named, in the operator's terms, rather
than judged harmless on their behalf.

### `fn vector_paths_cut_line`

# Why a success gets a line on a drawing

Because until v0.27.0 it did not happen. Lines ran straight through a
redacted rectangle, the file was reported as redacted, and nothing said
otherwise — the engine found it while verifying the image work and called it
*"the bigger one on a drawing"*. On a CAD sheet the geometry under a black
box can be as identifying as the text was.

So this is the count that turns *"the drawing under the box is gone"* from an
assumption into a statement. `dropped` is the subset that lay wholly inside a
region and was deleted outright, and it is named separately because a
deleted object and a trimmed one are different facts about the file.

### `fn residual_acknowledgement_checkbox`

Distinct from [`confirm_checkbox`] on purpose: a partial redaction must
never be mistaken for a complete one. Showing it always would make it a box
operators tick without reading, which is the failure mode that makes every
other acknowledgement in the program worthless.

### `fn confirm_disabled`

# It names WHICH box, because *"tick the box"* is ambiguous here

Two checkboxes gate this button and they do not both appear. The
acknowledgement is always shown; the residual acknowledgement is shown only
when the engine reported content it could not prove was removed — which is
precisely the situation in which an operator is reading carefully and is
least able to afford a vague refusal.

Four states, three of them reachable: the pair is only ever consulted when
the button is off, so `(true, true)` cannot be seen here. It is answered
anyway rather than left to a `_` arm, because a sentence that cannot be
reached is better than a panic and better than a wrong one, and because the
day the gate grows a third term this arm is where the omission shows.


The gate grew a third term: [`overwrite_acknowledgement_checkbox`], asked
for only when the operator has chosen to replace the open file. The comment
above predicted where the omission would show and it showed there — the
`(true, true)` arm stopped being unreachable and started meaning *"both the
boxes I know about are ticked"*, which on a replace would have greyed the
button and said **"Ready."**

So the parameters are now three, and all three are stated as
**outstanding** rather than as *acknowledged*. That is not tidying. A box
that is not being asked for is neither ticked nor untickable, and reading
`residuals_acknowledged == false` as *"go and tick it"* when no such box is
on screen is exactly the vague refusal this function exists to prevent. The
caller — which is the only surface that knows which boxes it drew — answers
the question *"is this one still owed?"*, and the answer for a box that was
never drawn is *no*.

# The order the boxes are named in

Top to bottom as they are drawn, because the operator is being sent to look
at one: the overwrite acknowledgement sits directly under the destination
choice, the residual acknowledgement under that, and the permanence
acknowledgement immediately above the button.

### `fn no_shortcut_note`

Visible text rather than an omission an operator has to notice: this shell
binds `Ctrl+Z`, `Delete` and the whole `Ctrl` chord family to
destructive-but-reversible actions everywhere else, so the ABSENCE of a
chord on the one irreversible action is a deliberate asymmetry worth
stating.


The fix is to make the sentence true on **every** destination rather than
to branch it, because the reason for the missing chord does not vary: what
this button starts always ends in a write that no Undo reaches. Branching
would have produced a second pair of strings to keep in step, which is the
condition that produced this defect.

### `fn suggested_suffix`

A suggestion, not a rule — the operator can type anything. It exists so the
default answer is never the file they opened, which on this operation is the
difference between a copy and the destruction of the only remaining source
of the content being removed. `crate::dialogs::ocr::suggested_suffix` and
`crate::text::files::save_copy_suffix` enforce the identical rule for the
two milder writes.

### `fn applied_clean`

**CORRECTED the same evening.** The replace form used to explain that
staleness with *"because pdfcer cannot apply a redaction into an open
document"*. That was true when it was written and stopped being true a few
hours later, when `Pass 250.1` shipped `EditSession::apply_redactions` and
[`destination_open_document`] became the default. The window is still stale
on the two write-now destinations — that has not changed — but the reason is
now a **choice the operator made**, not a limit of the program, and a
sentence that blames the program for a chosen behaviour teaches him the
wrong thing about a control he is holding.

### `fn applied_with_residuals`

Never shortened, never omitted, and never allowed to borrow the clean form's
wording: an operator who acknowledged a residual in a dialog and then closed
it is still owed a standing record of what remains. This is rule 1 —
**the residual is named in the same sentence as the success.**

### `fn refusal_message`

Every variant of [`crate::redact::RedactApplyRefusal`] gets its own sentence
rather than one "redaction failed", for `crate::text::ocr`'s reason: the
engine refuses by name because the causes have different remedies, and
folding four named causes into one message throws that away at the last
step.

### `fn save_refused_message`

It is separate from [`refusal_message`] because the moment is different
and so is what the operator is holding. That one is read when he presses
*Review & apply* and nothing has been decided. This is read after he pressed
**Save** and expected a file, so it leads with the fact that no file was
written and ends with the one control that unblocks him — named, because a
refusal an operator cannot act on is a refusal he learns to ignore.

The `_` arm is not a shrug. Every other [`crate::redact::RedactApplyRefusal`]
reaching this path means the engine declined the removal itself — an
undecodable image, an encrypted document, a hybrid base — and those already
carry [`refusal_message`]'s own worded cause. Repeating the cause here in
different words is how two sentences about one event come to disagree; what
this adds is the part [`refusal_message`] cannot know, which is that the
operator was trying to **save**.

### `fn write_failed`

Distinct from [`refusal_message`] because the two happen at different
moments and mean different things to the operator: a refusal happens when
they press *Review & apply* and nothing has been decided yet, and this
happens after they have confirmed, named a destination and expect a file to
be there.

### `fn save_kept_pending_marks`

Fires in ADDITION to the save's own outcome, never instead of it: the save
genuinely succeeded, and the operator also needs to know what it did not do.
This is the sentence that stands between `file.save_copy` and a marked file
leaving the building.

### `fn appearance_intro`

It says **applied**, twice over, because that is the distinction the
whole panel turns on. Nothing chosen here changes anything until the
operator applies — a mark is a red outline whatever fill is set — and an
operator who expected the swatch to recolour their marks would otherwise
conclude the control does nothing.

### `fn fill_transparent_note`

The one fill an operator can misread as "do not redact". It removes the
content exactly as the others do; what it omits is the box that says so.
Said at the control rather than in a tooltip, because a tooltip is not
read before a choice is made and this is the choice with a surprise in it.

### `fn overlay_illegible_warning`

The engine hard-codes black text in the `/DA` it authors and said so when
it shipped the burn-in: *"Wire a fill-colour picker, let someone choose a
dark red, and the caption will be black on dark red — we saw it in our own
verification render."* There is no overlay-text colour on the API yet.

So this is a **disclosure of a known engine limit**, not a style opinion,
and it names the two ways out rather than only the problem.

### `fn overlay_bound`

Two engine limits in one sentence, both stated when the burn-in shipped:
the face is Base-14 Latin, so anything outside it becomes `?`; and the
size is auto-chosen within a clamp, so a long caption on a small mark is
not going to be readable however it is justified.

### `fn unreadable_warning`

The strongest wording anywhere in this catalogue, and deliberately so.
Every other disclosure in pdfcer reports something the operator can take or
leave. This one reports that **an operation they believe completed may not
have**, on the one action with no undo and the one whose failure they will
discover after sending the file.

It does not say "0 results". It says what was searched and what could not be
searched, and it says the consequence in the operator's own terms — the text
is still there — rather than in the mechanism's.

### `fn marked_selection`

`OPERATOR_REQUESTS.md` **O60**. The third marking route's disclosure.

# Why it says MARKED and not REDACTED

Because nothing has been removed, and the difference is the single most
important thing about this whole feature. A `/Redact` annotation is a
**mark** (§12.5.6.23): it covers nothing, deletes nothing, and is perfectly
reversible until *Apply* is pressed.

An operator who read *"3 objects redacted"* would reasonably believe the
content was gone, stop reviewing, and save a document that still contains
every word of it. That is the one mistake in this feature that cannot be
undone by undoing — because it is a mistake about what to do next.

⇒ So the sentence names the state and the next step, in that order.

# Why it counts OBJECTS and not marks

One gesture makes one annotation carrying one quad per object, so the mark
count is always 1 and would tell the operator nothing. What they chose was
objects; what they should be told about is objects.
