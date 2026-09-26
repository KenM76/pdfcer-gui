# `canvas::stampfit` — the operator's standing answer to *"what happens
when a stamp's words do not fit its box?"*

One preference, read and written through the egui context, shared by the
**two** surfaces that can ask a stamp's label to change size:

* `dialogs::textannot` — placing a new stamp, where the operator picks a
  size before the mark exists;
* `panels::properties::markup::textannot` — retyping the size of a stamp
  already on the page, which `pdfcer-core` `Pass 292.0` made possible.

## Why this is a preference and not a property read off the file

Because **nothing in a PDF records the author's fit intent**, and the
engine says so in the field's own doc rather than leaving it to be
discovered:

> It is a caller's choice rather than a recovered one because *nothing in
> the file records the author's fit intent*, and a guess from the current
> geometry would invent a decision nobody made. Same reason
> `text_spec_from_dict` does not recover it.

⇒ A stamp on the page can be asked *what size are your words* (there is a
`Tf` in its appearance, or a `/DA`) and cannot be asked *what did your
author want to happen if they stopped fitting* — that question has no
answer in the bytes. A control that opened on a per-stamp value would
therefore be showing a value it had made up, differently for each stamp,
with nothing to compare against. A **standing preference** is the honest
shape: it says *this is what I want done*, which is a fact about the
operator and is exactly what it claims to be.

## Why the two surfaces share ONE preference rather than each keeping
## their own

Because they are the same question asked twice about the same stamp, and
an operator who has answered it once at placement time has not changed
their mind by opening a properties panel. Two independent stores would let
a stamp be placed under *make the stamp wider* and resized under *cut the
words off*, with no surface anywhere showing that the rule had changed
between the two acts.

⚠ **It is `insert_temp`, so it does not survive a restart**, exactly like
`canvas::textedit::pen`. That is deliberate for now: a preference that
persists into the settings file is one that needs a row in the settings
window, a default the settings window can restore, and a migration when the
engine's enum grows a fourth arm. If the operator reports re-choosing it
every session, promote it — the read/write pair here is the seam that makes
that a one-file change.

## The default, and why it is the engine's default rather than a choice
## made here

[`pdfcer_core::annot_author::StampFit::GrowToText`]. Not because it is
first in the enum but because it is the only one of the three that **cannot
be quietly wrong** (R8b rule 4): the box widens, the operator sees a wider
stamp, and the inference discloses itself by being visible. The other two
change what is drawn *without changing what is seen to have been asked
for* — a smaller label, or a shorter one — and each therefore owes an
off-canvas sentence, which is why
`text::panels::textannotstyle::stamp_label_shrunk` and `…_clipped` exist.

## Item notes

### `fn the_default_is_grow_to_text`

`TextAnnotStyle::stamp_fit`'s doc: *"`None` means
`StampFit::GrowToText`, the authoring default"*. If this constant ever
disagreed, the properties panel and the placing dialog would apply
different rules to the same stamp and neither would say so.

### `fn every_policy_is_offered_exactly_once`

The failure this catches is a list that quietly holds two of the
three — a policy an operator can never reach, with no error anywhere.
The same test `pen::FACES` has, for the same reason.
