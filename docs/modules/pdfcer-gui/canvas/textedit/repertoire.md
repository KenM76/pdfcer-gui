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

## ★★★ Ask ONCE. The alternative is a whole-page walk per character typed

The engine says which verb not to use here, by name:

> *"`preview_font_resources_for` … answers by walking **every operation in
> the page's content stream**, re-locating the run and scanning every font
> resource. Per keystroke that is a whole-page walk per character typed. Ask
> it once when the caret lands; ask this one once per run and keep the set."*

On the benchmark CAD sheet a page walk is a walk over 129,758 objects. So the
measurement is taken exactly once per `(page, run, edit_epoch)` and parked in
`egui`'s temp memory, which is where every other per-draft fact this shell
keeps already lives.

## ★★★ The FAILURE is cached too, and that is the load-bearing line

[`Held::rep`] is an `Option`, and the slot is written **even when the
measurement failed**. This is the finding `app::cache::provenance` was
corrected for on the same day: what stops a per-frame retry is not "set the
key before the work", it is *recording the attempt on the failure arm at
all*. A version of [`of_run`] that returned early on `None` without writing
the slot would re-walk the page's content stream on **every frame** for as
long as the caret sat in an unmeasurable run — invisibly, because the
feature would still behave correctly.

## ★★★ What `None` means, and it is "not measured", never "yes"

[`of_run`] answers `None` when the run cannot be pinned or the engine
returned an error. Every caller must read that as *do not gate* and let the
keystroke through, for `place::has_no_anchor`'s reason: refusing on an
unmeasured answer blocks editing everywhere on a guess. The commit-time
refusal is still there and is still correct — a keystroke that gets through
this gate is exactly as safe as it was before this module existed.

## ★★ The direction of the engine's guarantee, and why the converse holds

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
