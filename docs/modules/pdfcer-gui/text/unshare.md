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
