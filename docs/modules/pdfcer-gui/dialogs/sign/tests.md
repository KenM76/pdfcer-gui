# `pdfcer-gui/dialogs/sign/tests`

## Item notes

### `fn filling`

Built directly rather than through [`super::SignDialog::open`], which
needs an `OpenDoc`. Every field this file asserts on is set here explicitly,
so a test cannot pass because a default happened to line up.

### `fn the_confirm_control_is_dead_until_the_certificate_has_been_opened`

Not until one has been chosen, and not until a passphrase has been typed —
until the container has actually been unlocked and its subject is on screen.
That is §1 of the module header, and it is the one guard this surface offers
against the mistake that matters: an operator who can fill in a reason,
choose a destination and press *Sign* with the certificate still unopened
has been allowed to make the decision before the identity check.

⚠ A build that enabled the button on `certificate.is_some()` would pass
every screenshot and every ribbon test, and would sign with whichever file
was picked.

### `fn the_disabled_hover_names_the_first_outstanding_thing`

R9: greying is only ever for temporarily unavailable, and it is **always
explained on hover**. `OPERATOR_REQUESTS.md` O77's sweep found seven greyed
controls with no explanation.

The order matters because both can be outstanding at once, and the
certificate is the one the operator must deal with first — a sentence about
a tick-box on a form whose first section is not finished sends them to the
wrong end of the window.

### `fn changing_the_destination_retires_the_overwrite_acknowledgement`

`crate::dialogs::redact::choose_destination`'s rule. Without it an operator
could tick the box, think better of it, select *a new file*, change their
mind again, and arrive back at *replace* with the consent still standing
from a decision they had explicitly withdrawn in between — on the one
control in this window that writes over a file with no picker in front of
it.

It fires on **any** change rather than only on leaving the replace choice:
retiring a tick that was not needed costs nothing, and deciding which
changes matter is where a future edit gets it wrong.

### `fn no_phase_but_filling_offers_a_confirm`

The `Signing` arm is the one worth having: it lasts one frame in practice,
and "in practice" is an assumption about a machine. Without it a second
press on a slow document signs twice — two files, two signatures, and the
second one written over the first if the destination was *replace*.

### `fn the_debug_impl_carries_neither_the_passphrase_nor_the_certificate`

The mechanism, asserted rather than trusted. `crate::secret`'s header
records what the alternative costs: `Action` derives `Debug`, this crate
traces to stderr under `PDFCER_DIAG`, and **`tools/ui-verify` captures that
stderr to a file it keeps as evidence** — so a single `{:?}` on this struct
would write the operator's passphrase to disk, in a directory whose whole
purpose is to be kept and read.

The **path** is asserted absent too, which goes further than
`crate::dialogs::protect`'s equivalent. A path is not key material; it is a
durable pointer at where somebody keeps their digital ID.

### `fn choosing_a_new_certificate_clears_what_the_old_one_said`

Leaving either would show the operator a read-back of the certificate they
just replaced — which is the one sentence on this window that must never
describe a different file from the one that will sign — or an error about a
file they are no longer using.

### `fn the_five_refusals_are_five_different_sentences`

The cheap test that catches the expensive mistake: a `match` whose arms
were filled in by copying the one above it. Five refusals, five different
next moves for the operator, and a build that gave two of them the same
words would send somebody to take the password off a document that has a
redaction armed.

### `fn nothing_on_this_surface_claims_a_signature_is_trusted`

`crate::text::sign`'s first rule, enforced rather than remembered.
Authoring a signature and a recipient trusting it are different facts
settled by different parties, and this surface only ever performs the first.
[`crate::panels::signatures`] is the only place in pdfcer that reports the
second; it reports three facts that never collapse into one, and one
cheerful word here would undo that design before the panel is opened.

⚠ The word list is deliberately blunt and will catch an innocent sentence
one day. That is the right failure: the fix is to re-word the sentence, and
a reviewer who thinks the word is fine has to say so in a commit.

### `fn the_box_is_described_as_carrying_the_name_and_the_date`

Until `Cargo.lock` moved to `d6b998f` (v0.42.0),
[`crate::text::sign::placement_note`] told the operator *"The box is an
empty frame: pdfcer does not yet draw your name or the date inside it."*
Engine `Pass 10.14` composes the signer's CN, the date and the reason and
location into it — so the sentence became false the moment the pin moved.

⚠ **And it became false in the direction nothing reports.** An operator told
the box would be empty, who then finds his name in it, has been
under-promised; he files nothing, no screen looks wrong, and no test was
asserting the claim. What caught it was a doc comment carrying its own
expiry date and the engine commit that would void it.

⇒ **Where a claim about the engine CAN be an assertion, make it one.** This
is that assertion. It names both old wordings so a future "simplification"
cannot reinstate one out of git history, and it requires the two facts the
box now actually carries.

### `fn an_author_imposed_refusal_names_the_author_and_not_pdfcer`

The single most important property of any string added on 2026-09-06.
`Pass 10.13` enforces a signature field's `/SV` dictionary in full and is
deliberately stricter than Acrobat, so the operator **will** meet refusals
here on documents another reader signs. The general wording,
[`crate::text::sign::engine_refused`] — *"pdfcer did not sign the document:
…"* — would tell him, in plain English, that pdfcer is broken; he would be
right to conclude it from the sentence and wrong about the program, and a
working feature would be reported as a defect.

So: the document's preparer is named, and named FIRST; the strictness is
admitted as a choice rather than hidden; and a remedy is offered that does
not require pdfcer to change.

### `fn the_written_summary_carries_the_reuse_the_lock_and_the_notes`

The rule-4 disclosure has to distinguish the two outcomes an operator cannot
tell apart from the file: signed IN the sender's box, or signed beside it.
And `SignReport::notes` — the seed-value constraints the author RECOMMENDED
and this signature does not meet — must reach the screen, because they are
the ones that did **not** refuse and would otherwise be silent.

### `fn the_written_summary_carries_the_text_the_signature_box_shows`

`SignReport::appearance_lines` is the text the engine composed into a
visible signature's appearance — the certificate's subject, the time of
signing, and the reason and location as given. The operator never wrote that
composition, and he cannot read it off the document he still has open,
because that one is the unsigned original he started from. A fact about what
was written that is obtainable no other way is rule 4's whole subject.

### `fn the_appearance_counter_measures_the_sentence_not_the_slice`

`appearance_shown` is the shell's own account of whether the page's composed
text reached the operator, and `ui-verify` believes it. So it is asserted on
three shapes: every line present, a line dropped, and a line that appears in
the sentence but not as an appearance line — the last because a counter
matching a bare substring would score the subject and report a disclosure
that never happened.

### `fn choosing_the_senders_box_produces_no_rectangle`

The one that matters: choosing the sender's box must NOT produce a
rectangle. `SignRequest::visible` beside a resolving `field_name` is
`RectRefusedForExistingField`, so a build that carried both would refuse the
ordinary case the feature exists for.

### `fn a_field_index_with_no_field_draws_nothing`

Unreachable from the window — `Place::Existing` is only offered when a
selectable field exists — so this pins the *direction* of a guess rather
than a live path. When a surface has to guess on a branch it believes
impossible, it should guess toward writing LESS into the operator's file: a
fallback of `Visible` would stamp a box carrying his name on a page he never
asked to have marked.
