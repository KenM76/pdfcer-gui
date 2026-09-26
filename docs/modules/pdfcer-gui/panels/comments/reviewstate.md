# `panels::comments::reviewstate` — a comment's **review status**

`/State` and `/StateModel` (§12.5.6.3, Table 171; 2.0's Table 174), read
into a per-reviewer history, shown on the row, filtered beside the existing
sort, and recorded through
[`pdfcer_core::edit::EditSession::add_review_state`].

Everything this feature does that is not a `Vec` of `String`s or an
`egui::Ui` call lives here, in one file, for
[`super::filter`]'s reason: the interesting decisions are all
**classifications**, and a classification is only testable when it is
separable from the widget that shows it.

---

## FACT ONE — A STATUS IS **APPENDED**, NOT SET

This is the fact the whole module is shaped around, and it is the engine's
own, twice over, in `add_review_state`'s doc comment:

> *"THE STATUS IS NOT WRITTEN ONTO THE ANNOTATION IT DESCRIBES —
> §12.5.6.3 puts it on a **separate** `/Text` annotation that points at the
> reviewed one through `/IRT`, and says so with a `shall`. That is why this
> verb returns a new `ObjId` rather than mutating the target, and why
> nothing about the target changes."*

> *"AND A SECOND STATUS CHAINS ONTO THE FIRST, PER AUTHOR — The clause's
> last sentence is also a `shall`: 'Additional state changes shall be made
> by adding text annotations **in reply to the previous reply** for a given
> user.' So this verb walks the `/IRT` graph rooted at `target`, keeps the
> annotations whose `/T` matches `author`, and attaches to the deepest one —
> building a per-author chain rather than a star."*

⇒ **The file holds a log, not a field**, and three things follow that would
each be wrong under the other reading:

1. [`Statuses`] is `target → Vec<Recorded>` — **one entry per reviewer**,
   never one value per comment. Two people may hold opposite views of the
   same mark and both are in the file.
2. Each entry carries [`Recorded::depth`], and the row prints it whenever it
   exceeds one. A panel that showed only the tip would be indistinguishable
   from a panel over a format that stores a mutable field, which is exactly
   what §12.5.6.3 is not.
3. The control is called **Record status**, and its action is
   [`crate::app::actions::Action::RecordReviewState`]. Nothing in this
   feature is named *set*.

### There is deliberately NO resolver in the engine, and that is a
decision this module inherits rather than works around

> *"Deciding which of several state annotations is current is left to the
> caller, as `pdfcer-gui` asked ('Give us the annotations and the keys; we
> will pick') — and the standard supports that: it says **nothing whatever**
> about ordering or currency. … `/M` is not required and empirically ties,
> so a `/M`-sorted resolver would be guessing. Shipping none is
> spec-correct."*

So this module picks, and it picks the **only** ordering the standard does
define: the `/IRT` chain. The tip of a reviewer's chain is their current
status because §12.5.6.3 says each change replies to the previous one — not
because it is the newest by `/M`, which would be the guess the engine
refused to make. [`read`] implements exactly the walk `add_review_state`'s
own `deepest_state_for_author` implements, including its `MAX_CHAIN` bound,
and for the same reason: *"a `/IRT` cycle is legal syntax … and this must
terminate on a malformed file rather than hang."*

---

## FACT TWO — THE ENGINE READS THE STRINGS **VERBATIM**

`Annotation::state` and `Annotation::state_model` are `Option<String>`, and
`state`'s own doc says why it is not an enum:

> *"Neither key carries a 'shall be one of' anywhere in either edition, so a
> value outside Table 171's vocabulary is **unhandled, not illegal**.
> Reporting it verbatim is therefore reading the file rather than tolerating
> it. `ReviewState` is the closed set pdfcer AUTHORS; this is the open set
> it reads."*

⇒ **The vocabulary is this shell's to present**, and an unrecognised value
must be *shown*, never normalised to the nearest thing pdfcer knows.
[`StateReading`] is that presentation, and it takes
[`crate::text::buttonaction`]'s posture rather than inventing a second one —
the reasoning is already written and reusing it is what keeps the two
surfaces saying the same thing about the same problem:

| reading | what it means | what the row offers |
|---|---|---|
| [`StateReading::Modelled`] | Table 171's value, in Table 171's model | show it; **Record status** continues the same vocabulary |
| [`StateReading::Unmodelled`] | `/StateModel` is one pdfcer **authors**; `/State` is not in its vocabulary | show both verbatim; **Record status** still continues **that model** |
| [`StateReading::Foreign`] | `/StateModel` is a vocabulary pdfcer **will not author** | show both verbatim; pdfcer offers **nothing in it**, and says what it will do instead |
| [`StateReading::ModelMissing`] | `/State` present, `/StateModel` absent — Table 171's one non-conforming combination | show it as the malformed thing it is |

**Collapsing `Unmodelled` into `Foreign` would grey a row pdfcer writes
happily.** A `/StateModel` of `Review` carrying a `/State` of `Deferred` is
a document pdfcer can add its own `Accepted` to — same model, same chain
rule, same file. Saying "pdfcer does not author this vocabulary" about it
would be this shell claiming an incapacity it does not have, which is the
failure `crate::text::buttonaction`'s four-state table exists to prevent.

And the difference is **visible**: [`StateReading::authorable_model`]
answers `Some` for the first three and `None` for `Foreign`, and that is the
one thing the row's note is keyed off.

---

## R8b — THE STATUS IS OFF-CANVAS, ALWAYS

*"Fuzzy, never sneaky."* A review status is a **disclosure about a
comment**, not part of the comment's appearance, so it is drawn in this
panel and in the status row and **nowhere else**. A mark whose status is
`Rejected` is drawn on the page exactly as the file will draw it — no
badge, no tint, no dashed outline. The clause this is quoting is the
operator's own: *"the nagging and red flagging in the original GUI made for
a lot of extra bugs in the visibility when editing."*

`crate::canvas::notepopup`'s header already refuses to show `/State` for the
same reason, and stays untouched by this Pass.

---

## WHAT THE PANEL LOOKS LIKE AFTERWARDS, AND THE ROW NOBODY ORDERED

A status annotation is a `/Text` with `/IRT`, `/State`, `/StateModel`, a
`/T` — and a **deliberately empty** `/Contents`: *"A status is not a
comment, and inventing 'Accepted' as the body would put a sentence in the
operator's comment list that the operator never wrote."*

⇒ The moment a status is recorded, a **blank row** appears in the Comments
list. Excluding it was considered and rejected: this panel's founding rule
is that *nothing is silently omitted*, and each of its three existing
exclusions is counted and disclosed in numbers. A fourth, silent one would
be the panel deciding what the document contains. So the row stays and
[`crate::text::reviewstate::row_is_a_status`] names it — and, because it is
a status rather than a comment, it is the one row that offers no **Record
status** control of its own (R83: an affordance nobody could want).

---

## Where the pieces live

| | |
|---|---|
| reading the document | [`read`] → [`Statuses`] |
| classifying one value | [`StateReading::of`] |
| narrowing the list | [`StatusChoice`], [`Statuses::keeps`], [`narrow`] |
| the chooser beside the sort | [`status_strip`] |
| the row's status lines and its control | [`row_status`] |

## Every test below has been FALSIFIED, and one of them was vacuous

Twenty-one guards, twenty-one mutations, one at a time, each restored from a
byte copy — never `git checkout`, because four other agents held uncommitted
work in this tree the day this was written. All twenty-one went red.

**The sweep earned its cost on the first run.**
`a_status_with_no_irt_reaches_no_target_but_is_still_read` asserted
`s.on(1).is_empty()` and stayed **green** when [`walk_up`] was broken to make
an `/IRT`-less status its own target — because that answer lands on
annotation *2*, and the assertion was only ever looking at *1*. A negative
assertion aimed at one address cannot see a wrong answer given at another.
It now asserts over the whole index. That test is the reason the rule is
*break the guard and watch it go red*, not *write the guard and watch it
pass*.

Two mutations are recorded in that test suite for a second reason: the
first fixture for [`MAX_CHAIN`] could not reach the bound at all (both
statuses in a two-cycle are superseded, so the walk never runs), and the
`#[non_exhaustive]` note on [`OFFERED`] exists because no `match` here can be
made to fail when the engine adds a variant.

The filter *state* lives on [`super::filter::Filter`] rather than here, so
that **Show all** clears it and [`super::filter::Filter::is_narrowing`]
counts it — the two properties the panel's whole disclosure discipline hangs
off. What could **not** move there is the predicate: see
[`super::filter::Filter::status`] for why `keeps` cannot answer it.

## Item notes

### `const MAX_CHAIN`

The engine's own bound, by the same number and for the same stated reason:
*"a `/IRT` cycle is legal syntax (nothing in §12.5.6.2 forbids one) and this
must terminate on a malformed file rather than hang"*
(`edit.rs`, `deepest_state_for_author`). Matching it rather than picking a
different number matters: a shell that walked further than the engine would
show a history the engine will refuse to extend.

### `struct StateAnnot`

# Why the walk does not run over `pdfcer_core::annot::Annotation`

Two reasons, and the second is the one that matters.

1. `Annotation` carries twenty-four fields, none of which but these four is
   read here, and it derives no `Default` — so every fixture would be a
   twenty-four-field literal that changes whenever the engine's read model
   grows a field. That is a test suite bound to the engine's *shape* rather
   than to its *behaviour*.
2. **The `/T` trim happens exactly once**, on the way in.
   `deepest_state_for_author` compares `/T` bytes as the file carries them,
   and this panel's `super::keeps_author_name` trims — so a chain rule
   written against the raw field and a display rule written against the
   trimmed one would disagree about whether `"Ken "` and `"Ken"` are one
   person. Normalising at the boundary makes that unrepresentable.

### `fn same_author`

An absent `/T` is **never** equal to anything, including another absent one
— `deepest_state_for_author` compares `a.title.as_deref() == Some(author)`,
and `author` is a `&str`, so `None` can never match. Mirrored exactly rather
than improved: a shell that chained unsigned statuses would show a history
the engine will not extend.

### `fn walk_up`

Returns the reviewed annotation and the chain's depth (`1` for a lone
status). `None` when the tip has no `/IRT` at all — a status that refers to
nothing describes nothing, and §12.5.6.3 defines the state as living on an
annotation *"that refers to the original annotation by means of its `IRT`
entry"*.

The walk is depth-bounded at [`MAX_CHAIN`]: a bounded stop rather than a
hang, on a `/IRT` cycle the standard does not forbid.

### `fn statuses`

[`assemble`] is called rather than re-implemented, which is the point of
splitting it out of [`read`]: a helper that repeated the algorithm would
be a test of the helper.

### `fn every_authored_state_is_recognised_as_modelled`

A test asserting *an unknown state is not normalised* is **vacuous if
nothing is ever recognised** — a `StateReading::of` that returned
`Unmodelled` for every input on earth would pass it. So every one of
the seven values pdfcer authors is round-tripped through the classifier
here, and this test is what gives the next one its meaning.

The trailing count assertion is the one instrument this side of the
boundary that can notice [`OFFERED`] falling behind: `ReviewState` is
`#[non_exhaustive]`, so no `match` and no iteration over the array can
be made to fail when the engine adds a variant. See [`OFFERED`].

### `fn an_unknown_value_in_a_known_model_is_shown_verbatim`

The `Unmodelled` half of `crate::text::buttonaction`'s table. Both
halves are asserted: the value survives verbatim, **and**
`authorable_model` is `Some`, because collapsing this into `Foreign`
would grey a row pdfcer writes happily.

### `fn a_state_paired_with_the_wrong_model_is_unmodelled`

Asserted because the obvious implementation matches `/State` alone and
would report `Modelled(Accepted)` for a file saying something pdfcer
would never write.

### `fn a_state_of_none_is_recorded_and_not_unrecorded`

`Annotation::state`: *"`None` is a writable value, not a spelling of
'the key is absent'."* A row carrying `/State (None)` must not be
selected by the **No status recorded** filter.

### `fn a_reviewers_second_status_chains_and_the_depth_is_reported`

The property the whole feature is shaped around. Annotation 1 is the
comment; 2 is Ken's first status; 3 replies to 2 with his second. Only
the tip is shown, and `depth` is 2 — which is what
`crate::text::reviewstate::row_status_history` prints so the operator
cannot mistake a log for a field.

### `fn two_reviewers_are_both_reported`

§12.5.6.3 chains per user, so the file genuinely holds both. A resolver
that picked one would be inventing the currency rule the engine
explicitly refused to ship.

### `fn an_unsigned_status_never_chains`

Both statuses stay tips and both are reported, because neither can be
the other's parent. A shell that chained them would show a history
`add_review_state` will not extend.

### `fn an_irt_cycle_terminates`

Legal syntax — nothing in §12.5.6.2 forbids one — and the engine bounds
its own walk at 64 for exactly this reason.

**The obvious fixture does not test the bound**, which is worth
recording because the first attempt was that fixture: two statuses
pointing at each other are BOTH superseded, so [`assemble`] skips both
before [`walk_up`] is ever called and removing [`MAX_CHAIN`] leaves the
test green. Three are needed — 3 is a tip nobody replies to, and its
chain runs into a 4↔5 loop that only the bound stops. Delete
`depth < MAX_CHAIN` and this test does not fail; it **hangs**, which is
the failure it is asserting the absence of.

### `fn a_status_with_no_irt_reaches_no_target_but_is_still_read`

**The first version of this test was VACUOUS, and the mutation
sweep is what found it.** It asserted `s.on(1).is_empty()` — that the
status did not reach annotation 1 — and stayed **green** when
[`walk_up`] was broken to return `Some(current.id)` for an `/IRT`-less
status, because that makes the status reach annotation *2* instead. A
negative assertion aimed at one address cannot see a wrong answer given
at another.

⇒ The assertion is now over the WHOLE index: this status reaches **no
target at all**. `by_target` is reached directly rather than through
[`Statuses::on`] for exactly that reason — `on` can only be asked about
an id somebody already suspects.
