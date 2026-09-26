# `text::status` — every string the status bar shows

One area of the catalog described in [`crate::text`]'s header, and the
sole consumer is [`crate::app::status`]. Nothing here is read by the
ribbon: the status bar **mirrors** three View-tab commands under
amendment P1a (`RIBBON_IA.md` §2), and a mirror is a second surface for
one command, not a second command.

## The one place this file deliberately repeats the ribbon

[`fit_actual_size`] and [`fit_actual_size_tooltip`] say what
`crate::text::commands::view_zoom_actual` says, in the same words. That
is not an oversight and it is not a copy-paste that should be
de-duplicated into a shared constant:

- The two surfaces are mirrors of **one** command, so an operator who
  reads the ribbon's tooltip and then hovers the status bar's button
  must be told the same thing. Two paraphrases of one command is how a
  product acquires two different mental models of the same verb.
- They are nevertheless two *entries*, because the ribbon's catalog is
  keyed by command id and consumed by `crate::shell::commands`, while
  this one is keyed by control and consumed by a widget. Reaching across
  would make `text::status` depend on `text::commands`' `CommandText`
  type for no gain, and would put the first cross-area dependency in a
  catalog whose whole organising principle is one area per consumer.

**Both entries are now true**, and the wording being identical to the
ribbon's is what made fixing them one edit rather than two. The action
behind them raises `Action::ZoomTo(1.0)` (see the section of
[`crate::app::status`]'s module docs for that half), and the chord they
name has one owner (see [`crate::app::keyboard`]'s section for this
one). The same holds for the three mirrored fit tooltips: each is now
word-for-word its `crate::text::commands` twin, chord included.

## Why the arrows and the minus sign are in the catalog

`⏴`, `⏵`, `⏷`, `−`, `+`, `·` are *labels*: they are the entire visible
text of a control, and a control's visible text is exactly what rule R1
governs. The `check-ui-strings.sh` heuristic would never catch them —
it flags literals containing whitespace, and these contain none — so
they are here by the rule rather than by the gate, which is the
distinction that file's own header draws.

They are also the reason
[`crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`]
exists, and **that test has already paid for itself**. This file was
written with the obvious glyphs — `◀` `▶` for the page steps and `▸` `▾`
for the disclosure — and every one of the four is **missing from egui's
bundled font set** (Ubuntu-Light + NotoEmoji + emoji-icon-font). On
screen they would have rendered as tofu boxes: defect D2's shape, an
invisible label, with the operator's page position behind it. What the
font set does carry, measured rather than assumed, is `⏴ ⏵ ⏶ ⏷ ‹ › « »
○ • · – — − + % /`.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.** Every tooltip below is prose and ends in a
  full stop; every label is a name and does not.
- **Name the chord only when the chord works.** Actual size names
  `Ctrl+0` because the manifest keymap binds it there and
  `crate::app::keyboard::commands` enacts what the keymap says. Fit page
  and Fit width name **none**, because none reaches them: `Ctrl+0` is
  actual size's and `Ctrl+2` is `mode.review`'s. A string that named
  either here would be claiming half of a chord that has another owner.
  Do not invent a replacement chord to fill the gap; bind one in the
  manifest first, and it may be named the same day.
- **Never state a capability the build does not have.** The Find toggle
  `RIBBON_IA.md` §6 specifies has **no strings here**, and that is now a
  filing decision rather than an absence: the toggle exists, and its label
  and tooltip live in [`crate::text::find`] beside the rest of the Find
  surface's copy. One area per consumer is this catalog's organising
  principle, and the toggle's consumer is a control the Find module owns.

## Item notes

### `mod diagnostics`

Re-exported whole, on [`selection`]'s precedent: a catalog area is keyed
by the consumer it serves — here `app::status::notes` and nothing else —
so no call site changed and `t::diagnostics_clean` resolves where it always
did.

### `fn a_page_with_no_recognised_text_says_so_instead_of_counting_to_zero`

A build that formatted `0 blocks` would satisfy every check that only
looked for a non-empty string, and would tell an operator staring at an
un-OCR'd scan that the mode is working.

### `fn the_disclosure_reads_differently_open_and_closed`

A toggle whose two states read identically is a toggle whose state
the operator has to discover by clicking it, which is exactly the
affordance a disclosure triangle exists to remove.

### `fn the_edit_disclosure_frames_cores_sentences_without_altering_them`

Two properties, and both are load-bearing:

- **Verbatim.** The disclosure prose is written where the fact is known
  — inside the planner that decided to rewrite a rectangle as four
  lines. A shell that paraphrased it would put two descriptions of one
  surgery into the product, and the one further from the code is the
  one on screen. So this asserts *containment*, which is the mechanical
  form of "we added framing and changed nothing".
- **One line.** `crate::app::status` draws this inside a row whose
  height may not vary (R128), eliding what does not fit. A newline
  would defeat that from the string side, where no layout assertion is
  looking — the label wraps, the row grows, and the page re-fits itself
  at the moment the operator finishes a drag.

### `fn a_decline_does_not_read_like_a_disclosure`

The whole reason the worded decline got its own strings — rather than
borrowing the line the vector verbs already put in this bar — is that
*"this happened, and here is the part you cannot see"* and *"this did
not happen"* are different speech acts. One wording in one slot would
make a completed gesture and a refused one indistinguishable, in the
same place, which is worse than the trace-only state these replace.

Four properties, and each would be silently lost by an edit that looked
harmless:

1. **The lead-in diverges.** Neither decline may open with the edit
   disclosure's *"About your last edit"*.
2. **The mark diverges.** `⊗` and not `⚑` — the mark is what tells the
   two apart at a glance, before either sentence is read.
3. **The two declines read differently from each other.** "Nothing is
   selected" and "the page is still drawing" have different remedies,
   and the operator gets one line to tell them apart — the same
   property [`the_two_page_box_notes_read_differently`] defends for the
   page box's pair.
4. **One line.** `crate::app::status` draws these inside a row whose
   height may not vary (R128), eliding what does not fit. A newline
   would defeat that from the string side, where no layout assertion is
   looking.

### `fn the_decline_reports_the_state_rather_than_instructing_the_operator`

`view.zoom_selection` is greyed on `selection.bounds`, so the sentence
is reached by a chord — or, worse, in the race where the selection
evaporates between the frame that drew the enabled control and the
frame that applied it. In that second case the operator clicked a
control that was offered to them, and an imperative telling them to
select something first would be instructing them to repeat what they
just did.

Asserted as an absence of imperatives rather than against a fixed
string, so the copy can be rewritten and the property survives.

### `fn every_counted_note_is_singular_at_one`

Not pedantry: these lines are read in a small weak font at the edge
of the window, and "1 glyphs" is the kind of thing a reader notices
*instead of* the number, which is the part that matters.

The property is asserted structurally rather than against a table of
expected sentences: **the singular must not be the plural with the
digit swapped.** That catches a missing branch on every entry,
including the ones whose noun is not the first word ("text from 1
font not drawn"), and it keeps working when the copy is edited.

### `fn the_clamp_note_names_what_was_asked_and_what_was_given`

The whole value of the note is that it distinguishes "your number was
out of range" from "the box ignored you". A note that named only the
page landed on could not do that.

### `fn the_two_page_box_notes_read_differently`

"I typed a page that does not exist" and "I typed something that is
not a page number" are different mistakes with different fixes, and
the operator gets one line to tell them apart.

### `fn the_three_fit_controls_are_distinguishable`

Three controls in a row that read alike is the failure the ribbon's
own salvage notes record (two adjacent Content buttons both reading
`Aa`, distinguished only by their tooltips). Here both halves are
asserted.

### `fn each_fit_tooltip_names_exactly_the_chord_that_reaches_it`

The property defended here is not that any particular one of the three
is silent. It is **"a status-bar tooltip names a chord if and only if
that chord reaches the control"** — which, with one owner per chord
(`crate::app::keyboard`), means Actual size names `Ctrl+0` and the
other two name nothing.

The three assertions below are the direct expression of that, and the
last two are the ones that matter most — a chord *removed* from the
keyboard leaves no compile error behind, so the only thing standing
between the operator and a tooltip that lies is an assertion that the
string stayed silent.

### `fn the_fit_mirrors_repeat_the_ribbon_word_for_word`

The header's claim, asserted rather than trusted. Three status-bar
controls mirror three View ▸ Zoom commands under amendment P1a, and a
mirror that paraphrases is how a product acquires two mental models of
one verb. It is also what let the chord defect live on one surface and
not the other for as long as it did.

### `fn the_page_box_tooltip_states_the_commit_rule`

It is the only place the operator can learn that typing does not
navigate, and that property is the reason the control is usable at
all — see `crate::app::status`.

### `mod waiting`

A catalog area is keyed by the **consumer** it serves, not by the file it
happens to live in, and `crate::app::status::selected` is the one consumer
of every one of these. `t::selection_one` therefore resolves from this
module whatever file the sentence lives in: no call site in this crate
should have to move when a catalog area is reorganised internally.
**Sentences about the program being busy** — a different species from
everything else in this catalog: a STATE rather than an event, with no
retirement rule. Its header carries the distinction.

### `fn edit_disclosure_line`

`notes` are `pdfcer-core`'s own sentences, in the order the planner pushed
them, unmodified. This function contributes three things and nothing else:

1. **A mark**, `⚑` (U+2691). The status bar's other left-hand line is
   *narration* — a census of what a raster contained — and this one is a
   fact about the operator's own document that they cannot see by looking
   at it. A mark is what tells them apart at a glance.

   **It is deliberately NOT `⚠`, and that is a measurement rather than
   a preference.** `⚠` (U+26A0) is what
   [`crate::text::forms::forms_fill_autosize_note`] and twelve other
   forms sentences carry, and **egui's bundled font set cannot draw it**
   — Ubuntu-Light + NotoEmoji + emoji-icon-font, which is the whole set,
   because nothing in this workspace installs a font of its own. Every
   one of those sentences renders `□` today, on the panel and in this
   bar. That is defect D2's shape and it is
   [`crate::app::status::tests::every_glyph_the_status_bar_draws_has_a_glyph`]
   that caught it, on its third sighting of the same hazard.

   The forms catalog is not corrected here because the convention is
   thirteen sentences wide and one of its assertions lives in
   `crate::panels::forms::tab_order`, outside this change's territory —
   it is **reported**, which is what a boundary finding gets. What is in
   this project's gift is not to add a fourteenth undrawable mark, and
   `⚑` is the closest drawable neighbour: measured present in the same
   bundled set, alongside `✱ ☆ ! ○ ■ • · † ‡ ⊗ ◊ №`. It is also the
   mark this file would recommend for the other thirteen, so a future
   correction converges rather than adding a third spelling.
2. **A lead-in naming the gesture**, because the sentence outlives the
   gesture: it stands until the next edit or an undo retires it (see
   [`crate::app::actions::last_edit_disclosure`]), and core's sentences
   open with *"This shape…"* / *"This point…"* — deictic words that are
   unambiguous at the moment of the drag and unanchored a minute later.
3. **A single space between sentences**, matching the way
   [`crate::app::status`] joins the two form-fill notes into its one row.
   Not [`diagnostics_join`]'s `·`: that separator exists because the
   render notes are independent *fragments*, and these are whole
   sentences with their own full stops.

# Why the lead-in does not say what changed

The obvious wording — *"that edit changed how the page is written"* —
would be a claim, and it is **false of at least one note core can
return**: the clipping-region disclosure fires on a move that rewrote
operands in place and rewrote nothing's form, and says instead that the
shape controls what other content is visible elsewhere. A lead-in that
asserted a rewrite would be contradicted by the sentence immediately
after it. "About your last edit" is true of every note in the list,
which is the property a frame has to have when it does not know which
note it is framing.

### `fn zoom_declined_no_selection`

# The three causes, and why they get one sentence

`crate::canvas::zoom::zoom_to_selection` raises
`ZoomOutcome::NoBounds` in three situations — nothing is selected, the
selection is on another page, or it does not resolve against the current
decomposition after an edit. That function's own docs rule that from the
operator's side those are **one** situation: *"there is nothing on screen
for this command to act on."* Three sentences would ask the operator to
care about a distinction that has one remedy.

# Why it describes the state and does not instruct

`view.zoom_selection` is greyed on `selection.bounds`, so this is *mostly*
unreachable from the ribbon. The two ways it is reached are both cases in
which blaming the operator would be wrong:

1. **By chord.** A keymap reaches any command from any state, and the
   manifest binds this one; nobody who presses a chord has clicked a
   control that promised anything.
2. **In the race.** The condition is evaluated on the frame that *draws*
   the control and the verb runs on the frame that *applies* it, so a
   selection that evaporates in between — a mode change that clears it, an
   edit that dissolves what it named — leaves the operator having clicked
   an enabled control and been declined. That is the case an operator finds
   most confusing, and a sentence reading *"select something first"* would
   tell them to do the thing they just did.

So it reports the state, at the moment the command ran: *nothing on this
page is selected right now*. "Right now" is doing work — it dates the
claim to the gesture rather than asserting a standing fact about an
operator who may already have fixed it.

### `fn zoom_declined_not_drawn`

`ZoomOutcome::NoCanvas`: there is no viewport, no page rect and no scroll
offset yet, so there is nothing to frame *into*. Kept separate from
[`zoom_declined_no_selection`] because the remedy is different and the
operator has to do nothing at all to reach it — it resolves itself on the
next raster, which is what "yet" and "has not finished" promise.

Reachable in practice only by a chord fired at a document that has just
opened, or on a very slow first raster of a dense CAD sheet, where ~99 % of
the render cost is resolution-independent and the first frame can take
most of a second.

### `fn save_copy_failed`

# Why a decline needs wording here more than anywhere else on this bar

The other two sentences beside it describe a command that was refused
*before* the operator invested anything. This one arrives after they opened
a dialog, chose a folder and typed a name — and the only other evidence they
would get is a file that is not there. Silence would make a save that failed
indistinguishable from a save that never ran, which is precisely the "the
button does nothing" state this project exists to remove.

# Why it does not carry the engine's reason

`crate::app::save::SaveError`'s `Display` output goes to the trace, and
`check-ui-strings.sh`'s exclusion 3 states in as many words that a `Display`
impl "is not permission to route UI text through an error type". A
cross-reference form that could not express an entry is a true sentence and
not one an operator can act on.

# Why it names the two things they CAN act on

The folder and the permission, because between them they are almost every
real instance: a path typed into the dialog whose parent does not exist, a
network share that went away, a read-only volume, a file open in another
program. It reports the check to make rather than blaming the operator —
[`zoom_declined_no_selection`]'s rule, which
[`tests::the_decline_reports_the_state_rather_than_instructing_the_operator`]
enforces for that sentence — because the commonest cause is not something
they did.

### `fn settings_not_saved`

# Why this is not [`save_copy_failed`]'s sentence, though both are writes

Because the two have to say **opposite things about the operator's work**,
and getting that backwards costs them either their trust or their time.

A failed save-a-copy produced no file: nothing happened, and the operator
should try again. A failed settings save is the reverse — pdfcer **adopted
the configuration anyway**, deliberately, because a disk that refuses should
not cost somebody a choice they deliberately made. So what is true is *"this
is in force now, and it will be gone when you restart"*, and the sentence
has to carry both halves or it is misleading in one direction or the other:

- Say only "settings were not saved" and the operator makes the choice
  again, or concludes the setting does not work.
- Say only "settings applied" and they restart and lose it silently, which
  is the failure the whole store exists to prevent.

# Why the reason is not in the sentence

The store's `SaveError` has a `Display` — *"no writable location"*, *"could
not write settings to {path}: {reason}"* — and it is a developer's sentence,
not an operator's. It goes to the trace beside the store kind, from
`crate::app::settings_window`. The operator's actionable half is *the
folder*, and the settings window itself states which folder that is, on a
line it draws every time it opens.

### `fn undo_declined_empty`

# Why this sentence exists at all, when the control is greyed

Because the route that reaches it is **the keyboard**, and the keyboard is
the one route on which the greyed control explains nothing. `edit.undo` is
gated on `undo.available`, so its quick-access button is un-pressable with an
empty log — but it is also bound to `Ctrl+Z`, and
`app::modes::capability::offers_command` lets it through in every mode
because it sits on no tab. An operator who presses `Ctrl+Z` is looking at the
page, not at an 18 pt icon in the title bar, and silence there is
indistinguishable from a chord that never arrived. **A gesture that is
refused must say so on the route it was made on.**

It is the same argument [`save_copy_failed`] makes about *its* route, one
step earlier: that one arrives after the operator invested a dialog, this one
after they invested the single most reflexive keystroke in any editor.

# Why it says *this document* rather than *nothing*

The log is per-[`EditSession`](pdfcer_core::edit::EditSession), which is
per-document: closing a document and opening another empties it. "Nothing to
undo" alone would read as a claim about the application, and an operator who
had just undone six things in the file they closed would read it as a defect.

# Why it does not name the remedy

There is none — nothing has been changed, so there is nothing to take back,
and the sentence is a complete report of the state. It reports rather than
instructs, which is [`zoom_declined_no_selection`]'s rule and the one
[`tests::the_decline_reports_the_state_rather_than_instructing_the_operator`]
pins for that sentence.

### `fn redo_declined_empty`

Kept separate from [`undo_declined_empty`] because the two states are
reached differently and a reader has one line to tell them apart. An empty
**undo** log means nothing has been changed; an empty **redo** stack is the
ordinary state of a document that has been edited and never undone — and it
is also what a *new edit after an undo* produces, because the engine clears
the redo stack when a fresh command is recorded. One sentence for both would
tell an operator who just pressed `Ctrl+Y` after ten edits that their
document has no changes, which is false.

### `fn zoom_percent`

A **readout**, not a control: this build has no way to set an arbitrary
zoom by typing, so an editable box here would be an affordance for
something that cannot happen. The page number beside it *is* editable
because `Action::GoToPage` exists; there is no `Action` that sets a zoom
to a named value, and inventing a text box in front of one would be the
placeholder the project's invariants forbid.

### `fn zoom_percent_tooltip`

Explains the ladder, because "why did 137% become 150%?" is the question
the readout provokes and the answer is a deliberate design choice
(`crate::viewer`'s module docs: a fixed ladder makes zoom-in-then-out
exactly reversible).

### `fn fit_actual_size`

**Identical to `crate::text::commands::view_zoom_actual`'s label**, on
purpose — see this module's header for why a mirror repeats rather than
paraphrases, and [`crate::app::status`]'s section for why the claim it
makes is not yet true.

### `fn fit_actual_size_tooltip`

**It names `Ctrl+0` again, and that sentence is now true.** The chord
had two owners — the manifest keymap bound it to `view.zoom_actual` while
`crate::app::keyboard` bound it to Fit page and reached it first — so this
tooltip had to advertise no chord at all, with a test pinning the
omission. `crate::app::keyboard`'s section has the whole account; the
outcome is that the manifest is the only place a chord is bound, and it
binds this one here.

Word for word `crate::text::commands::view_zoom_actual`'s tooltip,
including the chord — see this module's header on why a mirror repeats
rather than paraphrases.

### `fn fit_width_tooltip`

Says "and keep it fitted", because a fit is a **mode** here rather than a
one-shot: resizing the window re-fits. A viewer that stopped fitting on
the first resize would be conspicuously wrong, and the tooltip is where
the operator learns which of the two this is.

**It names no chord.** `Ctrl+2` belongs to `mode.review`
(`MODES_AND_PANELS.md` Part 1 §6, and `crate::text::commands::mode_review`
names it), and naming it here would claim half of a chord with another
owner. Fit width is reached from this button, from its View ▸ Zoom control
and from its `canvas.empty` context-menu entry; what it does not have is a
chord, and the rule in this module's header is to say so by omission
rather than to name one that does something else.

### `fn fit_page_tooltip`

**It names no chord.** See [`fit_actual_size_tooltip`]: `Ctrl+0` has one
owner, the manifest keymap, and the manifest binds it to actual size. Fit
page is reached from this button, from View ▸ Zoom and from the
`canvas.empty` context menu.

Word for word `crate::text::commands::view_zoom_fit_page`'s tooltip, which
names no chord either — two mirrors of one command saying exactly the same
thing, which is what the header requires of them.

### `fn fit_height_tooltip`

Word for word `crate::text::commands::view_zoom_fit_height`'s tooltip, as
this module's header requires of every status-bar mirror of a ribbon
command — and, like its two siblings, it names no chord, because it has
none.

It says nothing about the page overflowing sideways, deliberately. That
is what *"its full height is visible"* already means on a sheet wider than
the window, and a tooltip that warned about it would be describing the
operator's own document back at them.

### `fn wheel_flip_pages`

Two words, because it shares a 24-point bar with the page buttons, the zoom
readout and three fit controls. It names the state the control **turns on**
rather than the state it is in, which is what a pressed/unpressed toggle
already reports: *Flip pages*, lit, means the wheel flips pages.

### `fn wheel_flip_pages_tooltip`

It states **both** answers, because the label can only state one and the
operator needs to know what turning it off gives them back.

And it names the two things the setting does **not** touch. Ctrl+wheel
always zooms, and a continuous display always scrolls — an operator who
tried the toggle in a continuous mode and saw no difference would
reasonably conclude it was broken, which is why the control is not drawn
there at all and why this sentence says so.

### `fn prev_page`

**Not `◀` (U+25C0)**, which `RIBBON_IA.md` §6 spells the control with and
which egui's bundled fonts cannot draw — see [`diagnostics_toggle`] for
the measurement and the test that caught it. `⏴`/`⏵` are the same shape
at a slightly smaller optical size, and they are what this font set has.

### `fn page_number`

**1-based.** `crate::viewer::ViewState::page_index` is 0-based and the
conversion happens here, once, exactly as that module's own docs
prescribe: *"The UI displays it 1-based; the conversion happens once, in
the string catalog."*

### `fn page_of_total`

`/ 42` rather than `of 42`: `RIBBON_IA.md` §6 spells the control
`page ◀ n/N ▶`, and the slash is narrower — which matters on a control
that sits between two buttons in a fixed-height bar.

### `fn page_box_tooltip`

States the commit rule, because it is the one thing about this control
that is not visible: nothing happens per keystroke, so an operator typing
`42` must be able to trust that passing through `4` did not move them.

### `fn page_clamped_note`

**The point of this string is that the clamp is not silent.** Typing
`99` into a 42-page document and landing on 42 with no explanation is
indistinguishable from the box ignoring what was typed — and an operator
who cannot tell those apart stops trusting the control. Naming the number
that does not exist, and the page they got instead, makes the clamp a
*report* rather than a shrug.

`asked` is the 1-based number typed; `landed` and `total` are 1-based
page numbers.

### `fn page_rejected_note`

The operator's text is deliberately **left in the box** when this
appears, so the note explains something still visible rather than
describing a value that has already been thrown away.

### `fn adopt_declined_name_taken`

# Why the sentence explains the standard rather than just refusing

Because the refusal looks arbitrary otherwise. Every other program the
operator uses will happily hold two things with one name in one file, and
"that name is taken" reads as pdfcer being fussy about a namespace it made
up.

It is not pdfcer's namespace. ISO 32000-2 SS12.7.3.1 makes the fully
qualified name the field's **identity**: two top-level fields called
`Address` are one field with two boxes, and filling either fills both. So
the second half of the sentence is the part that does the work — it says
what would happen if pdfcer allowed it, which is the only thing that makes
the refusal obviously right rather than obviously annoying.

### `fn adopt_declined_no_name`

# The word this sentence must not use is "restore"

The operator's mental model at this moment is *"something was lost, and I
am putting it back"*, and for the common case that is exactly right — a
merged field-widget carries its own name, type and value, and registering
it recovers the field as it was.

This is the other case, and it is not that. The box was a **bare kid**: its
name, its field type, its radio flags and its value all lived in a field
dictionary that is not in this document. Naming it here **creates a new
field** with no type and no value. That is a legitimate thing to want, and
it is not a recovery — an operator told they had restored a radio button
would go looking for its group, and there is no group.

So the sentence offers the name box and says what naming it will produce,
and it names the only route that gets the original back.

### `fn adopted`

# Three facts, each conditional, and none of them is "done"

`AdoptOutcome` carries three things the operator cannot see and would not
guess, and each is dropped when it is not true rather than being reported as
a negative:

- **the name it went in under** — always said, because for a blank box it is
  the name the file already carried, which the operator has never seen;
- **`field_type: None`** — legal (`/FT` is inheritable) and useless, because
  a top-level field has nothing left to inherit from. No viewer knows how to
  render or fill it. This is the fuzzy-never-sneaky half that would
  otherwise be invisible: the registration **succeeded** and the box is
  still not fillable;
- **`acroform_created`** — the document had no interactive form at all and
  now has one, which changes what other software does with the file.

### `fn flatten_declined_certified`

# Why the ribbon control is live at all, when the panel's is greyed

The Forms panel asks `EditSession::flatten_refusal` every frame and greys
its own Flatten with the reason on hover, because it is already reading the
session to draw the field list. A **ribbon** `enabled_when` is a condition
name evaluated against a published set, and publishing this one would mean
a certification query per frame for a control that is almost never pressed.

So the ribbon control is `enabled_when("doc.pages")` and the arm declines in
words. That is this project's standing division and `app::dispatch::forms`'
own header states it: *greying is a hint; the worded decline is the answer.*

# What the sentence has to carry

**Which gate refused**, because flatten and fill take *different* ones and
an operator who has just successfully typed into the form will otherwise
conclude the button is broken. On the ordinary real-world shape — a
certified fillable form at `/P 2` — filling is permitted and flattening is
refused, by design and by the standard.

**What it would cost**, because "the signature would be broken" is the fact
that makes the refusal reasonable rather than arbitrary.

It does **not** offer a way round. There is one — remove the signature —
and pdfcer will not suggest defeating a certification as a workaround for a
convenience.

### `fn blend_space_status_line`

# Every word of this was chosen against a specific misreading

**"at this zoom"**, not "on this page". The operator's report was
*"different results depending on Zoom level"*, and the thing that must land
is that the page has not changed — the view has. A sentence blaming the
document would send him looking at the file.

**"zoom out"** rather than "reduce the zoom", because it is the instruction,
and it is the opposite of what somebody chasing a colour difference tries.
Measured on an A4 page the boundary is 534 %; naming a number here would be
worse than useless, because it depends on the page size and on the display
density, and would be wrong on the next document.

**"approximate"**, not "wrong". The fallback is a known, counted
approximation that pdfcer has shipped for its whole life and that most pages
never reach; calling it wrong would overstate it and invite a bug report
about a page that is fine.

It does not apologise and does not promise a fix. What it owes the
operator is the fact and the remedy, and it gives both in one line that fits
a status bar.

### `fn recovered_status_line`

One sentence, stating the fact and where to look, and stopping. It does
not warn, does not instruct, and carries no counters — the numbers live in
Properties, and a status line long enough to hold three of them would push
the zoom and page controls off a narrow window.

It says **"rebuilt"** rather than "repaired" or "fixed". Repaired implies
the file is now correct; rebuilt says what actually happened — pdfcer
reconstructed the index by scanning, which is a best reading of damaged
bytes and may or may not be the one the author intended. The operator's
trust in the page should follow the weaker word.

### `fn raster_stop_status_line`

> *"If this error is caused by some other limitation that will always happen,
> zoom should stop at the limit and not end up showing an error - the canvas
> will just stop zooming in and can still function. the error can still be
> shown on the bottom bar so the user has some idea as to why zooming stopped
> short of 1 trillion percent."*

That sentence is a complete specification and it has two halves. The first —
*stop, do not show an error* — is
[`crate::viewer::zoom_ceiling`]'s learned clause and
`crate::render::settle`'s pull-back. This is the second: without it the `+`
button and Ctrl+wheel would simply stop responding with nothing anywhere
saying why, which is the silently-inert control the project has already been
corrected about twice.

# What it does NOT say, and why each omission is deliberate

**It does not name the percentage.** The zoom readout is three controls to
the right on the same bar, showing exactly the number the zoom stopped at,
and `app::status::decline`'s header makes this same ruling for the
raster-ceiling-clamped region zoom: the framing verb carries the *clamped*
scale, so the readout states the truth on the same frame. A number repeated
in a sentence beside the control that already shows it is a number that can
disagree with it.

**It does not say "error", "failed" or "could not".** The whole of O186 is
that this state is *not* an error — the page is drawn, the canvas works, the
operator can pan and select and edit. Ken's complaint was an error sentence
painted across his drawing where a limit had been reached; wording the limit
as a failure would move the same mistake to a smaller surface.

**It does not offer a remedy.** There is nothing the operator can do: this is
the rasterizer's own wall on this page's geometry, not a setting. A line that
said "try zooming out" would be advice about the thing they were already
doing when it stopped.

# Why it says the limit was measured on *this sheet*

Because the number is genuinely per-page and the operator will otherwise read
it as a property of pdfcer. Measured: an E-size sheet gave out at raster
scale 284,964 where a business-card page reached 8,053,069 — a 28× spread in
one build. So *"other pages may go further"* is not hedging; it is the single
most surprising true thing about this limit, and the clause that stops the
operator concluding the application has a maximum zoom it does not have.

### `fn too_many_anchors`

# The state this exists for, and why silence was the wrong answer

`overlay::MAX_UNSELECTED_ANCHORS` is 400, and the cap is right: five
thousand hollow squares over a CAD path is noise rather than an answer, and
the cap's own note argues that at length.

But it means `view.show_points` **does nothing visible on exactly the
drawings this program is for**. A 5,000-node path toggled on and off looks
identical, and the operator's report would be *"Show points is broken"* —
which is what the toggle was wired to stop happening in the first place,
arriving through a different door.

Rule 4's half that survives: *an inference the operator cannot see still
owes an off-canvas report.* The canvas is not marked — nothing is drawn on
the page to indicate suppression — and the status bar carries the number.

It names **both** numbers. The count alone would not say the cap is the
reason; the cap alone would not say how far past it they are. An operator
who sees *"5,903 … 400"* knows immediately that no setting is going to help
and that the answer is to enter a part.

It also names the remedy, and the remedy is real: descending into a
subpath narrows the anchor list to that subpath, which is nearly always
under the cap. That is the route the Points tool takes and it is one click.

### `fn too_many_text_chunks`

Its sibling above carries the argument in full and it applies unchanged: the
canvas is not marked, the status bar carries the number, and it names both
numbers because the count alone would not say the cap is the reason. The
remedy differs — there is no rung below a chunk to descend into, so the way
to see fewer at once is to select less.

### `fn ocr_layer_line`

Said only while the mode is on, and it carries two facts a look at the
canvas cannot supply:

* **Where the slider is.** The blend is a continuum and the canvas shows a
  position in it, not the position — an operator who has half-faded a pale
  scan cannot tell 30 % from 45 % by looking.
* **Whether there is anything to draw.** A page with no recognised text
  draws nothing, and *nothing* is indistinguishable from a broken mode.
  That is R8b's second guard exactly: an inference the operator cannot see
  — here, *this page was never OCR'd* — owes an off-canvas report.

The count is in the operator's own word from O226, *"block of text"*,
rather than the engine's *run*.
