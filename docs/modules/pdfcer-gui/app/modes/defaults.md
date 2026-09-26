# `app::modes::defaults` — what a mode's arrangement *is*

One subject: the **default arrangement per mode**. Which panels Read,
Review and Edit mount, on which side, stacked with which others, and how
wide those docks start. It is a pure function from a mode id to a
[`DockLayout`], and nothing in it can reach a document, a dock, a layout
file or a [`super::Modes`].

[`super`] owns the other half of the same feature — how an arrangement is
**remembered** once the operator has rearranged it.

## Why this is its own file

Split from `app/modes.rs` when that file reached 1,512 lines against the
1,500-line gate (R2). `app/mod.rs` has been split twice under the same
rule, into `crate::app::dispatch` and `crate::app::conditions`, and the
standing instruction for this gate is the one written into the gate
itself: *"the right response to this gate firing is to SPLIT THE MODULE,
not to shrink the prose."* This project's documentation is the logic, so
trimming it to fit is the one response that would make the file smaller
and the program less well specified.

**The seam is a real one rather than arithmetic**, and the test for that
is whether the two halves change for different reasons. They demonstrably
do:

* This file changes when the **information architecture** changes — when
  `MODES_AND_PANELS.md`'s table is amended, when a panel is invented, when
  the operator answers a taxonomy question. Read carrying Forms on its
  right — the operator's own answer to a question this module had been
  holding open — is exactly such a change.
* `super` changes when **persistence** changes — a new workspace naming
  rule, a new upgrade-reconciliation case, a different start-up order. The
  `Unseen` stamp, so a panel added in a new release is not born invisible,
  is exactly such a change.

The two are also readable at different times. Someone asking *"why does
Read have no Objects panel"* never needs to know how a workspace is
named; someone asking *"why did my arrangement come back wrong after an
upgrade"* never needs the taxonomy. And the dependency runs one way only:
`super` calls [`layout_for_build`], while this file calls nothing in
`super` at all.

## What `egui-shell` cannot supply

`SHELL_FRAMEWORK.md` §4 and `egui-shell`'s workspace store between them
make Read/Review/Edit a *configuration* rather than a built-in — see
[`super`]'s header for that rule and for how [`super::Modes`] honours it.

What *is* pdfcer's business, and therefore is here, is the **default
arrangement per mode** — see [`layout_for`]. `egui-shell` cannot supply
that: it does not know what a panel is for, so a default arrangement
invented there would be the framework inventing an application's
information architecture.

## The three defaults, and where they come from

`MODES_AND_PANELS.md` Part 1's table, reduced to what the *dock* does:

| Mode | Left | Right |
|---|---|---|
| **Read** | Pages, Bookmarks | Comments, Forms, **Document properties** |
| **Review** | Pages, Bookmarks | Comments, Properties, Forms, Dimension groups, **Document properties** |
| **Edit** | Pages, Bookmarks, Layers, Signatures, Fonts | Objects / Properties, Comments, Forms, Redact, Dimension groups, Attachments, **Document properties** |

**Document properties is in all three, and it is the only panel besides
Pages that is** — the operator: *"the document properties are
still always visible in the properties tab. it needs to get out of there and
be in its own document properties tab."* Reading a document's title is
reading, so no mode is withheld from it; its command sits on File ▸ Document
and every mode is shown the `file` tab, so no mode can mount it without
being able to reopen it. The arms below carry the placement within each
stack.

Read is the point of the whole feature — *"a PDF viewer, with pdfcer's
inspection panels available but nothing that authors anything"* — so its
default mounts the two surfaces that answer *where am I* and nothing that
merely describes an object you are not allowed to edit. **Forms on its
right is the one amendment to that sentence**, on the operator's own
answer to the open question; the arms of `spec` carry the full reasoning
and it is not repeated here. Review adds the two surfaces
markup work needs. Edit is everything, with **Objects on the right**,
opposite the navigators, because an inspector and a navigator are
consulted in different directions.

A mode this module has never heard of gets the **full** arrangement: a
mode with no opinion recorded about it should not have panels taken
away, because removing is the opinionated act.

## Panels this build does not have

**None — [`ABSENT_PANELS`] is empty.** The list is kept anyway, because
there are two opposite ways to get it wrong and the same mechanism catches
both.

**A blocker recorded from the wrong end** holds back a surface nothing is
blocking. *"Annotation authoring does not exist yet, so neither does the
panel that lists comments"* is false on its merits: listing what a document
already carries needs no authoring, and the Comments panel works against
`pdfcer_core::annot` while this shell still cannot place a single markup.
An **id** recorded the same way is the same failure — `view.panel_comments`
is a guess, while `RIBBON_IA.md` §7's migration map puts the control on
Markup ▸ Comments, and an id no code has ever resolved is a guess by
definition.

**A panel whose body exists and whose COMMAND does not** goes the other
way: it is correctly not an absent panel and is still not reachable.

Both are the `SHELL_FRAMEWORK.md` §5b mechanism rather than an oversight:
[`layout_for_build`] filters every default through the live
[`PanelCatalog`], so an id nothing registers is simply not mounted —
whether what is missing is the body or the command.

Writing the *intended* arrangement and filtering it is strictly better
than writing only what exists today, because the alternative is that the
intent lives in a document nobody re-reads when the panel lands. The
Pages panel is the worked proof of that: it was built long after these
defaults named it, and the day its command is registered it appears in
all three with **no edit in this file at all**.
`every_default_panel_is_registered_or_declared_absent` is what keeps
[`ABSENT_PANELS`] honest in both directions.

## Item notes

### `type SideSpec`

A single column per side, deliberately. Multiple columns are what the
dock is *for* — a narrow navigator beside a wide inspector — but they
are an arrangement the operator reaches by widening and splitting, not
one to hand somebody on their first launch. The model expresses them;
the defaults do not use them.

**Owned rather than a `&'static` table**, for one reason worth stating
because the static form is the obvious first attempt and does not
compile: [`Panel::command_id`] is an ordinary function, so its result
cannot be promoted into a `'static` slice literal. The alternative is a
table of string literals plus a test asserting each one still matches
its panel — a second spelling of every id, kept in step by a test rather
than by construction. Two `Vec`s built on a mode change are cheaper than
that, in every sense.

### `fn comments`

**Asked of the panel, never spelled as a literal** — and both halves of
`const COMMENTS: &str = "view.panel_comments"` would be wrong, which is why
this is a function like [`pages`] rather than a corrected constant.

The *value* would be wrong: the panel's command is `markup.comments`,
because `RIBBON_IA.md` §7's migration map sends the control to
Markup ▸ Comments by name, and a ruling about one control beats §5.2's list
that merely contains its name. The *form* would be wrong for the reason
[`pages`] records:
a literal here is a second spelling of an id that
[`Panel::command_id`] already owns, kept in step by a test instead of by
construction. Asking the panel is how the two cannot drift.

### `const NAVIGATOR_WIDTH`

Wide enough for two columns of page thumbnails, which is the measurement
that decides this number: a thumbnail rail one column wide wastes the
dock, and three columns makes each too small to recognise a drawing by.

### `const INSPECTOR_WIDTH`

Wider than a navigator because its rows are `label: value` pairs whose
values are paths, font names and coordinate triples — content that wraps
badly and reads terribly when it does.

This is Read's and Review's width. Edit's is [`EDIT_INSPECTOR_WIDTH`], and
the two being different constants is the whole of what *"remembered per
mode"* needs from this file — see that constant's section.

### `const EDIT_INSPECTOR_WIDTH`

## "Remembered per mode" is already built, and this is the other half

A width is stored on [`egui_shell::dock::SideLayout::width_pts`], which is
per side, of a [`egui_shell::dock::DockLayout`], which is saved **per mode**
as a named workspace by `super::Modes::record_layout` every time the dock
reports `layout_changed`. So a splitter drag in Edit has never been able to
move Read's dock, and this change adds no mechanism — it changes what the
*unremembered* case starts from.

⇒ Which is why it is a second constant and not a runtime branch: an
operator who has dragged Edit's dock is unaffected by either number, because
their saved workspace wins. This is only the first frame of a fresh profile.

## Why 360 rather than "as wide as the widest row"

Because no width fits every row and a dock that tried would be one nobody
wants. The complaint this number answers is real: our object rows already
carry paint style, colour hex, line width, node count, text preview, font
name and size, image pixels **and a trailing diagnostic note the mockup has
no equivalent for**. They are never missing content; at 320 pt they are cut
mid-character.

## ⚠ No width stops the common row being cut, and this one does not either

Driven against `fixtures/a1-titleblock.pdf`,
`the_inspector_is_one_master_detail_column` reports **8 of 8 object rows
elided** at 360, and the panel's `objects-rows overflow=` field names the
numbers: the widest row wants **473.6 pt**, the narrowest **306.3 pt**,
against **296 pt** of text room. *Every* row of that sheet is over — so
"the common row" is the case a width lever fails at, not the case it
handles.

⚠ That run also traces `mode-changed … remembered=true`: the dock restores
a saved workspace and this constant is never consulted. **A default width
cannot reach an operator who has ever dragged the dock**, which is the
second and more permanent reason the width is the wrong lever.

⇒ The remedy lives in the row rather than in the width, and this number is
deliberately left where it is: `crate::panels::objects`' row work draws a
headline (widest on that sheet: 207.6 pt) and hovers the full description.
**Nothing here follows the content** — R128 — and a future reader tempted
to raise this constant because a row does not fit should read that module's
§1b first, and the `overflow=` field second.

⚠ **Widening this is a harness re-baseline.** The canvas rect moves when the
right dock widens, so every canvas-relative click coordinate in
`tools/ui-verify` shifts. It is a one-line constant change that is a
suite-wide event, and it is the single most under-estimated edit this file
offers.

### `fn spec`

The `match` is the one place in this crate that knows what "read" means
as an *arrangement*. Note what it is not: it is not a list of the modes
that exist. [`super::Modes`] takes that from the manifest, so a mode with
no arm here still works — it simply starts from the full arrangement.

### `fn registry`

Duplicated in `super`'s own test module rather than shared, because
a `#[cfg(test)]` helper reachable across module boundaries has to be
made visible in the non-test build too. Six lines of fixture is the
cheaper of the two costs.

### `fn the_inspector_is_wider_in_edit_than_in_the_reading_stances`

Asserted per mode rather than as one constant, because *"remembered per
mode"* is the operator's phrase and the failure it guards against is the
tempting one-line version: bumping `INSPECTOR_WIDTH` alone, which would
widen Read's and Review's docks too and take that room from the page in
the two modes whose whole subject is the page.

The left widths are asserted in the same test on purpose. Edit's left
side became ONE stack of five tabs in this change, and a five-tab bar in
a 280 pt navigator is where the dock's overflow affordance starts to
matter — so a future widening of the navigator is a decision somebody
should have to change a test to make.

### `fn edits_navigators_share_one_stack`

> *"Layers, Signatures and Fonts join Pages and Bookmarks as tabs in one
> dock instead of a second dock with a fixed split."*

The count is asserted as well as the membership, and the count is the
half that matters: a build that put all five panels back into two stacks
would satisfy a membership assertion exactly, and would be the fixed
split he asked to be rid of.

### `fn edits_right_side_is_objects_over_properties`

> *"Objects and Properties become master–detail in one panel with a
> draggable split … I'd also like those one to appear in the space where
> the tool dock currently shown."*

Two adjacent stacks in one column is the master–detail shape, and the
split between them is the dock's own draggable stack splitter. What this
test pins is the part a refactor could undo without anybody noticing:
that **Objects is the first stack**, which is only true because the Tool
panel's stack was removed rather than merely emptied.

### `fn the_three_defaults_are_the_specified_arrangements`

Asserted on the *unfiltered* defaults, because that is where the
intent lives: filtering is what this build's panel set does to it,
and asserting the filtered form would make the test say less every
time a panel is missing.

### `fn read_mode_can_reach_the_comment_list_by_both_routes`

# The report this exists for

Ken: *"I could add a yellow sticky note but even in read mode I don't
think I could figure out how to read it."*

He is right, and it is an **absence**, not a discoverability problem.
Two independent barriers stand between him and a comment he has just
written, and each one alone is sufficient:

1. Read's default dock held Pages, Bookmarks and Forms. **No comment
   list was mounted at all.**
2. The panel's only command, `markup.comments`, sits on the **Markup**
   tab, and the mode table shows Read `["file", "view"]`. **So the
   toggle could not be reached to fix (1) by hand.**

⇒ **This test asserts BOTH**, deliberately in one place, because
that is the property that was violated. Two separate tests, each
passing, would each have been green on a build where he still could
not read his note — a barrier removed while another remains is
indistinguishable, from his chair, from nothing having been done. This
project has a standing lesson for that shape: *an absence claim is a
claim about EVERY route.*

# What it is NOT

It is not a claim that the popup on the canvas works, or that the
panel renders the words. It says the surface is **mounted and
reachable in Read**, which is the barrier this pair of lines removed.
A rendered screenshot is still the only oracle for the rest.

### `fn a_default_drops_panels_this_build_does_not_register`

`SHELL_FRAMEWORK.md` §5b applied to the *defaults* rather than to a
saved file: the intended arrangement names Pages and Comments, this
build registers neither, and what the operator gets is the rest of
the arrangement — never a tab whose body cannot be drawn, and never
an empty compartment where a panel would have gone.

### `fn every_default_panel_is_registered_or_declared_absent`

[`ABSENT_PANELS`] is the `PLANNED` discipline applied to panels, and
this is what keeps it honest in both directions: a default may not
name an id that is neither implemented nor declared absent, and an
id declared absent may not already exist. The second half is the one
that matters over time — it makes the day a Pages panel lands a
failing test rather than a stale comment.

### `const ABSENT_PANELS`

`(id, reason)`, in the shape and for the reasons
`crate::shell::manifest::PLANNED` uses for absent *commands*: an
omission that is data can be tested, enumerated and grepped, whereas an
omission that is a comment becomes stale the day it stops being true.

Tested in both directions by
`every_default_panel_is_registered_or_declared_absent`: nothing in a
default layout may be missing from both `Panel::ALL` and this list, and
nothing in this list may already exist as a panel. So the day either
panel lands, the suite fails until this entry is removed — which is the
same commit in which the default starts mounting it.

### `fn pages`

A function rather than a `const`, because the id must come from
[`Panel::command_id`] like every other one — a second spelling of the same
string is a second thing to keep in step, and [`SideSpec`]'s own doc
comment explains why that matters here.

It is a function and not an inline call only so the three arms below read
alike, and so this doc comment has somewhere to live.

`pub(super)` rather than private: it belongs with the arrangements, and
`super`'s upgrade-reconciliation tests name the Pages panel the same way
its own arms do. Deliberately not `pub` —
outside this module the id comes from [`Panel::command_id`] directly.

### `fn layout_for`

The intended arrangement, naming every panel the mode is specified to
offer whether or not this build has it. Almost every caller wants
[`layout_for_build`] instead; this exists so the intent is expressible,
testable and readable on its own.

An unrecognised `mode_id` gets the full arrangement — see the module
header on why removing is the opinionated act.

### `fn layout_for_build`

This is the one an application calls. `SHELL_FRAMEWORK.md` §5b: a
capability's presence is expressed by registering it and by nothing
else, so a default that mounts a panel nothing registers must mount
nothing rather than produce a tab whose body cannot be drawn.

The filter runs over the same [`PanelCatalog`] the dock and the layout
loader use, so "what a fresh profile starts with" and "what a saved
layout is allowed to contain" cannot disagree.
