# `canvas::textedit::repertoire` — the alphabet this run accepts, asked once
when the caret lands instead of discovered one refused keystroke at a time

## The operator problem this module exists for

Before `Pass 280.0` the shell learned that a run could not take a character
**at the commit**. He clicked into a title-block cell, typed a word, pressed
`Ctrl+Enter`, and the engine refused the whole edit because one character in
it had no code in the run's font. The word was gone — `commit_into` calls
`abandon` whether or not the engine accepted — and the sentence explaining
why arrived after the loss rather than before it.

This project's own request, quoted back in the engine's rustdoc:

> *"the refusal arrives **at commit**, so he types a whole word and then
> loses it. The alphabet is knowable before the first keystroke and we do not
> use it that way yet."*

[`pdfcer_core::edit::EditSession::run_repertoire`] is the answer, and this
module is the whole of the shell side of it: ask once when the caret lands,
keep the set, and decline the keys that are not in it **as they are pressed**
rather than after the last one.

## Ask ONCE. The alternative is a whole-page walk per character typed

The engine says which verb not to use here, by name:

> *"`preview_font_resources_for` … answers by walking **every operation in
> the page's content stream**, re-locating the run and scanning every font
> resource. Per keystroke that is a whole-page walk per character typed. Ask
> it once when the caret lands; ask this one once per run and keep the set."*

On the benchmark CAD sheet a page walk is a walk over 129,758 objects. So the
measurement is taken exactly once per `(page, run, edit_epoch)` and parked in
`egui`'s temp memory, which is where every other per-draft fact this shell
keeps already lives.

## The FAILURE is cached too, and that is the load-bearing line

[`Held::rep`] is an `Option`, and the slot is written **even when the
measurement failed**. This is the finding `app::cache::provenance` was
corrected for on the same day: what stops a per-frame retry is not "set the
key before the work", it is *recording the attempt on the failure arm at
all*. A version of [`of_run`] that returned early on `None` without writing
the slot would re-walk the page's content stream on **every frame** for as
long as the caret sat in an unmeasurable run — invisibly, because the
feature would still behave correctly.

## What `None` means, and it is "not measured", never "yes"

[`of_run`] answers `None` when the run cannot be pinned or the engine
returned an error. Every caller must read that as *do not gate* and let the
keystroke through, for `place::has_no_anchor`'s reason: refusing on an
unmeasured answer blocks editing everywhere on a guess. The commit-time
refusal is still there and is still correct — a keystroke that gets through
this gate is exactly as safe as it was before this module existed.

## The direction of the engine's guarantee, and why the converse holds

The engine guarantees one direction only:

> *"A character in `RunRepertoire::accepted` is one `edit_text` will not
> refuse for this run."*

This module relies on the **converse** — a character *not* in `accepted` is
one `edit_text` *would* refuse — because that is what licenses declining the
key. That is a stronger claim than the engine wrote down, and it holds for a
reason the engine states elsewhere in the same function: the candidate domain
is the font's own inverse map (`candidate_chars`), acceptance is decided by
calling the accepting code itself, and the `prefer` seed *"decides WHICH code
a character gets, never WHETHER it is accepted"*. A character outside
`accepted` is outside it because the accepting code refused it or the subset
floor removed it — both of which the commit would hit again.

⚠ **If that ever stops being true, the symptom is a key that does nothing on
text the engine would have accepted**, and it is invisible to every test that
drives the commit path. The two halves are driven separately below, against
the engine rather than against this module's own set.

## Item notes

### `const MEMORY_KEY`

One slot, not a map. A draft has one anchor, so at most one run is being
typed into at any moment, and a map would be a cache of measurements for
runs nobody is editing — held across page changes, invalidated by nothing.

### `struct Held`

# Why `epoch` is in the key

Because an edit changes the answer. `edit_text` rewrites the run's show
operator and may narrow which codes the embedded subset carries; a
repertoire measured before it describes a page that no longer exists.
`OpenDoc::edit_epoch` is this shell's one monotonic *the document changed*
counter and is what every other derived-from-the-page cache here is keyed
on.

`Clone` and `Send + Sync` are not stylistic: `egui::Context::data`'s
`get_temp`/`insert_temp` require `T: Clone + Send + Sync + 'static`, which
is why the payload is an [`Arc`] and not an `Rc`. The handle is cloned once
per read; the repertoire behind it never is.

### `fn measure`

# An EMPTY `find`, with the pin — the shape `pin::font_preflight` already
# uses, and for the same reason

`Pass 147.0` taught the engine to resolve a pinned operator's own characters
through `effective_find`, so `""` plus a pin means *the whole pinned
operator*. Passing the run's extracted text instead would be a second
description of what the pin already names, and on a run whose extraction
synthesised a space the two disagree — which is how a preflight came to
report every face on the page as acceptable for one day in August.

⚠ An empty find with **no** pin is refused by the engine by name, which is
why this answers `None` rather than falling back to an unpinned query when
[`super::pin::resolve`] answers `None`. An unpinned empty find would be
answered about the first operator on the page — a different run's alphabet,
presented as this one's.

### `const FIXTURE`

`fixtures/subset-font-floor.PROVENANCE.md` argues at length why no other
document here can stand in for it, and the short form is that every other
one either carries a non-embedded standard-14 face (whose
`WinAnsiEncoding` accepts anything Latin-1), or a fully embedded
non-subset face (the floor never fires), or a symbolic face that refuses
for an unrelated reason. A check driven against any of those would be
**unable to fail**, which is the failure mode this project has spent more
sessions on than any other.

⚠ The provenance note ends *"do not improve it"* and means it: the subset
tag, the `/FontFile2` and the three-letter single-operator run are each
load-bearing here.

### `const OUTSIDE_THE_SUBSET`

A plain lowercase `q`, deliberately, and not `€` or an accented letter:
the operator-facing point of the whole gate is that the wall is **not**
about symbols. A subset face built from a page that prints six capitals
cannot type an ordinary lowercase letter either, and a check that only
ever probed with a euro sign would let a reader believe otherwise.

### `fn raw_session`

Deliberately **not** `open_local_fixture(..).session`: that one is an
[`Arc`], because the shell shares one session across panels, and
`edit_text` needs `&mut`. The two tests that drive the engine want a
session they can spend.

### `fn pin_of_run_zero`

Re-measured on every call rather than computed once and reused, for the
reason `facewall` writes out at length: a span pinned before a stream was
rewritten explains any downstream refusal, and leaving that explanation
available is how a check comes to measure the harness instead of the
program.

### `fn the_fixture_run_is_editable_and_its_alphabet_is_the_subset`

The control for everything below: if this run were not editable, or if
its repertoire happened to contain every character, none of the other
checks here could fail and the module would be untested while reporting
green.

### `fn the_engine_accepts_every_character_the_repertoire_names`

This is the direction the engine wrote down. It is driven anyway, because
a guarantee in a doc comment is a claim about someone else's function and
this project's standing rule is that such a claim gets measured — the
engine moves daily and this shell is pinned to a commit, not to a
promise.

A fresh session per character, because `edit_text` mutates the run and
the second character would then be typed into whatever the first one
left.

### `fn the_engine_refuses_a_character_the_repertoire_omits`

# Why this is the most important check in the file

The gate this module feeds *stops the keystroke*. If the converse does
not hold — if some character outside `accepted` would in fact have been
accepted — then the shell refuses input the document could have taken,
and does it silently from the operator's point of view, because the
character simply never appears. That is a worse defect than the one the
gate was built to fix: losing a word at commit is at least visible.

The engine's guarantee is one-directional by construction (its own source
tests candidates and collects the ones that survive), so the converse can
only be held by measurement, and only on a font whose floor bites.

### `fn an_unmeasurable_run_records_the_attempt`

The check that pins the module header's load-bearing line. Move the
`write` behind the `?` in [`of_run`] — the shape a reader will reach for,
because writing a `None` looks like caching nothing — and this test goes
red while every other test in the file stays green. Without it, a run the
engine cannot answer for is re-walked on every frame the caret sits in
it, which on the benchmark sheet is a 129,758-object walk per frame and
presents to the operator as the application hanging while he types.

### `fn an_edit_invalidates_the_measurement`

`edit_text` rewrites the run's show operator and can narrow which codes
the embedded subset carries, so a repertoire measured before it describes
a page that no longer exists. Drop `epoch` from [`of_run`]'s key
comparison and this goes red on its own.

### `fn the_sieve_splits_a_mixed_keystroke`

The unit the keystroke handler consumes, over the fixture whose floor
actually bites — so `kept` and `refused` are both non-trivial in one
call and a build that returned the input unchanged fails here.

### `fn the_sieve_names_the_first_refusal_only`

One bar, one sentence. A build that reported the last one would look
identical on a single keystroke and differ on an IME commit, which is
exactly the class of difference nobody notices until an operator with a
compose key does.

### `fn the_sieve_keeps_everything_when_nothing_was_measured`

The permissive arm, and the one that must never regress: a run the engine
could not answer for has to type exactly as it did before this module
existed.

### `fn forgetting_clears_the_slot`

Belt and braces over [`of_run`]'s key check, for the reason
[`forget`] documents: a slot that outlives its subject is a fossil, and
this project has already spent a session on one.
