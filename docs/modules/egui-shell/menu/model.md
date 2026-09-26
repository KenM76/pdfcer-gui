# `egui-shell/menu/model`

## Item notes

### `static EMPTY`

A `static` so [`menus_of`] can return a reference with no allocation
and no lifetime gymnastics. `Vec::new` is `const`, so this costs
nothing at run time.

### `const IMPLICIT_SOME`

Named separately from [`ron_options`] so the two spellings — the
parser's and the pretty printer's — cannot drift apart within this
file.

### `fn ron_options`

# This must stay identical to [`crate::manifest`]'s

It is a second copy of one decision, which is a drift hazard, and the
alternative was worse: `manifest::ron_options` is private and
`manifest/` is not this module's to edit. So the copy is made
deliberately, kept to one constant, and pinned by
`the_menu_ron_dialect_matches_the_manifests`, which parses the same
implicit-`Some` spelling through both types.

`IMPLICIT_SOME` is not cosmetic. Nearly every field here is an
`Option`, because the `Option` is what distinguishes "set this to
empty" from "do not mention this". Without the extension a present
value must be written `items: Some([…])`, and the obvious spelling —
the one every example in this module uses — fails to parse with
`ExpectedOption`, a message that means nothing to someone who has never
seen a Rust `Option`. The operator's file is precisely the one that
must not fail.

### `fn shortcuts`

Defaults to none, which is the honest answer for a bare [`Menus`]:
a catalog of menus carries no key bindings, and inventing some
would be the second copy of a keymap that
[`crate::menu::shortcut`] exists to prevent.

[`Shell`] overrides it with its own keymap, which is the whole
reason a menu wants to be in the same document as the ribbon. An
application whose accelerators live elsewhere supplies them
explicitly with
[`crate::menu::ContextMenu::with_shortcuts`].

### `fn a_menu_document_round_trips_through_ron`

The whole value proposition is that a menu is a *file*: an operator
edits it, an application ships one. Every one of those claims fails
if the round trip is lossy, and both forms are checked because a
pretty printer that emits something its own parser rejects is
discovered by the operator rather than by CI.

### `fn the_menu_ron_dialect_matches_the_manifests`

[`ron_options`] is a second copy of one decision, made because
`manifest`'s is private and `manifest/` is not this module's to
edit. A copy is only safe while something checks the two agree, and
the way they would silently disagree is `IMPLICIT_SOME`: with it,
`items: [...]` parses; without it, the same text fails with
`ExpectedOption` and every hand-written menu file in existence
stops loading.

So the same implicit-`Some` spelling is pushed through both types.

### `fn an_unstated_item_list_stays_unstated_through_a_round_trip`

This is the property [`Menus::overlay`] rests on: `None` means "do
not mention this" and must not come back as `Some(empty)`. If a
layer's omitted `items` round-tripped into `Some(vec![])`, saving
and reloading a customization would silently empty every menu it
mentioned — turning a reference into a deletion.

### `fn a_layer_replaces_adds_and_leaves_alone`

The three rows of [`Menus::overlay`]'s table, asserted together
because the interesting part is that they coexist — a layer that
mentions three menus must be able to change one, extend the set,
and leave the third alone in a single document.

### `fn one_command_may_appear_in_several_menus`

`RIBBON_IA.md` §5: the context menu *"carries the same commands
again … that is not duplication in the P1 sense — context menus are
not tabs"*. If a future edit extends the one-command-one-tab rule
over menus, this is the test that says no.

### `fn a_shell_carries_its_menus_and_an_absent_field_offers_none`

The `None` arm matters as much as the `Some` arm: a manifest that
declares no menus is the common case, and it must resolve to "no
menu for this context" — which the renderer turns into a right-click
that does nothing — rather than to a panic or an empty popup.

### `fn one_shell_document_carries_both_the_ribbon_and_its_menus`

The claim under test is that a `menus:` key in a shell document
survives the parse. It is worth a test of its own because the failure
is silent: `serde` ignores a field it does not know, so a shell that
did not carry menus would load the file, draw the ribbon, and leave
the operator's context menus doing nothing with no error anywhere.

The input is deliberately **hand-written** rather than produced by
the serializer. A round-trip test cannot detect an ergonomics defect
— writer and reader agree by construction — and this dialect needs
`IMPLICIT_SOME`, without which every one of these fields would
require a `Some(…)` wrapper that no operator would think to type.
See `D:/dev/rag/rust/ron_without_implicit_some_makes_every_optional_field_unwritable_by_hand.md`.

### `struct Menu`

Field-for-field the same shape as [`crate::manifest::Group`], for the
reason the module header gives. In particular [`Self::items`] is
`Option` rather than `Vec` for the *same* reason a group's is: the
`Option` is what distinguishes **"this menu is now empty"** from **"do
not mention this menu"**, and a customization layer needs to be able to
say both.

### `struct Menus`

A `Vec` rather than a `BTreeMap` because the on-disk form must be
hand-editable and a list of `Menu(context: "…", items: […])` reads far
better in RON than a map whose key is repeated inside its own value.
Lookup is linear, over a collection whose realistic size is under
twenty, once per right-click.

Duplicate contexts are refused by [`Self::validate`]; [`Self::get`]
returns the **first** match, so an unvalidated document degrades to
"the first definition wins" rather than to something order-dependent
and invisible.

### `fn overlay`

This is the menu half of `SHELL_FRAMEWORK.md` §5's customization
contract, and it follows [`crate::manifest::merge`]'s rules rather
than inventing softer ones:

| The layer says | The result |
|---|---|
| `Menu(context: "canvas.object", items: [...])` for a known context | that menu's items are **replaced** |
| `Menu(context: "canvas.object")` — no `items` key | nothing changes; the entry is a reference |
| `Menu(context: "new.thing", items: [...])` | the menu is **added** |

Replacement rather than element-wise splicing, because an item list
has no per-item key to merge on — `Separator` is not identified by
anything, and an operator reordering a menu is expressing an order,
not a set of moves. This is the same conclusion the manifest's
group merge reaches for the same reason.

**It is deliberately fail-soft and validates nothing.** A layer
naming a command that this build does not have is not an error
here: [`crate::menu::plan::resolve`] omits it at render time and
discloses the omission, which is what
`GUI_ROADMAP.md`'s no-placeholders rule requires anyway. Rejecting
the layer instead would throw away an operator's whole
customization over one stale id — the failure mode `merge`'s header
exists to argue against.

An application that wants the stale id reported at *load* time
rather than at render time calls [`Self::validate_against`].

### `fn validate_against`

**Opt-in, and never a precondition of rendering.** The renderer's
contract for an unregistered id is *omission with a disclosure*
([`crate::menu::plan::resolve`]), because
`GUI_ROADMAP.md`'s no-placeholders rule says a command that does not
exist in this build must be absent, not greyed. This function
exists for the application that would rather find its own typo at
start-up than at right-click time, which is a different question
from what the operator should see.

# Errors

[`MenuError::UnknownCommand`], naming the context and the id, so
the message points at the line in the file rather than at the file.

### `fn from_ron`

The ordinary path is [`Shell::from_ron`], which reads the menus out
of the same document as the ribbon. This is the escape hatch for an
application that would rather keep its menus in a file of their own,
or ship them without using the manifest at all.

# Errors

[`MenuError::Parse`], carrying RON's line and column.

### `fn to_ron`

# Errors

[`MenuError::Serialize`] if RON refuses the value, which for this
type's fields should not be reachable.

### `fn to_ron_pretty`

# Errors

As [`Self::to_ron`].

### `enum MenuError`

Every variant carries the offending identifiers as fields rather than
interpolated into prose, for the reason
[`crate::manifest::ManifestError`] gives: the document is hand-edited,
so an error must say *which line*, and an application must be able to
act on it — offering "reset this one menu" rather than "your file is
broken".

### `trait MenuLookup`

# Why the entry points take this rather than a `&Shell`

[`Shell`] is where menus belong, and it is not the only place they can
come from: an application may build a menu in code, or load one from a
file of its own, and neither has a `Shell` to hand. A renderer that
demanded one would force such an application to construct an otherwise
empty manifest, and a rendering test to do the same — which is how a
suite ends up green because there was nothing for it to fail against
(the failure mode `ribbon/testfont.rs` exists to prevent).

A trait removes the choice. [`Menus`], a single [`Menu`] and [`Shell`]
all implement it, and every entry point in this module accepts whichever
of the three the caller already has.

### `fn menus_of`

Everything in this crate that asks a `Shell` for a menu comes through
here, so the field has exactly one reader and the absent case has
exactly one answer.

A manifest that declares no menus resolves to the empty catalog rather
than to an error: no menu for a context is a right-click that does nothing,
which is the correct behaviour and not a failure.
