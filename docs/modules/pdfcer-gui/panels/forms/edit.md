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

## Item notes

### `static LAST_FILL`

# Why a thread-local and not a field on `OpenDoc`

It should be a field on [`OpenDoc`], beside `edit_epoch`, dropped with
the document — and the constraint is a **boundary rather than a design
judgement**, the same one [`crate::panels::forms::FormsUi`]'s header
records for the drafts: `OpenDoc` is declared in `crate::app::state`,
which this work may not extend. Stated here rather than left for
somebody to find, so that whoever lifts the constraint knows what the
preferred shape is.

Why it is nonetheless sound rather than a smuggled mutation: this is
**not document state**. It is a note about an edit that has already
happened through the funnel, it cannot change a pixel of the page, and
nothing reads it except a panel deciding whether to draw a sentence.
It is also correctly scoped — `eframe`'s update loop is one thread, so
the writer and the reader are the same thread, and a test running on
another one gets its own empty slot rather than another test's leftovers
(which a `static Mutex` would hand it).

Staleness is handled by the `epoch` rather than by clearing: a
disclosure is shown only while it describes the revision on screen, so
an undo silences it without anything having to remember to.

### `fn label`

Separate from `Debug` on purpose: `Debug` prints the operands, which on
a `Recompute` is the whole plan and on a `FillText` is whatever the
operator typed — including into a `/Ff` `Password` field. A trace line
is written to stderr and read by whoever is diagnosing a machine they
cannot see, and neither of those belongs there.

**That is not a hypothetical.** `crate::text::forms::form_field_password_tooltip`
exists to tell an operator that a masked field is stored as plain text
in the PDF; echoing it to stderr as well would be pdfcer widening the
exposure it just warned about.

### `fn scope_of`

`OPERATOR_REQUESTS.md` O74. `None` means *"not established"* and is the
answer for everything this function has not proved, which is most of it.

# Why every branch below defaults to `None`

A `Some(page)` is a promise that no rasteriser drawing any **other** sheet
would produce a different picture. If that promise is ever false the page
rail shows the operator content he has already changed — rule 4's *sneaky*,
which outranks the slowness this exists to fix. So the shape of every arm
is "prove it or say `None`", and the four whole-form verbs never even ask.

| verb | answer | why |
|---|---|---|
| `FillText`, `ConvertRichTextToPlain`, `SetButtonState`, `SetChoice` | the field's page, if its widgets share one | one `/T`, and §12.7.3.1 lets one field have widgets on several sheets — so this is checked, not assumed |
| `Recompute` | `None` | N fields, N undo entries, no reason to think they share a page |
| `Reset` | `None` | every eligible field in the document |
| `RegenerateAppearances` | `None` | every field's `/AP`, and `/NeedAppearances` on the catalog |
| `Flatten` | `None` | burns every widget into page content and removes the form |

# Why it reads the form AFTER the edit rather than before

Because the pages that need invalidating are the ones the widgets are on
**now**. None of these verbs moves a widget between sheets, so the two
readings agree today — but "they agree today" is the kind of premise that
stops being true silently, and reading after costs the same.

# What it costs

One `parse_acroform` walk per fill. On the operator's own set that is a
small fraction of the single thumbnail it saves, and it replaces twelve.

### `struct Applied`

A struct rather than a tuple because the two travel for different reasons —
the count decides whether to invalidate the page, the disclosure decides
whether the panel says a sentence — and a `(usize, Option<_>)` at four call
sites is four chances to read the pair in the wrong order.

### `fn bound_token`

# Why this exists rather than `{:?}`

**Never `Debug`-format a field a machine reads.** `Debug` is a derived,
unstable rendering owned by another crate: a rename upstream, a
`#[derive]` change, or a variant gaining a payload all change the string
with no compile error here, and a driven check keyed on it goes quiet —
or, worse, reports the opposite of the truth while quoting the truth in
its own failure message. This project has that exact defect on the record.

⇒ Spelling the tokens here makes the trace vocabulary **this shell's**, and
makes changing it a deliberate edit next to the checks that read it.

# ⚠⚠⚠ And a worked example of the trap, committed by this very function


> *"The `match` is exhaustive and must stay that way. `AutoFitBound` is
> **not** `#[non_exhaustive]`, so a new variant upstream is a compile error
> here … Do not add a wildcard to silence a future build; the error is the
> feature."*

**Wrong.** `AutoFitBound` **is** `#[non_exhaustive]` — the attribute sits on
the line *after* the `#[derive]`, and the check that produced the claim
grepped the derive line. The compiler rejected it immediately (`E0004`), so
it cost two minutes.

It is left here because of *when* it happened: **within the hour of
writing a RAG lesson titled "`#[non_exhaustive]` removes the compile-time
guarantee, and comments keep claiming it anyway"**, after that same class
had bitten twice the same evening in unrelated modules. Knowing the rule is
not the same as checking the attribute, and *"I grepped for it"* is not
checking when the grep can miss by one line.

⇒ **Grep for the type name and read the lines above it, not for `derive`.**

## So the wildcard below is mandatory, and it returns a real token

`"other"` rather than a panic or an empty string: a bound this build has
never met is a fact worth seeing in a trace, and a check reading `bound=`
can tell *"a new upstream variant arrived"* from *"no bound was decided"*
(`bound=none`) from any of the three known ones. The operator-facing side
makes the matching choice — an unknown bound takes the general sentence,
which is true of every auto-size, rather than a claim about a constraint
this build cannot name.

### `fn run`

Split out from [`apply`] so the borrow of `doc.session` ends before the
epoch bump touches `doc`'s other fields, and so the verb dispatch is one
readable `match` uncluttered by the protocol around it.

# Why a command COUNT rather than `()`

Because two of the eight can legitimately do nothing, and "nothing
happened" must not look like "something happened":

- [`FormEdit::Recompute`] with an empty plan writes no field.
- [`FormEdit::Reset`] on a form that already holds its defaults commits a
  command, but `ResetOutcome::fields_reset` is 0.

Returning the count lets [`apply`] skip the invalidation, which is the
difference between a no-op and a no-op that discards the page raster, the
canvas selection and the Objects panel's expansion state.

**It is a count of commands, not of fields**, and the two differ on exactly
one variant: `Recompute` pushes one per change. That is the distinction
`pdfce_FeatureRequests/README.md` warns about in general terms — a number a
verb hands back is not automatically the number the caller wanted — so the
unit is named in the return type's doc rather than left to the reader.

### `fn every_form_edit_traces_under_its_own_name`

The labels are how a refusal is identified in a log from a machine
nobody can reach, and two verbs sharing one would make the log say
which pair of things might have failed.

### `fn a_trace_label_cannot_contain_a_typed_value`

The label is what reaches stderr, and `FormEdit::FillText` carries
whatever the operator typed — which, on a `/Ff` `Password` field, is a
value pdfcer has just warned them is stored in the clear. Widening that
exposure into a log would be pdfcer doing the thing it cautioned
against.

Asserted by construction rather than by inspection: a `const fn`
returning `&'static str` **cannot** interpolate a field, so the only
way this test fails is if someone changes the signature to build a
`String` — which is exactly the change that would need reviewing.

### `fn an_empty_recompute_plan_changes_nothing`

Pins the reason [`run`] returns a count at all. A plan with nothing in
it is reachable from a real click — the section recomputes its plan
every frame it is open, and a form whose calculations are already
correct produces an empty one — and treating it as a change would drop
the page texture and clear the canvas's resolved selection for nothing.

Driven through a real `EditSession` so it is the actual code path
rather than a restatement of the match arm.

### `fn a_form_verb_on_a_formless_document_is_an_error_not_a_panic`

The reachable case this guards: the panel draws Flatten, the operator
clicks it, and between the two frames an undo removed the form. Every
one of these verbs answers `EditError` for that, and the whole of
[`apply`]'s error arm is built on their doing so.

### `fn a_fill_discloses_a_substitution_and_an_ordinary_fill_says_nothing`

Driven through a real `EditSession` and a real form, because the whole
claim is about what `FillOutcome` reports rather than about what this
module remembers.

Both halves matter:

* an **ordinary** fill discloses nothing, so the panel draws no
  sentence. A disclosure line under every edit would train the operator
  to stop reading the ones that matter — the same argument
  `crate::app::status::page_box`'s `Note` makes for having no `Ok`
  variant;
* a fill of text the field's font **cannot encode** discloses the
  substitution. That is the one this exists for: the saved value IS the
  substituted one, so re-reading the field afterwards reports what pdfcer
  wrote and never that it wrote something else.

### `fn a_disclosure_is_hidden_once_the_document_moves_past_it`

The staleness rule, which is what lets an undo silence the sentence with
nothing anywhere having to remember to clear it. Verified by driving as
well — an unrelated check-box toggle made a live auto-size line
disappear — and pinned here because the epoch comparison is the whole
mechanism.
