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

**Before a row is written, prove the measurement was of the thing the row
names.** A defect report is itself a claim, and it gets the standard this
file applies to everybody else’s. The failure is not rare and it is not
obvious from inside: D39 is two driven checks that reported being unable to
detect their own planted defect, and the reason went unidentified because
the reports were believed. The questions that catch it are *what did the
instrument actually sample*, *would the healthy case produce this same
reading*, and *does a control that differs in exactly one property still
reproduce it*.

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

### D13 — RULE: a source scanner's skip is bounded, and a scanner that crashes is not clean

A skip that begins at a marker and never ends converts a checker into a
formality. **508 files here carry a column-0 `#[cfg(test)]`, and 66 of them
declare shipped items below it** — so a scanner that stops at the first one
reads almost nothing and prints the same words as one that read everything.

The convention cannot be the remedy, because the files are not breaking one.
The dominant shape is `#[cfg(test)] mod tests;` — a one-line declaration whose
body lives in a sibling file — sitting with the other `mod` declarations near
the top. "Keep the test module last" does not describe it: there is no test
module in those files to be last, and the declaration gates out nothing.

The three parts of the repair, each of which a `--self-test` assertion
falsifies on its own:

- **Inspect the attribute line.** A `;`-terminated declaration skips that line
  only. A braced item is skipped to its close, in both spellings — attribute on
  its own line, and attribute and item on one line. A skip that never starts
  and a skip that never ends both leave a bare "does the dirty fixture fail?"
  assertion green, so each is planted separately.
- **End the skip at `}` in column 0**, never a brace counter, which desyncs
  silently on a brace inside a string literal and is confidently wrong for the
  rest of the file.
- **Drop a cfg-gated FILE whole.** `#[cfg(test)] mod X;` names a file the
  compiler never emits into a release build. The in-file skip cannot see that
  — it reads the named file from line 1 as shipped source — so the declaration
  is resolved to `<name>.rs` / `<name>/mod.rs` and that subtree is excluded.
  In this tree that is the difference between 24 reported literals and 5.

**A scanner that crashes must not report clean.** The gate captured `awk` into
`$( )` and discarded both its stderr and its status, so a syntax error in the
scanner made every file fail to parse and the run still exited 0 — the same
fail-open shape, arriving through the tool rather than through the rule. A
nonzero `awk` is now fatal, with a headline that says the fault is in the gate
rather than in the tree.

`icons::glyphs` has an independent implementation of the same job and asserts
the same property itself, in
`a_mid_file_test_module_does_not_blind_the_scanner`.

Enforced by `tools/gates/check-ui-strings.sh --self-test`.

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

### D47 — RULE: a destination that names a point scrolls, and never sets the magnification

§12.3.2.2 Table 151 lets a destination leave any of `left`, `top` and `zoom`
unstated, meaning *leave that one as it is*, and states the 0-means-null
equivalence for `zoom` alone. So a `/XYZ` with a null zoom — the shape every
Word table-of-contents link has — asks for a position and for no magnification
whatsoever, and a `/FitH` or `/FitV` names exactly one magnification, its own.

The arrival is decided per axis: an unstated axis defers to whatever placed the
view this frame; a stated horizontal already on screen **from where the operator
was looking when they clicked** is held there; any other stated axis puts the
point one `CANVAS_MARGIN` inside the viewport edge. The vertical carries no
visibility test, because a destination is normally reached by a click on the
page the operator is already reading and a vertical that declined to move would
make the link do nothing.

`canvas::destscroll::solve` is the mechanism, and it is a pure function of the
parked destination, the point in strip space, the wanted offset, the current
offset and the viewport. It returns an offset and which axes it held. **It
cannot raise a zoom because it does not return one** — the property that makes
the rule structural rather than a convention.

"Where the operator was looking" is `OpenDoc::dest_origin_x`, recorded by
`app::actions::view` on the frame the verb is raised.
`canvas::strip::page_scroll_offset` brings the named page into view and centres
it horizontally on the frame the page turns, so an origin read any later
measures that centring instead of the operator.

Enforced by `canvas::destscroll`'s unit tests — planting the pre-repair
behaviour turns `a_point_destination_can_only_answer_with_a_position` and
`a_visible_point_holds_the_horizontal_where_the_operator_left_it` red, and
nothing else — and by `tools/ui-verify/src/checks/point_destination.rs` over
`fixtures/xyz-null-zoom.pdf`, whose two links differ only in the destination's
`left`, so "held" has a control and "moved" has a witness.

---

### D66 — RULE: a text sweep's far end is clamped to the text, never cancelled

`EditableTextModel::hit_test` answers only within **one line-height of a line's
box on every side**, so a drag whose pointer runs past the end of a run, out
into a margin, or across the blank middle of a drawing sheet is asking it about
a point it will not resolve. A sweep is not cancelled by that.
`canvas::textsel::drag` uses the furthest point along `from`..`to` that the
engine still resolves as the focus, and keeps the selection.

The reach is a line's own **height**, not its font size — a vertical run of
12 pt glyphs standing 75 pt tall reaches 75 pt sideways — which is why a point
chosen by eye as "past the text" often is not, and why the tests probe the
fixture's measured line boxes before they aim.

`canvas::textsel::clamp_to_text` scans **backwards from the pointer** in 32
steps and then sharpens by 8 halvings toward the far edge of the text. Backwards
rather than a bisection from the anchor because the reachable set along a drag
is not an interval: a sweep crossing the gap between two columns leaves reach
and re-enters it, and a bisection seeded at the anchor would stop at the near
edge of the gap and silently under-select. The engine stays the only oracle for
where text is; nothing in the shell re-derives its geometry.

Enforced by `canvas::textsel`'s unit tests over `fixtures/rotated-text.pdf` —
reverting `drag` to `hit(to)?` turns `overshooting_the_end_of_a_line_keeps_the_selection`
and `the_clamp_finds_the_furthest_text_the_drag_passed_not_the_nearest` red and
leaves `a_sweep_begun_off_the_text_still_selects_nothing` green — and by the
three driven text checks over `fixtures/layered-drawing.pdf`, where a full-width
band on a 2,384 pt sheet leaves reach one sixth of the way along a 396 pt note.

---

### D67 — RULE: a gesture's outcome is read from the settled trace line, not the last matching one

`canvas-text-selection` is traced at every distinct state a drag passes through,
clears included. An instrument that filters the event on `chars > 0` and then
takes `.last()` reports the last state the gesture was ever *in*, which is not
the state it *left*. The two differ on exactly the gestures that matter, so such
an instrument cannot tell *the application is broken* from *my own gesture ended
off the text* — and what it does instead is click a correctly greyed control and
report the feature dead.

`checks::text_selection::settled` answers the last line **unfiltered**, paired
with the number of such lines so a caller can distinguish a gesture that said
nothing from one that said `chars=0`. `settled_selection` is the presence form:
new since a recorded count, and non-empty at rest. All three band ladders read
them, and a ladder that misses says which of the two it was by printing the
settled line.

The *absence* phases keep the filtered form deliberately — "no non-empty
selection appeared at any point in the gesture" is the stronger claim, and it is
the right one when the assertion is that a gesture produced nothing at all.

★ The rule generalises past this event. Any check whose subject is **what a
gesture left behind** must read the trace unfiltered and take the last line: a
filter on the property being asserted makes every intermediate state look like a
result.

Enforced by `checks::text_markup`'s
`a_sweep_that_ends_cleared_has_selected_nothing`, whose two synthetic traces
differ only in which line is last.

---

### D32 — RULE: a citation into the engine names a symbol, and never a line

`pdfcer-core/src/edit.rs` is tens of thousands of lines and **grows at the head**,
and the engine is pinned by *branch*, so a line citation into it drifts with no
event on this side at all — no `cargo update`, no commit here, nothing to notice.

The rule's mechanism is `tools/gates/check-engine-citation.sh`, registered in
`run-all.sh`, which fails on four shapes: a path naming the engine or the
archived GUI; an `ENGINE:`-tagged citation carrying a line number; any `.rs:N`
whose N exceeds `check-file-size.sh`'s limit, since no file here may be that
long; and a bare `` `:N` `` continuing the citation before it. A vendor citation
survives only version-anchored (`egui-0.35.0/src/style.rs:1135`), and an
evidence artifact is exempt only where it declares the revision it was measured
at. The gate states its own blind spot: a bare citation into a *small* engine
file is indistinguishable from a local one, and it is the shape a reader will
not re-check.

**What makes the class dangerous is that it does not dangle.** Upstream
insertion shifts a file uniformly rather than scrambling it, so the drifted
number lands inside readable prose about a real function in the right file and
reads exactly as a correct citation reads. Measured across the living documents
and the forms parity table: of 92 engine citations, 66 had drifted and **not one
dangled**; two pairs had come to name each other's type. A repair keyed on *does
this line exist* therefore confirms every wrong one — resolve the enclosing
symbol instead.

The same argument covers the archived GUI at `D:\Dev\pdfce\crates\pdfce-gui`:
a frozen tree is
not frozen-correct, because its citations kept drifting until the freeze, so
what froze was the error.

### D28 — RULE: an `f32` step size belongs to the CONTENT EXTENT, not to a zoom

The canvas's scroll offset is an `f32` over a content space of `page x zoom`
where one unit is one screen pixel, so the spacing between representable
positions is the `f32` ulp **at that extent**. Quoting it against a zoom instead
hides the page size, and a comment that did so was wrong by 256x: 2,048 px is
the ulp at an extent of 2.05e10, not at the trillion-percent zoom the sentence
named.

**State the extent, and state the sheet any percentage was derived on.** Two
documents at the same zoom are at different extents, so a step quoted against a
zoom is true of one sheet and false of the next.

### D29 — RULE: a zoom percentage derived from a constant is not written down

The sub-pixel hand-over predicate is `longest * zoom > SUB_PIXEL_CONTENT_EXTENT`
(`crates/pdfcer-gui/src/viewer/ceiling.rs`), and the constant is `1_048_576.0`
= 2^20 — about 132,000 % on US Letter and about 86,000 % on a large sheet. Four
prose sites had gone on quoting *"about two million percent"* and
*"~1,000,000 %"*, the figures that were right while the constant was 2^24.

**Cite `SUB_PIXEL_CONTENT_EXTENT`; derive the percentage only where a reader
needs a feel for the magnitude, and name the sheet it was derived for.** A bare
percentage is a claim that decays the moment the constant moves, and the file
that moves the constant does not contain the percentage — so nothing recomputes
and nothing goes red.

Two of the four sites were `detects:` lines, which print the stale figure in
**every sweep report**: a wrong number gains readership as it ages.

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

### D27 — OPEN: the maximum-zoom ceiling is the first rung measured not to draw

`app::prefs::MAX_MAX_ZOOM_PERCENT` is `1e12` — a trillion **percent**, which is
a zoom **factor** of 1e10. Driving the real binary found a US Letter page drawn
at 8.6e9x and **not** drawn at 1e10x, so the permitted ceiling is exactly the
rung that fails. `DEFAULT_MAX_ZOOM_PERCENT` aliases it, so a fresh install ships
at that rung.

The raster is not what fails: the strip renders with no failed tiles and shows
no page, because the strip's extent is still `page × zoom` in `f32` and reaches
6e12 points there.

Two resolutions, and choosing between them is the operator's: lower the constant
into the confirmed range, or stop building the strip in `page × zoom` space at
deep zoom. The constant's own doc comment states the measurement and names this
defect; what is open is the code change, not the description.

### D30 — OPEN: the Comments rail tab's reachability is asserted in no mode

`RailFold::Never` holds `view.panel_pages`, `view.panel_bookmarks`,
`view.panel_layers`, `view.panel_signatures`, `markup.comments` and `file.fonts`
(`crates/pdfcer-gui/src/shell/manifest/rail.rs`). `TABS` in
`tools/ui-verify/src/checks/left_rail.rs:75` lists five of the six and omits
`rail.tabs.markup.comments`, so the check whose whole subject is *"the one group
the rail may never fold"* does not cover the entry added most recently.

Repair: add the id to `TABS`. The assertion strings quote their own count, so
they follow from the array rather than needing a second edit.

No comment in `pdfcer-gui` states the **rail group's** count any more. That was
the shape where **the file that changed does not contain the number that went
wrong** — the rule the count protected is *the rail may never fold this group*,
so state the rule and let the count come from the manifest. Counts of other
sets survive and are correct: `app::modes::defaults` says Edit's left side holds
five navigators, which is a different set and a different claim.

The assertion strings in `left_rail.rs` still describe the group by the size of
this check's subset. They follow from `TABS`, so the repair above fixes them.

### D31 — OPEN: "the engine cannot create a document" is false

`crates/pdfcer-gui/src/app/blank.rs:9` heads a section *"1. The engine cannot
create a document, and that is deliberate"*, echoed at
`crates/pdfcer-gui/src/app/lifecycle.rs:366-367`. The engine has
`pdfcer_core::text_edit::placetext::blank_document`, `pub fn`.

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

`MediaBoxChange` has `crop_box_outside` for `/CropBox` overhang and no field
for the other three. Measured: a
`/BleedBox [10 10 1000 1000]` survives a resize to 595x842 with no disclosure, so
a press or CAD export gets one overhang reported and three not.

`FEATURES.md`'s `pages.resize` row records that the three boxes are left
byte-identical without
drawing the consequence. This is an engine gap and is **not** in
`ENGINE_BACKLOG.md`; it belongs there.

### D37 — OPEN: raster images are box-hit-tested, not alpha-tested

`pdfcer_core::vector::hit`'s `object_hit` dispatches
`VectorObject::Image(i) => i.page_bbox.inflate(tolerance).contains(point)`, while
the Path and Text arms call real geometry predicates. So a click anywhere in a
transparent image's bounding box selects the image, and a click on visible ink
underneath it loses.

This is the half-closed row of form recursion: the deep hit test descends into
form XObjects correctly, and the leaf predicate for an image never tightened.

★ **The form half was closed by EXCLUSION, and that is why this one cannot
reuse it.** A form is not its own variant — it is `VectorObject::Image` with
`source == ImageSource::Form` — so `hit_test_point_deep` closed its half with a
`continue` over exactly that pattern, on the argument that a form’s `/BBox` is
an extent declaration rather than ink. The arm this row asks to be tightened is
therefore **the same arm forms travel through**, minus the `continue`. Closing
D37 means writing an alpha test against the image’s own samples; there is no
second exclusion available, because a raster image genuinely is a candidate.

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

### D40 — OPEN: the OCR tooltip promises a frozen window, and the window does not freeze

`crates/pdfcer-gui/src/text/ocr.rs:302` ships *"It takes a few seconds per page,
and the window will not respond while it does."* The recogniser runs on a
detached worker (`crates/pdfcer-gui/src/ocr/job.rs:115` spawns it, and
`crates/pdfcer-gui/src/ocr/mod.rs:47` describes the shape as *"a `std::thread`
plus a channel"*), and the dialog draws a spinner, a page count, **Stop** and
**Cancel** while it runs.

The sentence therefore tells the operator the one thing that would stop them
starting a long run, and it is false. Fix is a string change in `ui_text`, not a
behaviour change.

### D41 — OPEN: every resize refusal is recorded at epoch 0 and is never readable

`crates/pdfcer-gui/src/canvas/resizing.rs::decline` calls `record_note` with a
literal `0` where every other caller passes the document's epoch.
`crate::app::actions::last_edit_disclosure` is an **equality** filter
(`d.epoch == epoch`, `crates/pdfcer-gui/src/app/actions/disclosure.rs:110-116`)
and the status bar passes `doc.edit_epoch`, so the note is readable only until
the document's first edit. From then on every refused resize is recorded and
invisible for the life of the session.

The site carries a long note saying exactly this, which is why it is easy to
mistake for handled. It is not: the symptom is **silence**, and nobody reports
silence. A fixed sentinel cannot express *"retire on the operator's next act"* —
that needs an ordering the filter does not have.

### D42 — OPEN: a pasted object does not arrive selected; a placed one does

`crates/pdfcer-gui/src/app/actions/apply.rs` calls
`doc.selection.select_placed(..)` after a place, so the operator can move or
style what they just put down. `VectorAction::PasteObjects`
(`crates/pdfcer-gui/src/app/actions/vector.rs`) never touches `doc.selection`,
so a paste lands unselected and the next gesture starts from nothing.

Every application in the class selects what it just pasted. The engine returns
`PasteOutcome::objects_pasted`, and the paste appends, so the indices are
derivable the same way the place path derives its one.

### D43 — OPEN: arrow-key nudge moves annotations only, and `MANUAL.md` says otherwise

`crates/pdfcer-gui/src/canvas/moving/nudge.rs` is built entirely on
`EditSession::move_annotation`. `MANUAL.md:836` tells the operator the arrow
keys *"Nudge what is selected one point — a quarter point with Ctrl"*.

With a content object selected the keys do nothing and say nothing, which reads
as a dead keyboard rather than an unsupported operand. Either the manual row
names annotations, or the nudge routes a content selection through the transform
verb the drag path already uses.

### D44 — OPEN: a resize grip is offered on objects the engine refuses to transform

`crates/pdfcer-gui/src/canvas/resizing.rs:172-176` records that the preflight is
**not built**. `transform_preview` is `&self` and side-effect-free, so
`preview(..).is_ok()` *is* the predicate, and the engine's own guidance maps
`DegenerateCtm` to **do not offer a handle**. Today the handle is offered, the
drag is accepted, and the refusal arrives at the end — where D42 then swallows
it.

### D45 — OPEN: a path that both fills and strokes reports only its fill colour

`visible_colour` (`crates/pdfcer-gui/src/panels/objects/summary.rs:540-548`)
tests `style.fill` first and returns, so a `B`-operator path — filled *and*
stroked, and the common case for a CAD-exported hatched region with a border —
reports one colour and gives no hint a second exists.

The priority itself is right: it was written to stop a stroke-only path
advertising a fill colour that appears nowhere. What is missing is the
both-present case, which needs two fields rather than a different winner.

### D46 — OPEN: `Session::launch` names a cause it never measured, and the failure is SKIPPED

`tools/ui-verify/src/launch.rs:344-348` returns *"no window appeared for pid …
On a platform that cannot enumerate windows this is always the outcome, and the
check is correctly reported as SKIPPED rather than failed."*

The function measured a timeout. It did not measure whether the platform can
enumerate windows, and the same branch is reached by a binary that crashed at
startup, a modal that stole the window, or a machine under load. Because the
outcome is classified **SKIPPED**, a launch failure removes coverage without
turning anything red — the exact shape this project has been bitten by before
(*"a SKIP is not red"*).

Distinguish "no window enumerator on this platform" — which is a property of the
build host and can be probed once — from "this launch did not produce a window",
which is a failure.

### D48 — OPEN: the Objects row menu's one-item shape rests on a condition that expired

`crates/pdfcer-gui/src/shell/menus.rs` gives `OBJECTS_ROW` a single item,
`file.properties`. `format.delete` was kept off it because an Objects row click
wrote a panel-local **focus**, not a selection — so a Delete gated on
`selection.any` would have removed whatever was selected on the canvas rather
than the row under the pointer.

A row click now raises `SelectionAction::SelectObject`
(`crates/pdfcer-gui/src/panels/objects/mod.rs`), which is the exact condition
both comment sites named as the day Delete could be added. Nobody revisited
them. Deletion is not lost — the click selects and the canvas Delete then
applies — so this is a discoverability gap, not a dead end, and the decision to
add or not add is the operator's.

The wider point is the register's: **a rationale that names its own expiry
condition needs something that fires when the condition is met.** Two files
carried this one for the life of the selection model without noticing.

### D49 — OPEN: four Select-filter rows are inert, and their tooltips promise otherwise

`PickFilter` (`crates/pdfcer-gui/src/canvas/pick.rs`) carries eleven classes and
the popup writes all eleven. The pick path reads three:
`crates/pdfcer-gui/src/canvas/input.rs:131` (`Part`), `:137` (`Node`) and `:218`
(`filter.allows(class)`), where `class` comes from `PickClass::of_object`, which
can only ever return `Path`, `Text`, `Image` or `FormXObject`.

**Markup, Dimensions, Form fields and Characters can therefore never reach a
read.** The annotation pick (`canvas::selection::annot::under_pointer`) takes no
filter at all, and `canvas::textsel` contains no `allows` call.

Their tooltips say the opposite — *"Off: clicks pass through notes, shapes and
stamps."*, *"Off: clicks pass through form fields, and none can be filled in."*
(`crates/pdfcer-gui/src/text/pick.rs`). `PickClass::Link` is inert too and its
tooltip **says so**, which is the shape the other four need until they are
wired.

### D50 — OPEN: the resize and rotate grips vanish below the Object rung

`crates/pdfcer-gui/src/canvas/pressing.rs` binds `at_object_rung` and offers
`GripSet::all()` only there; every deeper rung gets `GripSet::default()`, which
is both flags false — no resize grips, no rotate handle — and drops the outline
too.

Delete no longer has this shape (it asks `canvas::deleting::subject`, which has
an arm per rung), so the ladder is now inconsistent with itself: descending it
keeps deletion and loses transformation, with nothing said. Either the grips
follow Delete down the ladder, or the disappearance is disclosed.

### D51 — OPEN: invisible OCR text is an object-selection target, against this project's own spec

This project's rule for OCR-invisible text (`Tr 3`) is that it **is not** an
object-selection target and **is** a target for the character sweep — a
recognised layer exists to be searched, not to be clicked. Neither half is
implemented: there is no render-mode predicate anywhere in this crate's
hit-test or selection path, and the engine deliberately bounds invisible text
(`pdfcer_core::vector::decompose`, `text_hit` tests run bounds only).

So a scan with a recognised layer over it answers a click with the invisible
text rather than the image the operator can see. The one mitigation that exists
runs the other way and only in Read: `crates/pdfcer-gui/src/canvas/clicking.rs`
makes the image arm yield when a word is under the pointer, gated on
`!caps.edit_content` — so it does not apply in Edit, which is the mode where
object selection is the point.

The engine's behaviour is right for the engine. The filter belongs here.

### D52 — OPEN: `page_cache` warms the cache it then measures

`tools/ui-verify/src/checks/page_cache.rs` drives forty wheel notches away and
forty back, and asserts no page is rastered twice. A smooth continuous scroll
rehomes each page it passes into the strip cache
(`crates/pdfcer-gui/src/render/settle.rs`: *"scrolling to a page is precisely
the gesture that guarantees the page is already cached"*), so the gesture fills
the cache by travelling through it, and scrolling **further** makes this more
true rather than less.

Both halves are affected: nothing is evicted either, so the return leg asserts
against a precondition that never occurs. The repair is a discontinuity — a page
jump or Ctrl+End — and an assertion on `strip-raster-evicted`.

### D53 — OPEN: `egui-shell`'s rail fixture is named for this application and no longer resembles it

`crates/egui-shell/src/dock/rail.rs` builds its test rail in `fn pdfcer_rail()`,
whose `tabs` group lists `view.panel_pages`, `view.panel_bookmarks`,
`view.panel_layers`, `view.panel_signatures` and **`view.panel_fonts`**. The
real group is in `crates/pdfcer-gui/src/shell/manifest/rail.rs` and holds six
items, the last two being `markup.comments` and **`file.fonts`** — and that
manifest carries an explicit warning against the very id the fixture invents,
because `Panel::command_id` is the source of truth and a symmetric-looking id
would give the rail a tab that opens nothing.

The fixture does not need to mirror the application: its subject is *a
`RailFold::Never` group is drawn entire at every rung*, which is a property of
any such group. The name is what makes it a trap. A reader who reaches into
`egui-shell` for the reference shape gets an id this repository has already
decided does not exist, and the assertion that the group "still holds all five
panel tabs" reads as a statement about the product rather than about a local
array.

It is also the one place where R7 is bent in spirit while passing in letter:
`check-shell-purity.sh` looks for `pdfcer-*` dependencies and for
`pdfcer_core` / `pdfcer_render` in source, so a function name naming the
downstream application goes through.

Repair: rename it to `sample_rail()` and let its ids be plainly synthetic. The
fixture's value is the shape, not the names.

### D54 — OPEN: the pixel oracle's only calibration input is deletable, and its absence is green

`tools/ui-verify/src/profile.rs`'s `SETTINGS_HEADINGS_LEGACY` region set is
`Calibration::Image("evidence/crop_settings.png")` — seven headings expressed as
fractions of that one 1860×1035 image, valid against nothing else. It is the
harness's own acceptance evidence: `tools/ui-verify/tests/`
`pixel_oracle_against_real_evidence.rs` is what demonstrates the oracle
separates text a person can read from text a person cannot, on a real
antialiased screenshot rather than on synthetic images whose legibility was
decided by whoever drew them.

**That file is an input no committed tool can rebuild**, and every other route
to `evidence/` writes rather than reads — `ctx.out(...)` output, `make-icon.py`'s
review strip, a sweep's traces. It is therefore the one artefact in this
repository whose deletion silently disarms a test, and it has already happened
once: it was removed with the rest of the directory's binaries and the suite
stayed green for twelve days, because the test's missing-file path prints a
reason and returns rather than failing. That path is correct in itself — an
absent artefact is not a regression in the code — but it means **the only
signal is a line in `--nocapture` output nobody reads**.

Repair: a gate that resolves every `Calibration::Image` path in `profile.rs`
against the working tree and fails when one is absent. The input set comes from
the source that declares it, so a new calibrated profile is covered the day it
is written rather than the day somebody remembers the gate exists.

Related: D2, whose evidence this image is.

### D55 — OPEN: the line-weight disclosure is written, tested, and spoken to nobody

`text/resizing.rs:103` `line_weight_disclosure()` returns the sentence that
explains why a scaled object's stroke width did not change with it. A crate-wide
grep finds exactly two references: the definition, and the unit test at
`text/resizing.rs:221` that asserts the sentence explains itself. **No
production code calls it.**

This is a Rule 4 failure of the half that survives. A resize that leaves `/LW`
alone is an inference the operator **cannot see** — the line looks the same
before and after, which is the point — and the rule says an invisible inference
still owes an off-canvas report. The report exists as a string and never reaches
a surface.

The test is what keeps the function compiling, so the usual signal is absent:
`pub` suppresses `dead_code`, and a unit test discharges the rest. The shape is
the one `check-region-names.py` was written for, one layer up — *a declaration
with a test but no caller looks exactly like a feature*.

Repair: emit it from the same place the refusals reach, on any resize whose
selection carries a stroke and whose `scale_stroke_width` is false. Note that
the refusal path itself is compromised by D41, so fixing this without fixing
that produces a sentence stamped epoch 0 that the status line will not show
after the document's first edit.

Related: D41 (the epoch-0 stamp that hides resize refusals), and
`OPERATOR_REQUESTS.md` O51, which added the switch this sentence describes.

### D56 — OPEN: a pushed grip box can never earn a mid-edge grip, and says nothing

`handles.rs:156` `MIN_BODY_STRIP_PX` = `GRIP_SIZE_PX + 2 × (GRIP_SIZE_PX / 2 +
GRIP_GRAB_SLACK_PX)` = **20**, and `grip_bounds` pushes a small object's anchor
box out to exactly that. `handles.rs:117` `MIN_MID_GRIP_EXTENT_PX` =
`GRIP_SIZE_PX × 3.0` = **24**, and `handles.rs:785` measures the piling test
against the **pushed** box, deliberately: whether a mid-edge grip lands on its
corner neighbours is a question about the spacing it is drawn at.

Both decisions are right on their own and their composition is not: 20 < 24, so
**every object small enough to be pushed is below the mid-edge threshold by
construction**. The four mid-edge grips are unconditionally withheld from
precisely the objects the push exists to make grabbable. A 0.85 pt cell gets
four corner grips; a comfortable selection gets eight; nothing tells the
operator why, and the difference is not recorded anywhere outside this row.

Not necessarily a bug in either constant — it may be the right outcome, since a
mid-edge grip on a 20 pt box would sit 8 pt from both its neighbours. What is
wrong is that it is **undisclosed and unasserted**: no test pins the relation
between the two constants, so changing either silently changes which objects get
eight grips.

Repair: assert the relation where both constants are declared, so a future
change to `GRIP_SIZE_PX` or `GRIP_GRAB_SLACK_PX` has to state which side of it
is intended; and decide whether a small object should have eight grips at all.

### D57 — OPEN: Delete inside a form XObject refuses in the engine's name, and the engine allows it

`canvas/deleting.rs`'s `Refusal::InsideForm` fires from `part_rung` and
`node_rung` whenever the entered target `is_leaf()`, and the operator is told
this is a limit of `pdfcer-core`. It is not. The **pinned** engine — `Cargo.lock`
names `git+file:///D:/Dev/pdfcer?branch=main#b2f54228eb53884367ca32016b9ff94ee8c48970`
— declares `delete_text_run_in_form`, `delete_subpath_in_form` and
`delete_node_in_form` as `pub fn` on `EditSession`, beside the six form-interior
move verbs this shell already calls. Nothing upstream forbids the delete.

**Two defects, and the second is the one that costs.** The refusal itself is
defensible while the wiring is absent: a key that silently does nothing is worse
than one that says why. What is not defensible is **attributing it upstream**. A
refusal worded as *the engine cannot* closes the question for everyone who reads
it — the next session does not check, `ENGINE_BACKLOG.md` keeps a `wanted` row
pointed at work that arrived, and the capability stays unreached for as long as
the sentence stands. This one stood across a pin bump that shipped the verbs.

The genuinely missing half is local and named: a leaf carries no page
paint-order index, so `part_hits_of` matches nothing for it and the Part rung
cannot be **entered** inside a form at all. That seam is the work, and it is the
same seam the in-form move verbs were taken through, so there is a worked
example in `canvas::moving`.

Repair, in order: open `part_hits_of` to leaves; route the three verbs through
the existing `vector_edit_on_page` funnel so the delete gets an undo entry and a
cache invalidation like every other; retire the variant. Until the first step
lands, the refusal stays and its wording is already corrected in source.

★ The generalisation, because this is the third instance: **a comment asserting
that the engine cannot do something is a citation with a shelf life of hours,
and it is the only kind of claim in this repository that no gate can falsify.**
`check-engine-backlog.sh` reads the engine's `FEATURES.md` table and cannot see a
verb this shell decided not to call; `check-verb-coverage.sh` scores a verb as
consumed on a doc-comment mention. An absence claim about the engine belongs in
`ENGINE_BACKLOG.md`, where it is re-triaged, and never in a refusal's prose,
where it is believed.

Related: D41 (resize refusals stamped epoch 0), and `ENGINE_BACKLOG.md`'s
*"Delete a subpath, a node or a text run INSIDE a form XObject"* row, whose
verdict is correct — it is `wanted` because **we** do not reach it — and whose
reason paragraph should name this row.

### D58 — OPEN: three tests read a fixture directory that no longer exists, and one of them passes

`D:/Dev/temp/pdfcer/` is gone. The operator's drawing is at
`D:/Dev/pdfTests/SW41177/SW41177.pdf`. Three tests still name the old path:

| site | what it does when the file is absent |
|---|---|
| `canvas/textsel/tests.rs:286` | `assert!(path.exists(), …)` — fails, and the message says where to re-point it. `#[ignore]`d, so only a deliberate run sees it. |
| `ocr/fixture.rs:731` | `Document::load(…).unwrap()` — panics. `#[ignore]`d. |
| `app/actions/forms/tests.rs:635` | `if !path.exists() { return; }` — **returns, and the test reports green.** It is not `#[ignore]`d, so it runs in the ordinary suite. |

The first two are the right shape and merely carry a dead address. The third
is the defect. `a_field_can_be_authored_and_only_the_tooltip_is_required`
exists to assert a **pair** — that authoring a text field succeeds with a
tooltip and fails with exactly `TooltipDecisionRequired` without one — and
that pair is the whole reason the test distinguishes "authoring works" from
"authoring happens to work on this fixture". With the fixture gone it asserts
nothing at all, and the suite's green count does not move, because a skipped
body and a passing body are the same number.

The early `return` was defensible when written: the fixture is a customer
drawing, it is not committed and never will be, and a developer without it
should not get a red suite. What is wrong is that the skip is **silent**. A
fixture-dependent test that cannot find its fixture has two honest options —
`#[ignore]` so the runner reports it as ignored rather than passed, or
`eprintln!` the reason so the skip appears in `--nocapture`. It currently
takes neither.

Repair: re-point all three at `D:/Dev/pdfTests/SW41177/SW41177.pdf`, and give
the third the same `assert!`-with-an-address shape the first already has, or
`#[ignore]` it. All three are **code** and are out of scope for a
comment-only pass.

The generalisation is D54's, pointed at a path instead of an image: **an
input a check reads from outside the repository is a dependency with no
build system behind it.** Nothing rebuilds it, nothing notices when it moves,
and the check that depends on it goes quiet rather than red.
`tools/ui-verify/src/checks/text_edit_real.rs:50-64` recorded this move when
it happened and is the reason it was findable at all.

Related: D54 (a deletable calibration input whose absence is green).

### D59 — OPEN: the Security tab’s boundary sentence has no call site

`text/security.rs` `cannot_author()` returns the sentence that tells the
operator what pdfcer can and cannot do about passwords, permissions and
signing, and where the honest limit falls — that pdfcer authors a signature
and reports what it could check about one, and that whether a recipient
trusts it is not pdfcer’s to say. A crate-wide grep for the name returns two
test **function names** that happen to contain the phrase, two comments that
cite it as a cautionary example, and the definition. **No surface draws it.**

The function is deliberately kept rather than deleted, and its own doc
comment argues why: a tab that states a boundary is informative, and a
boundary that is merely absent reads as a half-built feature. That reasoning
is sound and is not what this row disputes. What is wrong is that the
boundary is stated to nobody, so the reasoning is currently paying for
nothing.

**A refusal string nothing draws cannot be caught by looking at the screen,
and cannot be caught by a driven check.** It has no pixels, publishes no
region, and is corrected only when somebody greps past it — which is why
this one has now been wrong about encryption, wrong about permissions and
wrong about signing, in that order, each time by outliving a capability’s
arrival. `check-unreachable-refusals` does not cover it and cannot be made to:
that gate photographs every **engine** code line mentioning a symbol some
sentence here depends on being dead, and fails when the photograph and the
engine disagree. It is an instrument pointed upstream. A shell string with no
caller is a fact about this crate alone, and nothing currently looks for one.

Repair, in either order: draw it on the Security tab, or widen the
unreachable-refusal gate to shell-authored sentences so the next one is
found by the build rather than by a reader. Drawing it is the smaller job
and settles this row; widening the gate is what stops the shape recurring.

Related: D55, the same shape one layer down — a disclosure written, tested
and spoken to nobody.

### D60 — OPEN: a press-drag beginning on an unselected object rubber-bands

`canvas/gesture/meaning.rs` classifies a drag by where the press landed, and
`DragKind::Marquee`’s own doc states the rule: *“The press was on empty paper,
**or on unselected content**: rubber-band.”* Only a press already inside the
selection’s body is a `Move`. So picking an object up requires two gestures —
click it, then drag it — and the one-gesture form silently does something
else.

**This is a convention defect rather than a broken mechanism**, which is why
no test catches it: every marquee check passes, the move checks pass, and the
behaviour is exactly what the code says. Acrobat, Illustrator, Inkscape and
SolidWorks drawing views all select-and-move on a press-drag over an object,
and reserve the band for a press on empty paper. An operator who drags an
object and gets a selection box reads it as the drag having missed.

★★ **The repair is not a free swap, and that is the open question.** The band
is direction-sensitive (`FEATURES.md`: right-to-left crosses, left-to-right
encloses), and on a dense CAD sheet nearly every point is on ink — so making
a press on ink mean *move* removes most of the places a crossing band can be
started from. That trade is the decision this row is waiting on, not the
classification change, which is one arm.

### D61 — OPEN: the engine’s `ImageSource::Form` doc describes a build without leaves

`pdfcer-core`’s `vector::decompose::ImageSource` documents `Form` as *“treated
as one opaque selectable object bounded by its `/BBox`; 9a does NOT recurse
into the form’s own content (per-form path decomposition is a fast-follow)”*,
and the enum’s own header adds that *“all three are bbox-selectable”*. Both
halves are false in the same file: `PageObjects` carries `leaves`, and
`vector::hit::hit_test_point_deep` skips forms outright so that the leaves
painted from inside them can win the click.

This shell depends on the behaviour, not on the comment — `FEATURES.md`’s
*“clicking an object inside a form XObject selects that object”* is built on
`hit_test_point_deep` and is driven. The cost is to the next reader: the
comment is the most authoritative-looking statement about forms in the engine,
it sits immediately above the variant, and it says the opposite of what the
crate does.

`D:\Dev\pdfcer` is read-only from here, so **this is a row and a hand-off,
never an edit**. Recorded under D32’s precedent: a stale engine comment is not
this project’s to fix and is this project’s to notice, because believing it
costs a wrong diagnosis rather than a compile error.

### D62 — OPEN: the ribbon’s re-wrap rung cannot fire, because both row limits are 3

`plan::GROUP_ROWS` and `plan::MAX_GROUP_ROWS` are both `3` (`plan/mod.rs:292`,
`:314`). A group’s natural width is `measure_group_rows(…, GROUP_ROWS)`
(`band.rs:979`) and its re-wrapped width is `measure_group_rows(…,
MAX_GROUP_ROWS)` (`band.rs:433`) — the same function, the same group, the same
ceiling — so `Candidate::rewrapped` always equals `Candidate::natural`,
`gains_from(State::Rewrapped)` is therefore always false (`collapse.rs:163`),
and `fit` never advances a group to that rung (`collapse.rs:228`). S5’s middle
rung is unreachable in the running program: the band steps from natural
straight to collapsed, which is the *stall at one width and jump at the next*
the rung exists to prevent.

The compressed-height branch at `band.rs:864` is dead by the same arithmetic.
`wrap_group` never returns more rows than the ceiling it is handed, and both
ceilings are 3, so `rows.counts.len() > plan::GROUP_ROWS` cannot hold.

**Nothing is red, and that is the reason to write this down.** `collapse.rs`’s
ladder tests build `Candidate` literals whose `natural` and `rewrapped` are
deliberately different, so every assertion about the middle rung passes
against widths the measurement can no longer produce. A test that calls the
verb cannot see the chain in front of it.

Repair is a decision, not a typo. Either lower `GROUP_ROWS` so the natural
split is narrower than the ceiling — `MAX_GROUP_ROWS`’ own doc cites Word’s
Font group at two rows at 1900 pt and three at 1000 — or delete
`State::Rewrapped`, `Candidate::rewrapped`, `rewrap_is_legible` and the
compressed-height branch and say in `plan/mod.rs` that the ladder has two
rungs. What is not available is holding both constants at 3 while the
machinery and its doc comments describe a rung that cannot fire.

Related: D55, the same shape one layer up — built, tested, and reaching
nobody.
### D63 — OPEN: the zoom anchor is lost on about one wheel notch in a hundred and thirty

`zooming_does_not_throw_away_where_the_operator_panned` fails in roughly half of
runs against the same binary, the same fixture and the same aim point. Four
consecutive runs gave FAIL at 2314%, FAIL at 56770%, PASS, PASS. The steady
per-notch drift is 27-36% of the tolerance across every one of about 130
notches; the failing notch jumps to about 460% of it — at 2314% zoom that is
1.8737 pt of drift against a 0.4062 pt tolerance, which is some forty screen
pixels. It is the gesture the operator uses to find their way around a drawing.

**It is not the check, and two sampling hypotheses were measured dead before
this row was written.**

- `held()` (`checks/zoom_keeps_place.rs`) assembles one reading from two
  independently emitted trace events: `zoom` from the last `canvas` line, `at`
  from the last `canvas-pos` line. They are not 1:1 — `canvas-pos` is emitted
  about twice as often, with runs of up to seventeen consecutive `canvas-pos`
  lines between two `canvas` lines — so a reading can pair one frame’s zoom with
  another frame’s position. But `at` never moves inside such a run (42 runs, zero
  change), so the stale pairing cannot produce the jump.
- `settled()` accepts two byte-identical reads three frames apart, which a render
  stall could fake mid-animation. Each trace holds exactly one
  plateau-then-resume and it is the idle before the first notch, so no settle was
  faked.

Both readings are therefore of genuinely settled views and the jump is the
application’s. The failing notch sits at a different magnification every time, so
it is not a threshold; the randomness points at state carried between notches
rather than at arithmetic at one zoom.

Where to look: `canvas-place src=` flips between `zoom-anchor` and `deep`, and one
of the two failures was the first notch after that handover.
`geometry::zoom_anchor_offset` and `DeepAnchor::zoomed_about` are the two sites
the check’s own failure text already names, as O24e and O24f.

**Do not widen the tolerance.** `settled()`’s doc comment argues that at length
and the argument holds here: widening is the one change that removes the only
evidence the defect exists.

### D64 — OPEN: the Set-scale window never appears, and the frame-ordering explanation for it is disproved

The command runs and its state is built. The live `measure_calibrate` trace
carries `ribbon-command-invoked id=measure.set_scale`, then
`measure-group-fallback reason=no-measure-state`, then
`scale-open group=GroupId(0) path=cold` and
`scale-seeded group=GroupId(0) name="Default" ratio_paper=1 ratio_real=100`, so
`Dialogs::scale` is `Some` with a seeded group. After that the trace carries no
`viewport-inner` line and no `ui-rect name=dialog:set-scale` — the region name
`ScaleDialog` emits from inside `Host::show`'s closure (`dialogs/scale.rs:115`,
recorded at `:519`). The trace then stops, having produced one frame where the
check's `settle(16)` asked for sixteen.

**Three explanations are ruled out by measurement, and one of them was published
here and is wrong.**

*Frame ordering is not the cause.* `app::frame`'s `ui` is a single function: the
ribbon is Step 1b at line 724, the `Action::Command` drain is at 954, and
`dialogs.show` is Step 2b at 995. A dispatch therefore always precedes
`dialogs.show` **within the same frame**, and the `export_text` trace shows the
consequence directly — `ribbon-command-invoked` opens frame 88 and
`ui-rect name=dialog:export-text` is emitted later in that same frame 88. A
dialog draws in the frame of the press, not the frame after it. Any reading that
counts a frame boundary at `canvas-pos` will cut the unit in the wrong place and
manufacture a one-frame lag that is not there.

*`ScaleDialog::hidden` is not the cause.* It returns true only when the selected
canvas tool is `Measure(MeasureKind::Scale)`. The trace carries zero
`measure-tool` lines, and `CanvasTool`'s `#[default]` is `Select`
(`canvas/tool/mod.rs:176`), which is what `canvas::tool::selected`'s `get_temp`
falls back to.

*The no-document guard is not the cause.* `dialogs::show` returns early at
`dialogs/mod.rs:908` when the status is not `Status::Open`, and
`close_document_scoped` would drop `self.scale` on the way out. But `scale-open`
read the dimension model through `doc.session` in the same frame, so the status
was `Open` when the command ran.

**What is left, and it cannot be settled by reading.** Either `dialogs::show`
does not reach `self.scale.as_mut().map(|d| d.show(...))` at `mod.rs:1026`, or it
reaches it and `Host::show`'s closure does not run. The trace cannot distinguish
them: a frame with no dialog open ends at `canvas-pos`, and so does this one, so
"the frame finished and drew nothing" and "the frame was cut short" leave the
same evidence.

Next step is R1 — drive the binary with the Set-scale command and a long settle,
and add a trace line inside `dialogs::show` immediately before the `scale` call
so the two survivors become distinguishable. Until that runs, nothing here
prescribes a fix; the repaint-at-dispatch fix this entry previously proposed was
derived from the disproved ordering and must not be applied on its authority.

For the operator this is a ribbon button that does nothing.


### D65 — OPEN: a check occasionally gets no window in 30 s, and both instances followed an abnormally torn-down predecessor

Two of the 100 verdicts in the clean re-run skipped with *"no window appeared for
pid N within 30s"*. The application is not silently failing to start: the
`dimension_groups` trace carries nine lines — `start`, `shell commands=162`,
`layout-load`, `layout-skip`, `mode-restore`, `mode-changed`, `recent-load`,
`paste-chords`, `pick-filter-load` — and then stops. `pick-filter-load` is
`PdfcerApp::new`'s last traced field (`app/mod.rs:1094`); the next trace a
healthy launch emits is `ui-scale-initial`. So `new` completed and the stall is
in the window and renderer creation that follows it, which traces nothing.

**The ordering is the lead.** `ui-verify` runs one check at a time — it takes the
real cursor — so nothing is contending in parallel. But in the log each
no-window skip sits immediately after a check whose application was killed while
it held more than a plain window:

| killed predecessor | state when killed | next check |
|---|---|---|
| `measure_calibrates_by_picking_two_points` (FAIL) | `Dialogs::scale` is `Some`; a child viewport was being created — see D64 | `set_scale_reads_the_group_it_is_about_to_overwrite` — no window |
| `measure_hover_shows_what_it_will_take` (SKIP) | `Measure(Linear)` armed, mid-render | `dimension_groups_panel_makes_a_group` — no window |

`Session`'s `Drop` (`tools/ui-verify/src/launch.rs:737`) does `kill()` then
`wait()`, so the predecessor's *process* is reaped before the next launch. What a
`TerminateProcess` does not reap synchronously is the graphics device, and a
dialog opened through `show_viewport_immediate` is a second surface on it.

**Not yet established, and it must not be written down as though it were.** The
same two checks also skipped in the previous sweep, for a different reason
(foreground-void), so "these two are simply fragile" is not excluded, and neither
is an ordering coincidence in a sample of two.

The experiment, which needs the machine: run the two no-window checks on their
own, in isolation, with no predecessor — and then run each immediately after its
observed predecessor. If isolation passes and the pair reproduces, the cause is
the teardown and the fix is in the harness, not the application. A control
binary that does nothing but open a window belongs in the same run, so "the
machine could not make a window at all" is separable from "this application could
not".

### D66 — OPEN: a recovered document cannot be saved, and the operator finds out after he has edited it

`EditSession::to_incremental_bytes` refuses a document whose base was loaded
through cross-reference recovery, with `WriteError::RecoveredBaseForbidsIncremental`.
That refusal is the engine's **first** guard, ahead of the encrypted-base one —
a recovered base's cross-reference table was invalid, so §5.6's never-normalize
rule does not bind it and its save has to be a full rewrite.

`app::save::write_copy` has exactly one fork, and it is the staged redaction
(that module's §1.1). Nothing on the save path asks
`Document::loaded_via_recovery()` — `grep -rn 'loaded_via_recovery' crates/`
returns one hit, and it is `sign::Standing`'s `recovered` field, where the
signing window reads it and refuses **with a sentence, before the operator
commits to anything**. Save has no equivalent.

So a recovered document reaches `to_incremental_bytes` from `write_copy`, the
error propagates through `SaveError::Serialize`, and that variant's `Display`
arm renders it as:

> the engine could not build the update: incremental save of a recovered
> document is refused; its base cross-reference was invalid, so its save must
> be a full rewrite (save_full)

`save_full` is an engine function. It is not a command in this shell, it is not
in `RIBBON_IA.md`, and there is nothing the operator can do with the name.

**Two separate defects, and the second is the worse one.**

1. The sentence is the engine's, in the engine's vocabulary, for a state the
   operator cannot act on. That is a text defect and it is cheap.
2. **The shell knew before he started.** `loaded_via_recovery()` is answerable
   at open. Letting an hour of editing accumulate against a document that
   cannot be saved is the failure mode; saying so at open, or routing to the
   full rewrite the engine names as the supported path, is the fix.

`app/save.rs`'s §1 argues at length that incremental must never silently fall
back to a full rewrite, and lists the staged redaction as *the* exception. A
recovered base is a second case where the engine itself requires the full
rewrite — the module's argument does not cover it, and the route does not
exist. Whether the shell routes automatically or refuses at open with a
sentence is an operator decision, because a full rewrite discards the previous
revision and destroys existing signatures.

---

## Do not weaken

- `resize_scales_a_shape` targets a **page-sized** selection deliberately: it is
  the regression test for the floating-scrollbar defect. Narrowing its target to
  something more convenient silently retires that coverage.
- `overflow_probe` and `RESIZE_FLOOR_PT` hold a measured floor the dock's
  overflow behaviour depends on. See D25.
