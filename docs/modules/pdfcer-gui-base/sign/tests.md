# `pdfcer-gui-base/sign/tests`

## Item notes

### `fn clean`

Built field by field rather than read off a document, because every test
below is about **the ladder**, not about the reading. `Standing::read` is
exercised by the driven check, where a real document is open.

### `fn with_field`

Built field by field rather than read off a document, for [`clean`]'s
reason: the tests below are about what this shell DOES with a field, and the
reading of one out of an `/AcroForm` is exercised by the driven check on the
engine's own `sig-field-empty.pdf`.

### `fn a_pre_placed_field_is_named_and_carries_no_rectangle`

The whole of `Pass 10.13`'s contract from this side, and the one property a
build could get wrong in a way nothing else would notice: sending `visible`
beside a `field_name` that resolves is
`SignApplyError::RectRefusedForExistingField`, so the operator would meet a
refusal on the ordinary case the feature exists for.

⚠ Asserted through `Placement` rather than through `prepare`, which needs a
real `EditSession` and a private key. What this pins is that the three arms
are exclusive **by construction** — the enum cannot carry both — which is
the mechanism, and the mapping in `prepare` is one `match` in view of it.

### `fn a_kids_field_is_listed_and_refused_by_name`

Listed WITH its reason rather than filtered out, which is the rule this
shell applies to every unreachable option on this window: an operator hunting
for the box the sender told him about must find it and be told why it is not
offered. An absent row is indistinguishable from a document that never had
one.

### `fn a_lock_and_a_seed_value_survive_the_reading`

Both are disclosed BEFORE the press — the lock because signing freezes
fields the author nominated, the constraints because a refusal is possible
and it will be the author's rule rather than pdfcer's. A build that read the
field and dropped these two would produce an identical list, an identical
signature, and an operator who learned about the freeze from the summary
afterwards.

### `fn a_clean_document_may_be_certified`

The negative control for the two below: without it, a build whose
`may_certify` returned `Err` unconditionally would pass both refusal tests
and never offer a certification.

### `fn a_signed_document_cannot_be_certified`

§12.8.2.2.1: the certifier is the author, *"the person applying the first
signature"* — a later certification could not govern the changes made before
it. Refused HERE, when the window opens, rather than by the engine after the
form is filled in and a save path chosen.

### `fn an_already_certified_document_names_the_certification_not_the_count`

A document that is both certified and signed satisfies both guards. The
engine checks `AlreadyCertified` first, so this shell does too — two surfaces
disagreeing about which of two true things to say is how an operator learns
to distrust both.

### `fn a_clean_document_is_not_refused`

The negative control for every test below it. Without this one, a build
whose `refusal` returned `Some` unconditionally would pass all five of the
refusal tests and never sign anything.

### `fn an_encrypted_document_is_refused`

One of the two refusals the build brief names as *reachable rather than
theoretical*: Security ▸ Encrypt… ships, so this shell can produce an
encrypted document and then be asked to sign it, in one session, without
leaving the application.

### `fn a_pending_redaction_is_named_before_encryption`

Both are true of a document that was redacted and then encrypted — which
this shell can produce, because `set_encryption` ignores a pending redaction
(a defect already filed at the engine). One sentence is shown, so which one
matters: the redaction is **one press away** from being applied or called
off, and the encryption is a wall. Naming the wall while the gate beside it
is merely latched spends the only sentence the operator reads on the answer
they cannot act on.

A build that reversed this would look correct in every screenshot.

### `fn only_the_strictest_certification_refuses_a_signature`

The arm most likely to be written wrong, and the engine's own comment
says why: *"Table 254: P = 1 permits NO changes; 2 permits form fill-in AND
signing; 3 adds annotations. Adding a signature is the act P = 2 exists to
allow."* A shell that refused every certified document would refuse the
commonest legitimate case there is — a document certified precisely so that
other people could sign it.

### `fn a_recovered_base_and_an_unsaved_document_are_both_refused`

The second has no engine counterpart — see §4 of the module header. The
engine would sign a document with no file behind it; this shell will not,
because an incremental update is an appendix to a specific file and the
operator's next ordinary Save would write a different one.

### `fn a_document_that_is_already_signed_can_be_signed_again`

The one place this surface differs from [`crate::protect`], which refuses
a signed document outright because encrypting rewrites every byte a
signature covers. Signing appends, so a second signature is legitimate and
PDF is built for it. The window says so rather than staying silent, but it
does not stand in the way.

### `fn the_visible_box_sits_where_the_window_says_it_does`

Asserted as four numbers rather than as "near the corner", because the
box is **content written into the operator's file** — R8b Rule 4 — and
`crate::text::sign::placement_where` states the measurements on screen. A
sentence on a window and a constant in a file that could disagree is the
shape of a disclosure that stops being true.

### `fn the_visible_box_is_clamped_onto_a_small_page`

The clamp, and it is not defensive tidiness. The engine accepts whatever
rectangle it is handed; a widget laid partly outside the media box is
present in the file and drawn by nothing, so the operator would tick
*"draw a signature box"*, get a valid signed document, and see no box. That
is a feature that traces perfectly and does nothing — the exact class this
project has shipped before.

### `fn the_visible_box_follows_an_offset_page_origin`

Real CAD exports carry offset media boxes. Computing from `0` rather than
from the page's own lower-left would put the box a page-width away from
where the operator expects it, on exactly the documents this operator works
with and on none of the fixtures.

### `fn an_empty_or_blank_field_is_left_out_of_the_signature`

`/Reason ()` in a signature dictionary is a claim that the operator gave a
reason and it was nothing — a different statement from a key that is not
there. And a field holding one space is an untouched field as far as anybody
looking at the screen is concerned; writing `/Reason ( )` into a legal
document because of a stray keystroke is the kind of thing nobody ever
finds.
