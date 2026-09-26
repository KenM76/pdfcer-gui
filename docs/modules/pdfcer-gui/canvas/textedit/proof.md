# `canvas::textedit::proof` — **the tail did not move, proved in the bytes**

`crate::redact::proof`'s shape applied to `DEFECTS.md` **D4b**: a claim about
what an edit does to a *file*, asserted against the file, with the falsifying
run beside it.

## What is being proved, and why nothing else in the suite proves it

D4b says two things are wrong on commit, and both are about **text the
operator did not touch**:

1. a right-aligned / centred / justified tail is pushed off the edge it is
   flush against;
2. a rotated line's tail is displaced along user-space x, which is not the
   direction its baseline runs.

Neither is visible in any assertion about the edit itself. The edited run
comes out correct in both builds — the right glyphs, the right place, the
right font. What differs is a `Tm` belonging to something else, twenty bytes
further down the content stream, and the only oracle that can see it is the
stream.

## The falsifying run is the point of the module

Every assertion here is made **twice**: once through
[`super::disposition::options`], which is the shipped decision, and once
through `EditOptions::default()`, which is what a shell that made no
decision at all would pass. The second run is not decoration — it is what
makes the first one evidence:

* a build with the rule removed produces exactly `EditOptions::default()`,
  so the falsifying run *is* the broken build, executed;
* a fixture that does not actually exercise the defect makes both runs
  agree, and the `assert_ne!` between them then fails — so this cannot pass
  by flattering the thing it measures.

## Why the fixture's content stream is uncompressed

Because the oracle is a byte scan for a `Tm` operand triple, and a
Flate-compressed stream would answer "absent" for a correct build and a
broken one alike — a false pass in the only direction that matters. See
`tools/gen-textedit-fixtures.py`, which says the same thing from the
producing side.

## Where the *second process* proof lives

Not here. This module proves the arithmetic against the written bytes in one
process; `tools/ui-verify`'s `text_edit_pins_an_aligned_tail` drives the real
binary, saves a copy through the real command, and re-opens it in a second
process — which is the only honest proof that an edit reached the bytes *by
the route an operator takes*. The two answer different questions and neither
substitutes for the other.

## Item notes

### `fn fixture`

In **this** repository's `fixtures/`, not the engine's, and it is the one
fixture in either tree that carries right-aligned and rotated text — see
`DEFECTS.md` D4's closing section, which is an inventory of the conditions
every existing fixture omits by construction.

### `fn extract`

Panics rather than returning an `Option`: a fixture that stopped containing
its own text would otherwise make every test below pass vacuously, which is
the failure mode `run-all.sh`'s three-state model exists to refuse.

### `fn rotated_run`

**Why the rotated line cannot be found by its text, and this is a real
finding rather than a test detail.** `extract_page_view`'s line clustering
groups glyphs by horizontal proximity, so a line whose baseline runs *up* the
page is not clustered at all: `TITLE VERTICAL` comes back as the fifteen runs
`"T"`, `"IT"`, `"L"`, `"E"`, `" V"`, … one or two glyphs each.

Two consequences, both true of the shipped shell and neither a problem:

* a caret placed on rotated text lands in a **fragment**, so the edit replaces
  that fragment. It is still the right operator, because the provenance pin
  identifies the show operator rather than the run, and the fragment is by
  construction a substring of that operator's decoded text.
* this helper has to ask the *geometry* rather than the text, which is what
  `disposition::choose` reads anyway.

### `fn reason_for`

This is [`super::plan`]'s body with the document plumbing removed — the same
three derivations in the same order, against the same engine calls — because
`plan` needs an `OpenDoc` and the point here is the arithmetic.

### `fn appended_after_edit`

The base revision is excluded deliberately, and it is the whole reason this
helper exists rather than a `std::fs::read`. §7.5.6 forbids an incremental
update from rewriting what came before it, so the original `Tm` is *always*
still present in the first `base.len()` bytes — of a correct build and a
broken one alike. A scan over the whole file would therefore answer "the tail
is unmoved" for every build ever written. What is being asked is what the
**new** content object says, and that is what was appended.

### `fn a_right_aligned_block_reaches_the_pin_rule`

The unit tests in [`super::disposition`] assert what the rule does with a
finding; this asserts that the finding the engine actually produces for
right-aligned text is the one those tests assume. Without it the whole fix
could be correct and unreachable — which, per that module's header, is
exactly what happens if the *default* block recogniser is used instead of the
relaxed one.

### `fn a_rotated_line_reaches_the_rotation_guard`

`[0 1 -1 0 e f]` — the shape a SolidWorks title block's side text has, and
the case `DEFECTS.md` D4b says *"bites rotated CAD title-block text
specifically, which is exactly this operator's documents."*

### `fn upright_left_aligned_text_still_reflows`

A build that answered `Pin` unconditionally would satisfy both tests above
and would not be the fix: it would freeze every line on every ordinary
document, so an edit that lengthened a word would overlap the next one
instead of pushing it along. This is the assertion that tells the fix from a
blanket pin, and it is the reason the fixture carries a third text object
that neither of the other tests looks at.

### `fn the_right_aligned_tail_is_left_exactly_where_it_was`

The fixture's three right-aligned lines share one `BT`/`ET`, so the engine's
reflow walk reaches lines 2 and 3 from an edit to line 1. Line 3's `Tm` is
`1 0 0 1 412.64 668.00 Tm` and the replacement is longer than what it
replaces, so:

* under the shipped rule (`Pin`) that operator is re-emitted **verbatim**;
* under `EditOptions::default()` — the old shell's only call site — it is
  rewritten with `e` increased by the advance delta, and the string is gone.

# The second run here is an INVERTED control

It asserts that `EditOptions::default()` — plain `Reflow` — *also* leaves
line 3's `Tm` alone, and that is the engine's property rather than this
shell's: the engine continues a line only through a `Tm` that differs in
`e` alone — same orientation, same scale, same baseline — and lines 2 and 3
of this block sit on different baselines.

That narrowing is worth a guard of its own. Against a walk that shifted
every absolute `Tm` until a `Td`/`TD`/`T*` boundary, a four-character edit
on the operator's real drawing reported `followers_repositioned=1676` and
changed **34,059 pixels across the whole sheet**, because a CAD stream
positions everything with `Tm` and never emits the `Td` such a walk looks
for. The same edit changes 42 pixels inside one label.

⇒ So the shell's `Pin` rule is belt-and-braces for this shape rather than
the only defence, and this assertion fires if the walk is ever loosened
again. The case where `Pin` IS the only defence has its own test and its
own block in the fixture: see
[`a_same_baseline_follower_is_the_case_pinning_still_prevents`].

### `fn the_rotated_tail_is_not_slid_along_the_wrong_axis`

The follower is `0 1 -1 0 90.00 420.00 Tm`. Its baseline runs **up** the
page — the text-space x unit vector maps to `(0, 1)` in user space — so a
text-space advance of `Δ` should displace it by `(0, Δ)`. The engine's reflow
branch writes `e + Δ`, i.e. `(Δ, 0)`: the right magnitude on the wrong axis.

D4b case 2, asserted in the bytes. It needs a fixture carrying rotated text,
which is why `fixtures/tail-alignment.pdf` exists in this repository rather
than being borrowed from the engine's.

### `fn a_same_baseline_follower_is_the_case_pinning_still_prevents`

# Why a fixture that cannot exhibit the hazard proves nothing

Reflow reaches no follower in blocks A, B or C — every one of them sits on a
different baseline or at a different orientation — so both falsifying
assertions above are inverted controls, and **a quiet falsifier is a test
that has stopped measuring**: the two `Pin` assertions beside them would go
on passing against a build that pinned nothing, because nothing was going to
move either way.

Block D is the one shape reflow still acts on: two show operators at the
same `f`, differing in `e` alone. That is a single visual line drawn as two
runs — a table cell beside another, a title-block field beside its label —
which is the overwhelmingly common shape on this operator's documents and
the one case where *"the rest of the line"* genuinely is the rest of a
line.

This is therefore the test that tells the shipped rule from a build that
pins nothing, and the `assert_ne!` is what stops it passing vacuously.

### `fn the_replacement_text_is_in_the_appended_revision`

The floor, and it is worth stating separately: every assertion above is about
text the operator did **not** touch, and all of them would pass against a
build whose edit did nothing at all. This is the one that says an edit
happened.
