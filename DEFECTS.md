# pdfcer-gui — defect register

What this project's defects taught it, and what is still wrong. Read it before
changing a surface it names; the gates read several of its entries by number.

Every entry is one of three kinds, and its heading says which:

- **RULE** — repaired in this codebase, and the repair left a rule that a gate
  or a test now enforces. The entry states the rule and its mechanism and names
  the enforcer. Nothing about the incident survives.
- **OPEN** — still present in this codebase, verified at the `file:line` quoted.
- **CONSTRAINT** — an upstream or engine property this shell has to design
  around. It will not be "fixed" here.

Defects that were only ever properties of the GUI this project replaces are not
here. Git has them.

D-numbers are stable because the gates and the source cite them by number
(`grep -rn 'DEFECTS.md D[0-9]' crates/ tools/`). Numbers are not reused.

Every `file:line` is in this workspace. `D:\Dev\pdfcer` is the engine, and it is
read-only from here (`DEVELOPING.md` §1): an engine defect is written up and
handed over, never applied.

---

## Standing rules

### D1 — RULE: "is the operator typing?" is asked in exactly one place

`ctx.egui_wants_keyboard_input()` means *any widget has focus*, not *a text
field has focus* — it is `memory().focused().is_some()`, whatever its name and
doc comment promise. The canvas takes focus on the click that selects an object,
so a guard spelled that way suppresses every unmodified key from the first
canvas click onward.

`ctx.text_edit_focused()` is the right half, and it is only half: this shell's
canvas caret is deliberately not a widget, so `text_edit_focused()` is `false`
for an operator who is visibly mid-word. Both claimants are asked, once, in
`canvas::textedit::composing`.

Enforced by `tools/gates/check-typing-guard.sh`. It is a source gate rather than
a test because a `Context` built in a test has no canvas draft and no focused
field, so both spellings answer identically for every input a test can produce.

### D2 — RULE: a foreground is assigned for the fill it is painted on

Assigning `widgets.active.weak_bg_fill` the accent while leaving
`widgets.active.bg_fill` unset gives every widget that paints from `bg_fill` —
`egui_tiles` tab buttons, `CollapsingHeader` headers — a near-white foreground on
a light background. `theme::Theme::write_style` sets both
(`crates/egui-shell/src/theme/mod.rs:1395`).

A gate over source can only say a colour is a named role; it cannot say what the
text landed on. `check-theme-colors.sh` asks the first question,
`check-plate-colour.sh` asks the second (a plate colour used as a foreground must
have its matching fill), and `theme::contrast` renders the `(fg_stroke, bg_fill)`
pairs of all five widget states in all three presets and measures them.

### D3 — RULE: a `file:line` citation names its repository

A citation is a claim about a particular document. A register entry that omits
which repository it means gets resolved against whichever file the reader has
open, and can be closed as fixed while the claim is still live somewhere else.
Every citation in this file is in this workspace unless it says otherwise.

The related standing rule is the operator's: a published capability claim is
sourced or escalated, never improvised. A claim about the engine's surfaces is
filed on the shared channel; a `reply_` with no `done_` is an answer nobody acted
on.

### D5 — RULE: a list that must agree with a table is derived from the table

A hand-maintained keyboard reference with a doc comment telling you to
hand-maintain it disagrees with the keymap. `crate::text::shortcuts` holds only
the wording; every chord and every count comes from the registry
(`text/shortcuts.rs:47-55`), so a binding added tomorrow is listed without anyone
remembering to list it.

### D6 — RULE: capability is derived from the manifest, not from a mode's name

A boolean "editing on" master toggle and a string comparison on `"read"` both
produce the failure where a surface says editing is off while a gesture still
edits. `app::modes::capability` derives capability from **the mode's tab list in
the manifest** (`app/modes/capability.rs`), so the ribbon and the canvas read one
sentence and the hole is unrepresentable rather than guarded.

Three things a gesture gate cannot close on its own, each found by asking what
*survives* rather than what is refused: a click is not a drag, so gating presses
alone leaves the commonest canvas gesture ungated; an armed tool outlives a mode
switch, because it lives in `egui::Memory`; and a selection outlives it too,
leaving grips on a page in Read.

Driven by `ui-verify`'s `read_mode_refuses_canvas_edits`, which clicks page
content in Read and asserts no selection, clicks *the same point* in Edit and
asserts one — so Read's silence is proven to be a refusal rather than a miss —
then re-enters Read and asserts the selection is dropped.

### D9 — RULE: do not compensate in the file for a renderer's defect

Markup opacity is written as the annotation's `/CA` alone (§12.5.2), with the
appearance stream's `ExtGState` left at `1.0` (`canvas/markup/pen.rs:283`,
`:476`). Writing `/ca` into the appearance to make it look right in one viewer
makes it half as opaque as intended in every other viewer, permanently, in
documents that outlive the bug.

Nor does a control ship before the capability behind it: a control that visibly
does nothing is not a partial feature (`RIBBON_IA.md` P3).

### D10 — RULE: a theme that is built is not a theme that is installed

`Theme::apply` does two things — it writes the `egui` style, and it stashes the
theme under a context id so `Theme::of(ctx)` can reach the roles that have
nowhere to live in `egui`'s `Style` (the content backdrop, the label plate).
Skipping the call leaves framework chrome painting from one palette and every
`egui` widget from another, with every source gate green. It is called once per
frame from `app::frame` (`app/frame.rs:274`), per frame rather than at startup so
a theme change takes effect with no restart and no cache to invalidate.

The only oracle that can say a theme is installed is a driven measurement of the
running window: `ui-verify --check settings_theme_takes_effect` expands the
Appearance group, clicks **Dark**, and measures the window body before and after.

Still worth someone's judgement: the ribbon's group captions measure about
4.2:1, which clears this project's 3.0 floor (WCAG AA for *large* text) and would
fail the 4.5:1 that their ~10 pt size implies. Re-measure with
`ui-verify --check ribbon_group_captions_legible`.

### D11 — RULE: no `RichText::strong()` without an explicit colour beside it

`egui` has no role for emphasised text — `strong_text_color()` returns
`widgets.active.text_color()`, the **active-widget** foreground, which this
theme fills with `on_accent` because `widgets.active` is the accent-filled state.
So `.strong()` on an ordinary panel is near-white on light grey, and it survives
`override_text_color`. There is no colour it can resolve to that is correct on
both an accent fill and a panel.

`tools/gates/check-strong-text.sh` enforces the narrow form: `.strong()` is a
defect **unless an explicit `.color()` is within two code lines of it.** That
admits the legitimate uses — ribbon and dock tab labels, drawn *on* the accent
fill, where R84 wants the weight because weight survives greyscale and
colour-vision deficiency — and refuses everything else.

Two mechanisms the gate had to learn, both still load-bearing:

- The window is measured in **code lines, not source lines**, so a well-commented
  pairing is not failed while a terse one passes. A gate that punishes
  explanation trains people to delete it.
- Weight and colour must be **one decision**, not two independent `if`s.
  `ribbon::tabs` nests them (`crates/egui-shell/src/ribbon/tabs.rs:404-437`) so
  the weight cannot be reached without the colour having been stated; two
  parallel conditions let a hand-built cue struct produce a bare `.strong()`.

`ui.spinner()` resolves its colour the same way and is covered by the same gate.

### D12 — RULE: ask whether the glyph drawn is the substitution mark

`epaint`'s `Fonts::has_glyph` does not ask *"is this codepoint drawable?"* It asks
*"is this codepoint drawn by a face other than the one that supplies the
substitution mark?"* — so every character whose first supporting face in the
fallback chain is that face is reported missing and draws perfectly. The clinching
reading is that `has_glyph(Monospace, 'A')` is `false`.

`icons::glyphs::GlyphProbe` is the correct predicate: lay the character out and
compare the glyph actually drawn against a fingerprint of the substitution mark.
`GlyphProbe::new` fingerprints that mark from **three** mutually unrelated
unassigned codepoints across three planes and panics unless all three agree, so a
future font set covering one fails at construction rather than silently reporting
everything drawable.

The gate `icons::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph` reads
every `.rs` under `text/` **from source** and checks every codepoint in every
operator-visible literal, so a string added tomorrow is covered. It skips the
braced test item and **resumes** (see D13), and a file it cannot parse is a hard
refusal by name, never a quiet zero.

The quarantine mechanism is kept with nothing in it: each entry asserts its
codepoint is *still* undrawable **and** still present in the catalog, so fixing
the string makes the gate fail telling you to delete the entry. A quarantine that
outlives its reason is how an exception becomes a convention.

Two lessons under this number, both general:

- *"The gate went red"* and *"the thing the gate names is broken"* are different
  claims, and only the first is measured until somebody looks. Inferring from a
  tool's answer without asking what question the tool answers is the same class
  as inferring from documentation.
- A codepoint shipping in the launch screen went unseen because the gate looked
  only at the status bar. Point a predicate at everything it is true of.

### D13 — OPEN: `check-ui-strings.sh` stops scanning at the first column-0 `#[cfg(test)]`

The truncation is deliberate — test assertion messages are prose nobody renders,
and they were the largest source of the noise floor this gate was written to
remove. The **assumption** underneath it is not: nothing requires the test module
to be last, and where it is not, every non-test item after it is unscanned **and
the gate reports clean**. `#![cfg(test)]` on a whole file exits the same way and
is correct.

The convention that makes the limit safe — test module last — is the thing to
hold. Candidates:

```sh
grep -rn '^#\[cfg(test)\]' crates/*/src --include=*.rs
```

and read what follows each hit's closing brace. Two candidate fixes, and the
second is better: scan the whole file and exclude only items *inside* a `mod
tests` block; or keep the early exit and add a gate assertion that the test
module is the last thing in the file, which a self-test can prove it catches.

`icons::glyphs`' own scanner does not repeat this — it skips exactly the braced
item and resumes, proven by
`a_mid_file_test_module_does_not_blind_the_scanner`.

This is the fail-open class `check-file-size.sh` records: *"found no violations"*
and *"looked at almost nothing"* are byte-identical output.

### D14 — RULE: a diagnostic prints its input beside its output

A line that prints only what a stage produced cannot distinguish *"worked"* from
*"had nothing to work on"*. `markup-commit kind=Ink raw=2 kept=2` says both; with
`kept=` alone, a build whose trail was empty and a build whose simplification did
nothing are the same line.

The defect it caught, which is why the pairing exists: a gesture machine drops its
own drag on the frame it reports `Complete`, so anything reading the in-flight
trail *after* `update` reads nothing. Read the trail **before** the machine
advances (`canvas/markup/ink.rs` §2).

No unit test can see the order in which a frame calls two functions — that is a
property of a call site, and a call site's effect is only observable in a running
frame.

### D16 — RULE: `return` is load-bearing in every arm of a split `match`

When one `match` over a value is split into a pre-guard pass and a post-guard
pass — the handful of actions that must answer with nothing open, then everything
else — an arm in the first pass that falls through runs the second pass too.
Where the second pass spells that case `unreachable!("handled before the document
guard")` (`app/actions/apply.rs:387`, `:396`), the missing keyword is a panic
rather than a silent double-handle.

Both halves type-check and neither is wrong alone: the `unreachable!` documents a
real invariant, and the earlier arm reads correctly because it does the work,
traces, and records the epoch. Falling out of a `match` arm into the following
statements is ordinary control flow, so the compiler has no opinion.

The crash is the better outcome, but only if somebody presses the key — which is
R1's whole subject. A save that writes the file and then dies reads to the
operator as *"pdfcer disappears when I press Ctrl+S"*, not as a lost save.

### D17 — RULE: retire a dialog only when it is off screen **and** holding nothing

`dialogs::retire(open, answered)` is `!open && !answered`
(`dialogs/mod.rs:307`). A window that parks its answer rather than acting — because
writing over the operator's own file must have exactly one route — answers *"stop
drawing me"* on the press that fills it, and an owner reading that as *"I am
empty"* destroys it with the answer inside, three call frames before the drain
looks.

The invariant `retire` creates: **every caller of `DialogsState::show` drains the
parked answers in the same frame.** There is one caller, `app::frame`, and it
drains immediately after, so a retained-because-answered dialog lives for zero
frames.

The defect is entirely in the lifetime between three individually correct
functions, and a lifetime is not a value any assertion over either half can name.
Headless tests of the dialog's verdict and of its construction both pass on the
broken build.

Its general form, and the reason `unsaved` was repaired in the same change with
nothing red to advertise it: **when a driven check finds a defect in one member
of a matched pair, the pair is the unit of repair.** Fixing only the observed half
leaves the survivor looking deliberate.

### D18 — RULE: a ratio's two operands are asserted to be in one space

`canvas::resizing::Frame` divides a pointer travel by a grip box. The gesture
machine works in **page** space by design; `grip_box` is screen space, the same
rectangle the outline is drawn from. Handing the page-space delta to a field
documented as screen points inflates every factor's distance from unity by
`1/zoom` — invisible at zoom 1.0, which is where every unit test lives.
`PageMapping::page_vec_to_screen` is the conversion, and the Resize arm names the
space at the call site rather than leaving it to a doc comment two files away.

A degenerate zoom answers `Vec2::ZERO` rather than a NaN: a NaN displacement
reaching a content stream is a corrupted file, a zero one is a gesture that did
nothing.

The general form: **a ratio whose two operands come from different call paths has
no test unless something asserts the ratio against a number chosen outside the
program.** Every assertion available here was of the shape *"the same quantity
twice"* — a resize happened; the locked factor equals the free factor — and a
common factor cancels in all of them.

### D19 — RULE: state a helper reads is passed by argument, not reached through `self`

`canvas::interact` opens by `std::mem::take`ing the selection off the document and
puts it back at the end, which is a sound Rust idiom and silently changes what
every helper called from inside that window can see. A predicate that reaches for
`doc.selection` answers about a `default()` for the whole canvas frame.

`annotdelete::refuses(doc, selection)` takes the selection by argument
(`canvas/interact.rs:1161`); `refuses_selected(doc)` survives as its one-line
wrapper for `app::conditions`, which runs in the panel pass where the document's
selection is intact. The regression test puts the object in a **detached**
`SelectionState`, leaves `doc.selection` empty, and asserts `refuses` still says
yes — both directions, so a gate that refused unconditionally fails it too.

**A convenience overload that reaches for state through a long path is a trap
when any caller holds that state detached.** `doc.selection` reads like a fact
about the document; inside a canvas frame it is a fact about a temporary. The
remedy is to put the borrow in the signature, not to remember.

Two consequences that outlive the fix: a gate whose input is filled at a call site
is not tested by any test of the gate, because every such test sets the input by
hand; and a driven check that presses a key must press **until the trace shows the
key was heard** (`driving::press_until_traced`), so a key that never arrived is a
SKIP rather than a confident accusation against code that is fine.

### D20 — RULE: only content-bearing streams can refuse a redaction

Every stream in the finished bytes is decoded and searched — narrowing the sweep
hides evidence. What a hit can *produce* depends on where it is: only a page
content stream, a form XObject (which is what an annotation appearance stream
is), a tiling pattern or a Type 3 glyph procedure can refuse the write.
`redact::proof::role_of` classifies them. Everything else — font programs, image
samples, ICC profiles, object-stream containers, attachments — is promoted into
the disclosure list **with the site named**, behind the acknowledgement gate, so
nothing that used to refuse now passes silently.

The mechanism: a byte run in a place nothing draws is a coincidence pdfcer cannot
rule out. That is why `MIN_VERIFIABLE_LEN` exists for the raw-byte half, and the
decoded half must apply the same reading to the same evidence. Without it, an
embedded font's own `name` table vetoes any redaction of any ordinary English
word on any document carrying an embedded font.

**The fixture that exercises the feature and the fixture that resembles the
operator's work are not the same fixture, and a suite needs both.** Fixtures
assembled so that every byte is one the suite put there are right for what they
assert and share the property that there is nothing in them for a coincidence to
hide in. `redact::tests::a_real_drawing_sheet_with_an_embedded_font_is_applied_rather_than_refused`
runs the whole pipeline — mark, apply, write, re-extract — on
`fixtures/a1-titleblock.pdf`, with a negative control so a build that blanked the
page fails it.

The dangerous repair is the one that stops refusing *and* stops telling. It is
invisible to uncompressed fixtures, where a dropped hit merely degrades to a raw-
byte residual; only a real document, whose font program is compressed, shows the
empty list.

### D21 — RULE: a refusal is worded in the slot that means "nothing happened"

The disclosure channel draws as *"About your last edit: …"*, truncated, past
tense, about a *previous* edit. A refusal routed there after a press where nothing
happened tells the operator a small lie confidently. **A sentence in the wrong slot
is indistinguishable, from the operator's chair, from no sentence at all.**

`crate::text::textedit::ReflowRefusal` carries one sentence per cause, routed
through `app::status::decline::record_reflow` so they read as *nothing happened*.
The engine's `ReflowApplyError` is mapped into the same enum through
`Result::inspect_err` **inside** the funnel's closure, which works because
`vector_edit` takes the decline floor *before* running it and `BeforeTheVerb`
fills the slot only if it is empty. A single collapsed refusal — nine cause-free
words for four causes with four different remedies — is the failure this
replaces.

The tooltip leads with the preconditions, before the press, per R9.

**A workaround kept past its cause rots, and it is not inert while it rots.** The
shell's pre-flight `edit_epoch` gate forecast an engine refusal, was correct when
written, and cost the operator reflow on every page he had touched — including the
ones that were always safe. It is gone, and it was **deleted rather than
narrowed**: narrowing it would have been a second implementation of the engine's
own predicate, in a second crate, over the same `/Contents` list — two
self-consistent predicates over one model, with no test of either able to see them
disagree. The engine owns the question; the shell words the answer
(`app/actions/textstyle.rs:1156`).

### D22 — RULE: one key, one meaning, everywhere

Enter means a new line in every text draft. `Ctrl+Enter` commits, so commit is
never mouse-only; Escape abandons and clicking away commits. Where a line break
cannot go — a caret inside an existing show operator — Enter **declines in words**
and leaves the draft alive, because that is the file's rule and not a shortcoming
to hide: `edit_text` re-encodes into the run's own font, a line feed has no code
in any standard encoding, and the engine refuses it by name.
`canvas::textedit::keys::enter_means` is a pure function, so *"the whole
interaction, not half of it"* is checked rather than asserted in a comment.

The half that is easy to miss: a multi-line draft needs caret motion. Up, Down,
Home and End must reach lines, not just the ends of the whole draft; shipping the
line break without them is a multi-line editor you cannot move around in.

A clicked draft that gains a line break is promoted to a **boxed** request at the
commit, with the box taken from the page's own crop box — the click across to the
right edge and down to the bottom. Nothing is invented, and the promotion is
disclosed. A one-line click still takes the point path byte for byte.

### D23 — RULE: geometry is planned against a frame record about the right page

The scroll offset moves on the frame the page turns; the strip's visible set does
not, because `canvas::strip` chooses which pages to lay out from the offset the
frame *inherited*. So for exactly one frame the canvas is scrolled to the
destination and still drawing the page it left, and `zoom::CanvasFrame` — written
from that layout — describes the wrong sheet. Anything that then frames a rect
stamps the wrong page into its anchor, and the solver honours it exactly.

`canvas::destination::arrive_step` is the gate (`canvas/destination.rs:213`):

| condition | step |
|---|---|
| the view is on another sheet | `Drop` |
| the frame record is about the destination's page | `Frame` |
| it is about another page, or there is no record yet | `Hold`, up to `MAX_WAIT_FRAMES`, then `Drop` |

`arrive` **reads** the parked destination rather than taking it, and calls
`ctx.request_repaint()` so a reactive shell cannot leave it parked until the
operator jogs the mouse. The hold is bounded because a destination held for ever
is spent later on an unrelated layout change — a view springing to a bookmark
clicked a minute ago.

The repair is on the destination path and **not** in `frame_rect`, which is shared
by the zoom marquee, `view.zoom_selection` and every bookmark: the only fresher
geometry available inside a draw is geometry the draw is still deciding, and
feeding a layout back into a fit-to-viewport zoom is R128 exactly. The gate's
question — *"is the frame record about my page?"* — is true on the first frame for
every other caller, because a marquee and a selection are by construction on the
page being drawn.

Four lessons kept, each one general:

- **A check that names a cause is naming a hypothesis.** Read the trace, not the
  verdict.
- **A trace that reports an intent and calls it an outcome will be believed.**
  `destination-arrive page=3 framed=true` was true and useless; the line now
  carries `step=`, `frame_page=` and `waited=`.
- **An ordering argument written about statements is not a claim about frames.**
  Source order is honoured and still defeated by a one-frame lag between deciding
  a zoom and applying it, unless everything in the queue lands together.
- A check that passes against a defect may be structurally unable to find it. The
  bookmark fixture's targets are on the sheet already showing, so they take
  `Frame` on their first frame.

Open design question, not a bug: a `/FitH` destination raises both a fit and a
point, and `arrive` frames a point as a 150 pt box
(`geometry::DESTINATION_CONTEXT_PT`), which can override the fit the same
destination asked for. §12.3.2.2 says `/FitH` means *fit the width and put `top`
at the top edge*, and Acrobat keeps the current magnification for an `/XYZ` with a
null zoom. `canvas::destination`'s header argues for the framing deliberately
(*"adding a second scroll-to-a-point solver would be two answers to one
question"*), so this wants its own driven pass: it would move every `/XYZ` and
`/FitH` bookmark.

### D24 — RULE: a keystroke is not a harness step while a dock panel is open

A chord is routed through whatever holds keyboard focus, and an open panel holds
it. In a harness a chord after a panel open is **intermittent** — the same step
arrives on one run and never on the next — which is the worst available property,
because every failure it produces is a confident, specific accusation against code
that is fine. The remedy is not a longer wait or a retry loop: **click the
command** (for the select tool, `ribbon.item.view.tool_select`) instead of
pressing its chord.

`tools/ui-verify/src/input.rs:148,1030` records the focus-follows-click mechanism.

OPEN: checks that still press a chord after opening a panel have not all been
converted. Find candidates with

```sh
grep -rln 'panel' tools/ui-verify/src/checks/*.rs | xargs grep -ln 'press_chord\|press('
```

and read each one's step order.

---

## Constraints

### D4 — CONSTRAINT: the edit unit is one show-text operator

`PendingEdit` pins to one run, and a `TJ` array is one operator. A visual
paragraph split across several `Tj` runs — the ordinary output of CAD title
blocks, Word and LibreOffice — is edited run by run. A selection spanning runs is
**declined in a sentence** (`crate::text::textedit::spans_runs`) rather than by a
keyboard that silently stops responding, and the caret and extent bracket claim
only what the shell can honestly know before a commit — no ghost glyphs in a
substitute typeface at the wrong widths. `canvas::textedit::preview` carries the
argument for why a prettier ghost is the wrong fix rather than a deferred one.

Lifting it needs a multi-run edit request in the engine that groups runs into a
line or block and re-emits them as a set.

**Nothing moves as you type, and that is deliberate.** There is no core call per
keystroke; real layout runs once, at commit. The blocker is not the arithmetic —
`plan_edit`/`EditPlan` already computes `advance_delta` before any write — but its
visibility: it is `pub(crate)`, and every public route either performs a full
incremental save or mutates the undo log. The engine request is a dry run,
`measure_edit(&Document, &EditRequest) -> Result<f64, _>` or simply making
`plan_edit` public (`canvas/textedit/cost.rs:88-91`). Re-measure the cost with
`canvas::textedit::cost`; on the operator's own sheets the current route is
multiple frames per keystroke.

Debouncing was rejected, not overlooked: a re-layout that arrives after you stop
typing is a *second* surprise, and this feature's sin is showing the operator
something the document will not say. Until the draft can move truthfully on every
keystroke, it does not move at all.

**Disposition on commit** is `canvas::textedit::disposition` — a pure
`choose(text_matrix, ctm, alignment) -> Reason` consulted at the single commit
site. Rotation is rung 1 and outranks alignment; non-left alignment is rung 2.
`Pin` is *correct* under rotation rather than merely less wrong: it writes no
follower `Tm` at all and its compensating `TJ` acts in text space, along the
rotated baseline, so the ported guard selects `Pin` for rotated text instead of
refusing the edit. A **single-line** right-aligned block still reflows, because
alignment is inferred from the agreement of several lines' edges and one line
cannot disagree with itself.

**Reflow is operator-invoked, never automatic on edit** (Decision 015 §3.3, R75):
re-wrap invents line breaks the file never stated. Its refusals that hit real CAD
and Word content are text inside a form XObject, more than one font resource in
the block, rotated or skewed `Tm`/CTM, mixed font sizes, and composite/CID fonts.
The limit that matters more than any of them is tokenisation: word breaks are
found at **real U+0020 space glyphs only**, and producers that position words with
`Td`/`TJ` offsets instead — extremely common in CAD output — present reflow with
one unbreakable word. Treating a derived word space as a break opportunity is the
change that would matter.

The auto-detected wrap width is taken from the block bbox, which is a union over
its lines, so one over-long line has already widened it. An operator who does not
read the disclosure gets a re-wrap to a width they never chose.

**Fixture gap, still true:** no fixture has a paragraph split across many runs,
mixed sizes or fonts in a block, rotated text, or words separated by positioning
rather than space glyphs. Every condition that fails in the field is absent by
construction.

### D15 — CONSTRAINT: `ocrs` collapses on a sparse clean page

`TextDetectorParams::default()` sets `text_threshold: 0.2`, under upstream's own
comment that *"ideally the threshold would be 0.5 as a neutral value"*. On a page
that is mostly empty paper the measured background straddles it, so the whole page
binarises as text and detection returns one rectangle. The probability map is
fine; the failure is downstream, in thresholding
(`crates/pdfcer-gui/src/ocr/fixture.rs:157-182`).

**A drawing sheet is exactly that shape** — a small title block, a handful of
callouts, a very large expanse of empty paper — so an operator OCR-ing a scanned
drawing is the most likely user of this feature and walks into its worst case.

The **fixture** was changed, not the threshold. Tuning a recogniser's internals to
make a test pass is how a shell starts carrying an engine's opinions: the number
would be ours, the failure would still be theirs, and the next `ocrs` release
would silently disagree.

Unquantified on real scanned material, because there is none in the tree. If a
scanned drawing arrives, measure this first; if it reproduces, the honest fix is
upstream or a documented refusal, not a magic number here.

### D32 — CONSTRAINT: engine line-number citations rot, and a rotted one protects its claim

`pdfcer-core/src/edit.rs` is tens of thousands of lines and **grows at the head**,
so every hard-coded line citation into it drifts downward and nothing in this
repository detects it.

```sh
grep -rnoE '(edit|document|page_tree|text_extract|settings|pageops)\.rs:[0-9]+' crates/ tools/ | wc -l
```

A rotted citation does not merely fail to support its claim: the reader who checks
it finds plausible code at that address and stops. The worked example is a
`style.dash` guard whose cited range has become embedded-file-stream and name-tree
documentation — the substance still right, only the address wrong, which is
precisely why it is invisible.

**Cite by symbol name, or pair the line with the engine pin.** OPEN: the existing
citations have not been swept; a sample of eight all landed on unrelated code.

---

## Open defects

### D25 — OPEN: `dock.<side>.body_min` is a trace region nothing consumes

Published at `crates/egui-shell/src/dock/mod.rs:803` and documented at
`crates/egui-shell/src/dock/overflow_probe.rs:20`. Delete the publication and the
lines in `draw_side` that feed it.

Do **not** delete `overflow_probe` itself or `RESIZE_FLOOR_PT`
(`crates/pdfcer-gui/src/canvas/fit.rs:128`): both hold a measured floor the dock's
overflow behaviour depends on.

### D26 — OPEN: `canvas.selection-outline` is published twice per frame with two rectangles

`crates/pdfcer-gui/src/canvas/overlay.rs:374` publishes the turned-grip frame at
loop entry; `overlay.rs:439` publishes `grip_box`; `canvas/forms/selecting.rs:269`
is a third site. Each carries a documented argument for using the shared region
name and each argument is individually sound, which is why nothing has caught it:
a harness reading the trace gets whichever was published last.

The shared-name decision needs **re-deciding**, not just renaming — the consumers
are `canvas/handles.rs:84`, `canvas/overlay/anchors.rs:275` and
`canvas/painting.rs:474`.

### D27 — OPEN: `MAX_MAX_ZOOM_PERCENT` is a trillion where its own comment says a hundred billion

`crates/pdfcer-gui/src/app/prefs/mod.rs:204` declares `1e12`. The doc comment
immediately above it (`:186-195`) calls the value *"a hundred billion percent,
which is the deepest zoom the page has been confirmed to actually DRAW at"* and
says it is *"set an order of magnitude inside the confirmed-working range rather
than at the edge of it"* — and records the measurement it rests on: drawn at
8.6e9x, not drawn at 1e10x. The literal is at the edge, not inside it, and
contradicts the measurement it sits on.

`DEFAULT_MAX_ZOOM_PERCENT` aliases it (`:176`), so a fresh install ships at the
disputed rung.

### D28 — OPEN: `viewpos.rs:195` quotes a jump size for the wrong extent

`crates/pdfcer-gui/src/canvas/viewpos.rs:195` reads `// 2,048-pixel jumps.` for a
trillion-percent zoom. 2,048 is the `f32` ulp at about 2.05e10 — the extent
`viewer/ceiling.rs` actually drove, which is 2.6 **billion** percent on a Letter
sheet. At a trillion percent the extent is 7.92e12 and the ulp is 2^19 = 524,288
px, so the comment is wrong by 256x.

Repair: quote the content **extent**, not a zoom, since the ulp is a property of
the extent.

### D29 — OPEN: the sub-pixel hand-over threshold is stated as a stale percentage

The predicate is `longest * zoom > SUB_PIXEL_CONTENT_EXTENT`
(`crates/pdfcer-gui/src/viewer/ceiling.rs:291`) and the constant is `1_048_576.0`
= 2^20 (`ceiling.rs:50`) — about 132,000 % on US Letter and about 86,000 % on a
large sheet. Four prose sites still say *"about two million percent"* or
*"~1,000,000 %"*, which was right for 2^24:

- `crates/pdfcer-gui/src/canvas/deep.rs:6,8-9`
- `crates/pdfcer-gui/src/viewer/ceiling.rs:10`
- `tools/ui-verify/src/checks/zoom_keeps_place.rs` and
  `tools/ui-verify/src/checks/zoom_out_keeps_place.rs` — in their `detects:`
  lines, so the stale figure prints in **every sweep report**.

Repair: cite `SUB_PIXEL_CONTENT_EXTENT` rather than restating a derived
percentage. A percentage derived from a constant is a claim that decays the moment
the constant moves, and `ceiling.rs:53` records the move in the same file whose
header was never updated.

### D30 — OPEN: "five panel tabs" is written where the rail declares six

`RailFold::Never` now holds `view.panel_pages`, `view.panel_bookmarks`,
`view.panel_layers`, `view.panel_signatures`, `markup.comments` and `file.fonts`
(`crates/pdfcer-gui/src/shell/manifest/rail.rs:119-182`). Live prose sites:

| site |
|---|
| `crates/egui-shell/src/dock/rail.rs:62`, `:808`, and the assertion string at `:1043` |
| `crates/egui-shell/src/manifest/rail.rs:68` |
| `crates/pdfcer-gui/src/shell/manifest/rail.rs:22` |
| `crates/pdfcer-gui/src/shell/manifest/mod.rs:278` |
| `crates/pdfcer-gui/src/app/rail.rs:4` |
| `tools/ui-verify/src/checks/left_rail.rs:34`, `:74`, `:230`, `:329` |
| `tools/ui-verify/src/checks/reaching.rs:157` |

Re-measure with `grep -rn 'five panel\|all five panels' crates/ tools/`.

This is the shape where **the file that changed does not contain the number that
went wrong**. The rule the count is protecting is the real content — the rail may
never fold this group — so state the rule and let the count come from the
manifest.

### D31 — OPEN: "the engine cannot create a document" is false

`crates/pdfcer-gui/src/app/blank.rs:9` heads a section *"1. The engine cannot
create a document, and that is deliberate"*, echoed at
`crates/pdfcer-gui/src/app/lifecycle.rs:366-367`. The engine has
`pdfcer-core/src/text_edit/placetext.rs:1375`, `pub fn blank_document(`.

The claim is load-bearing: it is the stated reason this shell synthesises blanks
from a 443-byte template asset rather than asking the engine. Either the reason
changes or the sentence does.

### D33 — OPEN: `wheel_toggle` copies one of its predicate's two clauses

`crates/pdfcer-gui/src/app/status/page_box.rs:234` guards on
`doc.view.display.is_continuous()` alone. The authoritative predicate is
`canvas::paging::flips_pages` (`crates/pdfcer-gui/src/canvas/paging.rs:69-70`):

```rust
doc.prefs.wheel_paging.flips() && !doc.view.display.is_continuous() && doc.pages.len() > 1
```

Consequence: on a single-page document the wheel-paging toggle draws, highlights
and accepts clicks and can never act. `paging.rs`'s own header states why there is
one spelling — two would eventually differ, and the frame where they did would
either scroll *and* page at once or do neither.

### D34 — OPEN: the redaction search-and-mark route is not driven

Redaction is the one operation that cannot be undone, and R1 makes the driven
harness the only oracle that counts. The mark-by-search route needs a query typed
into a field, and synthetic keystrokes do not reach the target window from the
session that writes them on this machine — so its rule is unit-tested and the
field itself is verified by nothing.
`tools/ui-verify/src/checks/redaction.rs:85` says so in its header rather than
leaving it to be discovered.

### D35 — OPEN: an author-imposed signing refusal publishes no named region

A refused signing emits only `sign-applied written=0`
(`crates/pdfcer-gui/src/app/actions/sign.rs:150`), which proves *a* refusal and not
*which* one. The wording is covered by unit tests over `worded()`
(`app/actions/sign.rs:283`) and by nothing driven. A named region on the failure
label closes it in about ten lines.

### D36 — OPEN: `/BleedBox`, `/TrimBox` and `/ArtBox` overhang is not disclosed on a sheet resize

`MediaBoxChange` has a field for `/CropBox` overhang and none for the other three
(`D:\Dev\pdfcer\crates\pdfcer-core\src\edit.rs:3151-3171`). Measured: a
`/BleedBox [10 10 1000 1000]` survives a resize to 595x842 with no disclosure, so
a press or CAD export gets one overhang reported and three not.

`FEATURES.md:1136` records that the three boxes are left byte-identical without
drawing the consequence. This is an engine gap and is **not** in
`ENGINE_BACKLOG.md`; it belongs there.

### D37 — OPEN: raster images are box-hit-tested, not alpha-tested

`D:\Dev\pdfcer\crates\pdfcer-core\src\vector\hit.rs:708` dispatches
`VectorObject::Image(i) => i.page_bbox.inflate(tolerance).contains(point)`, while
the Path and Text arms call real geometry predicates. So a click anywhere in a
transparent image's bounding box selects the image, and a click on visible ink
underneath it loses.

This is the half-closed row of form recursion: the deep hit test descends into
form XObjects correctly, and the leaf predicate for an image never tightened.

### D38 — OPEN: `summary::describe_object` runs per visible row on every frame with no cache

Called inside the row map at `crates/pdfcer-gui/src/panels/objects/mod.rs:797` and
`:829`. Its own doc (`panels/objects/summary.rs:395`) states it counts every
anchor of every subpath — which is why the cheap classifier `object_kind`
(`summary.rs:405`) was split out of it and is used at `mod.rs:333`. On the
benchmark CAD sheet the heaviest objects carry several thousand anchors each, so a
scroll over those rows re-counts them every frame. Re-measure with `BENCHMARK.md`'s
sheet.

### D39 — OPEN: two driven checks were reported unable to detect their own defect and never identified

An adversarial review of nine author-written driven checks found three that cannot
detect the defect they exist for and one that cannot pass at all. That review's own
addendum referenced two **further** unrun checks with the same property, and
neither was ever named.

Re-run the review over `git log 539835f^..HEAD` before trusting that part of the
suite.

---

## Do not weaken

- `resize_scales_a_shape` targets a **page-sized** selection deliberately: it is
  the regression test for the floating-scrollbar defect. Narrowing its target to
  something more convenient silently retires that coverage.
- `overflow_probe` and `RESIZE_FLOOR_PT` hold a measured floor the dock's
  overflow behaviour depends on. See D25.
