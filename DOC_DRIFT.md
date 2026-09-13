# DOC_DRIFT.md — the register of this project's own stale claims, and the instrument that finds them

**What this file is for.** R5 of this project's charter says *the documentation
is the logic*. The corollary nobody writes down is that **a wrong sentence in a
doc comment is a defect, and it is a defect no compiler, test or gate currently
catches.** This file is the register of the ones found so far, each dated
against the commit that invalidated it, together with the greps that found them
and an honest statement of what was not reached.

**Why it exists as a file rather than a set of commits.** Every row here is a
`.rs` edit, and the rows were found during a driven `ui-verify` sweep, when
editing any `.rs` aborts the sweep (two staleness guards turn the remaining
chunks into usage dumps). So the findings had to be parked somewhere durable
before they could be repaired. They are kept after repair because the *shapes*
recur: this project has now corrected a count-drift defect seven times, and the
value of the register is that it makes the shape legible rather than the
instance.

**How to use it.**

1. A row is **deleted** when its repair lands, in the same commit as the repair.
   A row with no repair is a claim still shipping.
2. The **calibration table** near the bottom stays. It is the audit's negative
   control, and without it the method is unfalsified.
3. The **Coverage** section states what was *not* examined. Read it before
   quoting this file as a census; it is a sample with a known shape, not a
   complete scan.
4. The greps in Coverage are the reusable part. Re-run them; the conclusion of
   the audit is that they should become a gate.

★ **The single most useful thing learned:** *confirm the engine pin equals the
engine's HEAD before auditing any cross-repository claim.* With a stale pin the
same work yields "cannot tell" on every row. With the pin confirmed equal to
HEAD, every row becomes decidable. At the time of this audit the pin was
`Cargo.lock` = `d86cb19` = `v0.53.0-69-gd86cb19` = engine `main` HEAD.

---

## S0 — the instance that triggered the audit: `MAX_MAX_ZOOM_PERCENT`

Recorded here because the audit below exists to answer the question *"is this
shape anywhere else?"*, and the answer was eleven times yes.

`MAX_MAX_ZOOM_PERCENT` is `1e12` (a trillion percent). Four sentences of its doc
comment describe `1e11`, including *"a hundred billion percent, which is the
deepest zoom the page has been confirmed to actually DRAW at"* and *"a trillion
... does not put a page on screen"*.

**All four were falsified by the very commit that raised the constant** —
`390fcc40`, 2026-08-22. That commit measured a page drawn at 10^12 % on the
operator's own file, deleted the clamp, **and renamed and rewrote the unit test
in the same diff while leaving the prose eleven lines above the constant
untouched.** Twenty-one days standing.

★★ **Why it survived: the wrong prose was the most authoritative-looking text in
the module.** A driven measurement to two significant figures, a unit
conversion, and a paragraph on what would have to change for the limit to move.
Everything that makes a comment trustworthy was present, and all of it was
evidence for a superseded number.

★ **The direction is the part to remember.** It was found while auditing a
public `README.md` *capability* claim, on the assumption that product copy
overstates and source comments are conservative. **The product copy was right
and the source comment was wrong.** Nobody audits in that direction.

**Verdict: STALE.** **Repair:** rewrite the doc comment to the `390fcc40` world.
Tracked with S9, which is the same shape in `viewer/ceiling.rs`.

---

## S1–S11 — the 2026-09-13 audit of every other measured claim in `crates/`

Dispatched after the `MAX_MAX_ZOOM_PERCENT` finding, to ask whether that defect
shape existed elsewhere. It did, eleven times, and one of them is systemic.

**Read-only.** No file was edited and no cargo command was run — the sweep was
in flight. **Every repair below is a `.rs` edit and is blocked until
`=== SWEEP-DONE`.**

The pin was confirmed equal to engine HEAD first (`Cargo.lock:2610-2612` =
`d86cb19` = `v0.53.0-69-gd86cb19`), which is what made every engine claim
**decidable** rather than unverifiable. That step is the reason this audit
produced verdicts instead of suspicions — do it first, every time.

---

## S1 — `canvas/viewpos.rs:194-195` — off by 256x, and PROPAGATED THE SAME DAY

> `Measured: at a trillion percent it moves in 2,048-pixel jumps.`

`git blame` gives `741bd66d`, **2026-09-12 14:37** — written hours before this
audit, by copying `canvas/deep.rs`. The real provenance is
`viewer/ceiling.rs:40-42`: *"driving to the top of the setting on a US Letter
page drew at a content extent of 20.5 billion — a 2,048 px step"*. 2,048 is the
`f32` ulp at about 2.05e10, i.e. a zoom of 2.05e10/792 = **2.6 billion
percent**. At a *trillion* percent the extent is 7.92e12, ulp 2^19 =
**524,288 px**.

**Verdict: STALE.** **Repair:** quote the content extent, not a zoom — or say
"two and a half billion percent", the figure that was actually driven.

★★★ **This is the finding that matters most, and not because of the number.** A
wrong measured sentence was copied into a second file on the same day the
original was found wrong, by a session that was *looking for* wrong measured
sentences. Copying a comment copies its staleness and resets its apparent age.

## S2 — `canvas/deep.rs:6,8-9` — "about two million percent" is the pre-O49 threshold

`62cc539d`, 2026-08-24. The hand-over predicate is
`longest * zoom > SUB_PIXEL_CONTENT_EXTENT` (`viewer/ceiling.rs:285-292`) and
the constant is `1_048_576.0` = 2^20. 2^20/792 = **132,396 %** on US Letter,
about 85,700 % on a large sheet. "Two million" was right for 2^24 and O49 cut it
to 2^20 on 2026-08-28 — and `ceiling.rs:53` *records that change*, in the same
file, naming both new figures. The header was never updated. Carries the same
2,048-px error as S1.

**Verdict: STALE.** **Repair:** cite `SUB_PIXEL_CONTENT_EXTENT` rather than
restating a derived percentage.

⚠ **The same stale threshold is in two check `detects:` lines** —
`tools/ui-verify/src/checks/zoom_keeps_place.rs:153` and
`zoom_out_keeps_place.rs:124`, *"past about two million percent"*. Those print
in every sweep report. Fix them in the same edit as R8, which touches both files
anyway.

## S3 — `shell/manifest/rail.rs:22,99-104` — "the five panel tabs" when the file's own test asserts SIX

`groups()[0]` holds six `Item::command`s: `view.panel_pages`,
`view.panel_bookmarks`, `view.panel_layers`, `view.panel_signatures`,
**`markup.comments`**, `file.fonts`. The test at `:384-393` asserts
`len() == 6` — *"all six panels, one click away — Comments joined 2026-09-05,
on his report"* — and the Comments item carries a 55-line star block explaining
its addition.

★★ **The author documented the new member thoroughly and left every count
around it at five.** Eight prose sites say five:
`shell/manifest/rail.rs:22`, `:99-104`; `egui-shell/src/manifest/rail.rs:68`;
`egui-shell/src/dock/rail.rs:62-63`, `:808`, `:1043`;
`app/rail.rs:4`; `shell/manifest/mod.rs:278`; `app/modes/defaults.rs:790`.

`egui-shell`'s test fixture `pdfcer_rail()` (`dock/rail.rs:677`) is internally
consistent at five, so its own `assert_eq!(…, 5)` is not wrong — but its doc
comment *"The rail the mockup draws"* describes a rail the product no longer
has. `rail.rs:18`'s "The four groups" is still correct.

**Verdict: STALE.** **Repair:** eight sites to six; add Comments to the
enumeration. ✓ **Checked: not operator-facing and not published** — grep found
no "five panels" in any `ui_text` string or any `.md`. So this is not
release-blocking.

## S4 — `app/status/disclosure.rs:452-463` — both halves of an "until then" were untrue within 5 h 20 m

Claimed (a) `strategy::for_page` ignores the pixel ceiling so 534 to 2071 %
mis-composites, and (b) `MAX_CMYK_BUFFER_BYTES` is `pub(crate)` in the engine
so the repair is blocked.

`12be31b5`, 2026-08-26 **12:46**. `82af210e`, the same day at **18:06**, says
in its own message: *"render::strategy::for_page takes a third argument and ends
the whole-page tier at whichever ceiling bites first."* `render/strategy.rs:330`
now calls `pdfcer_render::will_composite_in_cmyk(w, h, max_bytes)`. And the
constant is public at the pin: `pdfcer-render/src/lib.rs:266`,
`pub const DEFAULT_MAX_CMYK_BUFFER_BYTES`.

**Verdict: STALE.** **Repair:** one sentence — the pixel ceiling is respected
since 2026-08-26, so the disclosure now fires only when the operator's own
`max_cmyk_buffer_bytes` setting is the binding limit.

## S5 — `app/blank.rs:9-18,22,37-38` — "The engine cannot create a document" is FALSE

> `## 1. The engine cannot create a document, and that is deliberate`
> `Document` has exactly four constructors … **every one of them parses
> existing PDF bytes** … There is no path anywhere in the engine that conjures
> a page from nothing.

`pdfcer-core/src/text_edit/placetext.rs:1375` defines
`pub fn blank_document(media: Rect, count: usize) -> Result<Document, …>`,
re-exported at `text_edit/mod.rs:113`, with a doc example asserting three pages.
There are **six** public constructors, not four (`load` 385,
`load_with_password` 407, `load_with_options` 460, `from_bytes` 474,
`from_bytes_with_password` 524, `from_bytes_with_options` 543) and the cited
span `document.rs:360-404` contains only `load`. `blank_page_doc` is at
`edit.rs:48069`, not `:18066`. `delete_pages` 38519, `reorder_pages` 38865,
`rotate_pages` 40259 — the cited `edit.rs:3848`/`:14739`/`:15039` hold
unrelated ink-stroke, `move_node_in_form` and `unshare` code.

The invariant itself is intact: `blank_document` parses scaffold bytes through
`from_bytes`, so it does not breach *"no separate builder/generation model"*.
The shell already knows about the newer surface in six other files. This file
has exactly one commit — the pdfce to pdfcer rename — and has never been
revised.

**Verdict: STALE.** **Repair:** retitle section 1 *"The engine creates documents
only by parsing bytes it scaffolds itself"*, cite `text_edit::blank_document` as
the supported path, four to six with `document.rs:385-560`, drop the three
rotted `edit.rs` line numbers for function names.

## S6 — `app/lifecycle.rs:366-367` — the same sentence, in the doc that JUSTIFIES New

> `pdfcer-core` has no way to create a document and states in `document.rs:10-19`
> that it never will … so New parses a 443-byte template that ships as an asset

★★★ **The citation checks out and the conclusion drawn from it does not.** The
quoted invariant sentence *is* still in `document.rs:10-19` verbatim. A reader
verifying the quote finds it, and stops. The 443-byte figure is fine — it is the
shell's own asset.

**Verdict: STALE.** **Repair:** *"`pdfcer-core` exposes
`text_edit::blank_document`, but New predates it and parses a 443-byte asset
template — which makes it an open; see `app::blank`."*

## S7 — `app/cache.rs:208-210` + `app/state.rs:1135-1136` — a file that refutes itself 110 lines later

Claimed the pinned engine cannot edit text inside a `Do`-invoked form XObject,
citing `edit.rs:79`. Engine `text_edit/edit.rs:81-90` says the opposite **and
dates it**: *"Form-XObject content was a named non-goal of the 14.1 cut, and
that sentence stood here until 2026-08-20. It is now false."* Line 79 is about
the `'` and `"` show operators.

`git blame` gives 2026-08-20 10:36, hours before the engine doc was rewritten.
And `cache.rs:317-322` — **the same file** — already carries the correction:
*"This used to mean 'inside a form XObject', and it stopped meaning that on
2026-08-20 … The old reading refused a caret on 99 % of the text on a CAD
drawing."* `state.rs` still states the stale version as a field invariant.

**Verdict: STALE.** **Repair:** rewrite both headers to say what `cache.rs:317`
already says — form content is editable since `Pass 119.0`; the cache now exists
for the provenance cost, not for a refusal.

## S8 — `viewer/ceiling.rs:10,16-17` — 7.5x too high, and "not yet wired" is false

Header table: `SUB_PIXEL_CONTENT_EXTENT` … `~1,000,000 %`. The header read the
*content-extent* value as a percentage. The real bite point is 132,396 % on US
Letter and about 85,700 % on a large sheet — **and the `//` block at `:53-91` in
this same file states both**. `DeepAnchor` is wired: `canvas/viewpos.rs:205-206`
calls `viewer::deep_position_needed(…)` under a TIER 3 comment, and
`canvas/deep.rs` exists to do it.

**Verdict: STALE.** **Repair:** *"~132,000 % on US Letter, ~86,000 % on a large
sheet"*; "not yet wired" becomes *"wired at `canvas::viewpos`'s tier-3 branch"*.

## S9 — `viewer/ceiling.rs:33-34` — a `///` states 2^24 directly above a 2^20 literal

`pub const SUB_PIXEL_CONTENT_EXTENT: f32 = 1_048_576.0;` (2^20). The correction
exists only in the **non-doc** `//` block at `:53` — which rustdoc does not
render. A reader of the generated docs sees `///` asserting 2^24 above a value
of 2^20 with nothing to suggest the discrepancy is known.

★★ **This is the canonical `MAX_MAX_ZOOM_PERCENT` shape exactly**, and it is the
second instance, which is what makes the gate worth building:
`tools/gates/check-const-doc-magnitude.sh` would have caught both.

**Verdict: STALE.** **Repair:** *"— `2^20`, chosen by O49 on 2026-08-28 as the
point judder becomes visible, four orders below where drawing fails"*; keep the
2^24 history in the `//` block.

## S10 — `app/markupband.rs:955` (+ `markupband/tests.rs:366`) — a citation that now lands in embedded-file-stream docs

Cited `edit.rs:26463-26476` for the `style.dash` / `takes_border` guard. The
guard is at `edit.rs:30682-30692`; `:26453-26476` is now embedded-file-stream
and name-tree documentation. **The substance is still correct** — the predicate
*is* `takes_border` and a chooser drawn from it cannot produce
`StylePropertyNotApplicable`. Only the citation rotted.

**Verdict: STALE (citation).** **Repair:** cite `set_markup_style`'s
`!support.takes_border` guard by function name.

## S11 — SYSTEMIC: 168 hard-coded engine line-number citations, 8 of 8 sampled land on unrelated code

`grep -rn '(edit|document|page_tree|text_extract|settings|pageops)\.rs:[0-9]'`
over `crates/pdfcer-gui/src` gives **168**. Eight were opened at the pin beyond
the two reported above; **all eight** land on unrelated code. `edit.rs` is now
about 48,000 lines and **grows at the head**, so every citation drifts downward
monotonically, and nothing in this repository detects it.

★★★ **This is the mechanism generating several of the findings above**, and the
highest-leverage repair on the list.

**Verdict: STALE as a class (8/8 sampled, not a census).**
**Repair:** cite engine functions and doc headings **by name**, never by line
number. If a line number is genuinely wanted, pair it with the revision it was
read at (`d86cb19`) so the staleness is visible. A gate can require that
pairing.

---

## Checked and CORRECT — the calibration table

Reported deliberately. These all contain exactly the kind of number the audit
targeted, and they hold. An audit with no negative results is an audit whose
method was never tested.

| site | claim | measurement |
|---|---|---|
| `render/strip.rs:141-143` | 256 M texels is about **1 GB** of RGBA | 256,000,000 x 4 B = 1.024 GB. CORRECT |
| `find/mod.rs:125-128` | no cache in `pdfcer-core` and none here | `find_text_with` to `search_text` to `scan_text_matches`, a full extraction per call. CORRECT |
| `app/fontband.rs:95` | `CUSTOM_ITEM_WIDTH` is **96** | `egui-shell/src/ribbon/plan/mod.rs:173` = 96.0. CORRECT |
| `panels/properties/markup.rs:237` | same ceiling as `canvas::markup::pen` | `MAX_WIDTH_PTS = 12.0`. CORRECT, and so are its `DASH_WIDTH` / `POPUP_MIN_WIDTH` cross-refs |
| `build.rs:244` | `v0.1.0` … `v0.5.0`, all six pushed | `git tag` gives exactly six. CORRECT |
| `dialogs/offpage.rs:121` | "469 ms" decompose | matches the trace at `app/cache.rs:445`/`:523`. CORRECT |
| 20+ sites | the **129,758**-object benchmark figure | zero drift across every occurrence |
| `viewer/ceiling.rs:40-42` | extent 20.5 bn, 2,048 px step, stopped at 41 bn | `f32` ulp at 2.05e10 is 2^11. CORRECT — **this is the source the two stale copies garbled** |

★ **The pattern in the negatives is worth as much as the positives:** every
cross-reference to a *sibling constant in this repository* checked out. Numeric
drift against a local `const` is rare here. **All eleven failures are claims
about something outside the file** — the engine, another module's constant, or a
count the author did not own. That is where to aim the next audit.

## Coverage — what was and was not reached

**Instruments run:** a script over all 785 `.rs` files matching
`(pub )?const NAME: T = <numeric literal>;` with the comment block above it,
reporting those whose comment contains digits, gave **250 candidates**; narrowed
to those whose comment and literal have **disjoint number sets**, **128**.
Phrase greps for `Measured`, `measured by driving`, `as measured`, `benchmark`,
`took`, `ms`, `x faster`, `regression`, `confirmed to`, `the deepest`, `cannot`,
`does not`, `never`, `always` gave several hundred. The engine-limit vein
(`pdfcer-core (cannot|has no|does not|never)`, `named non-goal`, `this cut of`,
`filed as an engine request`) gave about 30 hits and **the richest vein in the
tree**. The citation vein gave 168. A countable-count vein
(`the (two|…|ten) ` plus a noun) gave about 90, which is what produced S3.

**Read in full:** about 40 candidates. **`git blame` was run on every reported
comment** to date it against the change that invalidated it — that is what
turned each one from an opinion into a measurement with an interval.

⚠ **Deliberately not reached, stated so nobody reads this as a census:** the
per-site verification of the remaining **160** engine line citations (S11's
verdict rests on 8/8, a sample); the measured-claim hits enumerated but not
opened under `panels/` (about 120 files), `dialogs/print/`, `ocr/` and
`egui-shell/src/ribbon/`; and **122 of the 128** disjoint-number-set candidates,
scanned by one line of context but not traced to their commits.

⇒ **There is more.** The 128-candidate filter is the instrument to re-run, and
it should become a gate rather than a session.

---

## The gate this audit argues for

Two of the eleven findings (S0 and S9) are one machine-checkable shape:

> a `const NAME: T = <numeric literal>;` whose doc comment contains a numeric
> literal or a magnitude word (*thousand, million, billion, trillion*) that
> **disagrees** with the literal.

`tools/gates/check-const-doc-magnitude.sh` would have caught both, at a cost of
one script. The instrument already exists in rough form — the 250-candidate
scan, narrowed to 128 by requiring the comment's number set and the literal's
number set to be **disjoint** — and six of those 128 were traced. The other 122
are unexamined, which is the strongest argument for making it a gate rather than
a session: a session samples, a gate enumerates.

A second, cheaper gate is argued by S11:

> fail any `\w+\.rs:\d+` citation in a doc comment that is **not** accompanied
> by the revision it was read at.

That does not stop drift; it makes drift *visible*, which is the whole problem —
a rotted citation is indistinguishable from a good one at the point a reader
checks it. Read
`D:/dev/rag/rust/gits_default_short_hash_length_grows_with_the_object_count_so_a_gate_joining_h_against_pinned_citations_fails_all_at_once_on_a_fresh_clone.md`
before building it; a naive `%h` join fails every row on a fresh clone.

## Related records

- `D:/dev/rag/rust/a_line_number_citation_into_a_file_that_grows_at_the_head_drifts_monotonically_and_8_of_8_sampled_were_wrong.md`
  — the full write-up of S11, with the ten-row citation table and the four
  remedies.
- `.claude/agent-memory/pdfcer-gui-engineer/feedback_a_measured_limit_belongs_to_a_revision_not_a_design.md`
  — S0, and the rule that a measured limit belongs to a revision.
- `.claude/agent-memory/pdfcer-gui-engineer/feedback_a_verbatim_quotation_of_another_files_count_goes_stale_invisibly.md`
  — S3's shape: the file that changed does not contain the number that went
  wrong.
