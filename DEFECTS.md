# pdfcer GUI — defect register

**Compiled:** 2026-08-12, against `D:\Dev\pdfcer` at the release build
dated 2026-08-12 19:54 (`target/release/pdfcer-gui.exe`).

Every entry below was verified against source at the quoted `file:line`,
or observed directly by driving the built binary. Screenshots are in
`evidence/`. Nothing here is inferred from documentation alone.

Ordering is by *cost to the user divided by cost to fix*, not by severity.

---

## D1 — The Delete key stops working the moment you click the canvas

**Severity:** critical · **Fix:** one line · **Regression dated:** 2026-08-10

This is the defect reported as *"I can't even click on an object and
delete it by hitting the delete key."* It is real, it is not a
discoverability problem, and the selection half works perfectly.

### Causal chain

1. **Click-select works with no gating.** With no tool armed the canvas
   falls to the modeless branch (`main.rs:17010-17041`) which hit-tests
   and assigns `doc.canvas_selection` (`main.rs:22123-22173`, applied at
   `22614`). No `editing_enabled` check, no armed-tool requirement, no
   Objects-panel requirement. The object visibly selects.

2. **`editing_enabled` is not the culprit.** It defaults to `true`
   (`main.rs:3624`), with the comment *"Editing starts ON… a new
   operator who finds every tool inert would reasonably conclude it is
   broken."* That instinct was right. It is not what is blocking Delete.

3. **The canvas grabs egui keyboard focus on every click**
   (`main.rs:16891-16895`):
   ```rust
   if image_response.clicked() || canvas::primary_drag_started(&image_response) {
       image_response.request_focus();
   }
   ```
   Deliberate, and reasonable — §1.4 wanted the canvas to be a real Tab
   stop rather than an inert image. Because the widget is recreated every
   frame its id stays live, so the focus never lapses.

4. **The keyboard guard tests the wrong thing** (`main.rs:13777`):
   ```rust
   let typing = ctx.egui_wants_keyboard_input();
   ```
   In egui 0.35 that is **not** "a text field is focused". Verified in
   the vendored source at
   `egui-0.35.0/src/context.rs:2884-2886`:
   ```rust
   pub fn egui_wants_keyboard_input(&self) -> bool {
       self.memory(|m| m.focused().is_some())
   }
   ```
   — *any* focused widget, including the canvas itself. The doc comment
   directly above it says *"egui is currently listening on text input
   (e.g. typing text in a `TextEdit`)"*, which is what the name and the
   comment both promise and what the implementation does not deliver.
   This is an egui API footgun, not a careless read.

5. **So the binding is never installed** (`main.rs:13875-13878`):
   ```rust
   if (!tool_active || canvas_delete_target) && !typing {
       pressed(Modifiers::NONE, Key::Delete, Action::DeleteSelection);
       pressed(Modifiers::NONE, Key::Backspace, Action::DeleteSelection);
   }
   ```
   `tool_active == false` and `canvas_delete_target == true` are both
   satisfied. `typing == true` from step 4. The branch never runs.

6. **The deletion logic downstream is correct and simply unreachable.**
   `Action::DeleteSelection` (`main.rs:11205-11290`) → `delete_selected_object()`
   (`main.rs:5250-5310`). Pass 47.0 had already removed an earlier
   `active_tool() == VectorEdit` gate here. That fix is intact; nothing
   calls it.

> **Root cause.** `collect_keyboard_actions` guards its unmodified-key
> bindings with a predicate that means "any widget has focus" rather
> than "a text field has focus", and the canvas takes focus on the very
> click that selects the object — so from the first canvas click onward
> the Delete key is permanently suppressed.

### Blast radius

The same `!typing` guard also suppresses, after any canvas click:

| Keys | Lost function | Line |
|---|---|---|
| `PageDown` / `PageUp` | Next / previous page | `13780-13782` |
| `Home` / `End` | First / last page | `13787-13790` |
| `[` / `]` | Rotate page | `13849-13852` |

So page navigation by keyboard is dead too, for the same reason and
from the same click.

### Why it was never caught

`collect_keyboard_actions` has exactly one test
(`main.rs:28338-28375`), which builds a bare `egui::Context::default()`
with **no widgets** — therefore `memory.focused()` is `None` and
`typing` is always `false`. The single property that breaks in the real
app is structurally absent from the only harness that exercises the
function. Object deletion is covered at the `Action` level, never
through the key.

The regression is self-declared in its own commit message: `e46c3a8`,
2026-08-10, *"a focused text field keeps its unmodified keys —
analysis-confirmed, NOT empirically verified."* It landed two days after
Pass 47.0 fixed the same key by a different route.

### Fix

**Primary** — `main.rs:13777`:
```rust
let typing = ctx.text_edit_focused();
```
This preserves `e46c3a8`'s intent exactly. `text_edit_focused()`
(`egui-0.35.0/src/context.rs:2889-2895`) resolves the focused id and
checks whether a `TextEditState` exists for it. A `DragValue` in
keyboard-edit mode registers its `TextEdit` under the *same* id it
focuses, so property-bar drag values still count as typing.

**Secondary, required alongside** — `main.rs:13341-13348`. Once `typing`
stops masking it, the `canvas_delete_target` escape hatch becomes
reachable while a text tool is armed with a stale `canvas_selection`,
which would steal forward-delete from the caret. The comment at
`main.rs:13872-13874` already promises this cannot happen (*"The text
tools are deliberately NOT given this hole"*) but nothing enforces it:
```rust
Status::Open(doc) => !matches!(
        doc.active_tool(),
        Some(CanvasTool::TextEdit | CanvasTool::AddText)
    ) && (doc.selected_dimension.is_some()
        || doc.entered.is_some_and(|e| e.subpath.is_some())
        || !doc.canvas_selection.is_empty()),
```

**Test that would have caught it, and should be added:** drive
`collect_keyboard_actions` through a context where a widget holds focus
(`ctx.memory_mut(|m| m.request_focus(id))`) and assert `Key::Delete`
still yields `Action::DeleteSelection` when
`CanvasKeys { delete_target: true, tool_active: false, .. }`.

### Two workarounds, until it lands

Both work today and both explain why this survived dev testing (egui's
default is `SurrenderFocusOn::Clicks`):

- Click the object, then click **any ribbon or panel chrome** — that
  surrenders canvas focus without clearing the selection — then Delete.
- Select from the **Objects panel** tree row instead of the canvas. A
  plain `Button` never calls `request_focus`, so Delete works at once.

---

## D2 — Section headings and dock tab labels are invisible in the default theme

**Severity:** high · **Fix:** small · **Evidence:** `evidence/crop_settings.png`, `evidence/crop_tabs_left.png`

Every collapsible section heading in the Settings dialog — *Appearance,
Theme, Colour, Images and transparency, Copying and extracting text,
Pages and printing, Saving files* — renders near-white on light grey. So
do the dock tab labels "Pages" and "Objects". At 1× they are simply not
readable.

### Cause

`theme.rs:434-444` loops over all five widget states setting
`corner_radius`, `bg_stroke` and `fg_stroke`. Then:

```rust
v.widgets.inactive.weak_bg_fill = p.panel;     // 447
v.widgets.hovered.weak_bg_fill  = p.surface;   // 448
v.widgets.active.weak_bg_fill   = p.accent;    // 449
v.widgets.active.fg_stroke = Stroke::new(1.0, p.label_backdrop); // 450
```

`label_backdrop` is `rgba(250,250,250,220)` (`theme.rs:290`). Pairing it
with the accent is correct — light text on an accent fill. But only
`weak_bg_fill` is assigned the accent. **`widgets.active.bg_fill` is
never set at all.** Widgets that paint their background with `bg_fill`
rather than `weak_bg_fill` — `egui_tiles` tab buttons, `CollapsingHeader`
headers — get the near-white foreground on a light background.

### Why CI did not catch it

Two tests look adjacent to this and neither covers it:

- `text_contrasts_with_its_background_in_every_preset` (`theme.rs:521`)
  checks `text` against `surface`/`panel` and `text_muted` against
  `surface`. It never tests `label_backdrop`.
- `label_plates_stay_page_facing_not_chrome_facing` (`theme.rs:553`)
  *asserts `label_backdrop` stays light* — correct for its stated
  purpose (labels sit over the white page) — without checking what is
  actually behind it in chrome.

`tools/check-theme-colors.sh` bans raw `Color32` literals outside
`theme.rs`. It never measures a rendered pair. The gate is structural,
not perceptual.

### Fix

Either set `v.widgets.active.bg_fill = p.accent` alongside line 449, or
stop overriding `active.fg_stroke` and let the accent-filled case handle
itself. Then add a test that asserts every place `label_backdrop` is
used as a foreground has the accent as its background — or, more
robustly, a contrast assertion over the actual `(fg_stroke, bg_fill)`
pairs of all five widget states in all three presets.

---

## D3 — README claims two capabilities that FEATURES.md says are stubs

**Severity:** high (it is a published claim) · **Fix:** edit three words

`README.md:20-22` lists under **"Working today"**:

> …markup annotations; redaction (mark, review and apply); **Bates
> numbering; PDF/A validation and conversion**; digital-signature
> inspection…

`FEATURES.md:29-31` states:

> `to-pdfa`, `validate-pdfa`, `sign` and `bates-stamp` exist in
> `pdfcer --help` as **stubs that print "not implemented"**. Not
> ticked anywhere; listed under *Planned*.

Confirmed at `FEATURES.md:224-225`, where both Bates numbering and PDF/A
conformance are unticked on core, CLI **and** GUI.

The same sentence claims printing *"with page placement, orientation,
duplex, copies and n-up/booklet/poster imposition"*. Imposition is real
in the CLI but `FEATURES.md:164` says it has **"No GUI surface at
all"**, and the sentence is describing the application.

`digital-signature inspection` is accurate — inspection only, no
cryptographic verification — and should stay.

This matters more than a normal doc error because the README's own
selling point, two lines above, is that it *"says plainly what does and
does not work today."*

---

## D4 — Text editing: three separate problems behind one complaint

Reported as *"text editing is weird and doesn't just edit the existing
box and move the text correctly as you type plus flow to the next line
doesn't work."* All three parts are correct. They have different causes
and very different costs.

### D4a — The edit unit is one PDF show-text operator, not a text box

**Architectural limit, honestly documented.** Editing genuinely is
in-place on the canvas — there is a real blinking caret painted in PDF
space (`main.rs:17820-17830`), keystrokes are consumed as raw
`egui::Event::Text` (`main.rs:18227-18243`), and no `TextEdit` widget is
in the typing path. But `PendingEdit` pins to one run
(`main.rs:2386-2400`): *"a commit may only span ONE run (§4.4)"*, and a
`TJ` array is one operator.

So a visual paragraph split across several `Tj` runs — the ordinary
output of CAD title blocks, Word and LibreOffice — must be edited run by
run. Dragging a selection across runs sets `cross_run`, which **silently
disables the whole typing loop** (`main.rs:18227`,
`canvas.rs:1489-1510`) behind this notice (`ui_text.rs:5770`):

> *"This selection spans more than one text run … pdfcer's first-cut
> editor edits one run at a time. Narrow the selection to edit or format
> it."*

**Second contributor to "weird":** while composing, what you see is not
your glyphs. It is ghost text in an egui proportional font over a
translucent mask (`main.rs:17868-17899` — *"NEVER a re-raster; the real
glyphs appear only after a real commit"*). You type in the wrong
typeface at the wrong widths, then it snaps to reality on Accept.

**To change it:** a multi-run edit request in core that groups runs into
a line or block and re-emits them as a set, plus dropping the
`cross_run` typing lock.

> **Status 2026-08-15 — still architectural, but it now refuses in
> words.** This shell's editor is still one-run; the multi-run request
> does not exist in `pdfcer-core` and was not built. What changed is the
> failure mode: a selection spanning runs is declined by a sentence on
> the status row (`text::textedit::spans_runs`, via
> `actions::record_note`) rather than by a keyboard that silently stops
> responding. The ghost text is also gone — not replaced with a better
> ghost, but with a caret and an extent bracket that claim only what the
> shell can honestly know before a commit. The argument for why a
> *prettier* ghost would be the wrong fix rather than a deferred one is
> in `canvas::textedit::preview`.

### D4b — Nothing moves as you type; two cases move wrongly on commit

> **Both wrong cases FIXED 2026-08-15** in this shell —
> `canvas::textedit::disposition`, a pure
> `choose(text_matrix, ctm, alignment) -> Reason` consulted at the single
> commit site (`app/actions/apply.rs`, `Action::CommitTextEdit`).
> Rotation is rung 1 and outranks alignment; non-left alignment is
> rung 2. **"Nothing moves as you type" is NOT fixed** — see the
> measurement and the engine blocker below.
>
> The line numbers in the prose below are **stale**: the engine's edit
> code has since moved into `text_edit/`. `FollowerDisposition` and its
> doc comment are now `text_edit/edit.rs:295`; the unguarded
> `emit_tm([*a,*b,*c,*d,*e + delta,*f])` is `text_edit/edit.rs:1505`.
> The claims themselves were re-verified against that source before the
> fix landed; only the addresses moved.
>
> **The honest limit.** A *single-line* right-aligned block still
> reflows, because the engine's `infer_alignment` reports
> `SingleLineDefault` when it has only one line to compare — alignment is
> inferred from the agreement of several lines' edges, and one line
> cannot disagree with itself. Multi-line right/centre/justified blocks —
> the CAD title-block case — are pinned correctly.
>
> **Why this refuses rotation less harshly than `reflow_apply` does.**
> `Pin` is *correct* under rotation, not merely less wrong: it writes no
> follower `Tm` at all, and its compensating `TJ` acts in text space,
> i.e. along the rotated baseline. The ported guard
> (`check_uniform_axis_aligned`, `MTX_EPS = 1e-6`, taken verbatim rather
> than re-chosen) therefore selects `Pin` for rotated text instead of
> refusing the edit. The argument is written out in `disposition.rs`.

The metrics path is **correct**: advance widths come from real font
metrics — `/Widths` for simple fonts, `/W` + `/DW` for composite
(`text_extract/font.rs:687-700`) — and §9.4.4 is implemented properly
(`edit.rs:1950-1967`) with `Tc`, `Tw` and `Tz` all tracked, `Tw`
correctly restricted to single-byte code 32. The 500/1000 fallback is
the third rung only and is disclosed. `TJ` kerning numbers are preserved
verbatim (`edit.rs:1983-2036`), not dropped.

But: **there is no re-layout per keystroke.** `main.rs:18208-18210` —
*"Typing → build/extend the `PendingEdit` (§6.1). **No core call per
keystroke.**"* Real layout runs once, in `commit_text_edit_draft`. So
"as you type", nothing moves at all. That alone accounts for much of the
complaint.

Two cases are then genuinely **wrong** on commit:

1. **Right-aligned, centred and justified text moves the wrong way.**
   `FollowerDisposition::Pin` exists precisely *"for a justified /
   right-aligned tail that must not move"* (`edit.rs:301-303`), but the
   GUI always passes `EditOptions::default()` — i.e. `Reflow` — at
   `main.rs:12438`, its only call site. Alignment is never detected on
   the edit path.
2. **Rotated or skewed text is shifted along the wrong axis.** The
   follower shift adds the advance delta straight to the translation
   component: `emit_tm([*a, *b, *c, *d, *e + delta, *f])`
   (`edit.rs:1503`), with **no rotation guard**. The reflow path does
   refuse rotated text (`reflow_apply.rs:757-760`); the edit path does
   not. This bites rotated CAD title-block text specifically.

There is also **no collision or margin-fit check anywhere in the edit
path** — the response to an overrun is a disclosure string
(`edit.rs:1527-1534`), not a re-layout.

**To change it:** re-measure and re-render the draft with real metrics
per keystroke; detect alignment and select `Pin` for right/centre/
justified tails; port the rotation guard `reflow_apply` already has.

**Per-keystroke re-layout: measured, and blocked on the engine.**
Release build, median of 5, via `canvas::textedit::cost`:

| document | extract (prov.) | recognize+align | plan+save | total |
|---|---:|---:|---:|---:|
| `tail-alignment` (3 lines) | 0.12 ms | 0.01 ms | 0.36 ms | **0.49 ms** |
| `SW41177` p1 (SolidWorks sheet) | 32.07 ms | 0.16 ms | 70.54 ms | **102.77 ms** |
| `ncored-benchmark` A3 | 356.53 ms | 2.79 ms | — | **356+ ms** |

102.77 ms on the operator's own sheets is six frames. The blocker is
**not** the arithmetic: `plan_edit`/`EditPlan` already computes
`advance_delta` before any write — exactly the number wanted — but it is
`pub(crate)`. Every public route either performs a full incremental save
or mutates the undo log, which is why "plan+save" dominates the table.

**This is a feature request for the engine**, and the smallest one that
unblocks it: a dry run — `measure_edit(&Document, &EditRequest) ->
Result<f64, _>`, or simply making `plan_edit` public. With it, the cost
falls to the middle column: extraction is already cached per
`(page, edit_epoch)`, and typing bumps no epoch.

**Debouncing was rejected, not overlooked.** A re-layout that arrives
150 ms after you stop typing is a *second* surprise, and D4a's lesson is
that this feature's sin is showing the operator something the document
will not say. Until the draft can move truthfully on every keystroke, it
does not move at all — and the caret and extent bracket that *are* drawn
promise nothing about widths.

### D4c — Reflow is unreachable in the sequence a user actually performs

Reflow is implemented and shipped. It is blocked by three gates in a row.

**By design it never happens while typing.** Decision 015 §3.3 and
standing rule **R75**: *"Within-block re-wrap is never automatic on
edit; it is an operator-invoked action producing a DERIVED preview
accepted/rejected before any mutation."* The reasoning — that reflow
invents line breaks the file never stated — is sound and should not be
overturned. But it means the line simply grows past the margin and you
must go and press a button.

**Gate 1.** The "Reflow paragraph…" button is disabled *while you are
typing*: `reflow_button_enabled` is `target.is_some() && !pending_is_some`
(`main.rs:2462-2464`). You must Accept first.

**Gate 2 — the serious one.** Having accepted, reflow then refuses
outright (`edit.rs:4279-4285`):
```
"the page's content was already edited this session; reflow is planned
 against the base content, so save and reopen before reflowing this page"
```
And the **preview still renders**, because it is computed from
`state.page_text` against the base document (`main.rs:18501-18520`). So
you see a correct-looking ghost, click Accept, and only then are refused
(`main.rs:18660-18669`). Edit text → reflow is a dead end that requires
save-and-reopen.

**Gate 3 — an open filed defect.** Pass 33.0 (`ROADMAP.md:43419`). Even
on a fresh open, the auto-detected wrap width is wrong after an
overflowing edit, because the block bbox is a union over its lines and
the one over-long line has already widened it (`reflow.rs:605`:
`req.wrap_width.unwrap_or_else(|| old_bbox.width())`). Measured on the
project's own fixture: a 156 pt block became 930 pt and the re-wrap ran
text off a 612 pt page. Only the *disclosure* option shipped; the
roadmap says plainly that *"an operator who does not read the disclosure
still gets a re-wrap to a width they never chose."*

**Additional refusals that hit real CAD and Word content hard**
(`reflow_apply.rs`): text inside a form XObject (`:658`), more than one
font resource in the block (`:669`), rotated or skewed `Tm`/CTM
(`:757`), more than one text-matrix scale — i.e. **mixed font sizes**
(`:768`), and composite/CID fonts.

**And a tokenisation limit that matters more than any of them**
(`reflow.rs:42-54`): word breaks are found at **real U+0020 space glyphs
only**. Producers that position words with `Td`/`TJ` offsets instead of
emitting a space glyph — extremely common in CAD output — present reflow
with one unbreakable word, so nothing wraps at all.

**To change it:** pick option (b) or (d) for Pass 33.0's wrap width;
make reflow plannable against staged session content so Gate 2
disappears; treat `DerivedWordSpace` as a break opportunity; relax the
uniform-font and uniform-size refusals.

### Why the tests do not catch any of this

`fixtures/synthetic/reflow/reflow.pdf` is 5 pages of one paragraph each,
Courier, emitted as **one `Tj` per line with real space glyphs and a
uniform font and size** (`tools/gen-reflow-fixtures.py:114-124`). The
most complex text-edit fixture, `tm_follower.pdf`, has **two** runs on
one line. No fixture has a paragraph split across many runs, mixed sizes
or fonts in a block, rotated text, or words separated by positioning
rather than space glyphs. Every condition that fails in the field is
absent by construction.

---

## D5 — The keyboard-shortcuts reference omits six live bindings

`ui_text::shortcuts_reference()` (`ui_text.rs:5143-5158`) lists 14
chords. Missing: **Ctrl+F** (Find), **Ctrl+P** (Print), **Ctrl+E** (Edit
text), **Ctrl+Shift+E** (Add text), **F11** (full screen), **Ctrl+H**
(read mode). The doc comment immediately above it
(`ui_text.rs:5138-5141`) says it *must* be kept in step with
`collect_keyboard_actions`.

**Fix:** derive the list from `collect_keyboard_actions`, or add a test
asserting the two agree. A hand-maintained list with a comment telling
you to hand-maintain it has already failed once.

---

## D6 — Review mode does not actually block object deletion — **CLOSED 2026-08-14**

> **★ Closed 2026-08-14, and by a different mechanism than either the
> original analysis or the 2026-08-12 supersession expected.**
>
> The supersession below was right that the `Editing on` master toggle had to
> go, and right that "delete the gate sites" was the fix for *that* toggle. It
> was wrong to conclude there was nothing left to enforce. The operator asked
> for a genuinely read-only stance on 2026-08-14 — *"in read mode the document
> shouldn't allow editing"* — and `MODES_AND_PANELS.md` had already specified
> it as a **named, visible mode** rather than a hidden boolean, which is
> exactly what `RIBBON_IA.md` §5.4 said a real read-only state would have to
> be.
>
> What shipped is `app::modes::capability`, and the mechanism is the part
> worth carrying: capability is derived from **the mode's tab list in the
> manifest**, not from the string `"read"`. The ribbon and the canvas
> therefore read one sentence, and the failure this defect describes — a
> surface that says editing is off while a gesture still edits — is
> unrepresentable rather than merely guarded. The hole this entry predicted
> could not be reopened by forgetting a check, because there is no check to
> forget.
>
> **It was verified the way this project says to verify**: `ui-verify`'s
> `read_mode_refuses_canvas_edits` drives the real window, clicks page content
> in Read and asserts no selection, clicks *the same point* in Edit and
> asserts one — so Read's silence is proven to be a refusal rather than a
> miss — then re-enters Read and asserts the selection is dropped.
>
> Three things the gesture gate could not close on its own, each found by
> asking what *survives* rather than what is refused: a click is not a drag
> (gating presses alone leaves the commonest canvas gesture ungated); an armed
> tool outlives a mode switch, because it lives in `egui::Memory`; and a
> selection outlives it too, leaving eight resize handles on a page in Read.
>
> The analysis below is kept for the reason the supersession kept it.

> **Superseded 2026-08-12.** The operator's decision is to remove the
> `Editing on` master toggle entirely and work the way other editors do
> (`RIBBON_IA.md` §5.4, `GUI_ROADMAP.md` Phase 1.7). With no review mode
> there is nothing to enforce, so the fix becomes *delete the four gate
> sites*, not *add the missing fifth*. The analysis below is kept
> because it documents the inconsistency, and because **if D1 ships
> before Phase 1.7 the hole is briefly live** — sequence them together
> or land 1.7 first.

**Latent today; becomes live the moment D1 is fixed.**

Neither `Action::DeleteSelection` (`main.rs:11205`) nor
`delete_selected_object` (`main.rs:5250`) checks `doc.editing_enabled`.
With editing toggled **off**, a canvas selection plus Delete still
rewrites the content stream. Every other authoring surface does check
(`main.rs:7095`, `8169`, `8194`, `16920`).

`main.rs:3225-3235` states the guarantee this breaks: *"no gesture able
to change it by accident."* Add the check before shipping D1's fix,
or the fix turns a dormant hole into a live one.

---

## D7 — Documentation drift

Three items, all small, all in files the project treats as authoritative.

**D7a.** `ROADMAP.md:43419` (Pass 33.0) states as a load-bearing
correction: *"**There is no on-canvas caret at all.** Text entry is a
**panel field**, not an overlaid editable widget."* This is false —
`main.rs:17820-17830` paints a blinking caret in PDF space, and the Pass
14.3 comment at `main.rs:16904` says *"the canvas is its own
caret/selection surface."* It appears to have been written to rebut a
third-party guess and over-corrected. It should be fixed, because
`ROADMAP.md` is declared to win any disagreement.

**D7b.** `FEATURES.md:73` marks reflow `[x]` on core, CLI and GUI with
no caveat, while Pass 33.0 is open and the session gate (D4c, Gate 2)
exists. At minimum it needs a footnote.

**D7c.** `FEATURES.md:119` says form flatten has no GUI surface. It does
— `Action::FlattenForm` at `main.rs:4701`, pushed by a button in the
Forms panel at `main.rs:8112-8116`. The doc understates the build.

---

## D11 — `RichText::strong()` is unusable in this theme, and six labels used it

**Found 2026-08-14, by looking at a screenshot**, while building the
tab-order view: a page heading drawn with `.strong()` came out near-white on
a light panel.

### The mechanism, which is a conflation in `egui` rather than a mistake here

```rust
// egui: style.rs
pub fn strong_text_color(&self) -> Color32 {
    self.widgets.active.text_color()      // == widgets.active.fg_stroke.color
}
```

`egui` has **no separate role for emphasised text** — it borrows the *active
widget* foreground. `egui-shell`'s theme sets that to `palette.on_accent`
(`theme/mod.rs:624`), which is correct and necessary: `widgets.active` is the
**accent-filled** state, and text on an accent fill must be `on_accent`.

So in any theme whose active state is accent-filled — which is every theme
this project ships — **`.strong()` on an ordinary panel is near-white on light
grey.** The two uses cannot both be served by one colour, and `egui` gives
only one.

It also survives `override_text_color`, which the theme sets to
`palette.text`: `.strong()` wins.

### Why no gate saw it

The contrast gate renders **pairs** — a foreground against the fill it is
painted on — and by that measure `on_accent` on `accent` is exactly right. It
has no way to know a `.strong()` label landed on a *panel* instead. This is
`D2` reached from the opposite direction: D2 was a foreground with no fill
assigned; this is a foreground assigned for a fill the text is not on.

### Fixed

Five panel labels drop `.strong()` and render as plain text — strictly better,
since the emphasis they were asking for was invisible: `panels/comments`,
`panels/properties` (×3), `panels/signatures`.

The sixth is a different case and takes the ribbon-tab fix instead:
`dialogs/print/mod.rs`'s selected tab is a `Button::selectable`, so it had
**both** halves of the problem — the plate filled from `selection.bg_fill`
(the translucent canvas tint, 27 % alpha) and the label from `on_accent`. It
now paints `accent` + `on_accent` explicitly, exactly as `ribbon::tabs` does.

**Verified for the panels, argued for the dialog.** The five panel labels are
the observed case. The print dialog's tab was fixed by identical mechanism
rather than by a second screenshot — the ribbon tab with the same two bugs was
photographed before and after, and this is that fix applied to the one other
`Button::selectable` + `.strong()` pair in the codebase. Someone opening the
print dialog should confirm it.

### The rule that follows

**Do not use `RichText::strong()` in this application.** There is no colour it
can resolve to that is correct on both an accent fill and a panel. Emphasis
belongs to layout and wording, or to an explicit `palette` colour chosen for
the surface the text is actually on.

### ★ The rule was broken again three days later, and now there is a gate

**2026-08-17.** The Settings window was built and its seven group headings and
thirteen setting titles used `.strong()`. On screen they were pale grey on pale
grey while the radio labels beneath them read normally — the same picture as
the six labels above, in a window whose whole job is to be read.

It was found the same way, by capturing the running program, and the person who
wrote it had read this entry. That is the finding worth keeping: **this rule was
written down, in the file whose purpose is to stop repeats, and the document did
not stop it.** A rule that lives only in prose is enforced exactly as often as
somebody remembers to read the prose.

So `tools/gates/check-strong-text.sh` now enforces it, and the exact form it
enforces is narrower and more useful than the prose:

> `.strong()` is a defect **unless an explicit `.color()` is within two code
> lines of it.**

That admits the two legitimate uses — `egui-shell`'s ribbon and dock tab labels,
which are drawn *on* the accent fill, where `on_accent` is correct and R84 wants
the weight because weight survives greyscale and colour-vision deficiency — and
refuses everything else.

**Its first real run found a third instance**, latent, in
`egui-shell/src/ribbon/tabs.rs`: the weight and the colour were applied under
two independent `if`s, so a `TabCues` with `emphasised_text` and not `filled`
produced a bare `.strong()`. Unreachable through this crate's own `tab_cues`,
which derives all four cues from one flag — and reachable by anyone building
the struct by hand, which its own tests do. The guarantee rested on a
coincidence between two lines. It is now nested, so the weight cannot be reached
without the colour having been stated.

The gate also had to learn something in that run: it measures its window in
**code lines, not source lines**, because the safest shape of the fix had four
lines of comment between the colour and the weight. A gate that failed a
well-documented pairing while passing a terse one would push the next person to
delete the explanation.

---

## D10 — The theme system is built, tested, gated, and never installed

**Found 2026-08-14, by measuring pixels.** A `ui-verify` check needed to know
what colour a pressed ribbon button is, sampled it, and got
`#90D1FF` — which is `egui`'s stock `visuals.selection.bg_fill`, not the
`quiet` preset's composited `#C1CFE6`. The unpressed fill measured `#E6E6E6`,
`egui`'s `widgets.inactive.weak_bg_fill`, where the preset specifies `#E8E8EA`.

`egui_shell::theme::Theme::apply` (`theme/mod.rs:456`) **is never called from
`pdfcer-gui`.** The only mention of `egui_shell::theme` in the whole crate is
inside a test module in `icons/paint.rs`. Three presets, a palette, a
role-per-colour discipline, a rendered-pair contrast gate over all five widget
states, and its own self-test — compiled into the binary, never handed to the
`Context`.

### It is worse than "the colours are egui's defaults"

`apply` does two things, and the second is the one that bites:

```rust
ctx.all_styles_mut(move |style| Self::write_style(style, &p, &m, preset));
ctx.data_mut(|d| d.insert_temp(egui::Id::new(Self::CTX_ID), *self));
```

The stash is how `Theme::of(ctx)` — called by `ribbon/render.rs:210`,
`dock/mod.rs:472` and the splitter — reaches roles that have nowhere to live in
`egui`'s `Style`, such as the content backdrop and the label plate. With
`apply` never called, `Theme::of` returns the **default** theme, so the
framework's own chrome paints from one palette while every `egui` widget paints
from another.

`apply`'s doc comment describes precisely this and calls it the thing the
module exists to prevent:

> a dark theme with light-theme overlays, which is the two-thirds-of-a-theme
> failure this module exists to prevent, **and no test would see it**.

It was right. No test saw it.

### Why every guard missed it

- **The theme gate (`check-theme-colors`) passes**, correctly. It asserts every
  colour is a **named role in the theme module** rather than a literal at a
  call site. That is a property of the source, and it is true. Whether the
  resulting theme is ever *installed* is a different question that no grep can
  ask.
- **The contrast gate passes**, because it renders pairs *from the theme* and
  measures those. It never asks the running application what it actually drew.
- **`FEATURES.md` ticked the row**, which is the part that stings: this file's
  own bar is *"a row is ticked only when an operator can reach it in a real
  build."* Three themes an operator cannot reach were ticked for the whole of
  their shipped life — the exact failure the bar was written against, in the
  document that wrote it.

### Blast radius

Everything the operator has ever seen in this shell is `egui`'s stock light
style. There is also **no way to choose a preset**: the settings dialog is one
of the unsalvaged Class-B surfaces, so even once `apply` is wired, the preset is
whatever the code picks until that dialog lands.

Note this also means **every screenshot in `evidence/`, and every legibility and
contrast assertion `ui-verify` has ever made against the running binary, was
measured against the wrong palette.** The assertions were not wrong — the
contrast they measured was real — but they were measuring `egui`, not pdfcer.

### ★ CLOSED 2026-08-17 — both halves, and the second is proved in pixels

The first half was fixed on 2026-08-14 by calling `apply`. **The second half —
*"there is also no way to choose a preset"* — was closed on 2026-08-17 when the
Settings window landed.** `dialogs::settings::appearance` is the chooser; the
per-frame install reads the token from the draft when the window is open and
from the live settings otherwise, so a theme takes effect *as you click it* and
Cancel puts it back.

The evidence is `ui-verify --check settings_theme_takes_effect`, which drives
the real binary with no document open, expands the Appearance group, clicks
**Dark**, and measures the window's own body before and after:

```text
window body Rgb { r: 232, g: 232, b: 234 } -> Rgb { r: 45, g: 49, b: 54 }
                (#E8E8EA — quiet's panel)      (mean channel drop 183)
```

That is the only oracle that could have said so. Every cheaper one was
available throughout D10's shipped life and every one was green — the theme
gate asserts a property of the *source*, the contrast gate renders pairs *from
the theme* and never asks the application what it drew, and `FEATURES.md`
ticked the row. D10's own summary was *"No test saw it."* One does now.

### Fix

Call `Theme::apply` once per frame from the application's update, which is what
its doc prescribes (*"applied every frame rather than once at startup so a theme
change takes effect immediately, with no restart and no cache to invalidate"*).

### ★ Measured consequence: the ribbon's group captions were below the floor

Established after the fix, by driving the **two packaged builds** and comparing
— `ui-verify --check ribbon_group_captions_legible`, the same check against
each binary:

| build | measured | verdict |
|---|---|---|
| `pdfcergui-20260813-2248` (pre-theme) | **2.82:1 – 2.89:1** | **FAIL** — all five captions below the 3.0 floor |
| `pdfcergui-20260814-0735` (themed) | 4.14:1 – 4.26:1 | PASS |

So every group caption in every build this project has shipped was rendering
below its own stated contrast floor, and installing the theme fixed it as a
side effect rather than by design. The foreground moved `#959595 → #737374`
against a background that also moved `#F8F8F8 → #F2F2F3`.

**The check existed the whole time and would have caught it.** It is not one of
the eight CI gates because it needs a display and drives the real cursor and
keyboard — which is a legitimate reason, and it is also why nothing ran it
between the theme landing unwired and today. Recorded rather than fixed by
adding it to `run-all.sh`: a gate that cannot run headlessly would report
SKIPPED, and *"told you nothing rendered as green"* is the exact failure that
harness exists to remove.

**Still worth someone's judgement:** 4.2:1 clears the 3.0 floor, which is WCAG
AA for **large** text. A group caption is ~10 pt, i.e. small text, where AA asks
**4.5:1**. The captions pass the bar this project set and would fail the bar
their size implies. Not changed here — the palette is the theme's to decide and
the contrast gate's floor is a deliberate constant — but the next person to
touch either should know the margin is 0.3, not comfortable.

---

## D9 — Every imported reduced-opacity markup renders solid — **FIXED 2026-08-14**

> **Closed.** `pdfcer-render` now reads §12.5.2 `/CA` and composites the
> annotation's appearance through a scratch pixmap at that alpha — engine
> commit `a84bdc3`, carried by the `pdfcergui-20260814-0735-e8e9881-9c81b04`
> build. `alpha >= 1.0` short-circuits, so the common path allocates nothing.
>
> Their commit records why it jumped their queue, and it is this document's
> argument returned: *"reported by the `pdfcer-gui` session, which correctly
> ranked it as a **fidelity defect in the current product** rather than a
> prerequisite of a future authoring control."* The entry below is kept
> because the reasoning is the durable part — a question asked about
> *authoring* that turned out to be about *viewing*.
>
> Still open, and unchanged: **do not compensate.** When markup opacity
> authoring lands, write `/CA` alone and leave the appearance stream's
> `ExtGState` at `1.0`.

**Found 2026-08-14, on the pdfcer side, by asking a question about
authoring.** Not observed here first — which is the notable part, and the
reason it is written down rather than left in the request channel.

`pdfcer-render` **does not read an annotation's `/CA`** (§12.5.2 constant
opacity) at all. Their measurement, quoted from
`archive/2026-08-14-markup-opacity-reply.md`:

> ```
> grep '/CA' in pdfcer-render        -> ONE hit, interpret.rs:2050
>                                      and it is ExtGState /CA (stroking alpha)
> annotation paint path (annot.rs)  -> ZERO reads of the annotation dict's /CA
> ```
>
> `paint_appearance` interprets the form XObject straight into the page
> pixmap. Nothing consults the annotation's constant alpha.

### Why it costs more than it looks

The question that uncovered it was about markup **this shell would author**,
and in that framing it is a Phase 6 prerequisite. It is not. **This shell is
a viewer before it is an editor, and the defect is shipping in that role
today.**

Reduced opacity is the house style for a shaded area or a fill placed over a
drawing — the whole point being that the drawing underneath stays readable.
Markup arriving from Bluebeam and Acrobat uses it constantly, and the stated
audience is drawing review. Rendered solid, it does not read as "the opacity
is wrong"; it reads as **the markup covered the drawing**, and the drawing is
what the operator opened the file for.

### Status

**Filed and scheduled on the pdfcer side as its own piece of work**, deliberately
not folded into the opacity feature — their words: *"it is a correctness bug
with a blast radius wider than this request and it should not be discovered
later as 'the opacity feature also changed how imported markup looks'."* The
fix is a real change to `paint_appearance` (composite the appearance through a
scratch pixmap at alpha rather than interpreting it into the page), not a line.

### What this shell must NOT do about it

**Do not compensate.** When markup opacity authoring lands, write `/CA` alone
and leave the appearance stream's `ExtGState` at `1.0`. Writing `/ca` into the
AP would make the markup look right in pdfcer and **half as opaque as intended
in every other viewer, permanently, in documents that outlive the bug**. That
is encoding a render defect into the file format. The full three-way table is
in the archived reply.

Nor should the Style group draw an Opacity control before the renderer lands:
a control that visibly does nothing in the application you are using is not a
partial feature (`RIBBON_IA.md` P3).

### Not measured

How much of the operator's own corpus carries reduced-opacity markup is
**unknown and is not asserted anywhere**. If that number would change
anyone's priority, it needs counting on a real corpus rather than estimating.

---

## D8 — Housekeeping

A stale worktree at
`D:\Dev\pdfcer\.claude\worktrees\agent-ad491473a5659e3eb\` contains an
older `main.rs` in which `editing_enabled` defaults differently and a
test asserts `!doc.editing_enabled` (line 23274). It pollutes repo-wide
greps and will mislead the next investigation. Delete it.

---

## Not defects — deliberate choices worth re-examining anyway

These are working as designed. They are listed because the design is
what generates the complaint.

| Behaviour | Where | Why it reads as broken |
|---|---|---|
| Zoom buttons pin the page's **top-left**, not the centre or the cursor | observed; `viewer.rs` ladder | Every mainstream viewer zooms about the centre or the pointer. Zooming in loses your place. Note this is about the *anchor*, not the smoothness — the whole-page-texture model is a deliberate and well-judged trade, see `GUI_ROADMAP.md` § Rendering. |
| The status bar opens with a substitute-glyph census | `main.rs:15576-15960` | The first thing a user reads is the app talking about itself. Excellent information, wrong prominence — put it behind the disclosure triangle that is already there. |
| Dock layout resets every launch | `dock.rs:50-67` — disclosed in-app | Being told your layout will be lost is better than losing it silently, and worse than keeping it. |
| No context menus anywhere | `grep context_menu` → 0 hits | Right-click is where users look for Delete after the keyboard fails them. Fixing D1 without adding these leaves the second-choice path also missing. |

---

## D12 — the glyph gate asked `egui` the wrong question — **CORRECTED 2026-08-14**

> ### ★★ Correction, 2026-08-14 — `⚠` draws. It always did.
>
> **The diagnosis below is wrong, and the thirteen sentences it condemns
> render correctly.** The heading used to read *"`⚠` has no glyph in this
> font stack, so thirteen shipped sentences draw `□`"*. It is kept, struck
> through in substance rather than deleted, because the wrong claim
> travelled: it was quoted into
> `app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`'s doc
> comment and into `text::status::edit_disclosure_line`, and a reader who
> finds only the correction will not know why those two files talk the way
> they do.
>
> **What is actually broken is the gate's predicate, not the font stack.**
> `epaint 0.35`'s `Fonts::has_glyph` (`epaint-0.35.0/src/text/font.rs:720`)
> is:
>
> ```rust
> pub fn has_glyph(&mut self, c: char) -> bool {
>     // TODO(emilk): this is a false negative if the user asks about the
>     // replacement character itself 🤦‍♂️
>     self.resolve_face(c) != self.cached_family.replacement_face_key
> }
> ```
>
> It does not ask *"is this codepoint drawable?"* It asks *"is this
> codepoint drawable by a face other than the one that happens to supply
> `epaint`'s substitution mark `◻` (U+25FB)?"* — and answers **false** for
> every codepoint whose first supporting face in the fallback chain is that
> one. Upstream's `TODO` names a single instance; the real blast radius is
> every character that face supplies first.
>
> For `FontFamily::Proportional` the chain is
> `[Ubuntu-Light, NotoEmoji-Regular, emoji-icon-font]`, and **`◻` and `⚠`
> have the same supplier — `NotoEmoji-Regular`.** So `⚠` is reported
> missing and drawn perfectly.
>
> **The mechanism reproduces the original's own two lists exactly, 31 for
> 31**, which is what makes it the mechanism rather than a theory. Reading
> the four bundled charmaps directly:
>
> | original verdict | real supplier |
> |---|---|
> | "available" — `✱ ⚑ ⚐ ☞ ⊗ ⏺ ◊ ★ ☆ ! ○ ■ • · † ‡ № ¶` | all `Ubuntu-Light` or `emoji-icon-font` — **never** NotoEmoji |
> | "absent" — `⚠ ‼ ℹ ❗` | **`NotoEmoji-Regular`. All four draw.** |
> | "absent" — `▲ △ ● ◆ □ ✓ ✗ ⓘ ※` | genuinely absent. That half was right. |
>
> The clinching reading is that `has_glyph(Monospace, 'A')` is **false**:
> the monospace chain starts with `Hack`, which supplies both `◻` and `A`.
> A predicate that denies the letter A is not a fact about a font.
>
> **Corrected measurement**, `FontFamily::Proportional`, by laying each
> character out and comparing the glyph actually drawn:
>
> - **Drawable:** `⚠ ‼ ℹ ❗ ✱ ⚑ ⚐ ☞ ⊗ ⏺ ◊ ★ ☆ ! ○ ■ • · † ‡ № ¶ — … × “ ” − ° ⏴ ⏵ ⏷`
> - **Absent:** `▲ △ ● ◆ □ ✓ ✗ ⓘ ※ ▸ ◀ ▶ ▾ � (U+FFFD)`, and all CJK.
>
> **Consequences of the correction:**
>
> 1. `crates/pdfcer-gui/src/text/forms.rs`'s `⚠` sentences are **fine** and
>    need no edit. There are **fourteen** of them, not thirteen — counted as
>    string literals opening with the mark, `grep -c '"⚠'`; the original's
>    thirteen appears to have missed one. They are the only `⚠` in the whole
>    catalog.
> 2. The assertion at
>    `crates/pdfcer-gui/src/panels/forms/tab_order/mod.rs:672`
>    (`s.starts_with('⚠')`) was never at risk. Unchanged, still passing.
> 3. The edit-disclosure line's `⚑` was chosen under the wrong reading.
>    **Deliberately left alone** — it draws, it is shipped, and re-opening a
>    settled copy decision on the strength of a corrected diagnosis is churn.
> 4. The operator's 2026-08-14 instruction — *keep the `⚠` mark, add font
>    coverage* — is satisfied with **no font added and no dependency added**,
>    because the coverage was never missing. See "Fixed" below.
>
> **The lesson, which is the durable part.** This entry was filed on a
> failing test, and the test really did fail — but *"the gate went red"* and
> *"the thing the gate names is broken"* are different claims, and only the
> first was measured. The original text's own standard, three lines up in
> this file, is *"Nothing here is inferred from documentation alone"*; the
> failure here was subtler and more ordinary — inferring from a **tool's
> answer** without asking what question the tool was answering. The
> substitution box was never photographed. One screenshot of the Forms panel
> would have closed this on the day it was opened.

### The original entry, kept — *wrong from the second paragraph onward*

**Found 2026-08-14, by measurement rather than by looking.** A new status-bar
line was drafted with `⚠` to match the forms convention, and the existing gate
`every_glyph_the_status_bar_draws_has_a_glyph` **failed** on it.

Nothing in this workspace installs fonts, so `egui`'s bundled set is the whole
set, and it cannot draw **U+26A0**. `crates/pdfcer-gui/src/text/forms.rs`
carries `⚠` in **thirteen** sentences — including
`forms_fill_autosize_note` and `forms_fill_unencodable_note`, which are drawn
in the status bar two lines from where the new one goes — and every one of them
renders as a tofu box today, in the Forms panel and in the bar.

> *Wrong on every count in that paragraph. The sentence count is fourteen,
> not thirteen. `egui`'s bundled set is indeed the whole set — and it draws
> U+26A0 perfectly well.*

This is **D2's shape, fourth sighting**: a thing that is built, tested and
shipped, whose visible result nobody looked at. A unit test on the *string* is
satisfied by any string; only asking the font whether it can draw the codepoint
catches it. The gate that caught it already existed and was never pointed at
`text/forms.rs`.

> *★ This paragraph is the part that survives, and it turned out to be truer
> than its author knew. "Nobody looked at the visible result" was the real
> defect — including here, where nobody looked at the visible result of the
> gate's own verdict. And "only asking the font whether it can draw the
> codepoint catches it" is exactly right; the mistake was believing
> `has_glyph` was that question.*

**Measured available** in the bundled stack: `✱ ⚑ ⚐ ☞ ⊗ ⏺ ◊ ★ ☆ ! ○ ■ • · † ‡ № ¶`
**Measured absent**: `⚠ ▲ △ ● ◆ □ ✓ ✗ ‼ ℹ ⓘ ※ ❗`

> *Both lists are `has_glyph` output. See the correction's table: the first
> is accurate, the second contains four false positives — `⚠ ‼ ℹ ❗`.*

**Not fixed**, deliberately, because it is wider than it looks: thirteen
strings, plus an assertion at
`crates/pdfcer-gui/src/panels/forms/tab_order/mod.rs:672` that tests
`s.starts_with('⚠')` and would silently stop matching. The new edit-disclosure
line uses `⚑`, measured present, rather than joining the convention.

> *The caution was well judged even though the premise was false. Had this
> entry been "fixed" as written, fifteen correct sentences would have been
> rewritten to work around a bug in a test.*

**The fix that would prevent a fifth sighting** is not a substitution: it is
pointing the existing glyph gate at *every* `text/` module rather than at the
status bar alone, so a codepoint the stack cannot draw fails at the gate rather
than in front of the operator.

> *★★ Right, and it paid off on its first run — see below. This sentence is
> the reason the entry was worth filing at all.*

### Fixed 2026-08-14

**No dependency added. No font data added. No catalog string changed.**

| what | where |
|---|---|
| A correct predicate — lay the character out, compare the glyph actually drawn against a three-sentinel fingerprint of the substitution mark | `crates/pdfcer-gui/src/icons/glyphs.rs` — `GlyphProbe` |
| The **widened gate**: reads every `.rs` under `crates/pdfcer-gui/src/text/` from source, extracts every operator-visible literal, and checks every codepoint | `icons::glyphs::tests::every_glyph_the_catalog_draws_has_a_glyph` |
| The gate's self-test, on a planted unrenderable codepoint with comment and test-module decoys | `icons::glyphs::tests::the_gate_catches_a_planted_unrenderable_codepoint` |
| The status-bar gate, repointed at the correct predicate and its doc comment corrected | `crates/pdfcer-gui/src/app/status.rs` |

The gate reads **source** rather than a hand-written list of labels, so a
string added tomorrow is covered without anyone remembering to add it. That is
`D5`'s lesson applied: *"a hand-maintained list with a comment telling you to
hand-maintain it has already failed once."*

Three fail-open shapes were designed out, each with its own test:

- **A sentinel that stopped being a sentinel.** `GlyphProbe::new` fingerprints
  the substitution mark from **three** unrelated unassigned codepoints across
  three planes and panics unless all three agree. If a future font set covers
  one, the probe fails at construction instead of silently reporting every
  codepoint as drawable.
- **`D13`'s truncation bug, not repeated.** `check-ui-strings.sh` stops
  scanning at the first column-0 `#[cfg(test)]`, so anything below a mid-file
  test module is unscanned while the gate prints clean. This scanner skips
  exactly the braced item and **resumes**;
  `a_mid_file_test_module_does_not_blind_the_scanner` proves it on the shape
  that defeats the shell gate.
- **A file that could not be parsed being silently skipped.** A raw string is
  a hard refusal that fails the gate by name, never a quiet zero.

### ★ The fifth sighting happened anyway — the widened gate found two on its first run

Both are **live tofu today**, both in `crates/pdfcer-gui/src/text/`, which is
not the territory of the work that found them. They are **quarantined in the
gate and reported here, not fixed.** The quarantine is self-tightening: the
gate asserts each entry is *still* undrawable **and** still present in the
catalog, so fixing the strings makes the gate fail telling you to delete the
entry.

| codepoint | where | what the operator sees |
|---|---|---|
| **`▸` U+25B8** — the menu-path separator | `text/mod.rs:125`, `text/commands.rs:722, 767, 1079` | `Choose File □ Open` — and `text/mod.rs:125` is the **empty-canvas message, the first sentence a new operator ever reads.** `›` U+203A, `>` and `→` all draw. |
| **`�` U+FFFD** | `text/panels/objects.rs:639` | The sentence *"Some characters … are shown as `�`"* names a mark the application cannot draw. It reads correctly only by accident: `epaint` substitutes `◻` **both** for the character in this sentence and for the undecodable characters the sentence is about, so the two happen to match. A coincidence of two bugs, not a design. |

`▸` is the more serious of the two by a distance, and it is the vindication of
this entry's closing argument: the codepoint had been shipping in the launch
screen the whole time, the old gate could not see it because it looked only at
the status bar, and the *corrected* diagnosis of `⚠` is what got the gate
pointed somewhere it could find it.

### ★ Both verdicts were photographed, not only computed

The mistake this entry records is *trusting a tool's answer without looking at
the result*, so neither half of the correction is left resting on another
assertion. Driving the release binary
(`target/release/pdfcer-gui.exe`, 2026-08-14 12:27, `PDFCER_DIAG=1`):

| what was opened | what was on screen |
|---|---|
| `qpdf/qtest/qpdf/button-set-broken-out.pdf` — a 15-field form with `/NeedAppearances` | The Forms panel drew **two `⚠` sentences as amber warning triangles**: *"⚠ This form asks viewers to draw field values themselves…"* and *"⚠ 2 field(s) have no drawn appearance in this document…"*. Two of the fourteen this entry condemned. Neither is a box. |
| the binary with **no argument** | The empty canvas read *"No document open. Choose File **□** Open, press Ctrl+O, or start pdfcer with a PDF path."* — the `▸` tofu, live. |

Corroborated at the pixel level by dumping the glyphs `egui` actually
rasterizes into its own font atlas at 48 pt: `⚠` is a filled triangle
enclosing an exclamation mark, 43×38 px; `▸` is a hollow 30×30 square, which
is `◻` — the substitution mark, not the separator.

`D:\Dev\temp\pdfcer\SW41177.pdf` was opened first, as directed, and reached the
Forms panel — but it carries **no** interactive fields, so its panel correctly
draws the *"this document has no interactive form fields"* sentence and no `⚠`
at all. It could not have settled the question either way, which is why a form
fixture was opened as well. Recording that rather than reporting the first
screenshot as if it had confirmed something.

---

## D13 — A mid-file `#[cfg(test)]` silently switches the ui-strings gate off for the rest of the file

**Found 2026-08-14.** `tools/gates/check-ui-strings.sh` stops scanning a file at
the first column-0 `#[cfg(test)]` — its own header records this as a deliberate
limit, on the reasoning that test code below it is not operator-facing. The
limit is sound; the **assumption** is not. Nothing requires the test module to
be last, and where it is not, every non-test item after it is unscanned **and
the gate reports clean**.

Proven rather than argued: a violation planted after line 262 of
`crates/pdfcer-gui/src/panels/forms/edit.rs` **passes the gate**.

Three files are affected today:

| file | non-test items below the test module |
|---|---|
| `panels/forms/edit.rs` | 7, including `pub fn apply` |
| `canvas/guides.rs` | 6 |
| `panels/layers.rs` | 5 |

This is the **`check-file-size` fail-open class again** — the same shape as
PORT CHANGE 1 in `check-ui-strings.sh`'s own header, where a flat glob scanned
three files out of forty and printed the same output a clean run prints. *"Found
no violations"* and *"looked at almost nothing"* remain byte-identical.

**Not fixed** — the three files are outside the territory of the work that found
this, and the fix is the gate's, not theirs. Two candidate fixes, and the second
is better: scan the whole file and exclude only items *inside* a `mod tests`
block; or keep the early exit and add a gate assertion that the test module is
the **last** thing in the file, which is a convention this codebase already
follows nearly everywhere and which a self-test can prove it catches.

---

## D14 — Every freehand ink stroke authored two points — **FIXED 2026-08-14, same session**

**Found by driving the binary; invisible to a green suite by construction.**

`canvas::markup::ink::sync` read the in-flight pointer trail *after*
`GestureState::update` had already advanced. `update` drops its own drag on
the frame it reports `Complete` — so on **exactly** the frame the release
arrived, `active()` answered `None`, and the accumulated trail was discarded a
few lines before the arm that commits it. A freehand stroke hundreds of points
long authored an annotation with **two**.

### Why no test could see it

Every unit test calls `drag` directly. **None of them can see the order in
which `canvas::interact` calls two functions**, because that order is a
property of a call site and a call site's effect is only observable in a
running frame. This is `HANDOFF.md` §2's recurring shape — the same one that
produced the icon painter that was never passed to the ribbon, and the
page-text extraction paid at open rather than on the gesture.

### How it was found

The trace line, on a drag the harness had made hundreds of points long:

```
markup-commit kind=Ink page=0 raw=2 kept=2
```

`raw=` is printed beside `kept=` **for this reason**: a build whose
simplification did nothing, and a build whose trail was empty, produce
otherwise identical lines. Without the pair the number would have read as a
successful simplification of a two-point drag.

### Fix

Read the trail **before** the gesture machine advances. Recorded at
`canvas/markup/ink.rs` §2 with the measured symptom, so the ordering is stated
where the next reader will meet it rather than rediscovered.

### The general lesson

**A diagnostic that prints only its output cannot distinguish "worked" from
"had nothing to work on."** Print the input beside it. That is cheap, and it
is what turned an invisible defect into a one-line read.

---

## D15 — `ocrs` collapses on a sparse clean page, which is the shape of a drawing sheet

**Found 2026-08-14 while building the OCR fixture. Not a pdfcer defect — an
upstream characteristic this project has to design around, and it matters here
more than for most consumers.**

A first OCR fixture of **two words on an otherwise empty page** produced a
detection result of *the whole page as one rectangle*. The probability map was
dumped and inspected: it was **perfect** — four clean blobs, four connected
components counted by hand. The failure was downstream, in thresholding.

`ocrs`'s `text_threshold` defaults to **0.2**. The measured background on that
page ran **0.148–0.208** — so the threshold sat inside the noise floor and the
whole page crossed it.

### Why this is not an academic edge case here

**A drawing sheet is exactly that shape**: a small title block, a handful of
dimension callouts, and a very large expanse of empty paper. `SW41177.pdf` and
the A1 benchmark are both far sparser than the scanned prose OCR engines are
tuned for. An operator OCR-ing a scanned drawing is the *most likely* user of
this feature and is walking into the worst case for it.

### What was done, and what deliberately was not

The **fixture was changed**, not the threshold. Tuning a recogniser's internals
to make a test pass is how a shell starts carrying an engine's opinions: the
number would be ours, the failure would still be theirs, and the next `ocrs`
release would silently disagree with us.

### What remains

Unquantified on real scanned material, because **there is none in the tree**.
If a scanned drawing ever arrives, this is the first thing to measure — and if
it reproduces, the honest fix is upstream or a documented refusal, not a magic
number in `pdfcer-gui`.

---

## D16 — Ctrl+S saved the file and then killed the application — **FIXED AND DRIVEN 2026-08-29**

**Present in the shipped build.** Every in-place save of a document opened from
disk wrote the file correctly and then panicked the process. Introduced
2026-08-20 with `file.save`; found 2026-08-29 by an agent wiring an unrelated
guard into that arm, **not** by a test, **not** by the audit that session was
running, and not by any of the 105 driven checks.

### The code

`PdfcerApp::apply` matches the action **twice**: once before the "is a document
open" guard, for the handful of actions that must answer differently with
nothing open, and once after it for everything else. Every arm in the first
match ends with `return`.

`Action::Save`'s did not.

```rust
Action::Save => {
    match &mut self.status { … }          // saved, correctly
}                                         // ← no `return`
…
_ => {}
}                                          // first match ends
let Status::Open(doc) = &mut self.status else { … };
match action {
    …
    | Action::Save                         // ← and here it is again
    | Action::SaveCopy
    | Action::Find(_) => unreachable!("handled before the document guard"),
```

`SaveCopy` and `Find`, its two neighbours in the first match, both return.

### ★★★ The class, which is what makes it worth a number

**A fall-through arm whose later twin asserts unreachability. Both halves
type-check and neither is wrong on its own.**

- The `unreachable!` is **correct**: the arm *is* handled earlier, and the
  assertion documents a real invariant.
- The earlier arm is **correct** except for one keyword, and it reads correctly:
  it does the work, it traces, it records the epoch.

Nothing about either site is suspicious in isolation, and a reviewer reading
either one alone would approve it. The compiler cannot help: falling out of a
`match` arm into the following statements is ordinary control flow.

⇒ The general form: **when one `match` is split into a pre-guard pass and a
post-guard pass over the same value, `return` is load-bearing in every arm of
the first, and the second pass's `unreachable!` converts a missing one from a
silent double-handle into a crash.** The crash is the better outcome — it is at
least loud — but only if somebody presses the key.

### ★★ Why no test and no driven check caught it

- `PdfcerApp::apply` is called with `&mut self` on a real application; the unit
  suite exercises actions through smaller seams.
- **No driven check drives a save.** `save_in_place` and `save_copy` have unit
  tests that call `crate::app::save::save_in_place(doc)` **directly** — which is
  the function that works. The defect is in the arm that calls it.
- The gap is the same one recorded twice this week for gestures: *which check
  drives this?* For `Ctrl+S`, the answer was **none**, and R1 exists for exactly
  that answer.

### The verification, both directions

Driven offscreen (`PDFCER_DIAG_VIEWPORT` + `PDFCER_DIAG_INVOKE=file.save`) against
a scratch copy of `fixtures/a1-titleblock.pdf`, so the operator's pointer and
focus were untouched:

| build | result |
|---|---|
| fixed (this commit) | `save-in-place outcome=ok`, `save-epoch-recorded epoch=0`, **process alive after 8 s** |
| the `return` removed again, deliberately | `save-in-place outcome=ok`, `save-epoch-recorded epoch=0`, then `panicked at apply.rs:333`, **exit code 101** |

★ The falsification is the half that matters: the fix was re-broken on purpose
and the crash came back, so the pass is a measurement of *this* change rather
than of something else that moved.

★★ Note the order in the trace — **the file is written before the panic.** No
work was lost; the application simply died immediately afterwards, which is why
the symptom is *"pdfcer disappears when I press Ctrl+S"* rather than *"my save
did not happen"*.

---

## D17 — The signature warning's *Save anyway* was inert, so no signed document could be saved at all — **FIXED AND DRIVEN 2026-08-29**

**Present for one day.** The guard that stands between a structural edit and a
signed document's next revision shipped 2026-08-28 and was found by driving the
next morning (`an_invalidating_save_is_warned_about`, sweep
`evidence/sweep-20260829/main.txt`). It stopped the save correctly and then
**never let it through**: an operator on a signed document could cancel the
question and nothing else. That is worse than the silence it replaced — the
feature turned a working save into no save.

### What the trace showed

`target/ui-verify-main/signature-save.trace.txt`, in order:

```text
pages-deleted removed=1 …              the save is structural
signature-asked pending=Copy           the guard held it and the window drew
viewport-inner id="2DBB" rect=…        the dialog is its own OS window
ui-rect name=signature.proceed … viewport="2DBB"
…
ui-rect-gone name=dialog:signature     ← the press LANDED: the window closed
                                       ← and no `signature-confirmed`, ever
```

★ The first suspicion was the harness — the button's rectangle is published in
the **child viewport's own frame**, and this project's record
(`a_child_viewports_ui_rects_are_relative_to_ITS_origin`) is six checks clicking
hundreds of points from the control they named. The trace rules it out in one
line: the window **closed**. Only the proceed button, Cancel or the ✕ can do
that, and the other two also close it without an answer — so the press was
delivered and the answer was lost afterwards.

### The code

`dialogs::signature::SignatureDialog::show` returns *"should I still be on
screen?"*, and pressing the proceed button is exactly what makes it answer
`false`:

```rust
open && !self.cancelled && !self.confirmed
```

Its owner read that `false` as *"this dialog is finished"*:

```rust
if self.signature.as_mut().map(|d| d.show(ctx)) == Some(false) {
    self.signature = None;          // ← with the answer still inside it
}
```

But this window deliberately **does not act**. It parks the answer, and
`PdfcerApp::resume_after_signature` performs it — later in the same frame —
because writing over the operator's own file must have exactly one route.
The dialog was therefore destroyed, with the confirmation in it, three call
frames before the drain looked; `take_signature_answer` found an empty slot and
returned `None`.

### ★★★ The class

**A slot whose occupant carries a value the owner has not collected, retired on
a signal that means "stop drawing me" rather than "I am empty".**

Every part is individually correct. `show` correctly wants to close.
`take_confirmation` correctly returns the answer when asked. `resume_after_
signature` correctly performs whatever it is given. **The defect is entirely in
the lifetime between them, and a lifetime is not a value any assertion over
either half can name** — which is why `dialogs/signature.rs`'s own headless
tests, which assert the engine's verdict *and* that `ask_for` builds the window,
all pass on the broken build. It is the whole-link failure class
`PROJECT_PLAN.md` §4 built the driving harness for, and the check's own header
had listed this exact outcome as row 3 of the builds it must fail against.

### The fix

One predicate and one rule, in `dialogs/mod.rs`:

```rust
const fn retire(open: bool, answered: bool) -> bool { !open && !answered }
```

A dialog is dropped only when it is off screen **and** holding nothing.
`SignatureDialog::answered()` and `UnsavedDialog::answered()` are the second
input. The invariant it creates is stated at `retire`: *every caller of
`DialogsState::show` must drain the parked answers in the same frame* — there
is one caller, `app::frame`, and it drains both immediately after, so a
retained-because-answered dialog lives for zero frames.

### ★★ Its twin was fixed in the same change, unprompted

`dialogs::unsaved` parks an answer the same way, two lines above, through the
same branch. **Nothing in the harness clicks it** — no check presses *Close
without saving* — so it was carrying the identical defect with no red run to
advertise it. Its symptom would have been worse: a *Close without saving* that
closes the question and leaves the document open, which reads as the whole
application ignoring the operator.

⇒ The general form: **when a driven check finds a defect in one member of a
matched pair, the pair is the unit of repair.** Fixing only the observed half
leaves the survivor looking deliberate.

### The verification

Driven, on the real binary, against `fixtures/signed-two-pages.pdf`:

```text
[PASS] an_invalidating_save_is_warned_about
  · the save is structural: pages-deleted removed=1 freed=2
  ★ the save was held and the window drew: signature-asked pending=Copy
  ★★ no file was written while the question was on screen
  ★ the operator authorised it: signature-confirmed pending=Copy
  ★ the write ran: save-copy … bytes=1973 … deleted=2 epoch=1
  ★★★ 1973 bytes of PDF reached target/ui-verify-sig\signed-copy.pdf
```

The guard now blocks **and** releases, which is the whole claim, and the two
halves are asserted in one run so a build that never writes cannot satisfy the
absence in the middle.

---

## D18 — Every resize runs `1/zoom` too fast, because the drag's travel arrives in a different space from the box it is measured against

**Severity:** high · **Fix:** one line, plus a doc comment · **Found:** 2026-08-29,
while proving which side of `shift_constrains_a_resize` was wrong · **FIXED AND
VERIFIED 2026-08-29.** `PageMapping::page_vec_to_screen` is the conversion, and
the Resize arm names the space at the call site rather than leaving it to a doc
comment two files away. Re-run on `polyline-nodes.pdf`: `resize-commit
sx=1.1654 sy=1.4410` where the same class of drag previously committed
`sx=1.5200 sy=5.9439`. `resize_scales_a_shape`, `shift_constrains_a_resize` and
`rotate_handle_turns_a_selection` all PASS. Three unit tests on the conversion
pair, including the degenerate-zoom case, which answers `Vec2::ZERO` rather than
a NaN — a NaN displacement reaching a content stream is a corrupted file, a zero
one is a gesture that did nothing. **Was: not fixed
here** — see *Why this entry is a record rather than a change*.

At any zoom below 1.0 a grip drag scales the object by far more than the
pointer moved, and the grabbed corner runs away from the hand. At the zoom the
sweep runs at — 0.2955 — a 60 px drag on a 390.6 × 41.0 px box committed
`sx=1.5200 sy=5.9439` and left the south-east corner **143 px** beyond the
cursor on both axes. The gesture works, commits, undoes and announces itself
correctly. It is simply the wrong size, and it is exactly right at zoom 1.0,
which is where every unit test lives.

### The contract, quoted from the module that owns it

`crates/pdfcer-gui/src/canvas/resizing.rs`, on `Frame`:

```rust
/// How far the pointer has travelled since then, in screen points.
pub delta: Vec2,
...
/// The selection's grip box in screen space, or `None` if there is no
/// outline to have grabbed.
pub bounds: Option<egui::Rect>,
```

and `factors` divides the first by the second:

```rust
let sx = if dw == 0.0 { 1.0 } else { (w + dw) / w };
```

Two quantities, one ratio, one stated space. The ratio is only meaningful
because both operands are promised to be in it.

### Where the promise is broken

**`bounds` keeps it.** `interact.rs`'s `GestureOutcome::Resize` arm takes it
from `pressing::grabbable` → `overlay::grip_box`, which is
`mapping.rect_to_screen(union)` — screen space, as documented, and the same
rectangle the selection outline is drawn from.

**`delta` does not.** The gesture machine works in **page** space by design.
`interact.rs` builds its `PointerFrame` as

```rust
pos: screen_pos.map(|p| map.to_page(p)),
press_origin: ctx.input(|i| i.pointer.press_origin()).map(|p| map.to_page(p)),
```

and `gesture::Drag::outcome` answers `let delta = self.latest - self.origin;`.
So `GestureOutcome::Resize.delta` is a **page-space** displacement, and the
Resize arm hands it straight to a field documented as screen points. The
committed factor is therefore

```text
s = 1 + (d_screen / zoom) / extent_screen      instead of      1 + d_screen / extent_screen
```

— every factor's distance from unity inflated by `1/zoom`.

### Measured three times, on two different verbs, in one sweep

All from `evidence/sweep-20260829/`, on `SW41177.pdf` at `zoom=0.2955`. The
selection box for the first two is `[[316.4 580.8] - [707.0 621.8]]`, i.e.
390.6 × 41.0 px.

| trace | drag, screen px | committed | what the contract predicts | what the mismatch predicts |
|---|---|---|---|---|
| `resize.trace.txt` | 60 × 60 | `sx=1.5200 sy=5.9439` | 1.1536 / 2.4634 | **1.5197 / 5.9512** |
| `shift-constrains.trace.txt` | 90 × 12 | `sx=1.7799 sy=1.9888` | 1.2304 / 1.2927 | **1.7798 / 1.9902** |
| `scale-switch.trace.txt` | 23 × 14 on a 94 × 54 box, `resize-annot-commit` | `sx=1.8282 sy=1.8775` | 1.2447 / 1.2593 | **1.8282 / 1.8775** |

The third is a **markup annotation**, through `resize_annotation` rather than
`transform_objects`, so this is not confined to page content: the mismatch is
above the branch and reaches all three destinations — page content, markup, and
a form field's box.

### The visible symptom is `drag-moves` D8, which this module claims

`resizing`'s own conventions table says:

> D8 grab-point: the pivot is the OPPOSITE corner, so the grabbed corner tracks
> the pointer and the far one stays still.

It does not. In `resize.trace.txt` the pointer released at window (767, 682)
and the outline's south-east corner landed at (910.1, 824.9). An operator
dragging a corner at a fitted zoom watches the shape shoot past their cursor.

### ★★ Why a green suite never said so

1. **Every unit test in `resizing.rs` is the zoom-1.0 case.** `factors(Grip::SouthEast, box_100x50(), Vec2::new(50.0, 25.0))` passes a box and a delta that are trivially in the same space, so the mismatch is unobservable by construction. The tests are correct and prove nothing about the wiring.
2. **`resize_scales_a_shape` asserts that a resize HAPPENED**, not that it matched the pointer. It passes on this build and would pass on any inflation factor.
3. **The one check that compares a committed factor against a number the harness chose** was `shift_constrains_a_resize`, and it compares `locked.sx` against `free.sx` — both inflated by the same constant, which cancels.

⇒ The general form, and it is the fourth time this project has met it: **a ratio
whose two operands come from different call paths has no test unless something
asserts the ratio against a number chosen outside the program.** Every assertion
here was of the shape "the same quantity twice", and a common factor is
invisible to all of them.

### The fix

One line, and there is a choice of which line:

- **At the call site** — `interact.rs`'s `GestureOutcome::Resize` arm converts
  the page-space delta back to screen before it becomes `Frame::delta`, so the
  field matches its documented space and nothing in `resizing` moves. Smallest
  diff; keeps a conversion the shell does twice.
- **In the space `Frame` speaks** — take `bounds` from
  `selection.outline_union()` (page space) instead of `grip_box`, restate both
  doc comments as page space, and drop the `map.to_page(anchor_screen)` hop
  under `grip.pivot(bounds)`, which then already holds a page point. Fewer
  conversions and one fewer chance to do one twice, which is
  `canvas::mapping`'s standing argument — but it moves the pivot arithmetic and
  needs the annotation and form-field branches re-read.

Either way the doc comments are part of the fix, not decoration: the field said
screen points and the caller passed page points for however long this has been
here, and the next reader gets whichever sentence is left standing.

### Why this entry is a record rather than a change

The session that found it was scoped to two driven checks and explicitly
forbidden from running the rest of the suite, and this change moves the numbers
that every resize-related check asserts on across all three destinations. It
also lands in `canvas/interact.rs`, which other agents had open at the time.
**A behaviour change of this reach that cannot be verified in the run that makes
it is the thing this project's rules exist to prevent**, so the evidence is
filed and the change is not made. It wants its own pass, with
`resize_scales_a_shape`, `shift_constrains_a_resize`,
`the_line_weight_switch_reaches_the_resize`, `widget_move` and the annotation
resize checks all re-run against it.

★ A note for that pass: `the_line_weight_switch_reaches_the_resize` SKIPPED in
this same sweep reporting *"a non-uniform drag"*, and D18 is **not** the cause —
its travel is equal fractions of the shape (23.5 and 13.5 px) which the driver
rounds to 23 and 14 integer cursor pixels, and 23/94 ≠ 14/54 whatever space the
ratio is taken in. What D18 does is **multiply that rounding error by 3.4**,
turning a 0.0146 spread into a 0.0493 one. Fixing D18 will not make that check
pass; it will make its failure smaller, which is worse. That check needs a
travel the driver can hit exactly.

---

## D19 — The Delete key's annotation gate read a selection that had been moved off the document, so it was `false` on every frame of the program's life — **FIXED AND DRIVEN 2026-08-29**

**Severity:** critical · **Fix:** one argument · **Shipped:** 2026-08-28,
found by driving on 2026-08-29, open for about eighteen hours.

This is R83's own subject surviving the change that closed R83, on the one
surface of the three that could not be checked by reading the code.

### What the operator would have met

Open a certified drawing. Click a comment. The Properties panel says, correctly
and permanently:

> this document carries a certification signature whose permissions are
> enforced (ISO 32000-1 §12.8.4, /Perms /DocMDP, P=2); structural page changes
> are not among the changes it permits, so pdfcer refuses rather than silently
> breaking it

The Format tab's *Delete* is withheld. The canvas menu's *Delete* is withheld.
Then press the **Delete key** — and the comment does not go, nothing is said,
**and the sentence disappears**, because the selection was cleared by a delete
that never happened. The one surface that cannot be undrawn was also the one
surface that never asked.

### Causal chain

1. `canvas::keys` grew the gate on 2026-08-28. Its annotation rung reads
   `Keys::annot_delete_refused` and, when set, writes
   `canvas-delete-declined … reason=annot-delete-refused` and returns without
   raising the action. That code is correct and has eleven unit tests.

2. `canvas::interact` fills that field, at what was `interact.rs:1242`:

   ```rust
   annot_delete_refused: crate::panels::properties::annotdelete::refuses_selected(doc),
   ```

3. `refuses_selected` asks `doc.selection.annot()`.

4. **`canvas::interact` opens by moving the selection off the document**, at
   `interact.rs:342`:

   ```rust
   let mut selection = std::mem::take(&mut doc.selection);
   ```

   and puts it back at `interact.rs:1493`. Every line between those two — which
   is the whole canvas frame, step 2 included — sees a
   `SelectionState::default()` on `doc`.

⇒ `doc.selection.annot()` was `None`, `is_some_and` short-circuited, and the
flag was **`false` for every document, on every frame, always**. The Delete key
raised `AnnotAction::Delete`, `EditSession::delete_annotation` refused it,
`app::actions::apply::vector_edit`'s `Err` arm wrote
`delete-annotation-refused` to the trace and — by that arm's own recorded
decision — said nothing to the operator, and `actions::annots::delete` cleared
the selection afterwards regardless, because it clears after the funnel rather
than on success.

### ★★★ Why nothing in the crate could have caught it

Every unit test of the ladder sets `annot_delete_refused` **by hand** — that is
the design, and a good one: `canvas_keys` takes no `&OpenDoc` so that its tests
can exercise the whole Delete/Escape rung order without opening a file. So no
test in `canvas::keys` is downstream of the call that was wrong.

And the panel's own test asserted
`refuses_selected(&doc) == doc.selection.annot().is_some()`, which on a
freshly-opened fixture is `false == false` — true of the fixed build and true
of the broken one.

The **only** instrument that could see it was a real keystroke into a real
window with a real selection, which is `ui-verify`'s `annot_delete_gate` phase
D, driven for the first time on 2026-08-29. R1, exactly as written: *a
capability is not verified until the running binary has been driven through
it.*

### The fix, and why it is structural rather than a comment

`annotdelete::refuses(doc, selection)` takes the selection **by argument**, so a
caller holding a detached one cannot silently ask about the wrong one.
`refuses_selected(doc)` survives as its one-line wrapper for
`app::conditions`, which runs in the panel pass where the document's selection
is intact. `canvas::interact`'s line becomes:

```rust
annot_delete_refused: crate::panels::properties::annotdelete::refuses(doc, &selection),
```

99 characters, so `interact.rs` stays on R2's 1,500-line ceiling exactly where
it was.

The regression test is
`annotdelete::fixtures::the_gate_reads_the_selection_it_is_given`: it puts the
square in a **detached** `SelectionState`, leaves `doc.selection` empty, and
asserts `refuses` still says yes — which is precisely the state
`canvas::interact` asks from, and which the broken build answers `false` to.
Both directions are asserted, so a gate that refused unconditionally fails it
too.

### ★★ The generalisation, which is not about annotations

> **A convenience overload that reaches for state through a long path is a trap
> when any caller holds that state detached.** The path `doc.selection` reads
> like a fact about the document; inside a canvas frame it is a fact about a
> temporary.

`std::mem::take` on a field for the duration of a function is a common and
sound Rust idiom, and it silently changes what every helper called from inside
that window can see. The remedy is to make the borrow explicit in the
signature, not to remember.

### The verification

The first run reported the failure as *"the keystroke did not reach
`canvas::keys` at all — check that the canvas had focus"* while the trace
carried `delete-annotation-refused` four rows above the region the same phase
went on to read. The check was right that the gate was broken and wrong about
every word of why, because it read only the line it hoped for.

Phase D now names all three lines the key can produce, and presses **until the
trace shows the key was heard** (`driving::press_until_traced`) so that a key
that never arrived is a SKIP rather than an accusation.

---

## D20 — Every real redaction was refused, because an embedded font's own `name` table counted as a leak — **FIXED 2026-09-04, NOT DRIVEN**

**Severity:** critical · **Fix:** one classification · **Reported:** 2026-09-04,
by the operator · **Files:** `crates/pdfcer-gui/src/redact/proof.rs`,
`crates/pdfcer-gui/src/redact/mod.rs`, `crates/pdfcer-gui/src/dialogs/redact.rs`

### What the operator said

> *"I really hate how when I search for text to redact, or select a text object
> on the screen to redact, pick the text to redact, then click apply redaction
> it refuses to redact anything because it always finds text that wasn't
> redacted, and it always finds all of the text is found that I selected. …
> What is the purpose of a redaction tool that refuses every time to do any
> work?"*

Every clause of that was a measurement and every clause was correct.

### The reproduction, on a fixture in this repository

`fixtures/a1-titleblock.pdf`, marking the word `construction`:

```text
REFUSED: VerificationFailed { survivors: [" construction"] }
```

Nothing was written. The removal had **succeeded** — 13 characters deleted from
1 content stream, 1 mark applied, 0 retained. What refused it was
`redact::proof`'s own absence check, and what it found was object 9: a stream
with `/Length1 19092` and no `/Type`, i.e. an **embedded TrueType font
program**. JetBrains Mono's `name` table describes its own stylistic sets as
*"Classic construction"* and *"Closed construction"*.

A font's description of its own letterforms had vetoed the operator's
redaction, and would have vetoed any redaction of any ordinary English word on
any document with an embedded font — which is every document anybody opens.

### The cause: an inverted classification, not a broken measurement

`proof::prove` ran two halves over the finished bytes:

| where the removed string still occurred | verdict |
|---|---|
| in **any decoded stream** | **REFUSE**, write nothing |
| in the **raw bytes only** | disclose as a residual, acknowledgement-gated |

The prose defending the first row said *"a decoded stream is content a renderer
or a text extractor will read back"*. That is true of a content stream and
false of most streams in a real file — font programs, image samples, ICC
profiles, object-stream containers, attachments.

★ **The two halves applied opposite rules to the same evidence.** The raw-byte
half already knew that a byte run in a place nothing draws is a coincidence
pdfcer cannot rule out — `MIN_VERIFIABLE_LEN` exists entirely because of it.
The decoded half took the identical coincidence and, merely because it happened
to sit inside a `/FlateDecode` stream rather than beside one, gave it the
harshest verdict in the module instead of the mildest.

### The fix

Every stream is still decoded and still searched — narrowing the sweep would
hide evidence. What changed is the **verdict** a blob can produce. `role_of`
classifies each decoded stream, and only a **content-bearing** one can refuse:
a page content stream, a form XObject (which is what an annotation appearance
stream is), a tiling pattern, a Type 3 glyph procedure. Everything else is
promoted into the disclosure list **with the site named**, so nothing that used
to refuse now passes silently.

The operator now gets what he asked for: the redaction is applied, the leftover
is named in the same sentence, and the acknowledgement gate — not a refusal —
stands between him and the write.

### ★★ Why the test suite did not catch it, which is the transferable part

Every unit test in `redact/` ran on `assemble`d fixtures: a handful of objects,
uncompressed streams, `/Helvetica`, no embedded font. `tools/ui-verify`'s
`checks::redaction` — the one check that drives the real binary — generates its
own fixture and its header says what it is: *"Two pages, uncompressed"*, two
ASCII strings, a Base-14 font.

Those fixtures are right for what they assert (*"every byte in this file is one
the suite put there"*) and they share one property that turned out to decide
everything: **there is nothing in them for a coincidence to hide in.** So the
feature had a unit suite and a driven check, and neither had ever seen a
document a person would open.

⇒ **The fixture that exercises the feature and the fixture that resembles the
operator's work are not the same fixture, and a suite needs both.**
`redact::tests::a_real_drawing_sheet_with_an_embedded_font_is_applied_rather_than_refused`
is the second, and it runs the whole pipeline — mark, apply, write, re-extract
— on `fixtures/a1-titleblock.pdf`, with `FOUNDATION` as a negative control so a
build that blanked the page fails it.

### The falsification

Five plants, each restored:

| plant | what it broke | which test went red |
|---|---|---|
| 1 | `/Length1` counts as drawn content again (the defect) | the font test, the site table, **and the real-sheet test**, the last with `VerificationFailed { survivors: [" construction"] }` |
| 2 | opaque hits dropped on the floor (the *dangerous* fix — stop refusing, and stop telling) | the font test and the real-sheet test. ★ On the synthetic fixtures the residual merely degraded to `RawBytes`; **only the real-document test saw `[]`**, because a real font program is compressed and the plaintext is not in the raw bytes. The dangerous fix is invisible to a suite of uncompressed fixtures. |
| 3 | `/Subtype /Form` classified opaque | `the_sweep_reaches_a_stream_that_is_not_page_content` |
| 4 | the content-stream guard removed from the disclosure half | `a_survivor_in_drawn_content_is_not_also_listed_as_a_residual` |
| 5 | the Type 3 `/CharProcs` walk removed | `a_tiling_pattern_and_a_type3_glyph_procedure_are_drawn_content` |

### Not driven

The GUI was **not launched and `ui-verify` was not run** — the operator was at
his keyboard and a watchdog kills GUI processes on sight. `checks::redaction`
gained a phase E2 that asserts the new destination control is drawn; that phase
has never executed. Everything above was proven headlessly.

---

## D21 — Reflow answered every press, in the slot that reads as a footnote about an earlier edit — **FIXED 2026-09-04, NOT DRIVEN**

`OPERATOR_REQUESTS.md` **O127**, defect 3. The operator:

> *"I also haven't seen the reflow option actually work with anything when I
> press it."*

### The finding, and it is not the one anybody expected

**`edit.reflow_block` was not silent. It answered him every single time.**

All four of its shell-side refusals — no caret, caret on bare page, run not in a
recognised block, session already edited — called
`crate::app::actions::record_note`. That is the **disclosure** channel, which the
status bar draws as:

> `⚑ About your last edit: <sentence>`

…truncated to 45 % of the remaining bar width (`NOTES_WIDTH_FRACTION`), with the
full text on hover only. For a press where **nothing had happened**, in the past
tense, labelled as a note about a *previous* edit.

`app::status::decline`'s own header had already ruled on this exact swap, for two
other sentences, in these words:

> *"an operator who reads 'About your last edit' after a gesture that did nothing
> has been told a small lie confidently."*

⇒ ★★★ **A sentence in the wrong slot is indistinguishable, from the operator's
chair, from no sentence at all.** That is the second time this project has proved
it and it is the transferable part. The engine's own refusal was worse again: it
collapsed into `Declined::EditRefused`'s nine cause-free words — *"That change
was refused, and the document is unchanged"* — for four causes with four
different remedies.

### The fix

`crate::text::textedit::ReflowRefusal`, eight variants with eight sentences, all
routed through `decline::record_reflow` so they wear `⊗` and mean *nothing
happened*. The engine's `ReflowApplyError` is mapped into the same enum through
`Result::inspect_err` **inside** the funnel's closure — which works because
`vector_edit` takes the decline floor *before* running it, and
`BeforeTheVerb::refused` fills the slot only `if slot.is_none()`. Reflow is the
first verb to use that mechanism for its own wording. The tooltip now leads with
the two preconditions, before the press, per R9.

### ⚠ The gate that was deliberately NOT removed, and why

`app::actions::textstyle::reflow` refuses whenever `doc.edit_epoch != 0` — any
edit at all, anywhere. That is far broader than the engine's own condition
(`state.contains_key(&page.contents[0])`), and removing it was drafted and
**rejected**:

`EditSession::add_text` appends a **new** content stream and never touches
`contents[0]`, so it does not trip the engine's guard. A reflow then plans from
the base document and writes the result into `contents[0]` through
`text_edit_command`, whose first-edit branch **empties every other `/Contents`
entry** — the one holding the operator's added text. It returns `Ok`.

⇒ Lifting the shell's forecast would trade a control that refuses too often for a
control that silently deletes work. It is filed as
`request_added_content_is_duplicated_by_the_next_content_edit.md` §6 and stays
until the engine can be asked.

### Not driven

The GUI was **not launched and `ui-verify` was not run** — another session held
the desktop. Proven headlessly: five new unit tests in `text::textedit`, each
falsified.

---

## D22 — Enter answered "can this make a new line?" by finishing the edit — **FIXED 2026-09-04, NOT DRIVEN**

`OPERATOR_REQUESTS.md` **O127**, defect 2:

> *"also can the enter key create new lines when we are editing or creating
> text?"*

Enter inserted a line break in a **dragged box** and **committed** in the other
two drafts. So the answer to his question, pressed with his fingers, was an edit
finishing under him — which looks like success and answers a different question.

### ★★ The half that was missing even where Enter worked

A box draft could hold two lines and the caret could not reach the second one.
`blocks::step` returns `false` for anything but `Anchor::Run`, and the arrow arm
had **no fallback** — so Up and Down did **nothing at all** in a multi-line box,
and Home and End jumped to the ends of the *whole draft* rather than of the line.
Shipping the line break without those four would have been a multi-line editor
you cannot move around in.

### What it is now

Enter means **one thing everywhere**: a new line. `Ctrl+Enter` commits every
draft, so commit is never mouse-only. Escape still abandons and clicking away
still commits, both unchanged. Where a line break cannot go — a caret in an
existing show operator — Enter **declines in words** and leaves the draft alive,
because that is the FILE's rule and not a shortcoming to hide: `edit_text`
re-encodes into the run's own font, a line feed has no code in any standard
encoding, and the engine refuses it by name.

The decision is `canvas::textedit::keys::enter_means`, a pure function, so *"the
whole interaction, not half of it"* is a claim four unit tests check rather than a
sentence in a comment.

### ★ Multi-line reaching the engine — read, not assumed

| path | a `\n` |
|---|---|
| boxed `add_text` | a **hard paragraph break**, each paragraph wrapped independently. Intact. |
| point `add_text` | a **named refusal** — no code in any standard encoding |
| `edit_text` | a **named refusal**, same reason |

So a clicked draft that gains a line break is promoted to a **boxed** request at
the commit — `app::actions::addtext` — with the box taken from the page's own
crop box: the click across to the right edge, and down to the bottom. Nothing is
invented (`canvas::textedit::place`'s *"a click would have to invent a width"*
still holds), and the promotion is disclosed under rule 4. A **one-line** click
still takes the point path byte for byte.

### Not driven

`tools/ui-verify/src/checks/enter_newline.rs` was written and **not executed** —
another session held the desktop. It is registered anyway, on the precedent
`left_rail`, `properties_tool` and `protect` set: a check that is not in the list
is a check nobody will ever run.
