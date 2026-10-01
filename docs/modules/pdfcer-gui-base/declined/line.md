# `pdfcer-gui-base/declined/line`

## Item notes

### `fn line`

The mapping is the whole of this module's contribution to the copy;
every word an operator reads is [`crate::text::status`]'s, under rule
R1.

# Why this returns a [`Cow`] rather than `&'static str`

Because exactly one decline in this enum has a **subject the operator can
see** — `OPERATOR_REQUESTS.md` O141's *"pdfcer cannot type a `q` into
this text"* — and while the return type was `&'static str` it could not
say which character. The sentence was written generically for that
reason, and the reason was recorded in three separate doc comments as
though it were a design choice; it was a return type.

The cost is one arm. Every other sentence here is still borrowed static
prose returned through `fixed` below, so the bar — which redraws every
frame — allocates nothing except on the frames that are reporting that
one refusal.
