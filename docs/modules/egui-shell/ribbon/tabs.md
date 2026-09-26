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
