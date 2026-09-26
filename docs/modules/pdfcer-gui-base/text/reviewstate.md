# `text::reviewstate` — every word the review-status control says

One module for one subject: `/State` and `/StateModel` (§12.5.6.3, Table
171 — 2.0's Table 174), read by `pdfcer-core` `Pass 253.1` and authored by
[`pdfcer_core::edit::EditSession::add_review_state`]. It covers the status
line on a comment row, the control that records one, the status chooser
beside the existing filter, and the two sentences the status row says after
the edit lands.

It is deliberately **not** in [`crate::text::panels::comments`], even
though most of these strings are drawn by that panel. The subject here is a
*vocabulary the standard defines and the engine refuses to interpret*, and
the wording decisions below are all consequences of that one fact rather
than of anything about panels. Keeping them together is what makes the next
reader able to check them against §12.5.6.3 in one pass.

## THE FACT THAT DECIDES ALMOST EVERY STRING BELOW

**A review status is APPENDED, not set.** `add_review_state`'s own doc
comment states it twice with the standard's `shall`:

> *"THE STATUS IS NOT WRITTEN ONTO THE ANNOTATION IT DESCRIBES … §12.5.6.3
> puts it on a **separate** `/Text` annotation that points at the reviewed
> one through `/IRT` … That is why this verb returns a new `ObjId` rather
> than mutating the target, and why nothing about the target changes."*

and

> *"AND A SECOND STATUS CHAINS ONTO THE FIRST, PER AUTHOR … 'Additional
> state changes shall be made by adding text annotations **in reply to the
> previous reply** for a given user.' So this verb walks the `/IRT` graph
> rooted at `target` … building a per-author chain rather than a star."*

⇒ The file holds a **log**, not a field. So nothing here may say *set the
status*, *change the status* or *the status is now*. [`record_label`] says
**Record status**; [`status_recorded`] says what was added and how deep the
operator's own history on that comment now is; [`row_status_history`] says
how many earlier statuses stand behind the one being shown. A control
labelled *Set status* would describe a different document format.

## THE SECOND FACT: THE ENGINE DOES NOT INTERPRET THE STRINGS

`Annotation::state` and `Annotation::state_model` are `Option<String>`,
decoded verbatim, and `state`'s own doc says why:

> *"Neither key carries a 'shall be one of' anywhere in either edition, so
> a value outside Table 171's vocabulary is **unhandled, not illegal**.
> Reporting it verbatim is therefore reading the file rather than tolerating
> it. `ReviewState` is the closed set pdfcer AUTHORS; this is the open set
> it reads."*

⇒ The vocabulary is **this shell's to present**, and an unknown value must
be **shown**, never normalised. That is why there are three families of
string for one value — [`state_name`] for the seven pdfcer authors,
[`state_unmodelled`] for a value in a model pdfcer authors, and
[`state_foreign`] for a value in a model it will not author. The
distinction, and the reason collapsing the last two would be wrong, is
[`crate::text::buttonaction`]'s, reused rather than re-derived; see
[`crate::panels::comments::reviewstate::StateReading`].

## Where these strings are NOT

**Never on the canvas.** R8b — *"fuzzy, never sneaky"* — and its clause
about the original GUI: *"the nagging and red flagging … made for a lot of
extra bugs in the visibility when editing."* A review status is a
**disclosure about a comment**, not part of the comment's appearance, so it
is drawn in the panel and in the status row and nowhere else. A mark whose
status is *Rejected* is drawn exactly as the file will draw it.

## Conventions, restated from [`crate::text`] because they bind here

- Sentence case, no trailing period on labels; full sentences for prose.
- The **file's own spelling** for any value that came out of the document,
  including `Cancelled`, which Table 171 writes in British English and
  which is therefore not "corrected" anywhere below.

## Item notes

### `fn state_name`

# These are the file's own words, and that is the decision

`Accepted`, `Rejected`, `Cancelled`, `Completed`, `None`, `Marked`,
`Unmarked` are Table 171's vocabulary and
[`pdfcer_core::edit::ReviewState::as_str`] writes exactly those bytes. A
friendlier relabelling — *Approved*, *Done*, *Ticked* — would give the
operator a fourth name for a value they will meet again in Acrobat, in the
saved file and in `pdfcer list-annotations`, and would make a screenshot of
this panel un-checkable against the document it describes.

The one place a word is added rather than changed is `None`, which alone is
ambiguous on screen: Table 171 makes it a **writable value** in the `Review`
model and *not* a spelling of "no status recorded" — `Annotation::state`'s
doc says so in as many words — and a bare *None* in a chooser would read as the
empty entry. See [`state_none`].

### `fn state_none`

# The distinction the standard makes and a menu would lose

`Annotation::state`'s doc: *"`None` is a **writable value**, not a spelling
of 'the key is absent'."* Recording `None` is an act — it is how a reviewer
withdraws a status they set an hour ago — and it produces a state
annotation with a `/State` of `None` in the file. A chooser entry reading
just *None* sits beside [`filter_status_any`] and
[`filter_status_unrecorded`] and would be read as one of them.

### `fn state_unmodelled`

# Why this is not folded into [`state_foreign`]

[`crate::text::buttonaction::button_action_current`]'s table, applied to a
different key for the same reason:

| state | what it means | what the row offers |
|---|---|---|
| modelled | Table 171's value, in Table 171's model | show it, record another |
| **unmodelled** | pdfcer **authors** this model and did not decode this value | name both, **still offer to record** |
| foreign | a `/StateModel` pdfcer will not author | name both, offer nothing in it |

Collapsing the two would grey a row pdfcer writes happily: a `/StateModel`
of `Review` carrying a `/State` of `Deferred` is a document pdfcer can add
its own `Accepted` to, in the same model, on the same chain rule. Greying it
would be this shell claiming an incapacity it does not have.

`model` is carried because the value alone is not interpretable —
`Annotation::state`: *"a caller that wants the effective state must read
both fields together."*

### `fn state_foreign`

§12.5.6.3 names two models and neither edition says *shall be one of*, so a
third is unhandled rather than illegal. pdfcer cannot record a status in a
vocabulary it does not know the values of, and
[`pdfcer_core::edit::ReviewState`] has no variant that could hold one — so
this is the `Foreign` row of the table above, and R9 makes it render the
fact rather than a greyed control.

It does **not** stop the operator recording their own status: their status
goes in the `Review` model, on their own `/IRT` chain, and leaves this one
untouched. [`record_other_model_note`] is the sentence that says so.

### `fn state_without_model`

Table 171 makes `/StateModel` *"Required if `State` is present"*, and
`Annotation::state_model`'s doc calls this out as the asymmetric half:
*"the only non-conforming combination is `/State` present with this
absent."* So this is a malformed file, surfaced rather than repaired — the
same posture `crate::panels::comments::model::CommentRow::subtype` takes
for a missing `/Subtype`.

pdfcer's own authoring cannot produce it: `add_review_state` derives the
model from the state.

### `fn row_status_unsigned`

Worded as a fact about the document rather than as *Anonymous*, for
`crate::text::panels::comments::comment_row_byline`'s reason: `/T` is a
Table 170 markup key and its absence is not a claim about a person.

It also has a consequence the operator can act on, and the sentence
carries it: `add_review_state` chains on `/T` equality, so an unsigned
status can never be continued by a signed one — a later status starts a
fresh history beside it.

### `fn row_status_history`

# This is the string that stops the panel lying about the format

The row shows one status per reviewer — the tip of that reviewer's `/IRT`
chain. `depth` is how many states that reviewer has recorded on this
comment altogether. Drawn only when it is greater than one, so the line
means something when it appears rather than reading *1 status recorded*
beside every row.

Without it the panel shows a single value per person and is indistinguishable
from a panel over a format that stores a mutable field — which is precisely
what §12.5.6.3 is not. The engine says so plainly: *"the history a
reviewer's chain encodes would simply not be there"* if the shape were
flattened.

### `fn row_status_none`

Drawn only under the status chooser's *No status recorded* filter and
nowhere else — see [`crate::panels::comments::reviewstate`]. On an ordinary
unfiltered list a caption on every unreviewed row would be forty repetitions
of "nothing has happened here", which is the noise
`crate::panels::comments`' disclosure discipline exists to keep out.

### `fn row_is_a_status`

# Why this exists, and why not filtering the row out instead

A status is a `/Text` annotation with `/IRT`, `/State`, `/StateModel`, a
`/T` — and a deliberately **empty** `/Contents`. `add_review_state`:

> *"`/Contents` is deliberately EMPTY. A status is not a comment, and
> inventing 'Accepted' as the body would put a sentence in the operator's
> comment list that the operator never wrote."*

So the moment the operator records one, a **blank row** appears in the
Comments panel. Excluding it was considered and rejected: this panel's
founding rule is that *nothing is silently omitted*, and its three existing
exclusions are each counted and disclosed in numbers. A fourth silent one
would be the panel deciding what the file contains.

So the row stays and says what it is. `status` is the reading of its own
`/State`.

### `fn record_label`

See this module's header: the file holds a log. *Set status* would name a
field that §12.5.6.3 explicitly does not define, and an operator who read
it as one would expect their second status to replace their first.

### `fn record_tooltip`

**Where it goes** (a new annotation, not this one), **that it is added**
(the earlier one stays), and **that it is signed** with the name from
Settings. All three are invisible from a screenshot of the row, and the
third writes a person's name into a file that may leave the building —
which `crate::app::prefs::Prefs::author_name` treats as a decision the
operator makes rather than one pdfcer makes for them.

### `fn record_other_model_note`

Says what the control will do rather than leaving the operator to infer it
from a value in a vocabulary nobody has explained: their status is recorded
in the `Review` model, on its own chain, and the foreign one is untouched.

### `fn filter_status_any`

Worded *Any status* rather than *All*, unlike the author and type choosers'
shared [`crate::text::panels::comments::comment_filter_all`], for one
reason: this chooser's other entries include [`filter_status_unrecorded`],
and *All* beside *No status recorded* reads as a pair of opposites rather
than as an entry and its negation.

### `fn filter_status_unrecorded`

This is the entry the whole filter is for. A reviewer's question on a
thirty-six-sheet drawing set is *"what have I not dealt with"*, and it is
unanswerable from a chooser that can only name statuses that exist. It is
distinct from a `/State` of `None` — see [`state_none`] — and the two
entries sit next to each other saying so.

### `fn status_recorded`

# Why `depth` is in the sentence

[`pdfcer_core::edit::ReviewStateAdded::chain_depth`] is *"how deep the chain
for this author now is — `1` for their first status on this target"*, and it
is the only observable that distinguishes a correct per-author chain from a
star of statuses all pointing at the comment. The engine is blunt about why
it is exposed: *"the wrong shape is invisible … a star of state annotations
all pointing at the target renders the same as a correct chain in every
viewer, so nothing would ever report it."*

So this sentence is the operator-facing half of that, and it does a second
job at the same time: on the second press it says **2**, which is the panel
telling the truth about a log rather than a field, at the exact moment the
operator would otherwise assume they had overwritten something.

### `fn status_recorded_unsigned`

`add_review_state` takes an author `&str` and uses it as the chain key —
its `deepest_state_for_author` matches on `title == Some(author)` — so an
empty one
writes an empty `/T` and **cannot be continued** by a later status recorded
once a name is set: that one starts a fresh history beside it.

That consequence is invisible — the row looks identical either way, and it
only shows up later as two parallel chains — so it is stated at the moment
it is caused, and it names the place to fix it.
