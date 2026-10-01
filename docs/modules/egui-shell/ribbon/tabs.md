# `egui-shell/ribbon/tabs`

## Item notes

### `fn has_something_to_show`

**A tab with no groups, or groups with no items, is `true`.** That is
deliberate and is not the case this rule is about: an empty tab is a
manifest under construction, and hiding it would make a half-written
manifest look like a working one with a missing feature. What this
suppresses is a tab whose items exist and are all **conditioned away**,
which is a statement the manifest made on purpose.

### `fn draw_tab`

`truncate()` is unconditional, and it is what makes the pinned active
tab's promise keepable: [`super::plan::plan_tab_strip`] guarantees the
active tab a slot but cannot guarantee that slot is as wide as its
label, so when the row is narrow the tab loses characters rather than
losing its place. See [`super::band::command_button`]'s `truncate`
section on why the band makes the opposite choice.

Drawn frameless with `tabshape::body` behind it on the band's `surface`
colour. In the overflow menu (`in_menu`) it is a plain selectable row instead.
The active tab's rect is stored for the pass so `strip_underline` can break
the baseline beneath it.

### `fn a_tab_with_every_item_conditioned_away_is_not_shown`

The symmetric completion of the band's *"a group with nothing left is
not drawn"*. Without it the two disagree, and the operator gets a tab
they can click with an empty band beneath it.

This is also the rule that makes a **generous tab list** safe, which
is the point of having it: a mode can name a tab it only sometimes
needs and the tab appears exactly when it has something to offer. It is
what lets a command live on the tab it belongs on rather than on the tab
a mode happened to be granted.

### `fn an_unconditioned_or_empty_tab_is_always_shown`

The guard that makes it safe to add to a shipped shell: the question
asked is *"is every item conditioned away?"*, and an unconditioned item
answers no. An **empty** tab is also still shown — that is a manifest
under construction, and hiding it would make a half-written manifest
look like a working one with a feature missing.

### `fn a_mode_shows_only_its_own_tabs`

`MODES_AND_PANELS.md` Part 1's whole premise: Read is *File ·
View*, Edit is everything. Nothing in this crate names those
modes — they come out of the manifest — which is the "Read/Review/
Edit is configuration, not a built-in" requirement made concrete.

### `fn a_modes_order_wins_over_the_manifests`

A mode is a workspace. A workspace that silently reordered itself
to match the underlying tab list would not be one, and the
operator who put Tools first would find it back in fourth place
with no explanation.

### `fn an_absent_or_unknown_mode_shows_everything`

Both are fail-soft in the safe direction: showing everything can
never hide a capability, and `SHELL_FRAMEWORK.md`'s rule for a
stale customization is that it loses one thing rather than the
layout.

### `fn a_contextual_tab_appears_only_while_its_condition_holds`

`RIBBON_IA.md` §4: Format appears when something is selected. It
is appended after the mode's tabs rather than inserted into them,
because a tab that changed the *position* of the others as it came
and went would move every target under the operator's cursor.

### `fn a_contextual_tab_with_no_condition_never_appears`

The opposite of the empty-string case in
[`super::ctx::condition_holds`], and deliberately so: a tab placed
in `contextual_tabs` with **no** `visible_when` key at all has not
said when it appears, and a contextual tab that is always present
is an ordinary tab in the wrong list.

### `fn a_hidden_tab_is_skipped_even_when_a_mode_names_it`

The operator's own hide outranks the mode's list — hiding is a
customization, and `SHELL_FRAMEWORK.md` §5 puts "hide them" in the
allowed column.

### `fn an_active_tab_that_disappears_falls_back_to_the_first`

Two ways a tab disappears while active: the operator switches to a
mode that does not contain it, or a contextual tab's condition
stops holding. Both must recover without a blank band and without
a click. See the module header on why the alternatives are worse.

### `fn the_active_tab_is_distinguished_by_more_than_colour`

A fill-only cue is invisible to a colour-blind operator, invisible
on a projector, and invisible in a greyscale screenshot — which is
also how it becomes invisible in a bug report. Two of the four
cues here are the presence or absence of a *shape*, and either
alone is sufficient to read the strip.

Written against the *count* of non-colour cues rather than against
the specific ones, so a future redesign may swap an underline for
a top rule or a border for a notch — but may not quietly reduce
the set to the fill.

**`emphasised_text` is deliberately not counted.** See the module
header: `RichText::strong()` in `egui` 0.35 is a stronger *colour*
at the same weight, and treating it as a weight cue is exactly the
mistake this project's own preferences document made.

### `struct TabCues`

A struct rather than three `if active` expressions scattered through
the drawing code, because R84 is a property of the *set* of cues and a
property of a set cannot be asserted about three separate expressions.

### `fn visible_tabs`

# The rules, and why each is what it is

- **A mode names its tabs, and the mode's order wins.** A mode is a
  *workspace*, and a workspace that reordered itself to match the
  underlying manifest would not be one.
- **No modes, or an unknown mode id, means every ordinary tab.** The
  manifest is allowed to have no modes at all — a small application
  does not need them — and an unknown id is a stale customization,
  which per `SHELL_FRAMEWORK.md` must lose one thing and not the
  layout. Showing everything is the safe direction: it can never hide
  a capability.
- **Hidden tabs are skipped.** [`Tab::hidden`] is the operator's own
  choice and outranks the mode's list.
- **Contextual tabs come last and are never mode members.** Their
  presence is decided by application state rather than by
  configuration, which is exactly why [`Shell::contextual_tabs`] is a
  separate field.
- **A tab with nothing left to show is not shown.** The rule that
  comes with [`crate::manifest::Item`]'s `visible_when`, and the symmetric
  completion of the band's own — *a group with nothing left is not drawn at
  all*. Without it the two halves disagree: hide every item on a tab and
  the groups all vanish, leaving a tab an operator can click and an empty
  band beneath it.

  It is also what makes a **generous tab list** safe, which is the point.
  A mode can name a tab it only sometimes needs, hide the items that do not
  apply, and the tab appears exactly when it has something to offer. That
  turns `Mode::tabs` from *"which tabs exist here"* into *"which tabs may
  appear here"*, and it is the mechanism by which a command can live on the
  tab it belongs on rather than the tab a mode happened to be granted.

  A tab whose items carry **no conditions at all** is never affected:
  the question asked is *"is every item conditioned away?"*, and an
  unconditioned item answers no. So this cannot hide a tab that a manifest
  written before conditions existed would have shown.

### `fn measure_tab`

# Why a tab is measured at all

[`super::strip`] must decide which tabs fit **before** any of them is
drawn, for the reason [`super::plan`]'s header gives at length: an
affordance emitted into whatever the content left is an affordance the
content can take. So a tab is estimated the same analytic way a band
item is — the galley `egui` will lay out, plus the padding `egui` will
add — and the estimate is floored at
[`super::plan::MIN_ITEM_WIDTH`] by [`ItemWidths::total`].

A tab has no icon slot and no gap, so this is text plus padding. The
active tab is *not* measured wider even though it is drawn with a
stroke and `RichText::strong()`: an `egui` stroke is painted inside the
button's own rect, and `strong()` in `egui` 0.35 changes the colour and
not the face (see this module's header). Neither costs a point of
width, so a strip does not reflow when the operator changes tab — which
it very visibly would if the estimate said otherwise.

### `fn render_tabs`

`visible` is the subset [`super::plan::plan_tab_strip`] decided to
draw, already in display order and already including the pinned active
tab. The tabs that did not fit are drawn by
[`render_overflow_menu`] instead; both paths call [`draw_tab`], so a
tab in the menu carries the same cues, the same accessible name and the
same tooltip as one in the strip. (That is the same rule
[`super::band`] follows for overflowed groups, and for the same reason:
a second, simpler drawing path for the menu is how the salvage
source's caption-less groups happened.)

Returns the newly clicked tab's id, if any. The caller writes it into
[`super::RibbonState`]; this function mutates nothing, so a test can
call it without owning shell state.

### `fn render_overflow_menu`

The menu is a vertical run of the *same* tab buttons the strip draws —
see [`render_tabs`] on why there is no second drawing path. Clicking
one activates it, and because
[`super::plan::plan_tab_strip`] pins the active tab, the tab the
operator just picked out of the menu is guaranteed to be in the strip
on the next frame. That is the property that makes the menu a *route*
to a hidden tab rather than a place a tab can be looked at.

`active_id` is passed even though the active tab is never in `hidden`:
it costs nothing, and it means a future change that unpins the active
tab cannot accidentally draw it in the menu without its cues.
