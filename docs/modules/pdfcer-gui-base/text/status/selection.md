# `text::status::selection` — what is selected, said in words

Every string [`crate::app::status::selected`] draws, and nothing else. One
subject, one consumer — the organising principle
[`crate::text`]'s header states for the whole catalog, applied inside an
area that had grown large enough to need it.

## Why this became its own file

Because R2 said so, and R2 was right. `text::status` crossed 1,500 lines
when the form-containment clause landed, and the rule this project was
founded on is that the limit is the signal to find the seam rather than to
raise the limit — the GUI being replaced reached 25,005 lines in one
`main.rs`, and *"nothing could be reasoned about locally"* is the direct
cause of most of what is wrong with it.

The seam was already there. Every function here is read by exactly one
widget, they are the only strings in the area that describe a *thing the
operator picked* rather than a control they can press, and they now include
the two sentences the form-XObject work turned on. The paths do not move:
`status`'s `mod.rs` re-exports this, so `t::selection_one` still resolves
and no call site changed. A catalog area is keyed by its consumer, and the
consumer did not change — only the file did.

## The four sentences, and the state each is for

| state | line |
|---|---|
| one page object | `Selected: Path · 120.0 × 40.0 pt` |
| one object inside a form XObject | `… · inside a form` |
| several things | `3 objects selected` |
| anything, with more underneath | `… · 1 of 5 here` |

**The containment clause is [rule 4](R8b) disclosure and it is
off-canvas.** A form-interior object is drawn on the page exactly as it
will be drawn when saved — no badge, no tint, no dashed outline. What
pdfcer had to do to find it is reported here, in words, on a bar; never by
marking the drawing.

## Item notes

### `fn coverage_line`

# Remedy first, which reverses the sentence when there is one

This module's rule is *remedy first in every arm that has one*, because the
operator is looking at text that did not change and the useful half is what
to do now. Without a list there is no remedy to lead with and the sentence
opens on the diagnosis; with one it opens on the faces. That is two
sentences rather than one with a clause bolted on, and it is deliberate: a
sentence that opens *"That face has no shape…"* and ends *"… Times-Roman
can"* buries the actionable half behind the explanation.

`"The face you picked"` rather than naming it. The name is in the face
chooser the operator is looking at, and repeating it costs width on a bar
that is already carrying up to fourteen face names in the first clause.

# Why `const WITHOUT` and not a second catalog function

Because it is the *same* refusal. Two catalog entries would be two
sentences that must be kept consistent with each other by hand, and this
project has a gate (`check-ui-strings`) that would be content with both.

### `fn join_or`

`or`, not `and`: the faces are **alternatives**, and `join_and` in
`crate::text::page_size` — whose subject is edges a drawing runs past, all
of which are true at once — would read as though the operator needed all
three. Copied rather than shared for exactly that reason: the two differ in
the one word that carries the meaning, so a shared helper would need a
parameter that is really a choice about a sentence.

### `fn the_rung_clause_counts_what_is_held`

The wording carried a literal `1` for as long as one chunk was all the
rung could hold. A Shift-click now adds a second, and the drag that
follows moves the set — so a sentence that cannot say *2 lines of 27* is
a confident, wrong statement about the operand of the next keystroke.

### `fn selection_one`

**What is selected**, for the status bar's left-hand readout.

# Why this line exists at all

The operator, 2026-08-26: *"when I click on one of the objects all I get is
the page selected."* He was right, and nothing on screen said so — the
selection outline round a page-sized object looks exactly like *"the page
is selected"*, which is a state this program does not have. This is the
sentence that turns that into a diagnosis.

# The wording

**The kind first**, because it is the word that answers his question:
*Form* is the one that explains a page-sized outline, and it is the word he
would have needed to ask the right next question.

**Size in points**, to one decimal, because the operator works in a CAD
world where a number is how you tell two similar things apart, and because
a page-sized object is obvious the moment its size is beside the page's.
Not millimetres: the rest of this bar and the geometry fields are in points,
and one surface in two units is worse than either unit.

### `fn selection_one_unsized`

Rare — it needs a page transform that will not invert — and it says less
rather than saying something invented. A size derived from a failed
projection would be a number the operator could act on and could not trust.

### `fn selection_one_in_form`

# The sentence this whole change exists to make sayable

The operator, 2026-08-26: *"when I click on one of the objects all I get is
the page selected."* He was clicking a real object; a page-sized form
XObject wrapped it, the form's `/BBox` won every hit test, and **nothing on
screen said the word "form" anywhere**. The selection outline round the page
edge looked exactly like a state this program does not have.

The engine now descends into forms, so the click lands on the object he
meant. This clause is what stops the *next* question — *"why can I select
it but not move it?"* — from being as unanswerable as the first one was.

# Why the suffix, and not a different sentence

Because it is the same selection, described more completely. Kind and size
are unchanged and still lead, because they are what the operator asked for
by clicking; the containment is the qualifier. A separate line would read as
a separate subject, which is `status::selected`'s standing rule about the
depth clause too.

# Rule 4: this is DISCLOSURE, and it is off-canvas

Nothing is drawn differently on the page. A form-interior object renders
exactly as it will render when saved, with no badge, tint or dashed
outline — the operator's own finding that *"the nagging and red flagging in
the original GUI made for a lot of extra bugs in the visibility when
editing"*. The fact that pdfcer reached inside a form to find this object is
reported here, in the status bar, and nowhere on the drawing.

# The count

`nesting` is [`pdfcer_core::vector::FormLeaf::containment`]'s length — how
many forms enclose the object, outermost first. One is overwhelmingly the
common case and gets the article rather than the digit, because *"inside 1
form"* reads like a computer counting. Deeper nesting is worth the number:
it is the difference between "this is in the title block" and "this is
three wrappers down", which changes what the operator does next.

### `fn inside_container`

`OPERATOR_REQUESTS.md` O70. The one state in the Smart-Selector arm with no
visible evidence anywhere else: no outline, no armed tool, nothing on the
page — just clicks that resolve differently from how they resolved a moment
ago.

It names **Escape** for the reason `text::placing::armed_instruction`
does: this is the only statement of the way out that the operator can read
at the moment they need it, and a scope with no visible exit is exactly the
stranding the design exists to prevent.

### `enum InsideFormRefusal`

# The sentence this replaced, and why it had to go


> *"That object is inside a form — pdfcer cannot edit inside one yet"*


`TextStyleRefusal::line` below states the rule this violates, and it states
it about exactly this species of mistake: *a refusal sentence that states a
limit the build no longer has is worse than no sentence — it teaches the
operator not to try something the program can do, and it does so with the
program's own voice.*

# Why splitting it was not optional once it was wrong

One string was serving two call sites whose facts had drifted apart in
opposite directions, which is how it stayed wrong: neither site could be
corrected without making the other one worse.

* The **move** path reaches this when a part or node inside a form is
  entered and the part's kind cannot be read. The address space is fine —
  `move_subpath_in_form` and `move_node_in_form` are wired and work — so
  the fact is *this is not a path*, not *this is out of reach*.
* **Select containing form** reaches it when there is no containing form to
  reach, which is the OPPOSITE condition: nothing selected is inside one.
  The old sentence told that operator their selection was inside a form at
  the exact moment pdfcer had established it was not.

# The second site was silent, and the split is what exposed it

`Declined::still_true` filtered `InsideForm` on `selection_in_form`, which
is the right predicate for a sentence about a form-interior selection and
the wrong one for a sentence about not having such a selection. So
**Select containing form pressed with nothing form-interior selected
recorded a decline that was discarded before the bar could draw it** — the
operator got no outline, no movement and no sentence. A variant per fact
gives each one its own retirement rule, and [`Self::NoContainingForm`]'s is
`true`.

### `fn selection_many`

No kinds and no size: a mixed selection has neither, and picking the first
object's kind to stand for all of them would be a claim about the set that
is false the moment the set is mixed. The count is the honest whole of what
can be said until a multi-selection summary is built.

### `fn selection_part_of_text`

# The gap this closes


⇒ That is the failure mode this module's header is organised around,
one rung down: *reporting the wrong thing is worse than reporting
nothing*, because a line that is confidently about the block gives the
operator no reason to suspect he is not holding the block.

# Why there is no index in it

The obvious wording is *line 4 of 27*, and it was refused. The Objects
panel already numbers the same thing, as `Line #3`, **zero-based on
purpose** — decision 025 §1.3(b), so that the number pdfcer shows and
the number a `pdfcer` command line addresses are one number. A status bar
saying *line 4* beside a panel saying *Line #3* about the same line is two
numberings of one thing, which is the drift this project spends its
corrections on.

*1 line of 27* needs no index space at all. It says what he has (one
line), and how much there is (twenty-seven), and it cannot disagree with
the panel because it does not name a position.

# `held` is counted, never assumed

A Shift-click at this rung adds a second chunk, and the operand of the next
drag or Delete is then the whole set. A sentence with the literal `1` in it
says *1 line of 27* while four are outlined and four are about to move,
which is the module header's failure mode exactly: confidently wrong beats
silent at nothing.

# Rule 4

Nothing is drawn on the drawing to express the rung. The selection
outline is the cursor and is untouched; this is the off-canvas half.

### `fn selection_part_of_path`

Two functions rather than one with a flag, because the two words are
the whole content of the difference and a flag would put the choice at
the call site with nothing beside it explaining which is which. The
caller asks `ObjectModelProvider::part_kind` — the one dispatcher the
Objects panel's row builder also asks — so the two surfaces cannot
disagree about which kind of part is selected.

### `fn selection_part_of_text_hint`

# Why the way out is stated and not assumed

A rung with no visible exit is a stranding, and this module's
`inside_container` note records the same argument for the scope a bare
Escape leaves behind. The Part rung is reachable in one right-click now,
which means it is reachable by an operator who did not set out to go
there — so the sentence that gets him back has to be somewhere he is
already looking.

**It names both verbs, because the engine has both.**
`delete_text_run` removes the line and `move_text_run` moves it, and
`canvas::moving` routes a Part-rung drag on a run to the second exactly as
it routes a Part-rung drag on a subpath to `move_subpath`. A hover that
named only Delete would be telling the operator a shipped gesture does not
exist — which is the failure mode O214 reported, in his words *"I thought
we worked on this … but it didn't make it here"*.

**The two lines the engine refuses are not hedged into this sentence.**
A run with no position of its own, and a run the next line's position is
measured from, each get their own wording from `crate::text::arrange` at
the moment the drag is declined. Folding a *"sometimes"* in here would cost
every other line its plain instruction to buy a caveat the refused ones
already state better.

### `fn selection_part_of_path_hint`

The verbs are the same as the text one's — both parts drag and both
parts delete — and what differs is the **nouns**: *part* of a *shape*
against *line* of a *block of text*. That is why they stay two sentences
rather than becoming one parameterised one: an operator told he is holding
a shape goes looking for corner handles a line of text does not have.

### `fn selection_with_depth`

Appended to whichever line above applies, because it is a fact about the
same selection: *"this one, and there were others."*

This is the half that makes `Alt`+click discoverable. A cycling gesture
nobody knows about is a gesture nobody uses, and the operator has no way to
learn that four more objects were under his pointer unless something says
so. *"1 of 5 here"* says both that this is not the only answer and that
there is a question worth asking.

`here` rather than `under the pointer`: the bar has finite width and the
word is doing one job — locating the count at the click rather than in the
document.

### `enum TextStyleRefusal`

# Why this is an enum here rather than a `String` from the engine

`FormatError` writes excellent prose about itself — the synthetic-italic
refusal explains the `Td` interaction, names §9.4.2 Table 108 and ends
*"Nothing was applied"* — and it is tempting to put it on the status bar
verbatim.

`check-ui-strings.sh` exclusion 3 says in as many words that an error type's
prose is **not** permission to route UI text through it, and the reason is
not tidiness. The engine's sentence is written for whoever is debugging: it
names the rule, the clause and the mechanism. An operator restyling a title
block needs the *remedy* first and does not need `Tm` at all. Two audiences,
two sentences; the engine's goes to the trace, where its audience is.

So this enum is the shell's own reading of which refusals an operator can
**act on**, and there are three. Everything else is either impossible from
this surface (a bad page index, an empty request) or is not improved by
being subdivided.


One variant now owns a list of face names the engine computed, so the enum
holds a `Vec<String>` and cannot be `Copy`. Three doc comments in
[`crate::app::status::decline`] used to argue that this type is `Copy`
*"so that `Declined` stays `Copy` and `Declined::line` stays
`&'static str`"*, and both halves of that sentence have now been overtaken:
`Declined::line` became a [`std::borrow::Cow`] on 2026-09-10 for O141's
*"pdfcer cannot type a `q`"*, and the `Copy` half went here.

What the argument was actually protecting is intact and is worth naming
so it is not lost with the derive: **the engine's prose must not reach the
status bar.** That is still true. What travels here is a list of
`/BaseFont` names — data pdfcer computed, not a sentence pdfcer wrote —
and the connective words around it are this catalog's own.

### `fn text_style_used_standard_face`

There was no sentence for this because there was no outcome for it. This
shell asked for bold with `set_synthetic`, whose gate only ever looks at
faces **already on the page**; a page carrying nothing but `Helvetica` —
which is most CAD-exported title blocks, and pdfcer's own `rotated-text.pdf`
— has no bold resource, so the gate passed and the strokes were thickened.
`Helvetica-Bold` was a standard-14 name the whole time: every conforming
reader carries it, ISO 32000-1 §9.6.2.2 says it needs no font file, and
binding it is one new `/Font` resource of about sixty bytes.

So the operator was getting a **faked** weight on the commonest page in
their working set, five days after the engine shipped the rung that binds a
real one. The sentence says which of the two happened, because after this
change "pdfcer made it bold" has two very different meanings and only one of
them survives being printed at 1:1 on a plotter.

It names the growth explicitly. An operator whose file must stay small —
a drawing going to a portal with an upload cap — is entitled to know that
this route did not embed a typeface, and the alternative reading ("pdfcer
added a font to my file") is the one they would otherwise assume.

### `fn text_style_already_that_way`

# Not a refusal, and the distinction is the whole point

Bold and Italic are buttons that APPLY, not switches that reflect — there is
no "is this run bold" bit in a PDF, so a pressed-in toggle would be claiming
to have read a fact that is not recorded. Pressing Bold on a run already set
in `Times-Bold` is therefore an ordinary, expected gesture, and the engine
answers it by taking the ladder's zeroth rung and changing nothing.

Reported rather than silent, because "I pressed it and nothing happened"
is indistinguishable from a broken button. This sentence is the difference
between a control that did nothing and a control that had nothing to do.

### `fn text_style_faked_warning`

# Why this is a separate sentence rather than louder formatting

The engine already reports a synthesis in `FormatReport::disclosures`, and
under `Auto` that quiet report is the whole obligation. `Warn` exists for
the operator for whom *"a faked weight in the output is a problem worth
noticing at the moment it is created"* — a drawing that will be printed, a
document that will be handed on — and a disclosure they have to go looking
for does not serve them.

It is prose rather than an alarm colour because the edit **happened**.
Rule 4's shape holds: the text renders exactly as it will render when
saved, and the fact about it is said off-canvas.

### `fn text_style_multi`

# Why this sentence exists at all

`EditSession` has no undo-grouping verb, so restyling N runs is N entries in
the undo log and N presses of Ctrl+Z. That is a limit of the engine that the
operator meets through this shell, and an operator who presses Ctrl+Z once,
sees two thirds of their change still there and concludes undo is broken is
the exact outcome this sentence prevents.

Filed with the engine rather than worked around here — a shell-side coalesce
would work and would leave every other consumer with the same defect.

### `fn too_many_anchors_in_part`

The sibling of [`crate::text::status::too_many_anchors`], and it exists
because that one's guard excluded the exact route the operator reported.

# What he saw, and why it read as broken rather than as limited

The Points tool puts the selection at the **Part** rung, so
`entered_object()` is `Some` — and the disclosure was gated on it being
`None`. A subpath with more than four hundred anchors therefore drew no
dots and said nothing: he armed the tool, clicked a shape, watched the
selection box change, and the program went quiet. A limit reported as an
absence is the failure `RESUME.md` records four separate occasions of.

# Why it is not the same sentence

[`crate::text::status::too_many_anchors`] ends *"Double-click into a part
of it, or use the Points tool, to see that part's"* — advice that is
correct at the Object rung and **wrong here**, because there is nothing
below a subpath to descend into. Reusing it would send him looking for a
rung that does not exist.


It lives here rather than beside its sibling in `text::status` because
that module is at 1,482 lines against R2's 1,500. The seam is noticed
rather than trimmed, which is that file's own standing note.
