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
