# `canvas::links` — **following a `/Link`, which this program could not do
at all until now**

Operator report, 2026-09-01: *"does a clickable table of contents work?"*

It did not, and the honest description of the state was worse than "there is
a bug". **There was no link-following code path in the shell whatsoever.**
Clicking a `/Link` did nothing, nothing on screen suggested it would, and a
drawing package whose entire navigation is a hyperlinked contents sheet
behaved like a stack of loose pages.


A link's destination **could not be read**. `pdfcer_core::annot::Annotation`
carries `action_type` — the `/S` name, so the string `GoTo` — by an explicit
and well-reasoned engine decision: *"the `/S` NAME only, deliberately — not
the action dictionary"*. That is right for `list-annotations`, whose job is
to print one token per annotation. It is useless to a viewer, whose entire
job with a `GoTo` is to **perform** it, and the shell has no raw object-graph
access with which to walk `/D` itself — nor should it, or the §12.3.2.2
name-tree walk would exist twice and the two copies would drift.

The engine shipped `outline::DestinationReader` and
`annot::page_link_destinations` in answer to that request. This module is the
shell half, and it is short because the hard half is elsewhere: hit-test a
rectangle, hand the destination to the pipeline the bookmarks panel already
uses.

## The five destinations, and why collapsing four of them is the defect

`Destination` has five variants and **only one navigates**. The engine's own
note on shipping the reader states the failure modes exactly:

> *"A viewer that maps the last four to 'no link here' reports a document
> full of working links as empty. One that maps them to a page jump lies
> about where it goes."*

So [`follow`] has five arms and no catch-all, and every non-navigating arm
raises a **different** sentence from [`crate::text::links`] — because the
four fail for different reasons with different remedies. A deleted target
page, a lost name table, another file, and an action pdfcer deliberately does
not run are four situations, not one.

## The affordance is a CURSOR, and there is no mark on the page

[`cursor`] sets a pointing hand over a link that can be followed and does
nothing over one that cannot. That is the whole of the pre-click disclosure,
and it is bounded by rule 4 in both directions:

* a cursor is an **affordance**, not content styling — the same clause that
  permits a snap indicator and a hover highlight, and the same reasoning
  `canvas::forms` gives for the hand it puts over a fillable widget;
* **nothing is drawn into the page.** No border, no tint, no dashed
  rectangle over the link's `/Rect`. A screenshot of this canvas is identical
  to a screenshot of the same document saved and reopened, which is the
  one-line test rule 4 is judged by.

A hand cursor over the **non**-navigable four was considered and rejected:
it advertises a capability that does not exist, and R9 says an unavailable
capability renders nothing. Their disclosure arrives on the click, where the
operator has actually asked.

The disclosure is raised on a **click only, never on hover.** A sentence
that appeared because the pointer crossed a rectangle would fire dozens of
times crossing a contents sheet, and a status line that changes without the
operator doing anything is a status line they stop reading.

## Which modes follow a link, and why Edit does not

Read and Review follow. **Edit selects**, because in Edit a `/Link` is an
annotation like any other and the operator is there to move it, resize it or
delete it — and a click that navigated away instead would make a link the one
annotation in the document that cannot be edited.

That split is `caps.edit_content`, the same predicate `canvas::forms` uses
for the identical reason: *"the same click cannot both type a value and
select the box to rename it."* It is also the convention — every program that
both reads and authors PDFs separates the two by tool or by mode, and this
project's standing rule is to use the conventional interaction rather than
invent one.

## Cost

One `/Annots` walk per `(page, edit epoch)`, cached on `OpenDoc` — see
[`crate::app::cache::LinkCache`], which also explains why the O(document)
`DestinationReader` is cached on a *different* key. [`cursor`] runs on every
frame the pointer is over a page, so without that cache this feature would
walk a 36-sheet drawing's page tree on every mouse move.

## Item notes

### `fn only_a_resolved_page_is_navigable`

The single most important assertion in this file, and the one the
engine's note is explicitly about: a viewer that treats `UnmappedPage`
or `Named` as navigable jumps to a defaulted page 0 and tells the
operator, confidently, that their link goes to the front of the
document. That failure has no symptom — the page turns, something is
shown — and it would never be reported as a bug.

### `fn each_destination_kind_has_its_own_trace_token`

Pinned because the trace is the only oracle a driven check has for
*which* of the five happened, and two variants sharing a token would
make the check unable to tell a followed link from a disclosed one.

### `fn a_remote_page_number_is_shown_one_based`

`RemoteTarget::Page` is 0-based, every page number this program shows an
operator is 1-based, and the engine's own reply flagged this as the
conversion it nearly got wrong in its CLI. A sentence naming "page 0"
would be wrong in a way the operator cannot check without opening the
other file.

### `struct Hit`

**Owned, not borrowed.** `OpenDoc::page_links` hands back a `Ref`, and a
caller holding one cannot then take `&mut OpenDoc` — which every consumer
here eventually needs, directly or through the action funnel. One
`Destination` clone per hit is a handful of bytes on a gesture the operator
made deliberately; the borrow it avoids is a whole class of runtime panic.

### `fn navigable`

The predicate the cursor is decided by, and deliberately narrow: only
`Destination::Page` is navigable. See the module header on why a hand
over the other four would be advertising a capability that is not there.

### `fn under_pointer`

`point` is in **canvas/page coordinates**, the same space
`PageMapping::to_page` produces and the same space every other hit test on
this surface takes.

## Last match wins

`/Annots` is painted in array order, so a later entry is drawn over an
earlier one and is the one under the pointer where two overlap. Overlapping
links are rare and are exactly the case a first-match scan gets backwards —
and a first-match scan looks correct on every document that does not have
them, which is almost all of them.

## A link with no `/Rect` is skipped, and that is not a filter

§12.5.2 makes `/Rect` required, so a link without one has a destination it
can never be clicked to reach. The engine keeps it in the list rather than
dropping it — so a repair tool can see it — and a hit test must skip it,
because there is no box to be inside.

### `fn follow`

Navigates when it can and says why when it cannot. Every branch does exactly
one of those two things and there is no silent arm — see the module header
on the four non-navigating variants.

### `fn cursor`

The whole of the discovery affordance, and the only thing this module puts
on screen. Called once per frame from `canvas::present`, after
`canvas::forms`' own cursor pass and before `canvas::interact` runs — so
`canvas::tool::cursor_for` still has the last word, which is right: a
cursor it has an opinion about is one describing a gesture already under
way, and that outranks a hover.

Does nothing in a mode that edits content, matching [`follow`]'s own gate.
A hand promising navigation in a mode where the click selects instead would
be a lie told sixty times a second.
