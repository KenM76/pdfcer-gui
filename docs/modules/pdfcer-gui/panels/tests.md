# `pdfcer-gui/panels/tests`

## Item notes

### `fn every_panel_is_reachable_from_the_ribbon`

The check three panels shipped without. The old shell's
`panels_structure.rs` header records what that cost:

> All three shipped with a `PaneSubject`, a panel body, a rail entry
> and a diagnostic step — and no control an operator could click.
> Their only callers were the harness step handlers, so every
> verification passed while the panels were unreachable in a real
> build.

Two assertions per panel, and both are needed. A command **the
manifest references** is one the ribbon draws a control for; a
command **the registry holds** is one that has a label, a tooltip and
an enable predicate. Either alone is half a control: an id on a tab
with no registration renders nothing, and a registration nothing
references is an orphan (`crate::shell`'s own
`no_registered_command_is_orphaned` catches the second direction from
the other side).

This is deliberately stronger than the gate it replaces, which read
`main.rs` as a **string** and looked for a `show_pane_subject(…)`
substring outside the harness function. A substring search cannot
tell a live call from one inside a `#[cfg(test)]` block, and it
silently stops working the day the call is spelled differently. This
one asks the same data the ribbon draws itself from.

### `fn the_documents_own_properties_are_their_own_panel_in_every_mode`

The operator, 2026-09-05: *"the document properties are still always visible
in the properties tab. it needs to get out of there and be in its own
document properties tab."*

# Why this is one test and not three

Because the interesting assertion is **negative** — *the metadata is not in
the Properties panel any more* — and a negative about a panel is satisfied
by every way of having no panel at all. Three of the failing builds it must
exclude:

| build | what the naive assertion does |
|---|---|
| the metadata form deleted outright | passes |
| the new panel written but mounted by no mode | passes |
| the new panel hung off `file.properties`, so only one of the two can open | passes |

⇒ So the positive controls are in the same test, deliberately: the new panel
is **mounted by all three modes**, the old panel is **still mounted** where
it was, and the two hold **different commands that each resolve back to
their own panel**. Split into three green tests, any one of them could pass
on a build where the operator has lost the metadata entirely — and this
project has a standing lesson about exactly that shape (`read_mode_can_reach
_the_comment_list_by_both_routes`, whose header argues it at length).

# What it does NOT assert

That the Properties panel's *body* no longer draws the metadata. There is no
headless frame runner in this crate, so the only oracle for what a body
draws is `ui-verify` — `the_inspector_is_one_master_detail_column`'s
assertion 2b, which was written for this and, as its own header says, **has
not been run**. This test pins the structure; that one pins the pixels.

### `fn document_properties_is_offered_in_every_mode_and_redact_is_not`

# ⚠ TWO vacuity holes, and the first one was found by falsifying this test

**1. An unknown shell.** `app::modes::capability::offers_command` falls
through to `true` when it is handed `None` — deliberately, because the mode
selector is an interface-complexity control and not a permissions system,
and failing closed would produce a keyboard that silently does nothing. A
test passing `None` therefore asserts that every command is offered in every
mode, including commands that do not exist. So the built-in shell is passed
explicitly, and the offer assertion is **paired with a refusal**:
`edit.redact` must NOT be offered in Read.


⇒ So the tab is asserted first, out of the manifest's own
`command_references()`, and the two halves together say the thing the name
claims: *the control exists, it is on the `file` tab, and every mode is shown
that tab.* `every_panel_is_reachable_from_the_ribbon` catches the deletion
too — it was the test that failed under that plant — but it asks *"is this
referenced anywhere at all"*, which a QAT slot or a key binding satisfies,
and a QAT slot is not a tab.

### `fn a_row_wider_than_the_viewport_widens_the_container`

If this returned the viewport width, `ScrollArea` would compare
content against viewport, find them equal, draw no bar, and the row
would be cut off at the panel's edge with nothing to say so. That is
the exact defect the Objects panel had, and it is why this is a pure
function rather than three lines inside a closure.

### `fn a_non_finite_row_width_cannot_blank_the_panel`

`f32::max` propagates `NaN` in one direction and swallows it in the
other depending on argument order, and a `NaN` container width makes
egui lay nothing out at all — a blank panel, which reads as a crash.
Filtering is cheaper than reasoning about which way round it went.

### `fn clicking_the_focused_row_again_clears_the_focus`

With no selection model there is no Escape ladder and no other route
back to "nothing focused". A panel an operator cannot get out of is
worse than one they cannot get into.

### `fn the_panel_focus_has_not_quietly_become_a_selection`

[`ObjectTreeUi::focus`]'s own docs say the field is **deleted** when
the real selection model lands, not extended — because two selections
that have to be kept in step will drift, and the drift is invisible
until an edit acts on the wrong object.

The danger is not that someone renames it in one commit. It is that it
grows into one an attribute at a time — a second index here, surviving
a page change there — until deleting it is a refactor nobody wants to
start. So the four properties that make it *not* a selection are
asserted directly, and the canvas's real selection model landing in
this same stage is exactly why they are asserted now:

1. **Single-valued.** A selection is a set; this is one `Option`.
2. **Does not survive a page change.** A selection is document-scoped;
   a paint-order index is a position on one page.
3. **Does not survive an edit.** Deleting one object renumbers every
   object after it, so a retained index describes a different object.
4. **Read by one panel, and drives nothing else.** No enable
   predicate reads it. `crate::app::PdfcerApp::conditions` **does**
   set `selection.any` as of S4 — but from `OpenDoc::selection`, the
   canvas's real selection, and never from this focus. That is the
   distinction this test defends: the two look alike, and the day
   someone wires the condition to whichever one is nearest, a row
   highlighted in a panel starts arming a destructive command.

### `fn per_char`

Deliberately NOT a real font. The property under test is the *decision*
— does this row need shortening, and to what — and a real font would make
every expected value a measurement nobody could check by reading.

### `fn a_row_that_overflows_is_shortened_to_fit`

The three conditions together are the whole contract. Asserting only the
first would pass on an implementation that returned the label untouched;
asserting only the last would pass on one that returned an empty string.

### `fn it_keeps_every_character_that_fits`

A correct-but-useless implementation returns the bare ellipsis every time
and satisfies every assertion above. This one pins the search: at 20 pt with
one point per character, nineteen characters plus the ellipsis is the answer,
and twenty would not fit.

### `fn a_multi_byte_row_is_never_split_mid_character`

Object rows carry the middle dot, the em dash and the multiplication sign,
and font names carry accents. A byte-offset slice would panic here rather
than misbehave, which is the good news; this test is what stops a future
"optimisation" to `&label[..n]` from reaching the operator.

### `fn a_pane_with_no_width_leaves_the_row_alone`

One frame of a zero-width pane is not a layout decision worth taking, and
returning `Some("\u{2026}")` for it would put an ellipsis in every row of a
panel that is merely mid-animation.

### `fn opening_a_document_does_not_throw_away_the_preview_preferences`

[`PanelsState::forget_document`] is `*self = Self::default()`, which is
exactly right for everything on the struct that describes a **document**
and exactly wrong for the two fields that describe the **operator**. The
tick and the time limit are read out of `preferences.txt` once, in
`PdfcerApp::new`; this function then runs when a document opens, which on
every real launch is a fraction of a second later, because pdfcer is
started on a file.


⇒ So the property is pinned here, at the seam that broke it, and it is
pinned for BOTH fields: a carry that moved only the tick would leave the
limit resetting to two seconds on every open, which is the half of O187
the operator would notice second and complain about just as much.
