# `pdfcer-gui/dialogs/redact/tests`

## Item notes

### `fn the_suggested_name_is_never_the_source_file`

The standing rule as a default, and the single most consequential
assertion in this module: the source file is the only remaining copy of
the content being removed, so a default that pointed at it would make
the safety of the operation depend on the operator reading a pre-filled
field before pressing Enter.

### `fn no_document_means_no_dialog`

The guard matters more here than for print: [`RedactDialog::open`] runs
the whole removal, so one built against an empty shell would be a window
that had done a full rewrite of nothing in order to refuse.

### `fn the_confirm_control_needs_every_gate_that_applies`

§3, asserted over the state machine rather than over pixels. The
interesting direction is the residual one: an operator who ticks only
the permanence box on a report with residuals must **not** be able to
commit, because the two boxes answer different questions and treating
one as both is how a partially-redacted file gets handed over as a
complete one.

It is asserted here as well as at
`crate::redact::PreparedRedaction::write_to` deliberately: this is the
drawing decision and that is the mechanism, and a test for only one of
them would leave the other free to drift.

### `fn changing_the_destination_retires_the_overwrite_acknowledgement`

The sequence this forbids is not exotic — it is *"I'll just look at what
the other option says"*: tick the box, select **a new file**, change
your mind, select **replace** again, and find the button already live
with a consent you had explicitly withdrawn in between.

It also asserts the *other* direction, which is the one a "tidying"
edit removes as pointless: arriving at [`Destination::NewFile`] must
clear it too. Retiring a tick that was not needed costs nothing;
deciding *which* changes matter is where the next edit gets it wrong.

### `fn the_outcome_sentence_says_the_open_window_is_now_stale_after_a_replace`

The strangest consequence of the replace path, and the one nothing else
on screen would say: the session was not touched, so the canvas still
shows the marks and the content underneath them, while the file those
bytes came from contains neither. *"The document you still have open is
unchanged"* is a reassurance after a copy and a falsehood after a
replace.

### `fn every_source_of_a_residual_reaches_the_disclosed_list`

One derivation for both, so a residual cannot be listed without being
acknowledgeable or acknowledged without being listed. The promotion
source is the one a tidying edit would drop, because it is the mildest —
and a report that silently drops the findings it judges harmless is one
whose judgement nobody can audit.

### `fn the_written_sentence_follows_the_residual_count`

The catalog's rule 1, at the one call site that chooses between the two.
A build that always used the clean form would produce a window saying a
file was *"verified absent"* over a report the operator had just
acknowledged as incomplete — which is worse than saying nothing, because
it contradicts the thing they read a moment earlier.

### `fn clean_session`

Built from `crate::redact`'s own fixture shape rather than from a file,
so every byte in it is one this suite put there — which is what makes
"the report has no residuals" a fact about the fixture rather than a
property of somebody's producer.

### `fn the_default_destination_writes_nothing`

The safety property the default has to carry, expressed as a property
rather than as an identity. Until this afternoon the default was
[`Destination::NewFile`], and it was safe because it never *overwrote*;
it is now [`Destination::OpenDocument`], which is safe because it never
*writes*. An operator who presses the confirm control without reading
the destination group loses nothing on disk either way, and that is the
invariant a future re-ordering of the radio buttons must not break.

It asserts through [`Destination::writes_now`] rather than by comparing
to a variant, so a fourth destination made the default would have to be
a non-writing one to pass.

### `fn the_staging_consequence_is_disclosed_before_the_operator_can_commit`

`Pass 250.2` charges no such price: the undo log survives. What is
disclosed in the same place, in the same warning role, from the same
region, is the fact that replaced it — **the page does not change**, and
the removal happens at the save. That is more surprising rather than
less, and it is the one thing about this destination an operator cannot
work out by looking.

The assertion is made over [`RedactDialog::staging_disclosure`] — which
[`RedactDialog::gates`] draws **between the destination choice and the
confirm control**, so a disclosure that exists is a disclosure the
operator passed on the way to the button.

The negative half matters as much: the two write-now destinations really
do produce a file at the click, so a sentence saying nothing is written
would be a false claim there.

### `fn a_staged_document_offers_the_control_that_calls_the_removal_off`

1. **the pipeline refuses by name.** `prepare_redaction_apply` on a
   staged session must answer [`RedactApplyRefusal::AlreadyStaged`] and
   not `FullRewriteUnavailable`. Without this the operator would be told
   *"this document cannot be rewritten in full"* — a true sentence about
   the wrong subject, because the engine declines `to_full_bytes` while a
   removal is armed;
2. **the dialog turns that into [`Phase::Staged`]**, which is the only
   phase carrying the control that unblocks him;
3. **the control pushes the cancel action and closes.** A build that
   pushed the *stage* action here would re-arm what he asked to call off,
   and one that pushed nothing would leave him with a document that
   cannot be saved by any ordinary means.

### `fn confirming_the_default_destination_pushes_an_action_and_writes_no_file`

The whole of `OPERATOR_REQUESTS.md` O125's second half, asserted at the
one method that could break it. Three things are checked and each has a
failure it is looking for:

1. **exactly one action, and it is the apply** — a build that pushed
   nothing would leave the operator with a dialog that closed and a
   document that never changed;
2. **the phase is still `Prepared`** — this route must not fabricate a
   `Written` outcome for a file that does not exist;
3. **the dialog asks to close** — the outcome is reported by the funnel's
   edit disclosure, and a window left open beside it would be a second
   account of one event.

### `fn the_overwrite_acknowledgement_is_owed_by_exactly_one_destination`

And the half that is the whole of O125: **Save-over-the-original
still warns.** It is a warning and not a refusal — the operator may do
it — but he may not do it without having said, at a control naming the
file, that he knows what it costs.

### `fn the_disclosed_list_is_the_domain_count_plus_promotion`

The other half of `crate::redact::tests::
the_residual_count_matches_the_disclosed_list_except_for_promotion`.
Two derivations exist because the deferred route has no materialisation
step of its own to observe a promotion in; this pins the difference at
exactly one item so it cannot quietly become two.
