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

## The 13 remaining misses, each with its reason

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

### Not gaps — alternate spellings of a verb the shell already calls

| Verb | What the shell calls instead |
|---|---|
| `rotate_page_by` | `rotate_pages(&[…], delta)` — the same act for a set rather than for one page, which is what the Pages panel's selection is. |
| `search_and_mark_redactions` | `search_and_mark_redactions_styled` — the styled variant, because a redaction mark whose appearance the operator cannot choose is a mark they cannot see against their own drawing. |
| `mark_redactions_by_pattern` | `mark_redactions_by_pattern_styled`, for the same reason. |
| `copy_annotations` | ⚠ **This one is a real fidelity gap and is listed again below.** The shell calls `copy_objects`, which does not carry annotations, and round-trips a copied markup through `MarkupSpec` instead. |

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
