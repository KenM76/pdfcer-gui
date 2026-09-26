# `app::status::decline::fresh` — is the sentence still true?


## The seam

[`super`]'s header sets itself two jobs: *what is a decline* and *how long
does it owe its sentence*. Every split so far has taken a question that was
neither of those — `floor` answers one about somebody else's protocol,
`record` answers *who says one*, `line` answers *in what words*. This one is
different and is the better cut for it: it takes **the whole of the second
job**, leaving [`super`] the enum and the store.

| question | answered in |
|---|---|
| what is a decline? | [`super`] — the [`Declined`] enum and its 40-odd variants |
| who says one, and from which phase? | `decline::record`, `decline::floor`, `decline::textedit`, `decline::clipboard` |
| in what words? | `decline::line` |
| **is it still true?** | **here** |
| where is it stored, and who clears it? | [`super`] — `LAST`, [`super::retire`], [`super::live`] |

## ★★ Why the PURE half, specifically

This is the project's standing split, quoted in `still_true`'s own doc from
`crate::viewer`'s header: *"this module is unit-testable and the widget code
is not."* [`super::live`] needs an `egui::Context` and an
[`OpenDoc`](crate::app::state::OpenDoc) to gather its facts and therefore
cannot be asserted headlessly; `still_true` takes four plain values and can
be, which is why `decline/tests.rs` calls it directly several hundred times.
Moving the testable half out leaves each file with one testability story
instead of two.

★ So the rule for what may be added here: **nothing that needs a document,
a context or a frame.** The parameter list is the contract — see
`still_true`'s own note on why it is a list of named facts rather than a
`&OpenDoc` — and a function here that could go and ask its own question
would dissolve that contract from the inside.

## ★★★ The two arguments every arm in here is making, and they are not the
same argument

A reader adding a variant will reach for `=> true` and should know which
`true` they mean, because the wrong one is invisible:

- **"the FILE cannot change under it"** — a certification, whether an
  appearance is pdfcer's own, whether an annotation is a fixed-size marker.
  The sentence stays true because the fact it reports is a property of the
  document, and the document does not change while the operator reads the
  status bar.
- **"NOTHING HAPPENED"** — a refusal raised before a byte was staged or an
  undo entry pushed. There is no later state for the sentence to be stale
  against, because there is no later state.

Both retire the same way — by [`super::retire`], on the operator's next
command — and that shared ending is what makes them easy to conflate. They
diverge the moment somebody adds a variant whose fact **is** re-askable: put
that one in the wrong group and the bar keeps a sentence the document has
already contradicted, which is the one failure mode this whole module exists
to prevent.

★ And a third group worth naming because it is the one people reach for and
should not: **re-asking by walking the document.** Every predicate in here
runs on every frame the bar draws. `parse_acroform`, a page walk, a text
extraction — none of them belong, and a fact that can only be re-derived
that way is a fact that should be answered by `true` plus [`super::retire`].
