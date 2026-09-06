# EDITABLE_SURFACES.md — every verb `pdfcer-core` implements, and where the operator reaches it

**Written 2026-08-28, in answer to a question this project could not answer from
its own documents:**

> *"confirm that you have built every editable surface into the GUI that has
> been implemented in pdfcer"*

`FEATURES.md` says what the GUI does. `NO_SURFACE.md` lists compiled-in values
with no control. `GUI_ROADMAP.md` says what is planned. **None of the three is
keyed on the engine's verb list**, so none of them could answer *"is there a
verb `pdfcer-core` implements that nothing in this shell calls?"*

The answer was **yes, twelve times**, and the pattern in the misses matters more
than the count: three were capabilities the engine had shipped **in answer to
this shell's own requests**, which this shell then never consumed, and **two
were settings the operator could change that were honoured by nothing**.

⇒ *A reply arriving is not a capability landing.* The engine session runs in
parallel and answers within the hour; its answers sat unread while this
project's own doc comments still recorded the capability as blocked.

---

## ★★★ The instrument, and why this file is not a hand-written list

`tools/verb-coverage.py`. It parses `impl EditSession` out of
`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs`, takes every `pub fn` declared in
it, and greps `crates/pdfcer-gui/src` for each name.

```
python tools/verb-coverage.py            # the misses, one per line
python tools/verb-coverage.py --all      # every verb with its occurrence count
```

**Re-run it before quoting any number in this file.** The engine moves daily —
five of the rows below were written on the day the verb behind them shipped —
and a register that is trusted rather than re-measured becomes the seventh
stale blocker in a project that has already found six.

### What the measurement is worth, stated plainly

- **A hit means the NAME appears**, not that a reachable operator route calls
  it. A call site behind a condition nothing sets is a hit here and dead in the
  running program. Only `tools/ui-verify` answers that question.
- **A miss is stronger**: no occurrence of the identifier means nothing here
  calls it, full stop.
- **A miss is not automatically a gap.** Roughly half are session queries or
  alternate spellings of a verb the shell calls in another form. That is what
  the table below is for: **every miss owes a reason**, and a reason that is
  merely *"not built"* is a reason to go and look.

---

## The state at the end of 2026-08-29 (re-measured after the preview work)

**157 `EditSession` verbs. 147 named somewhere in the shell. 10 named nowhere.**
At the start of the audit it was 135 / 22.

★★★ **Measured against the LOCKED revision, not the engine's working tree**, and
the distinction earned itself within a day. The first cut of the tool read
`edit.rs` off disk and reported `move_outline_item` and `set_outline_open` as
gaps — bookmark reorder, re-parent and open state, which the engine's own note
had said did *not* ship. They were **uncommitted work in the engine session's
worktree**, which that project edits continuously while this one runs. (It also
reported 159 verbs and 12 misses for the same reason; the numbers above are the
lock's.)

⇒ A verb in the worktree and not in the lock **is not callable from here**, and
a register listing it would send the next session to write a call that does not
compile while looking like a capability we were behind on. The tool prints those
under `COMING` and keeps the two facts apart: *"nothing here calls it"* and
*"we could not call it if we wanted to."* Both are worth knowing; they are not
the same thing.

★ `move_outline_item` and `set_outline_open` are the two to pick up the moment
the engine commits them — they are bookmark **reorder, re-parent and open
state**, which is the half the Bookmarks panel is still missing and which that
panel's own doc comment currently records as unbuilt on the engine's side.

The twelve gaps this audit found, and what happened to each:

| Verb | Engine Pass | Status |
|---|---|---|
| `set_markup_note` / `clear_markup_note` | 154.0 | ✅ the Comments panel writes notes |
| `add_markup_with` (opacity) | 81.1 | ✅ Markup ▸ Style ▸ Opacity, one undo entry |
| `set_outline_title` / `delete_outline_item` | 156.0 | ✅ Bookmarks rename and remove |
| `set_quad_point_order` | — | ✅ the fourth settings funnel — **it was a live defect** |
| `delete_pages_with` | — | ✅ the separation policy reaches the delete — **also a live defect** |
| `rotate_annotation` | 155.0 | ✅ a ninth grip on the selection box |
| `rotate_dimension` | 159.0 | ✅ the same grip, routed by kind |
| `attach_file` / `detach_file` | — | ✅ Edit ▸ Insert ▸ Attachments, with extraction |
| `unshare_form` | — | ✅ *"Give this page its own copy"*, seven worded refusals |
| `delete_field_group` / `field_group_deletion_preview` | — | ✅ Forms ▸ Field groups, previewed before the press |
| `signature_impact_of_save` / `changes_structure` | — | ✅ a window before an invalidating save, a note after a preserved one |
| `copy_annotations` | 120.x | ⬜ **open** — asked of the engine; the interim loss is closed |

**Seven driven checks were written for this work and none has run.** A wired
verb is not a verified one; see the caveat at the top.

### ★★★ The two that were live defects rather than missing features

**`set_quad_point_order`.** `Settings::quad_point_order` was parsed, defaulted,
validated, persisted, drawn in the Settings window — and honoured by nothing,
because every session was opened with `EditSession::new(doc)`, which takes the
engine's default.

⇒ ★★ **The lesson is about the shape of the guard, not the field.**
`app::settings` exists precisely to prevent this class and a `syn` check
enforces it — and both were built around **option constructors**. A setting
delivered by a **setter on the session** is invisible to that shape, and the
check reported green for the whole life of the shell. `Settings::separations`
was the same defect one file along: chosen by the operator, reported in the
disclosure after a page delete, and never passed to the verb that would act on
it.

The fix is a fourth funnel (`SettingsExt::open_session`) with `EditSession::new`
on the check's forbidden list. **A guard shaped around one delivery mechanism
cannot see a second one, and the way to find the second is to ask what the
engine offers rather than to re-read the guard.**

### ★★ And one defect the audit did not find, which is worth saying

`Ctrl+S` **saved the file and then panicked the application** — every time,
since 2026-08-20, in the shipped build. It was found by an agent wiring the
signature guard into that arm, not by this register and not by any of the 105
driven checks. `DEFECTS.md` D16 carries the class.

⇒ A verb-coverage sweep answers *"is every capability reachable?"* It does not
answer *"does the route work?"*, and the two questions need different
instruments. This one is cheap; the other is `tools/ui-verify` and it needs the
operator's machine.

---

## The misses, each with its reason

**Currently 41, at lock `03f6004` (2026-09-05, and read the next sentence
before comparing it with anything).** ⚠ **That number is not comparable with the
12 this line carried at lock `04f7ec0`**, and not because the engine grew by
twenty-nine verbs: `tools/verb-coverage.py` was tightened on 2026-09-05 to count
only **call-shaped** matches with comments blanked, so verbs that had always been
uncalled stopped being scored as consumed by prose about them. Roughly half the
jump is the instrument getting better eyesight rather than the shell falling
behind — the section at the foot of this file accounts for the twenty-six that
had no row at all. This heading used to carry the
number — *"The 13 remaining misses"* — and the number went stale twice while the
prose under it stayed true, which is the failure this file warns about in its
own opening: **re-run the instrument before quoting a count.** So the count now
lives in one line that says which lock it was measured at, and the heading does
not carry it.

★ **There are more rows below than there are misses, on purpose.** A row is
written when a verb becomes a miss and it is *not deleted when the verb stops
being one* — `copy_attachment` and `paste_attachment` were wired the day their
row was written and both rows are still here, marked SHIPPED. The reason is that
the argument is the valuable part: a row saying *why* a verb was left alone for
a fortnight is what stops the next session re-deriving the same conclusion, or
worse, reversing it without knowing one was ever reached. The gate only ever
asks whether a name is **present**; it does not ask that the section be a
snapshot, and it should not.

### Not gaps — session queries the shell has no use for

| Verb | Why nothing calls it |
|---|---|
| `into_document` | Consumes the session to get the base document back. This shell's session lives as long as the tab does. |
| `authored_source` | A `base ++ staging` memcpy — ~14 MB per call on the benchmark document. Its own doc comment says it is for `pageops` callers that serialise a whole file anyway and is *"completely unacceptable on a render loop"*. |
| `dirty_set` | What the writer would emit as an incremental update. The shell never needs to know before saving; the writer asks it. |
| `dimension_rects` | Hit-testing ce dimensions from their `/Rect`s. This shell hit-tests through the **decomposition**, which resolves the object under the pointer for every kind at once. Two hit tests would be two answers to one question. |

### ★★★ The five the GATE found, 2026-09-01 — three cut verbs and the attachment clipboard

`tools/gates/check-verb-coverage.sh` is new on this date, and it exists because
`set_button_action` shipped on 2026-08-30 and was consumed on 2026-09-01: the
instrument existed and nobody ran it. **A tool that must be remembered is a tool
that will be forgotten.** The gate now fails the build when a verb is named
neither in the shell nor in this file — and on its first honest run it named
these five, all of which had been silently uncovered.

| Verb | Why nothing calls it |
|---|---|
| `cut_objects` (context) / `cut_annotations` | ⛔ **Deliberate, argued in code.** `canvas::clipboard::cut` is copy-then-`DeleteSelection`, and the reason is in its own comment: routing the delete through the shell's funnel means it lands one `EditSession` command and one undo entry *by the same mechanism as every other edit*, and it leaves `canvas::clipboard` changing no document — which is what lets its refusals be unit-tested without one. The engine's ordering property (copy first, so a selection that cannot be carried is refused with nothing deleted) is honoured by the `?` on the copy. |
| `cut_field` | ◑ **A workaround, and it costs one undo entry too many.** `canvas::fieldclip::cut` is copy + `FieldAction::DeleteWidget`, and `DeleteWidget` is deliberate rather than incidental: the operator pointed at **a box**, and `cut_field` removes the whole *field* — on a field with three widgets that is not what was asked. So the two verbs are genuinely different acts. What is *not* deliberate is the undo cost, and it has the same cause as the button-action one filed the same day: `EditSession::coalesce_last` is private, so a shell cannot fold two commands into one. See `request_placing_a_button_with_an_action_costs_two_undos.md`. |
| `copy_attachment` / `paste_attachment` | ✅ **SHIPPED 2026-09-01, the same day this row was written.** Copy and Cut on every document-level row, Paste at the top of the panel — drawn only when the clipboard holds one, per R9. Driven by `an_attachment_moves_between_two_open_documents`, which crosses a document boundary deliberately: a same-document round trip exercises every line and does not test the defect. |
| `cut_attachment` | ⛔ **Deliberate, and the same argument `cut_objects` gets.** The panel's Cut is `copy_attachment` in the widget followed by an `AttachmentAction::Detach` through the funnel. The copy half is `&self` and commits nothing, so that is **one** `EditSession` command and one `Ctrl+Z` — the fold `cut_attachment` performs with the private `coalesce_last` buys nothing here. The widget route also lets the copy **fail before the delete is raised**, which is the ordering `cut_objects`' own doc comment insists on. |

⇒ ★★ Note what the three attachment verbs have in common with the button
action: **the engine shipped them and this shell said nothing about them
either way.** That is precisely the silence the gate exists to break — a verb
nobody has written a sentence about is indistinguishable from a verb nobody
noticed.

### ★★★ The two the GATE found, 2026-09-04 — encryption AUTHORING landed, and this is the fourth time

`cargo update -p pdfcer-core` moved the lock from `e27c3b4` to `04f7ec0` and the
gate went red on the next run with two names:

> **191 `EditSession` verbs (lock `04f7ec0`), 179 named somewhere in the shell,
> 12 named nowhere** — of which ten already had a row in this file, and
> `set_encryption` and `set_permissions` had nothing anywhere.

★★★ **This is O108's own request coming back answered, and nobody would have
noticed.** On 2026-09-03 the security audit measured `pdfcer-core` from its own
side and reported the finding that changed the shape of the ask: *"Every one of
them is READ-SIDE. `pdfcer-core` has **no** `encrypt_document`, no
`set_password`, no `remove_encryption`, **no `set_permissions`**, no
`sign_document`."* A Security tab was therefore scoped as an **information**
tab, and the authoring half was filed at the engine as
`request_a_document_cannot_be_encrypted_or_have_its_permissions_set.md`.

The engine's reply
(`reply_signature_integrity_first_then_encryption_and_your_two_sentences.md`)
ranked it second of three and closed with one line: *"Encryption authoring
(`Pass 5.4`) is next in the queue; nothing you need to change for it yet."*
**It is no longer next in the queue. It is in the lock.** And unlike the
signature-validation half, which arrived with a `★ UPDATE, same day` written
into the reply naming the entry point, this one landed with **no reply file, no
note, and no announcement** — the capability simply appeared in `edit.rs` under
`// -- encryption authoring (Pass 5.4, ISO 32000-2:2020 §7.6) --`.

⇒ ★★ So the count is now **four**: `set_button_action` (two days), the three
attachment verbs (silent), and this. Every one of them is a capability the
engine shipped *because this shell asked for it*, and every one of them sat
unconsumed because **an addition on the other side of a boundary is silent by
construction**. The gate is the only thing in this project that made a noise
this time, and it is the only thing that was keyed on the engine's API.

| Verb | Why nothing calls it — and note the reason this is NOT "should not call" |
|---|---|
| `set_encryption` | ⏸ **Real, unwired, and AWAITING AN OPERATOR RULING ON SCOPE.** It is a save transform, not an undoable edit: `(&EncryptionSettings, &SaveOptions) -> (Vec<u8>, SaveReport)`, writing AES-256 `/R` 6 and nothing else, refusing `AlreadyEncrypted` and `SignedDocument` by name. Wiring it means authoring a password dialog, a permission-bit chooser and a save path that is a full rewrite — which is a **new surface**, not a call site, and O108's tab was deliberately scoped read-side when the engine had no authoring half. That scope decision is the operator's and it is now stale. Surfaced to him as **O119**. |
| `set_permissions` | ⏸ **The same ruling, and it carries a precondition the shell must show first.** `(&mut self, &EncryptionSettings, &SaveOptions)` re-keys an already-encrypted document — `/P` is bound into `/Perms` by Algorithm 10 and cannot be edited in place, so it is a fresh full encrypt under a fresh file key. It is therefore **owner-only**, refusing `NotOwner { opened_as: AuthKind }`. Surfaced as **O119** with `set_encryption`, because they are one operator question. |

★★★ **The wording of those two rows is the point of this whole gate.** *"A verb
this shell should not call"* and *"a capability that landed, is real, and is
waiting on a decision that is not mine to make"* are different sentences, and
the gate's own message says the difference is the entire mechanism: silence
reads as the first when it is very often the second. These two are the second.
Neither row is a refusal, neither is a deferral on technical grounds, and
neither should be read as this project having declined encryption authoring —
it has declined nothing. It has **asked**.

#### ★★ And a false hit beside them: `remove_encryption`

Not in the gate's list, and it should have been. `remove_encryption` shipped in
the same `Pass 5.4` block, is the third verb of the same family, and is called
**nowhere** — `python tools/verb-coverage.py --all` scores it `1`, and that one
occurrence is a **doc comment** in `text::security::auth_line` quoting the
engine's instruction to surface `AuthKind` *"because `remove_encryption` will
refuse a user-authenticated session"*.

⇒ ★★★ **This is exactly the defect O108 recorded in the other instrument, one
tool along.** `tools/security-coverage.py` reported `load_with_password` as
*reached* on the strength of a single sentence in a doc comment — *"which would
have recorded the single most important missing capability in this whole area as
already built"* — and was fixed by stripping comment-only lines before
searching. `tools/verb-coverage.py` **has not had that fix**, so it is blind in
precisely the way its sibling was, and the blindness is worst on exactly the
verbs this register talks about most: a verb argued about in prose here or in a
doc comment there scores a hit and leaves the gate.

★ It is not fixed in this pass **deliberately**, because tightening the
instrument changes what the gate reports and that is a change to make on its own
and measure on its own, not as a rider on a documentation entry. It is named
here so the next session does not rediscover it, and `remove_encryption` is
named here so that when the instrument *is* fixed, this row is already written
and the gate does not go red for a verb that was known about all along.

### ★★★ The four the GATE found, 2026-09-05 — the undo-preserving redaction, and all four are WIRED

The lead bumped the lock to **`pdfcer-core` v0.38.0 (`b01964f`)** and the gate
went red on the next run with four names, all of one family:

> **197 `EditSession` verbs (lock `b01964f`), 184 named somewhere in the shell,
> 13 named nowhere** — of which nine already had a row in this file, and
> `apply_redactions_deferred`, `cancel_pending_redaction`,
> `has_pending_redaction` and `save_applying_redaction` had nothing anywhere.

**All four are now called**, so none of them needs a row here to keep the gate
green. They get one anyway, because the *arrangement* is the part worth
recording and because one of them is the second half of a request this project
filed — `Pass 250.2`, `41095eb`, the undo-preserving variant our own
`request_apply_redactions_into_the_session.md` asked for in the first place.

| Verb | Where it is called, and why exactly there |
|---|---|
| `apply_redactions_deferred` | `crate::redact::stage_into_session`, and **nowhere else** — pinned by `redact::sealed`. It arms the removal and touches nothing: base, overlay and the whole undo/redo stack survive. |
| `has_pending_redaction` | Four places, and they are four different questions rather than one repeated: `app::save::write_copy` (which writer runs), `app::save::has_unsaved_edits` (is this document dirty), `redact::prepare_redaction_apply` (refuse the second report by name), `redact::stage_into_session` (refuse the second arming). It is a query, so it is deliberately **not** in the monopoly's table. |
| `save_applying_redaction` | `crate::redact::save_applying_pending`, and **nowhere else** — pinned. It is the only save that succeeds while a removal is armed, and it takes `&self`, which is what makes undo survive the save. |
| `cancel_pending_redaction` | `crate::redact::cancel_staged_redaction`, and **nowhere else** — pinned, even though it removes nothing. It *disarms* a removal, which is the same surface seen from behind. |

#### ★★★ …and one verb that STOPPED being called: `has_applied_redaction`

The movement worth recording, because it is the shape this register exists for
and the gate cannot see it.

`has_applied_redaction` is `Pass 250.1`'s disclosure signal — *"a redaction has
been collapsed into this session"* — and on 2026-09-04 it was a live term of
`app::save::has_unsaved_edits`. On 2026-09-05 it stopped being one, because this
shell stopped collapsing: `Pass 250.2`'s staging replaced `Pass 250.1`'s
finalizing route entirely rather than joining it, so the flag that verb reads is
**`false` for the life of every session this shell now creates.** A term that
can never be true is worse than an absent one, because it reads as a guard.

★ **It is still called, and the call is an assertion rather than a use.**
`app::save::tests` and `redact::tests` both assert `!has_applied_redaction()`,
and that is deliberate: it is the tripwire for a build in which something has
reached the engine's *other* apply verb by a route `redact::sealed`'s call
counts did not see. If that assertion ever goes red, the monopoly is broken
somewhere the syntax sweep cannot reach.

⇒ ★ **The gate reads it as `named`, and that is the `remove_encryption`
blindness two sections above, from the other direction.** A verb whose only
remaining call sites are negative assertions in a test suite scores a hit on
`tools/verb-coverage.py` exactly as a verb mentioned once in a doc comment does.
The instrument is not wrong to report it; the row is here so that the day
somebody tightens the instrument, the reason is already written down and the
gate does not go red for a verb that was known about all along.

#### ⚠ AND ONE HAZARD THIS PASS INTRODUCED, NAMED RATHER THAN IMPLIED

**`EditSession::set_encryption` does not consult the pending-redaction flag, and
`crate::protect::prepare` calls it on the OPEN session.**

`crates/pdfcer-gui/src/protect/mod.rs:675` — `Job::SetPassword` calls
`doc.session.set_encryption(&settings, &options)`, which serialises base plus
the dirty set through the encrypting encoder. The engine guards
`to_incremental_bytes` (`edit.rs:8348`) and `to_full_bytes` (`edit.rs:8374`)
against a pending redaction and **does not guard this one**. So an operator who
arms a removal and then uses File ▸ Security ▸ `Encrypt…` gets an encrypted file
containing the un-redacted content *and* the `/Redact` marks — a marked file
that looks finished, which is the single most-cited real-world redaction
failure.

★ **It was not reachable before this pass.** Under `Pass 250.1`'s collapse the
content was already gone by the time the operator could reach the Security
window, so this is a hazard the deferred route brings with it.

★ **It was not fixed in this pass, and the reason is scope rather than
judgement.** The guard belongs as a `protect::Refusal` variant, which is
`protect/mod.rs`, `dialogs/protect.rs`, `text::security` and two test files —
four files belonging to no track that was running, edited in a session that owned
neither. It is written here instead of being fixed quietly, because a boundary
defect that is not reported is one that stays. **The honest fix is the engine's**:
`set_encryption` should return `RedactionPending` the way its two siblings do,
and the shell's guard is the belt to that braces.

> ### ✅ THE ENGINE FIXED IT — `Pass 250.3`, released in v0.39.0, 2026-09-05
>
> Filed at 02:23 and answered the same day. `set_encryption` now refuses a
> pending redaction by name, exactly as `to_incremental_bytes` and
> `to_full_bytes` do.
>
> ⚠ **NOT yet in our lock**, and everything above is therefore still true of the
> build this repository compiles today. It becomes false the moment the bump
> lands — `RESUME.md` carries it as the first queued job.
>
> ★★ **Annotated rather than deleted, and annotated NOW rather than when the
> bump lands**, because a limitation sentence is a citation with an hours-long
> shelf life and this project has been caught by exactly this class three times
> in two days: prose that was true when written, describing an engine that has
> since moved, read later as a statement about the present. The paragraph above
> is a **dated measurement**; this note is what stops it being read as a
> standing fact.
>
> ★ The shell-side `protect::Refusal` variant it argues for is **still worth
> building** — the engine's refusal is the braces and ours is the belt, and the
> operator meets ours with a sentence in his own terms rather than an error
> code. It is no longer urgent, and it is no longer the only thing standing
> between him and an encrypted file full of un-redacted content.


### Not gaps — alternate spellings of a verb the shell already calls

| Verb | What the shell calls instead |
|---|---|
| `rotate_page_by` | `rotate_pages(&[…], delta)` — the same act for a set rather than for one page, which is what the Pages panel's selection is. |
| `search_and_mark_redactions` | `search_and_mark_redactions_styled` — the styled variant, because a redaction mark whose appearance the operator cannot choose is a mark they cannot see against their own drawing. |
| `mark_redactions_by_pattern` | `mark_redactions_by_pattern_styled`, for the same reason. |
| `copy_annotations` | ⚠ **This one is a real fidelity gap and is listed again below.** The shell calls `copy_objects`, which does not carry annotations, and round-trips a copied markup through `MarkupSpec` instead. |
| `move_annotation_vertex` | `reshape_annotation(id, VertexEdit::Move { .. }, modified)` — and the third argument is the whole reason. The three wrappers are one line each and pass `modified: None`, so **none of them can stamp `/M`**: the engine reads no clock on purpose (determinism — the same edit on the same file produces the same bytes) and says the shell that knows the time supplies it. A reviewer's comment whose shape changed and whose modification date did not is a comment that lies about when it was last touched, so `app::actions::annots::reshape` calls the planner directly with `app::clock::pdf_date_utc()`. ★ `AnnotationReshape::mod_date_written` reports whether the stamp landed, and it is in the trace. |
| `insert_annotation_vertex` | `reshape_annotation(id, VertexEdit::Insert { .. }, modified)`, for the reason above. |
| `remove_annotation_vertex` | `reshape_annotation(id, VertexEdit::Remove { .. }, modified)`, for the reason above. |

### ★★ The preview and refusal queries — six closed 2026-08-29, one declined

These are `&self`, side-effect-free, and **share one body with the verb they
describe**, so `preview(..).is_ok()` *is* the predicate rather than a second
implementation that agrees until somebody changes one.

| Verb | Status |
|---|---|
| `rename_refusal` / `deletion_refusal` | ✅ **Wired 2026-08-28** — the Forms properties pane withholds Rename and both Deletes and puts a sentence in their place |
| `field_group_deletion_preview` | ✅ Wired — Forms ▸ Field groups, previewed before the press |
| `signature_impact_of_save` / `changes_structure` | ✅ Wired — a window before an invalidating save |
| `annotation_deletion_refusal` | ✅ **Wired 2026-08-29** — `format.delete` is *not drawn* on a certified or encrypted document, and the Properties panel says why |
| `annotation_deletion_preview` | ✅ **Wired 2026-08-29** — the collateral of a delete, stated before the press, memoised on `(id, epoch)` |
| `preview_style_resolution` | ✅ **Wired 2026-08-29** — Bold and Italic say which face they will use, or that the press will be refused |
| `paste_preview` | ⛔ **Declined, with the argument below.** Not "not built" |

⇒ Each is R9 and R83 quality work rather than a missing capability: the verb
runs either way, and the difference is whether the operator learns the answer
before the gesture or from a refusal after it.

#### ★★★ `annotation_deletion_refusal` — the forms defect, one `/Subtype` along

The 2026-08-28 audit found that **`deletion_refusal` (the forms one) was
consulted by nothing**: it appeared in this crate only inside three comments in
`panels::forms`, arguing correctly about which query *Flatten* should ask, while
Rename, Delete field and Delete this box asked none.

**The annotation half was the same defect and was still open.** On a certified
or encrypted drawing this shell drew *three* live Delete controls — the Format
tab's, the canvas object menu's, and the Delete key — and every press reached
`delete_annotation`, was refused, and landed in `actions::apply::vector_edit`'s
`Err` arm, **which writes one line to the trace and says nothing to the
operator**.

⇒ ★★★ **And it was worse than a silence.** `actions::annots::delete` clears the
selection *after* the funnel rather than on success, so the press removed the
Properties panel's own description of the annotation — the surface where the
explanation lives. A refused gesture that destroys its own explanation is the
worst shape this class can take, and no unit test in the crate could have found
it, because every one of its halves is separately correct.

**The fix**, following the forms pattern rather than inventing a second one:

* `selection.delete_permitted`, published by `app::conditions`, carries
  `format.delete`'s **`visible_when`** on the Format tab and on the canvas
  object menu — *absence*, not greying, because a certification signature is
  neither temporary nor arguable (R9);
* `panels::properties::annotdelete` draws the **sentence** that replaces it,
  from `annotdelete::gate` — the *one* derivation the condition also asks, so a
  control cannot be withheld for one reason while a panel explains another;
* `canvas::keys` and `dispatch::format` consult the same gate before raising
  the action, so the selection survives the press and the sentence stays on
  screen.

★★ **The sentence is in a panel and deliberately not in `status::decline`.** A
decline reports *a gesture just failed* and must be repeatable; this is a
**standing property of the open document**, true from the moment it was opened
and whether or not anything was pressed. Delivering it only after a press is the
one moment R83 exists to get ahead of.

#### ★ `annotation_deletion_preview` — and what was done about its cost

The query walks the page's whole `/Annots` looking for `/IRT` referrers —
O(annotations) per call. **The old shell gated it on hover, one row at a time**,
because its Comments panel would otherwise have paid O(rows × walk) per frame.

This shell does better, and the reason it can is structural rather than clever:
**it is not a list.** There is one selected annotation, so the worst case is one
call per frame — and even that is not paid, because the answer is memoised on
`(annotation id, edit epoch)`. In steady state the cost is one `Option`
comparison per frame and no engine call at all. A hover gate would have been
cheaper only in the frames where the answer is not wanted, and it would have hidden
the fact behind a gesture the operator has no reason to make.

#### ★★ `preview_style_resolution` — and a claim of this project's that it retires

Bold and Italic are *"buttons that apply, not switches that reflect"* and
**still do not grey**; the engine's instruction is unchanged. What changed is
the hover, which used to hand the operator a conditional (*"if this page carries
a real bold face…"*) and now evaluates it: which face will be used, or that the
letters will be thickened, or — the interesting one — that **the press will be
refused**.

That third answer is `app::actions::textstyle`'s own retracted claim, previewed.
`gate_synthesis` prefers a face by *family* and gates synthesis off; the face it
names may map none of the run's characters, and `set_font` then refuses it too.
Neither verb reaches bold. The shell can now predict it by comparing
`preview_style_resolution`'s `selector` against `preview_font_resources`'
accepted list — **two engine-issued selectors, compared for equality**, not the
coverage test re-implemented.

⇒ ★★ That retires this project's own note that *"greying would mean predicting a
refusal that depends on a per-run glyph-coverage test this shell cannot run"*.
It can. The buttons still do not grey, and the reason has changed from *we
cannot know* to *knowing is not a reason to withhold*: the engine has a queued
fix that turns this case into ordinary synthesis, and **a control withheld on
the strength of a defect that is about to be fixed is a control that stays
withheld for months after it starts working**. A stale sentence is corrected in
one line.

#### ⛔ `paste_preview` — declined, and this is the argument rather than a gap

Not "not built". Three reasons, and the first is decisive on its own.

1. **There is a recorded decision against it**, on `edit.paste`'s registration:
   *"`enabled_when("doc.pages")` rather than a selection condition: what is
   selected changes every click, and a control that greys and un-greys under the
   pointer is harder to aim at than one that answers in a sentence when
   pressed."* The engine's case for the verb is quoted from **the requesting
   shell**, which wanted to grey a menu item; this shell decided not to have one.
2. **The cost is per frame and real.** The clip lives in `egui::Memory` as a
   `Vec<u8>`, and `read()` clones it out. A condition backed by `paste_preview`
   would clone that vector *and* run `ObjectClip::from_bytes` — magic check,
   version, a length-prefixed COS parse — on every frame the ribbon draws, to
   produce an answer the recorded decision says must arrive as a sentence on
   press. That is precisely the shape the decision declined.
3. **The answer is already delivered**, on the path that has it: `paste_objects`
   returns the same `PasteOutcome` and its disclosures reach the status bar.

⇒ ★★ What *is* worth doing here is a different job with a different name: the
funnel's `Err` arm is silent for **every** verb, which is what made the
annotation delete a silence. Wording it is `FEATURES.md`'s "Worded decline" row
and a decision about placement, not a consumer for this query.

#### The fixtures this work had to author, and why none existed

`tools/gen-certified-fixture.py` builds two files that differ in **exactly one
dictionary** — the catalog's `/Perms`:

* `fixtures/certified-comments.pdf` — an enforced certification at `/P 2`;
* `fixtures/threaded-comments.pdf` — the same document without it.

Both carry a `/Square` markup with a `/Popup` companion and one `/IRT` reply, so
the collateral has two clauses rather than one. Nothing in `fixtures/` could
drive either branch: `signed-two-pages.pdf` is *deliberately* an approval
signature — no `/Reference`, no `/Perms` — so the gate is open on it, and no
fixture carried a markup annotation at all.

★ The pair is one document on purpose. A check comparing "withheld here" against
"offered there" across two *different* documents varies two things at once and
cannot say which caused the difference. `tools/ui-verify`'s `annot_delete_gate`
drives both and asserts the difference is the dictionary.

##### `fixtures/certified-nested-form.pdf` — certified AND nested, 2026-08-29

★ **This repository's fixture provenance lives in the generator's docstring**,
not in a `PROVENANCE.md` — every file in `fixtures/` is byte-authored by a
committed `tools/gen-*.py` whose header carries what it builds, why each
structural choice, and what the fixture is for. The engine's corpus uses a
`PROVENANCE.md` because its fixtures come from several sources; this one's do
not. The entries below are the index, and each points at the header that is the
record.

`tools/gen-certified-nested-fixture.py` builds one file, and it exists because
of an **intersection nothing occupied**:

| needed | had | short by |
|---|---|---|
| certified | `fixtures/certified-comments.pdf`, `fixtures/threaded-comments.pdf`, `D:/Dev/pdfcer/…/forms/certified-p2-form.pdf` | all three are **flat** — no dots in any field name, so `AcroForm::groups` is empty |
| nested | `D:/Dev/pdfcer/…/forms/nested-form.pdf` | **uncertified** — no `/Perms`, no signature, every gate open |

`tools/ui-verify`'s `structural_refusals_are_sentences_not_controls` asserts
that a certified document's Field-groups section lists its grouping nodes and
draws **no** Delete-group control (R9). On a flat form
`panels::forms::groups::section` returns before drawing anything, so that
assertion was true of a section that never drew — `crate::checks` rule 4
exactly. The check went vacuous → SKIP → conditional-with-a-note over three
revisions, and none of them was the fix; the fix was the file.

It is `nested-form.pdf`'s field tree (`Personal` ▸ `Personal.Address` ▸ three
terminals, `Personal.Name` one level shallower on purpose) under
`certified-p2-form.pdf`'s certification: `/Perms << /DocMDP >>` on the catalog
and a `/Type /Sig` whose `/Reference` names `/DocMDP` with **`/P 2`**.

★★★ **`/P 2`, and the reason is R162.** `/P 1` refuses *everything* — filling as
well as restructuring — so a check written against it *"passes whether or not
those gates differ at all"* (the engine's own `PROVENANCE.md`). At `/P 2` the
two gates disagree **on one file**: `EditSession::fill_refusal` answers `None`
and `EditSession::deletion_refusal` answers `Some`. That puts the control group
*inside* the document, which is why this fixture needs no uncertified twin the
way the annotation pair does — a run that finds no Delete-group control has
found a *withheld* control rather than a dead panel, because the fill controls
beside it are still live.

★★ **Verified with the engine, not by eye.** A fixture that loads and whose
`AcroForm::groups` is empty makes the check pass while testing nothing, which is
worse than the SKIP it replaces. So the four properties are pinned by a unit
test beside the fixture's users
(`crates/pdfcer-gui/src/app/actions/forms/delete.rs`,
`the_certified_nested_fixture_is_both_certified_and_nested`): it loads,
`deletion_refusal` is `Some`, `AcroForm::groups` is exactly
`["Personal.Address", "Personal"]` (post-order, deepest first), and
`fill_refusal` is `None`. Cross-checked against `pdfcer`: `list-fields`
reports the four terminals, `list-signatures` reports
`signatures=1 certifications=1`, `delete-field` exits 9 naming `P=2`, and
`fill-field` exits 0.

### ⬜ / ⛔ What is left, and why

| Verb | Reason |
|---|---|
| `copy_annotations` | ⬜ **Open, and narrowed.** The object clipboard copied a markup by reading it into a `MarkupSpec` and authoring a new one, so everything a spec cannot express was lost — and on 2026-08-28 that came to include the note, the author, the date and the opacity, all of which this shell had just learned to author. `carried_options` closes those four. The general fix (`copy_annotations` → `ObjectClip` → `paste_objects`) is **asked of the engine rather than assumed**, because it is not known whether a `/Popup`, an `/IRT` reply chain or an `/RC` rich-text body survive that path either, and a paste that silently orphans a reply is worse than the loss it replaces. ⇒ The general form: **a copy implemented as a re-author loses ground every time the authoring side gains a key**, silently, in a direction no screenshot can see. |
| `add_named_destination` | ⛔ **Not a gap — a deliberate absence, and the engine agrees.** Nothing in this shell constructs a `Destination`: the one authoring call passes `Destination::Page { view: DestView::Fit }` and cannot pass anything else, because there is no destination chooser. The engine's own note says why that is right: *"a destination chooser offering fits pdfcer cannot write would be a control whose options are mostly refusals."* The **reading** side already resolves named destinations, so the Bookmarks panel navigates them in CAD and Word exports today. |
| `field_defaults` | ⛔ **Not a gap.** *"Make another field like this one"* is already how this shell behaves — `FormDefaults::next` carries the previous field's settings forward, with the **name** the one thing that deliberately does not carry. What the verb adds is copying from *any named* field rather than the last one placed, which is a chooser. An operator call, not a hole. |

---

## ★★★ The twenty-one the TIGHTENED gate found, 2026-09-05 — an argument in a comment is not an entry in the register

`tools/verb-coverage.py`'s `gui_hits` scored a verb consumed when its **name
appeared anywhere** under `crates/pdfcer-gui/src`, comments included. The
`remove_encryption` section above named that blindness and deliberately did not
fix it, on the grounds that tightening an instrument is a change to make and
measure on its own. It was made on 2026-09-05, and the case that forced it is
the worst one yet: `pdfcer-core` shipped **`pdfcer_core::sign`**, 101 public
items, an entire digital-signing subsystem written in answer to *this shell's
own* request, and this gate scored `EditSession::sign` **consumed** because the
word *sign* occurs in `app/actions/bookmarks.rs` in a doc table about the
arithmetic **sign of `/Count`**. A capability the operator asked for was
discharged by a comment about positive and negative numbers.

A hit must now be **call-shaped** — the name followed by `(`, no identifier
character before it — with comments blanked first. Both filters are needed and
neither subsumes the other: the first kills prose, the second kills aspirational
examples. Read `gui_hits`' own docstring before re-litigating either.

The gate then said:

> **203 `EditSession` verbs (lock `03f6004`), 162 named somewhere in the shell,
> 41 named nowhere.**

Twenty-three of the forty-one had no row anywhere when this section was started.
(It was twenty-six an hour earlier: the track wiring the markup-vertex verbs
landed three rows for its own verbs in the *alternate spellings* table above
while this was being written, which is the register working as intended and is
why a count in this file always names the moment it was taken.)

**Twenty-one of the twenty-three are below. The other two are deliberately not
here, and are deliberately not spelled in backticks anywhere on this page:**

- ⚠ **delete\_node was held back on a MIS-ATTRIBUTION, corrected 2026-09-05
  the same evening.** It was withheld as *"being wired by the vertex track"*.
  It is not, and could not be: `EditSession::delete_node` (`edit.rs:12148`)
  removes a node from a **vector path in the page's content stream**. The
  vertex track's verbs are `reshape_annotation` and its three convenience
  wrappers, which edit a markup **annotation's** `/Vertices`. Two unrelated
  capabilities, and what joined them was that this shell's own private helpers
  in `app/actions/annots.rs` happen to be named `move_node` and `remove_node`.
  ★ **A name collision inside our own crate deferred a real gap for an evening.**
  It now has a row of its own below, as a gap, beside its twin `delete_subpath`.
- **sign** is a 101-item subsystem this build does not even compile:
  `crates/pdfcer-gui/Cargo.toml` takes `pdfcer-core` with
  `default-features = false` and forwards only `jpx`/`ocrs`, stripping the
  default-on `signing` feature. It gets an `ENGINE_BACKLOG.md` row **and** a
  row below, because the two documents answer different questions: the backlog
  asks *"what did the engine ship that we want?"* and this file asks *"why does
  the shell not call this verb?"*. The answer to the second is short and is not
  a decision anyone made.

⚠ ★★★ **The backtick discipline in those two bullets is load-bearing, and the
first draft of this section got it wrong.** `check-verb-coverage.sh`'s rule is
*"`EDITABLE_SURFACES.md` must mention it by name, in backticks"* — a fixed-string
`grep -qF` for the backticked name, anywhere in the file. So a paragraph written
to say *"this verb is somebody else's and has no row here"* **discharges the gate
for it** if it spells the name the way this register spells verbs. The first
version of the six lines above did exactly that, and the gate went from naming
five unexplained verbs to `PASS: all 41`, on a change that wired nothing. Two
names left the failure list because a sentence disclaiming them was written.

⇒ **The gate cannot read English and must not pretend to** — its own header says
so, and calls that weakness deliberate. The corollary nobody had written down is
that **the weakness runs in both directions**: prose *about* a verb is
indistinguishable from prose *accounting for* a verb, so a register can silence
the instrument by discussing what it is not covering. Spelling the two names in
bold rather than in backticks keeps them on the failure list where their own
tracks will meet them.

### ★★★ What the split says, and it is not what a gate failure usually says

**Four are gaps. Seventeen are verbs this shell should not call.** That ratio is
the finding, because of *where* the seventeen reasons already were: **twenty of
the twenty-one had their argument written out, in full, in a doc comment inside
this crate.** `cut_pages`' is a paragraph in `app/dispatch/pageclip.rs` ending
*"Recorded here so the next reader does not rediscover the constraint by
trying"*. `find_text`'s is a table under the heading *"The trap, stated first
because it is the whole reason this module is written the way it is"*.
`transform_preview`'s is a note whose first line is **"THE PREFLIGHT IS NOT
BUILT, AND THIS IS THE NOTE THAT SAYS SO."**

⇒ ★★ So the tightening did not mostly find unconsidered verbs. It found
**considered verbs whose consideration was filed where the register could not
see it** — and, because the old instrument counted that prose as a call, filed
in the one place that also switched the gate off. The three "not built" notes in
that set had been true and unactioned for days precisely because writing them
down felt like discharging them.

★★★ **And the exception is the whole point.** `set_media_boxes` is the one verb
of the twenty-one with **no sentence anywhere in this crate about why it is not
called** — and it is the largest of the four gaps. The verb nobody had written a
sentence about is the capability nobody had noticed. That is the gate's own
message, arriving as data rather than as advice.

### The gaps — three of them CLOSED 2026-09-05

⚠ **This section was headed "The four gaps" and the count was already stale when
it was written**, which is the failure mode this whole register exists to catch,
arriving in the register itself: the table under it listed **six** rows. Three of
those six were the same defect three times — `delete_text_run`, `delete_subpath`
and `delete_node`, each with its MOVE twin wired and itself called by nothing —
and they are now wired, so what is left below is **`sign`, `set_media_boxes` and
`transform_preview`**. The heading no longer carries a number, deliberately: a
number in a heading is a claim nothing checks, and this one was wrong on the day
it was typed.

★★ One further gap was found while closing the three and is recorded here
because nothing else in this file is keyed on it: **the Part and Node rungs
INSIDE a form XObject still cannot delete, and that is an engine gap rather than
a shell one.** `pdfcer-core`'s six `*_in_form` verbs are five moves and one
whole-object delete; there is no `delete_subpath_in_form`,
`delete_node_in_form` or `delete_text_run_in_form`. `canvas::deleting` declines
those two rungs with `Refusal::InsideForm` and says so on the status row,
pointing at what does work (Escape out to the whole shape, then Delete).

| Verb | Engine Pass | Status |
|---|---|---|
| `delete_text_run` | 32.0 | ✅ **WIRED 2026-09-05, and DRIVEN — one label off a sheet whose 237 share a text object.** The Part rung on a text object reaches it through `VectorAction::DeleteTextRun`, routed by `canvas::deleting::subject`, which the Delete **key** and the ribbon's `format.delete` both ask. ⚠ **Wired is not reachable, and for one commit these were different things**: the key inherited `canvas::interact`'s conditional decomposition, which was gated on a list of gesture outcomes, and a keystroke is not one — so the key declined `NoObjectModel` while the ribbon (which asks for the model unconditionally) worked. `canvas::modelneed` fixed it; `deleting_a_label_leaves_the_other_labels_alone` PASSES on `SW41177.pdf` at `0,1140,62`, **18 runs → 17, page objects 5,903 unchanged**. ★★ **The operator reaches that rung with the Points tool (`A`), not with a double-click** — O70 gave the double-click on text to the caret, so `canvas::clicking` opens a caret and returns before the ladder is touched; the node-tool branch is the one route that descends on text. Before this, Delete over a selected label traced `canvas-delete-declined … reason=no-verb-for-rung` and nothing happened, silently. ★ The R83 pre-condition is **no longer dead**: `text_run_delete_would_move_next` is asked ahead of the press, and a run whose successor inherits its position is refused with the remedy in words (*delete the later label first*) rather than with the engine's cause-less decline — `EditError` is `Display` output and `check-ui-strings.sh` exclusion 3 forbids routing it to a surface, so pre-empting is the only way the operator can be told what to do. Disclosures: **always empty**, measured — `plan_delete_text_run` returns `Vec::new()` on both arms. ⚠ **These are pdf dimensions** (R8b Rule 15): page content pdfcer reads and must not silently alter. |
| `delete_subpath` | — (`move_subpath`, Pass 28.0, calls it *"the companion to `delete_subpath`"*) | ✅ **WIRED 2026-09-05, and DRIVEN — one line out of a drawing view.** `deleting_a_line_leaves_the_rest_of_the_shape_alone` PASSES on `fixtures/hole-in-a-big-object.pdf` at `0,336,500` — **41 subpaths → 40, page objects 1 unchanged**; that fixture is one path object holding a circle and forty unrelated segments, which is the shape of his own export. ⚠ It first SKIPPED on `polyline-nodes.pdf`, whose single object holds ONE subpath: the delete was correct and the check could not tell a right build from a wrong one, which is the fixture guard working rather than a defect. `VectorAction::DeleteSubpath`, from the same rung its twin `move_subpath` has been draggable from since Pass 28.0. On the measured CAD export where **one path object holds 1,194 subpaths**, *"delete this line"* is what an operator means and the only Delete previously offered removed the whole view. Disclosures: **always empty**, measured — both arms of `plan_delete_subpath` return `Vec::new()`. ★ It is still the sole `EditSession` verb in `edit.rs` with **no doc comment at all**, which remains the best account of why it was read past for a fortnight; the shell's own `VectorAction::DeleteSubpath` now carries the explanation the engine did not. |
| `delete_node` | — (`move_node` / `move_nodes`, Pass 28.0) | ✅ **WIRED 2026-09-05, DRIVEN, and it is the one of the three that owes a SENTENCE.** `deleting_a_point_leaves_the_rest_of_the_line_alone` PASSES on `fixtures/polyline-nodes.pdf` at `0,150,260` — **6 anchors → 5, page objects unchanged**, `disclosures=none` because the anchor it picked sits on no curve. `VectorAction::DeleteNode`, from the Node rung. ★★★ `delete_node` returns `PlannedEdit::disclosures` **non-empty when deleting the point discarded a curve** — *"The curve that ran into this point was removed along with it, so the shape now goes straight from the point before to the point after."* — which is a shape change re-adding a point cannot undo, and the engine's doc says rule 4 forbids letting the operator find that out from a diff: *"the caller must surface these."* The surfacing is **structural rather than hand-written**: the arm returns the list from `vector_edit_on_page`'s closure and the funnel records every verb's disclosures to the status bar's row, stamped with the epoch the edit produced. A `record_note` beside it would have been a second mechanism for one sentence — the one that later forgets to retire itself. ★ A multi-anchor selection is **refused by name** (`Refusal::ManyNodes`) rather than looped: there is no `delete_nodes`, each excision renumbers, and acting on the entered one alone is the `selected_nodes_on` defect. ⚠ It is **not** the markup-annotation vertex verb; this shell's private `move_node` / `remove_node` in `app/actions/annots.rs` belong to `reshape_annotation`'s family and share nothing with it but a name. |
| `preview_font_resources_for` | 142.2 (shipped 2026-09-05, **in answer to this shell's own request of that morning**) | ⬜ **A gap that is one argument wide, and the surface it closes already apologises for being open.** `preview_font_resources` coverage-tests the characters **already in** the run; `_for` takes a `candidate` string and tests the characters the operator is **about to type**, through the same gate `set_font` applies, embedded-subset floor included. That is exactly the caveat `panels::properties::refusedchar`'s header states in its own words — *"the offer is UNTESTED against the character, and says so"* — and the sentence `text::panels::face::refused_char_untested` exists to carry. ★ The call site is **one**: `canvas::textedit::pin.rs:425` passes `(page, "", Some(span))`, and the engine's doc says an empty `candidate` behaves exactly as the old verb, so the change is additive by construction. ⚠ **Not built here, and named rather than implied**: it retires an operator-facing sentence and re-words a chooser, so it needs its own driven check (the face list must be seen to *shrink* for a character a page face cannot hold) and it is not this track's subject. Verdict **wanted**, not declined — the engine built it because we asked. |
| `sign` | 10.7 / 10.8 / 10.9 | ⬜ **Not declined, not deferred — not compiled.** `pdfcer_core::sign` is 101 public items answering *this shell's own* 2026-09-03 request (*"a document cannot be signed"*), and `crates/pdfcer-gui/Cargo.toml` takes `pdfcer-core` with `default-features = false` while forwarding only `jpx` and `ocrs`, so the default-on `signing` feature is stripped from this build. ★★ **That is the JPX incident repeating, and our own manifest carries the warning about it** — the comment at `Cargo.toml:47` records the day the GUI silently lost JPEG 2000 decoding to the same omission and says *"forgetting to forward does not fail to compile"*. ★ It was scored **consumed** for two days because `verb-coverage.py` matched the bare word `sign`, which occurs in `app/actions/bookmarks.rs` in a doc table about **the arithmetic sign of `/Count`**; that is what prompted the instrument to be tightened to call-shape. ⚠ The remaining input is the operator's, not an engineer's — key store, visible or invisible signature, `/Reason`, `/Location` — and two engine limits belong in that conversation: signing refuses an **encrypted** document outright, and refuses one carrying a **pending redaction**, both of which this shell can now produce. `ENGINE_BACKLOG.md` carries the capability rows. |
| `set_media_boxes` | — (shipped 2026-08-18 beside `set_media_box` and `pdfcer_core::paper`) | ⬜ **A gap: an open drawing's sheets cannot be resized, at all.** `set_media_box` is called exactly once, on a brand-new blank document's page 0 (`app/blank.rs:334`). The plural verb — written for the drawing-set case, *"a sheet set is resized as a set"*, one undo entry however many sheets, refusals raised before anything is committed — is called nowhere, because there is no `pages.resize` command: the Pages tab has insert, cut, copy, paste, delete, extract, move and rotate, and no size. ★ The chooser is **already built and unreachable**: `dialogs::new_document` offers `PaperSize::ALL`, both orientations and a custom size, and can only be opened while creating a file. This is the row that had no sentence anywhere. |
| `transform_preview` | 113.1 | ⬜ **A gap this shell has already written the confession for**, at `canvas/resizing.rs:172` — *"THE PREFLIGHT IS NOT BUILT, AND THIS IS THE NOTE THAT SAYS SO."* The engine distinguishes two refusals and gives the shell an instruction for each: `SingularTransform` means *this drag* is degenerate (offer the handle, refuse on release — `is_usable` does this), and `DegenerateCtm` means the object can never be transformed **at all**, for which the instruction is *do not offer a handle*. The shell offers one, and the operator finds out by dragging it. Small — a singular CTM is a producer emitting `0 0 0 0 x y cm` — and real. Not built because the preview **decomposes the page**: ~4 s on the 129,758-object benchmark in a debug build, so the engine's own advice is to call it on selection change and gesture start, which means a cache keyed on `(page, edit epoch, selection)` in `app::cache::FormRunCache`'s shape rather than a line of code. |

### ⛔ The seventeen this shell should not call

| Verb | Engine Pass | Status |
|---|---|---|
| `add_text_annotation` | 6.2 | ⛔ **Alternate spelling, and the plain door can never be the right one here.** `app/actions/textannot.rs:172` calls `add_text_annotation_with`, because this shell has a `MarkupNote` (the operator's author name, a UTC `/M`) **and** the pen's opacity to pass on every sticky note, text box and stamp — and the plain verb is `_with` under `MarkupOptions::default()`. Exactly the relation `add_markup`/`add_markup_with` has two tables up, where the `_with` door is the one that shipped. |
| `cut_pages` | 171.0 | ⛔ **Declined, argued in code at `app/dispatch/pageclip.rs:48`, and the constraint is structural rather than stylistic:** the clipboard lives in `egui::Memory` and the action applier has no `egui::Context`, *"so a single-call cut could not put its own clip anywhere."* `pages.cut` is `copy_pages` then `PageAction::DeletePages` — one extra page-tree walk, and the undo entry count stays at **one**, which is the property the engine's verb exists to guarantee. ★ The copy runs first and unconditionally, so a cut whose delete is refused leaves the sheets on the clipboard rather than losing them. |
| `cut_outline_item` | 172.0 | ⛔ **Declined for the identical reason, stated at the site** — `panels/bookmarks/clip.rs:74`. The engine's verb *is* `copy_outline_item` followed by `delete_outline_item`; the panel performs those two in that order, and `BookmarkAction::Delete` already drops the selection, warns how many descendants travel and lands one undo entry. The copy's success **gates** the delete (`take(…) && cut.clicked()`), so a cut whose copy half failed cannot silently become a delete — the failure an operator would discover by pasting. |
| `cut_selection` | 168.0 | ⛔ **Declined — `cut_objects`' argument, already in this register, extended to the mixed selection.** `canvas::clipboard::cut` is `copy_selection` + `Action::DeleteSelection`; the copy is `&self` and commits nothing, so *"the cut is one undo entry because only one half of it is an edit"* (`canvas/clipboard.rs:936`). ★ The engine's refuse-before-deleting contract is honoured **ahead of** the copy rather than inside it, by `canvas::cutgate::blocker` (`clipboard.rs:990`), and that ordering is stronger than the verb's: it is what lets `Ctrl+X` over an annotation the clipboard cannot carry refuse the *whole gesture*, instead of leaving the operator with the annotation still on the page and a copy of it on the clipboard. |
| `paste_pages` | 171.0 | ⛔ **Declined so the disclosures have exactly one wording.** The verb is `Document::from_bytes(clip.bytes)` then `insert_pages`; `app/actions/pages.rs:1080` does precisely that and hands the result to `insert_from_view` — the one function that reports `orphaned_widgets`, `orphaned_widgets_unrecoverable`, the dropped source outline and the two page-label facts, and then moves the view to what landed. Its own comment says why a fourth copy was refused: it *"would have been a second wording of the most consequential disclosure in this file"* — the orphaned widgets the engine flagged as *"the one that produces a document that looks right and is not."* |
| `delete_object` | decision 011 §2.5 op 2 | ⛔ **The singular of a verb this shell only ever needs plural, and looping it is a documented hazard.** `app/actions/vector.rs:77`: `delete_objects` resolves **every** index before planning, so one stale or duplicated entry refuses the whole call rather than deleting the prefix that happened to resolve. A loop would be N undo entries for one Delete and — the correctness half — each call re-splices the content stream, so the second index is planned against byte offsets the first already invalidated. `docs/core-api/02` states it in a box: *"Never loop the singular verbs over a selection."* |
| `move_object` | decision 011 §2.5 op 1 | ⛔ **The same ruling, one verb along** — `app/actions/vector.rs:106`. A released move-drag raises `MoveObjects` and reaches `move_objects(page, &objects, dx, dy)` with the whole selection, for the identical two reasons. ★ And there is no second claimant: the Object rung's resize and rotate both go to `transform_objects`, and the deeper rungs have `move_subpath`, `move_node` and `move_nodes` of their own. |
| `delete_dimension` | 25.6 | ⛔ **It is called — by the engine, on this shell's behalf.** `delete_annotation` **routes** a **ce dimension** to `delete_dimension` (`edit.rs:25327`, `AnnotationDeletionRoute::Dimension`) exactly so a front end needs no `match`: *"a `/PieceInfo` sidecar record backs it; leaving it would keep a dimension the annotation no longer supports."* So `format.delete` and the canvas Delete key over a selected ce dimension reach it through `app/actions/annots.rs:66`, and `annots.rs:16` records that `delete` is the one verb in that file carrying **no** ce-dimension routing obligation of its own. ★ One consequence worth knowing: a delegated route runs the **destination's** gate, and `delete_dimension` keeps the strict one because it also rewrites the catalog sidecar — so on a `/P 3` document a ce dimension is refused while every other annotation deletes. The standard's answer, not pdfcer's preference, delivered to the operator by the funnel's decline channel. |
| `move_dimension` | 25.5 | ⛔ **Declined for the drag on the engine's own instruction** — `canvas/dimdrag.rs:16` carries the two-row table. Dragging a **ce dimension** is `place_dimension`, which writes `offset` and `text_along` only, so **no drag, however far, can alter the printed number**; `move_dimension` translates the measured points and would take the ce dimension off the feature it annotates. ⚠ **The residual, named rather than left inside a half-promise:** that header ends *"remains available only where the operator has said they mean it"*, and **no such surface exists**. Nothing in this shell translates a ce dimension's measured points, so a ce dimension does not travel with page objects moved under it. Declined verb, real remainder; whoever builds the *"move the ce dimension with the geometry"* gesture should correct that sentence in the same commit. |
| `delete_dimension_group` | — (the safe door over `delete_dimension_group_with`) | ⛔ **Alternate spelling, and this shell always holds the argument the plain door lacks.** `app/actions/dimensions.rs:935` calls `delete_dimension_group_with(group, policy)` for **both** policies, `Refuse` included — which is exactly what the no-argument verb does internally. One call site rather than two, *"because the difference between them is a value this variant already carries, and a `match` here would be a second place for the default policy to be decided"* (`dimensions.rs:901`). The engine's pair exists for callers with no policy to express; the ce-dimension-group dialog always has one, because what happens to the members is the operator's decision and it is asked before the press. |
| `find_text` | — | ⛔ **Refused deliberately, and two tests keep it refused.** It passes `with_wildcards(true)`, so `#` matches any ASCII digit and `?` matches **every character on the page** — the defect the old shell's Find bar shipped with, fixed in the front end because the verb's pattern behaviour is its documented contract. `find/mod.rs:12` carries the table, and `tests::the_default_search_is_literal` and `tests::a_wildcard_search_is_only_ever_asked_for_explicitly` fail the moment anybody reaches for the shorter verb. |
| `find_text_with` | — | ⛔ **Superseded by a strict superset, and the header naming it is now its only mention.** The Find bar called it until `pdfcer-core` v0.11.0 shipped `search_text` — which `find_text_with` now delegates to, so the scan and the hits are identical — and which additionally returns the extraction diagnostics that say whether a zero-result answer can be trusted. `find/mod.rs:815`; `Results::unsearchable_fonts` is built from them, and the only cost is holding a `TextDiagnostics` that was previously computed and thrown away. |
| `mark_redactions_by_search_with` | — | ⛔ **Subsumed twice over.** The shell calls `search_and_mark_redactions_styled` (`app/actions/redact.rs:193`) **with** a `TextSearchOptions`, so the options this verb exists to accept are already being passed; and that verb is a superset of this one on the appearance and the diagnostics as well. |
| `mark_redactions_by_search_styled` | — (the `_styled` verbs, `a7210a4`, 2026-08-17, shipped in answer to this shell's filing) | ⛔ **Declined for the one verb that also hands back the diagnostics, and on this operation that is not a nicety** — `app/actions/redact.rs:166`. Both run the identical scan and author the identical marks. Only `search_and_mark_redactions_styled` distinguishes the two causes of an empty result: *the term is not in the document*, and *the document's text was never recoverable as Unicode, so no term could ever have matched it*. For a search that ambiguity wastes a minute; **for a redaction it fails in the direction nobody catches** — the operator asked for every occurrence of a name to be removed, the run reported success, the file still contains it, and both populations render perfectly. |
| `embed_refusal` | 67.0 phase E | ⛔ **Declined as a duplicate guard, argued at `app/actions/fonts.rs:34`:** `embed_fonts` runs it itself *"before any mutation"* and returns the refusal as an `Err`, so a pre-flight here would be a second implementation of a guard the engine already owns. ⚠ **The residual, named:** it is a pure query safe to call every frame, so a *window* gated on it would be R83 work this surface has not done — on an encrypted or certified drawing the Embed-fonts window still opens, the operator chooses donor faces, and the decline arrives from the funnel afterwards. Quality work on an existing surface, not a missing capability. |
| `unembed_refusal` | 67.0 phase B | ⛔ **The same ruling, the same file (`fonts.rs:72`), the same residual.** ★ And note what the engine deliberately leaves **out** of it: PDF/A. Unembedding genuinely breaks ISO 19005 conformance, *"but it is a consequence the operator may knowingly accept, not a structural impossibility. The core reports it and the shells gate on it."* This shell's gate on that is the sentence in `dialogs::unembed`, read before the press — so even wired, this query would not be where the PDF/A decision lives. |
| `info_bytes` | — | ⛔ **Session query, superseded by the sibling that carries the disclosure.** `panels::docprops` reads every `/Info` field through `info_text` (`docprops/mod.rs:269`), which returns `InfoText { text, exact }`, and `exact` is the whole reason: when it is `false`, re-encoding the string would **not** reproduce the document's own bytes, so the panel must not write the field back and says so on the row. Raw bytes have no operator meaning and would discard the one flag stopping this shell from replacing a `/Title` with pdfcer's guess at it. ★ The name has been in this crate all along — in that module's header, arguing that an old blocker had cleared — which is precisely the prose the tightened instrument stopped counting as a call. |

---

## What this register does NOT cover, said so nobody reads it as complete

- **`pdfcer-render` and `pdfcer-print`.** This is the editing surface only.
- **Verbs on other engine types** — `MarkupNote`, `NewTextField`,
  `FieldEdit`, `MarkupStyle` and their builders. The tool scopes itself to
  `impl EditSession` deliberately: those types are *operands*, and an unused
  builder means a field of an operand the shell never sets, which is a
  different and much longer question.
- **Whether a wired verb is reachable.** See the caveat at the top. A hit is a
  name, not a route. `tools/ui-verify` is the instrument for reachability, and
  the standing rule stands: **a capability is not verified until the running
  binary has been driven through it.**
