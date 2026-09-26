# `ui-verify/checks/signature_save`

`an_invalidating_save_is_warned_about` — **the window that stands between a
structural edit and a signed document's next revision.**

# What this is for

`pdfcer-core` publishes two verbs written specifically so a front end could
answer *"what will this save do to the signatures already in this
document?"*:

```text
session.signature_impact_of_save(mode: SaveMode) -> SignatureImpact
session.changes_structure() -> bool
```


## ★★★ Why no unit test can make this claim

`crates/pdfcer-gui/src/dialogs/signature.rs` already asserts, headlessly and
against this same fixture, that the engine reports the invalidation and that
`ask_for` builds a dialog for it. Every one of those assertions passes on a
build where:

* `DialogsState::ask_signature` is never called from the save arms, so the
  window is constructed by nothing and the save runs unannounced;
* the guard is called and its `bool` is dropped, so the window appears **and
  the file is written anyway**, which is the worst outcome available — the
  operator is asked a question whose answer is ignored;
* the window draws and its proceed button is never wired, so the save can
  only be cancelled and `Ctrl+S` looks broken.

Each of those is a whole-link failure between a passing decision function
and a running application, which is the class `PROJECT_PLAN.md` §4 built
this harness for. So this check asserts the link in **both directions**, and
that pairing is the point:

| # | assertion | the build it fails against |
|---|---|---|
| 1 | the page delete happened | (precondition — SKIP, not FAIL) |
| 2 | `signature-asked` is traced and `dialog:signature` is declared | the guard is not called; the save is silent |
| 3 | **no `save-copy` line exists yet** | the guard is called and its answer discarded; the write already happened behind the window |
| 4 | after the click: `signature-confirmed`, then `save-copy` | the proceed button is inert, and Save cannot be completed at all |
| 5 | the file on disk begins `%PDF-` | the write was traced and went nowhere |

★★ **Assertion 3 is the one worth the whole check.** It is an absence, and
`checks/mod.rs`'s rule 4 says an absence is only evidence once the thing
that would have produced it is shown to work. It is admissible here for
exactly that reason: the same `save-copy` line is then *demanded* in
assertion 4, in the same run, from the same build. A build that never writes
satisfies 3 and fails 4.

## ★★ The fixture, and why this repository had to author one

`fixtures/signed-two-pages.pdf`, built by `tools/gen-signed-fixture.py`,
whose header carries the argument. The short version is that the engine's
own signature corpus
(`D:\Dev\pdfcer\fixtures\synthetic\signature\`) is three **one-page**
documents — they were built for `signature::byte_range_coverage`, which is
arithmetic over byte offsets and needs no pages — and this check has to make
the save **structural**, which it does by deleting a page. A one-page
document has no page it can spare: `pages.delete` over the only page is a
refusal, and the check would SKIP for a reason that has nothing to do with
signatures.

It carries an **approval** signature with no `/Reference`, deliberately, so
`SignatureImpact::documentation_basis` answers `ImpactBasis::ConservativeReport`
— the arm where ISO 32000-1 is silent and pdfcer reports the cautious answer
under rule 4, which is the wording hardest to get right and therefore the
one worth driving.

## ★ Why it drives Save-a-**copy** and never Save-in-place

Not a preference: `Action::Save` writes over the document's own file, and
the document here is a **committed fixture**. A check that drove it would
rewrite `fixtures/signed-two-pages.pdf` on every run — the exact hazard
`checks/ocr.rs` records having hit, one directory over — and the fixture's
whole value is that it is byte-authored and stable.

`file.save_copy` answers its picker from `PDFCER_DIAG_SAVE_PATH`, so the
bytes land in the run's own output directory and the fixture is never
opened for writing. The guard under test is the same one on both routes —
`crates/pdfcer-gui/src/app/actions/apply.rs` asks it in both arms — so
nothing about the assertion is weakened by taking the safe route.

## What this does NOT cover

* **The certification wording.** `ImpactBasis::SpecSourced` needs a
  `/DocMDP` transform in a signature's `/Reference`, and this repository has
  no such fixture. The two footings are asserted apart in
  `crates/pdfcer-gui/src/text/signature.rs`'s own tests; what is undriven is
  the certified *window*, which differs from this one only in three strings.
* **The `ByteRangePreserved` note.** It appears on the status bar's
  disclosure row after a save with no structural change, and reading a bar
  sentence needs a pixel oracle rather than a trace one. Its decision is
  unit-tested against this same fixture
  (`the_signed_fixture_moves_from_a_note_to_a_window_when_a_page_goes`).
* **Cancel.** The window's Cancel must leave no file, and asserting it needs
  a second launch. Stated as a gap rather than folded in, because a check
  that clicked two buttons in one run could not say which one the failure
  belonged to.
