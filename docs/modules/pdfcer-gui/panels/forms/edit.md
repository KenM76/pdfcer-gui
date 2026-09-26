# `panels::forms::edit` — the eight things a Forms panel can ask for

One enum and one function. The enum is the complete vocabulary of what
filling a form means in this build; the function is the only place any of
it reaches an [`EditSession`].

## Why one [`crate::app::actions::Action`] variant and not eight

[`crate::app::actions`]' header claims four properties for the action
funnel, and the fourth — *"every state change is greppable"* — is the one
that decides the shape here. It survives a nested enum intact:
`FormEdit::Flatten` is exactly as greppable as `Action::FlattenForm` would
be, and `grep FormEdit::` answers "what can change a form?" completely.

What a nested enum buys is that the **panel owns its own vocabulary**.
Eight flat variants would put eight form-shaped concepts —
fully-qualified names, on-state names, a recompute plan — into a module
whose other variants are zooms and page steps, and every one of them would
need an arm in `PdfcerApp::apply` that reached back into this module for the
verb anyway. So the seam is drawn where the knowledge is: `Action::Form`
carries the intent across the funnel, and [`apply`] is what knows how to
honour it.

## The four-step mutation protocol, and why it is repeated here

[`crate::app::actions::vector_edit`] is the same protocol for the vector
verbs, and this is deliberately **not** a call into it. Two reasons, and
the first is decisive:

1. **The signatures do not unify.** Every vector verb returns
   `Result<Vec<String>, EditError>` — a disclosure list. The form verbs
   return six different outcome types (`FillOutcome`, `ResetOutcome`,
   `FlattenOutcome`, `RegenOutcome`, `()`, and a `Vec` of fills), none of
   which is a `Vec<String>`. A shared helper would need a type parameter
   per verb and a closure per call, which is the same code with a generic
   bolted on.
2. **`vector_edit` is private** to `crate::app::actions`, and this module
   may not edit that file this round.

So the protocol is restated, and stated in full, because **each of the four
steps is a separate way to end up with an edit that is silently declined or
a page that silently keeps drawing what was just changed**:

1. **Stop the render worker.** `OpenDoc::session` is an `Arc` precisely so
   a worker can hold a clone while it rasterizes, and `Arc::get_mut` fails
   while any other strong reference exists. Cancelling first is what turns
   "sometimes refused, depending on how fast the page rasterized" into
   "always applied".
2. **Mutate through `Arc::get_mut`.** A `None` is not a panic: it means
   something else still holds the session, which is a bug in the caller's
   ordering rather than in the operator's document. Traced and declined,
   because declining an edit is recoverable and corrupting one is not.
3. **Bump `edit_epoch`.** Filling a field rewrites its widget's appearance
   stream, which is page content, so the canvas's decomposition and the
   Objects panel's paint-order indices are stale. It is also what
   `crate::panels::PanelsState::sync` keys on.
4. **Drop the cached texture.** Nothing else notices an edit: the render
   key compares page index and raster scale, and a fill changes neither.
   Without this the page keeps showing the empty box until the operator
   zooms or pages away.

## Almost nothing travels back — and the exception is the interesting part

### The rule

The old shell wrote an operator-facing note into `doc.pending_note` after
every one of these verbs: *"Filled X"*, *"X changed, but this document has
no drawn appearance for that state"*, *"saved, but this form also carries
an XFA packet"*. This build has no such channel, and the panel does not ask
for one, because **every fact those notes carried is derivable from the
document the panel re-reads on the next frame**:

| Old note | Where it is now |
|---|---|
| "Filled X" | The row shows the new value. |
| XFA may disagree | [`crate::text::forms::forms_xfa_note`], stated **before** anything is typed, because `AcroForm::xfa` is a property of the file. |
| no appearance for that state | The control is disabled up front — the state pdfcer would write does not exist, so the call would refuse (see [`crate::panels::forms::rows`]). |
| "Reset N fields" | The reset preview recomputes and lists nothing left to clear. |
| "Recomputed N fields" | The plan recomputes and reports every calculation already correct. |
| "Flattened N fields" | There is no longer an `/AcroForm`, and the panel says so. |

This is `crate::panels::layers`' lesson one surface over: that panel
computes "how far has the operator diverged?" by **comparing sets** rather
than by counting clicks, and its own docs record that counting clicks was
subtly wrong. Deriving from the document is both simpler and more correct
than carrying a note, because a note can outlive the fact it describes and
a derivation cannot.

### The exception: two facts a fill knows and the document does not

The argument above has one precondition — *the fact is re-derivable from the
document next frame* — and exactly two things `FillOutcome` reports fail it:

| fact | why it is not re-derivable |
|---|---|
| `applied_autosize` | the field's `/DA` asked for size 0 and **pdfcer chose a number**. What lands in the file is the chosen size; the fact that pdfcer chose it, rather than the document stating it, is gone the moment the command returns. |
| `unencodable_chars` | characters with no `WinAnsi` code were **silently replaced with `?`**. The saved value is the substituted one, so re-reading the field tells you what pdfcer wrote and never that it wrote something else. |

Both are inferences pdfcer made on the operator's behalf, and rule 4's whole
subject is inferences: *"pdfcer inferred something, and another reader may do
otherwise"*. So both are captured — see [`FillDisclosure`] — and shown
**off-canvas**, in the panel, where rule 4 says a disclosure belongs.

The four remaining `FillOutcome` fields stay discarded, and each for the
reason above: `widgets_updated` is `Field::widgets.len()`, `top_index` is
`/TI` and is in the file, `xfa_may_disagree` is `AcroForm::xfa` and is
stated **before** anything is typed, and `field_id` is the field the panel
is already looking at.

**What is left over is a refusal**, and it is traced rather than surfaced —
the same posture, and the same acknowledged gap, as
`crate::app::actions::vector_edit`, whose own header names it. That is
defensible here for a reason it is not there: every refusal these verbs can
raise is **asked about before the control is drawn**
(`EditSession::fill_refusal`, `EditSession::deletion_refusal`, the per-row
block reasons), so reaching the trace at all means a precondition changed
between the frame that drew the control and the frame that applied it. See
this module's `KNOWN GAPS` section below for the two cases where that is
genuinely reachable.

## KNOWN GAPS — reported, not worked around

Both are `pdfcer-core` boundary findings rather than shell defects, recorded
here because `pdfce_FeatureRequests/README.md`'s decision 058 says a
workaround that is not reported is a boundary defect that stays.

1. **`EditSession::fill_refusal` is a strict subset of what a fill
   enforces.** The verbs call the private `fill_guards`, which checks
   `/Encrypt`, the `/P`-aware certification gate **and** the suppressed-
   object guard; `fill_refusal` checks only the middle one. So it can
   answer `None` on a document where the fill then returns
   `DocumentEncrypted` or `ObjectCreationWouldExposeHiddenObjects`. The
   encryption arm is unreachable from this shell — an encrypted document is
   refused at open, see `crate::text::open_needs_password` — and the
   suppressed-object arm has no public accessor at all, so the panel cannot
   compensate for it. `fill_refusal` should mirror `fill_guards`.
2. **RESOLVED — `EditSession::flatten_refusal` now exists** (pdfcer
   `fa243df`), and `panels::forms::mod` asks it. The gap was real:
   flatten shares deletion's strict certification gate but additionally
   creates page content, so it carries a suppression guard deletion does
   not — two checks of three, which works until it does not.

   The half of the same report that claimed `deletion_refusal`
   under-reported was **rejected, and rightly**: it predicts DELETION and
   matches `deletion_preflight` exactly. The comparison had been against
   flatten. Acting on it would have disabled a Delete control that would
   have worked, and core now carries a test whose job is to stop a future
   reader "correcting" a correct function on the strength of it.
