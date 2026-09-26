# `text::unshare` — every sentence "give this page its own copy" can say


## Why this feature needs the biggest refusal catalog on the canvas

Because **the refusals are the feature's whole shape**, and because the
commonest one is not an error at all.

`EditSession::unshare_form` takes `(page_index, form: ObjId)` and clones one
form XObject's stream so that this page — and only this page — names the
copy. Every other invocation site keeps naming the original and is left
byte-identical. That is a **structural** edit: it allocates an object, it
rewrites the page's `/Resources`, and it therefore runs the same guard
ladder every structural verb in the engine runs (encryption, certification,
`/Size` suppression) before it does anything at all.

⇒ So a control that says *"give this page its own copy"* declines for
**several distinct reasons** — the list is [`UnshareRefusal`]'s variants and
this sentence deliberately no longer counts them, for the reason
`tools/gates/run-all.sh`'s header spends six corrections on: *a number
written in prose beside the thing it counts is a claim that decays*. Two of
them — [`UnshareRefusal::Nested`] and [`UnshareRefusal::NotShared`] — are
considered design positions rather than limits, and none of them is
visible on the page. The operator presses a button, the drawing looks
identical (it *must*: the copy is byte-identical to the original, which is
the point), and without a sentence the only difference between success and
every failure is a status row that says nothing either way.

This is the project's founding defect shape with the volume turned up:
*a gesture that is made, is refused, and reports nothing.* `DEFECTS.md` D4a.
And it is worse here than for a drag, because a successful unshare also
looks like nothing happened — see [`unshared`], which is why the success
path owes a sentence too.

## The vocabulary, decided once

| the file's word | the operator's word here | why |
|---|---|---|
| form XObject | **drawing** / **the shared drawing** | §8.10.1's own illustration is a CAD system's standard component; the operator calls their title block a drawing, not an XObject |
| invocation | **place it is drawn** | an invocation is a `Do` operator; a place is something they can point at |
| page | **sheet** *(only where the fan-out is the subject)* | a 36-sheet drawing set is "sheets" in every room this software is used in. Elsewhere "page", because that is what the page box in the status bar says |
| `ObjId` | **not named at all** | see [`unshared`]: an object number is evidence, and evidence goes to the trace |

[`crate::text::rotating`]'s rule is inherited unchanged: **name the thing
the operator can see, never the thing pdfcer models.** A refusal phrased in
the file format's vocabulary reads as an internal error, and an internal
error is a thing an operator reports rather than acts on.

## What is deliberately NOT worded here

**Nothing.** That is unusual in this directory and it is the point: every
`EditError` this verb can return has a variant below, including the three
that are unreachable on a well-formed file. [`crate::text::rotating`]'s
argument for keeping its two unreachable variants is the argument for all of
these — *"a routing bug with a sentence is a bug report; a routing bug
without one is a handle that does nothing"* — and this verb has more ways to
be routed wrongly than a rotation does, because its operand is derived
through two hops (a selected leaf, then that leaf's outermost enclosing
form) rather than being the thing that was clicked.

## Item notes

### `fn other_pages_clause`

# The three shapes, and why none of them is a format string with a
# number in it

| measurement | clause |
|---|---|
| nothing to say (`other_pages == 0`) | *"anywhere else it is drawn keeps the original."* |
| exactly one | *"1 other page that draws it keeps the original."* |
| more than one | *"N other pages that draw it keep the original."* |

The zero case does **not** print "0 other pages" and does not claim
there are none. It is reachable only from an incomplete walk (or from a
form the walk could not see at all), and on an incomplete walk *"no
other page draws it"* is precisely the claim that has not been measured.
The wording it uses instead — *anywhere else it is drawn* — is true
whether that set is empty or not, which is the only kind of sentence a
failed measurement is entitled to.

Verb agreement is why one and many are separate arms rather than one
`format!` with a pluralised noun: *"1 other page … keeps"* against *"3
other pages … keep"* differ in two places, and "page(s) … keep(s)" is
the shape [`crate::text`]'s rules exist to keep out of an operator's
status bar.

### `fn every_refusal_is_a_sentence`

The check that a variant added later cannot ship silent — the founding
rule applied to the enum that exists to serve it. Copied deliberately
from `crate::text::rotating::tests::every_refusal_is_a_sentence` rather
than generalised: a shared helper would be one more thing to keep in
step, and the assertion is three lines.

### `fn every_refusal_says_where_the_sharing_stands`

The reachable failure this pins: somebody adds a variant, writes a
perfectly good sentence explaining *why* pdfcer declined, and leaves out
the half that stops the operator going straight on to type into a title
block that is still shared by thirty-six sheets. The refusal would read
as complete and would omit the only part that prevents damage.

Asserted on a **word**, not on a phrase, because the wording of each
sentence is deliberately different and pinning a phrase would either
force a set of identical endings or fail on the first rewrite. Every
sentence must mention what is *shared* or that nothing *changed*; that
is the property, and it is the weakest assertion that still catches the
omission.

[`UnshareRefusal::NotShared`] satisfies it by saying *nothing was
changed* while asserting the opposite about the sharing — see
[`UnshareRefusal::line`]'s docs. The property is *"the sentence settles
where the sharing stands"*, not *"the sentence says the page still
shares it"*, and the name below says so.

### `fn the_disclosure_names_a_count_only_when_there_is_one_to_name`

Both halves matter. A build that dropped the count on the multi-name
case would leave an operator who knows the sheet draws its title block
three times to guess whether one of the three moved or all of them; a
build that printed "1 place" on the ordinary case would put a number in
front of every operator for no reason at all.

### `fn the_disclosure_claims_other_pages_only_when_it_measured_some`

The shipped sentence ended *"every other page still shares the
original"* in both branches, on a command that had never counted the
invocations. This pins the property that made it a defect rather than a
wording preference: a claim about other pages appears **only** when
[`Fanout::other_pages`] is non-zero.

### `fn a_lower_bound_is_said_to_be_one_and_a_total_is_not`

`InvocationSet::is_lower_bound`'s own documentation is the reason:
*"an under-count presented as a total is the same class of defect as a
silent edit."* Both directions are asserted, because a build that said
"at least" unconditionally would be hedging a number it had actually
measured, which teaches the operator to discount the hedge on the one
document where it means something.

### `fn the_single_other_page_reads_as_english`

One other page "keeps" the original; three "keep" it. The arm exists
because the alternative — one `format!` with a pluralised noun — puts
"page(s) … keep(s)" on an operator's status bar, and this directory's
rules exist to keep that out of it.

### `fn the_not_shared_decline_is_good_news`

The sentence reports the one outcome in this enum where the operator did
nothing wrong and the document is in perfect health, and the whole
reason it is a *decline* rather than a pointless structural edit is that
the truthful sentence is better than the edit. A rewrite that opened it
with "pdfcer could not" would keep every fact and lose that.

Asserted on the absence of failure vocabulary and the presence of the
two facts the operator acts on — *it is already private* and *nothing
was changed* — rather than on the sentence itself, which is free to be
reworded.

### `fn the_remedy_puts_undo_first`

The one assertion in this file that pins a *sequence* rather than a
property, and it is pinned because getting it backwards produces a
sentence that reads perfectly and causes the damage it warns about: at
the moment this is read the edit is already in the shared stream, so
unsharing first copies the edited version and leaves every other page
changed as well. See [`shared_content_remedy`]'s docs for the full
account.

### `enum UnshareRefusal`

# A `Copy` enum rather than the engine's own `Display`

[`crate::text::status::TextStyleRefusal`]'s reason, adopted unchanged and
for the third time: a `format!` of an `EditError` would route **diagnostic
prose into the UI**, which `tools/gates/check-ui-strings.sh`'s exclusion 3
names in as many words — *"this exclusion is not permission to route UI text
through an error type."* An enum keeps
[`crate::app::status::decline::Declined`] `Copy`, keeps its `line()`
returning `&'static str`, and keeps every operator-visible word in this
file under **R1**.

# The one variant that carries no number, and why that is deliberate

[`Self::WouldExposeHiddenObjects`] is raised by the engine with a `count` —
how many cross-reference entries the file's `/Size` is currently hiding —
and this enum drops it. Two reasons, either sufficient:

1. Carrying it would make this type non-`Copy`-friendly in the sense that
   matters: `Declined::line()` returns `&'static str`, and a counted
   sentence needs a `String` and an allocation on a path that runs while a
   status bar is being laid out.
2. **The number is not actionable and is barely meaningful to the reader.**
   "17 hidden cross-reference entries" tells an operator nothing they can
   do. What they can act on is *this file is damaged in a way that makes it
   unsafe to add anything to*, and that is what the sentence says. The count
   goes to the trace, where evidence belongs — the same split
   `canvas::textedit::report` makes for `followers_repositioned`.

# Ordering

The variants are in **the order the engine checks them**, which is also the
order of decreasing "this is about the whole document" and increasing "this
is about what you just clicked". A reader comparing this enum against
`EditSession::unshare_form`'s body should be able to walk both top to bottom
together.

### `fn line`

# Every one of them ends by saying the sharing is unchanged

That clause is not padding, and it is the clause that took the longest
to get right. The operator pressed this button **because they are about
to edit something**, and the thing they need to know after a refusal is
not "it failed" — it is ***do not now go and type into that title
block***, because doing so will change thirty-six sheets.

A refusal that says only "pdfcer could not do that" leaves them believing
the safe state might have been reached. Every sentence below therefore
closes the loop explicitly: the page still shares the drawing.

**[`Self::NotShared`] is the one exception, and it is the same rule
rather than a break from it.** The clause exists to tell the operator
*what is true about the sharing before they type*. For every other
variant that fact is "you still share it"; for that one it is "there was
never anything to share", which is the same clause with the opposite
value and is exactly as load-bearing. What is forbidden is a sentence
that leaves the question open, not a particular answer to it — and the
test below (`every_refusal_says_where_the_sharing_stands`) asserts the
property that way rather than pinning one of the two answers.

# Remedy first where there is one

[`crate::text::resizing`]'s rule, inherited: the operator is looking at
something that did not happen, and the useful half is *what to do now*.
[`Self::Nested`] and [`Self::NothingInAForm`] both name a next act;
the rest have none, and none is invented for them.

### `fn unshared`

# Why a SUCCESS owes a sentence at all, which is unusual

Most disclosures in this crate exist because a consequence is invisible.
This one exists because **the whole act is invisible, by design**.

`EditSession::unshare_form` clones the form stream verbatim — the engine's
own comment says the copy *"carries the ORIGINAL's value verbatim, span and
all"*, so unsharing costs no duplicated bytes until the copy is actually
edited. The page therefore renders **pixel-for-pixel identically** before and
after. Nothing moves, nothing changes colour, nothing appears or disappears.

⇒ Without a sentence, the operator's evidence that the command worked is
indistinguishable from their evidence that it did nothing — which is the
same state as an unworded refusal, arriving through the success path. R8b
rule 4 as narrowed by pdfcer's decision 059 (*render normally, report
separately*) applies with unusual force: there is nothing to render.

# What the engine asked a shell to say, verbatim

[`pdfcer_core::edit::UnshareFormReport`] is documented as naming the copy and
how many references moved *"so a shell can say what happened rather than
only that it worked"*. This is that sentence.

# What it does NOT say: the object numbers

`UnshareFormReport::original` and `::copy` are `ObjId`s, and neither reaches
the status row. That is the split `canvas::textedit::report` states as a
rule and this file follows: *a number about a content stream is evidence, not
a disclosure.* An operator cannot act on "object 47"; a driven check can, and
a regression then names itself. Both numbers go to the trace from
`app::actions::xobject`, where the object-clipboard and text-edit arms
already send theirs.

# The plural, and why it is a branch rather than a format string

`references_moved` is *"usually 1. Greater than 1 when the page invoked the
same form under several names"* — a real case on CAD output, where one title
block is drawn once per view. The two sentences are genuinely different
statements, not one sentence with a number in it:

- at 1, the operator needs to know the change is now local to this page;
- above 1, they additionally need to know that **all** the places this page
  draws it moved together, because the alternative reading — "one of the
  three title blocks on this sheet is now private and two are not" — would be
  a genuinely alarming and genuinely wrong thing to infer, and it is exactly
  what an operator who knows the page draws it three times will infer from
  silence.

That plurality is the engine's decision, stated in the verb's own docs: *"the
unit of this operation is the PAGE"*. The sentence says so.


The sentence used to end *"every other page still shares the original"* in
both branches, unconditionally, on a command that had never asked how many
other pages there were. On a single-invocation form that clause was **false
about the operator's own file**; on a genuinely shared one it was
indistinguishable from the false version, so the operator who *did* have a
thirty-six-sheet title block learned nothing from it either. One
unconditional clause managed to be both a lie and useless.

[`Fanout`] carries the measurement, and every claim about other pages is now
made from it or not made at all. See its docs for the three shapes and for
why "at least" is not a hedge.

### `struct Fanout`

# Why this type exists rather than two loose parameters

Because the two fields are only ever meaningful **together**, and read apart
they produce the exact sentence this type was introduced to delete. `3`
alone says *"three other pages draw it"*; `3` with `lower_bound` says *"at
least three, and pdfcer could not finish looking"*. A caller handed two bare
arguments eventually passes them in the wrong order or forgets the second,
and the symptom of forgetting the second is **an under-count presented as a
total** — which `pdfcer_core::text_edit::invocation_set`'s own documentation
calls *"the same class of defect as a silent edit"*.

It is the same argument [`crate::app::status::decline::History`] makes for
pairing undo and redo: *"a caller that had to pass two loose booleans in the
right order would eventually pass them in the wrong one."*

# Where the numbers come from, and what they are NOT

`crate::app::actions::xobject::fanout` walks the document once, on the
press, through `pdfcer_core::text_edit::invocation_set`. Both fields are read
off the returned `InvocationSet`:

| field | source | measured **before** or **after** the copy |
|---|---|---|
| [`Self::other_pages`] | `set.pages`, minus this page | **before** |
| [`Self::lower_bound`] | `InvocationSet::is_lower_bound()` | **before** |

*Before* is load-bearing and is not an implementation detail. After the
copy is made, this page's invocations name the copy, and a walk run then
would report the original's fan-out with this page already subtracted. The
number the operator needs — *how many sheets are still on the original* — is
the same either way only because the subtraction is done here rather than by
the file. Measuring after would give the right answer for the wrong reason
and would break the moment the verb's granularity changed.

# "At least" is a statement of fact, not a hedge

`InvocationSet::is_lower_bound()` is true when some page's scan hit the
depth guard or a form pdfcer could not decode. Those pages may or may not
draw this form; nothing in the count knows. Printing the bare number would
present an under-count as a total, and an operator who reads *"2 other pages
keep the original"* on a document where the true answer is nine will not
check the other seven. So the sentence says *at least*, and it is the
**honest** wording rather than the cautious one.

### `fn shared_content_remedy`

# Why the shell adds a sentence to the engine's own list

`pdfcer-core` already puts a `"SHARED CONTENT: …"` sentence into
`text_edit::EditReport::disclosures`, worded for direct display, and
`canvas::textedit::report`'s header rules — correctly — that **re-wording it
here would be a second account of one fact, free to drift**. Nothing below
re-words it. This is a second, *different* fact, and it is one the engine
cannot state: **pdfcer-core does not know what this shell's commands are
called.**

The engine's sentence says *what happened* — the edit changed every place
this form is drawn, because the standard binds a form to no page and there is
exactly one stream holding those glyphs. Complete, and true. What it cannot
say is *what to do about it*, because the answer is the name of a control in
a program it has never seen.

⇒ The precedent for appending is already in the same apply arm:
`crate::text::textedit::pinned_tail_disclosure` is a shell-authored sentence
pushed onto the engine's list, for the same shape of reason — the engine says
nothing about a pinned tail because from its side pinning is what was asked
for, and the operator still owes it.

# The sequence is UNDO first, and getting that wrong would be a lie

This is the sentence's load-bearing clause and it is worth the paragraph.

The naive remedy — *"press Unshare now"* — **does not work, and would make
things worse.** The edit has already been written into the one shared stream
object; every page that draws it already shows the change. Unsharing at that
point copies the **already-edited** stream to this page and re-points this
page at it. The other thirty-five sheets keep the original object, which is
the one that was edited. The operator would end up with the change on every
sheet **and** a redundant private copy, and a sentence that told them to do
that would have caused the damage it was warning about.

The order that works is **undo, unshare, edit again**, and it is stated in
that order with no room to read it otherwise.

# Why it is worded as a future-tense offer, not a warning

Because at the moment it is read, the fan-out has already happened and may
well have been wanted — §8.10.1's whole purpose for the feature is that one
component appears on many sheets, and a drawing-office correction to a title
block is *supposed* to reach all of them. This shell must not imply the
operator has made a mistake. It names the option they did not know they had.

# Why it is appended rather than replacing the engine's sentence

The engine's sentence carries `InvocationSet::describe()` — the actual
counts, "3 pages, 5 places" or whatever this document is — and that is the
fact that makes the disclosure *startling*, which is the property
`canvas::textedit::report` says it is meant to have. Dropping it to make room
for a remedy would trade the alarming half for the useful half. Both, in the
engine's order: what happened, then what to do.
