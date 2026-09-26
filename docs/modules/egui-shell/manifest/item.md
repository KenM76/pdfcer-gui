# `egui-shell/manifest/item`

## Item notes

### `enum ItemSize`

`RIBBON_SCALING.md` §5.1 has the measurements. Word has exactly three
sizes and a group mixes them freely — one Large button beside a column of
three Small ones is its Clipboard group — and that mixing is where its
density comes from: at 884 client points Word fits **ten** groups on the
band, which a band of uniformly Medium controls cannot approach, because
nothing in it can be narrower than a label.

[`Self::Medium`] is the default, so a manifest that states no size
renders exactly as one written before sizes existed. A vocabulary whose
default is the status quo can be introduced in one change rather than
behind a flag.

### `fn is_default`

Keeps `Command(id: "file.open")` in the manifest file rather than
`Command(id: "file.open", size: Medium)` on every one of a hundred
lines. The on-disk manifest is meant to be read and edited by an
operator, and a field that is the default everywhere is noise that
hides the two places it is not.

### `fn sized`

A builder rather than a second constructor, so the common form above
stays the short one and a sized item reads as *"this command, but
large"* — which is what it is.

A separator and a custom item have no size to set, and this returns
them untouched rather than panicking. A manifest is **data**, and the
honest response to nonsense in data is that it does nothing, not that
the application stops.

### `fn provided_by`

The builder for [`Item::Command::capability`] — read that field's
documentation before using this, because the distinction from
[`Self::shown_when`] is the whole of it: this is about the *build*, that
is about the *frame*.

Named `provided_by` rather than `when_available` or `requires` because
the sentence it makes at a call site is the true one: `file.sign`
**is provided by** the `signing` capability. `requires` would read as a
precondition the shell checks, and the shell checks nothing — see the
field's *"carries no meaning beyond its presence"*.

A separator and a custom item are returned untouched, for
[`Self::sized`]'s reason: a manifest is data, and the honest response to
nonsense in data is that it does nothing. A custom item drawn by the
application is the application's to omit; it has no command id, so there
is nothing here that could be absent from a registry.

### `fn visible_condition`

`None` means *always*, which is what the overwhelming majority of
items are.

**This function is where the rule lives**, which is what makes it
safe for two variants to declare the field. [`Item::Custom`]'s
`visible_when` carries that argument in full, and names the trigger
that would move the field onto a wrapper instead.

A **separator** still cannot carry one, and deliberately: a rule for
when a divider disappears is a rule about its *neighbours*, which is
the record-shaped problem the wrapper exists for. A separator between
two hidden items is a cosmetic defect; a separator with its own
condition, set independently of the items it divides, is a
contradiction that renders.

### `fn capability`

`Some` means *a build without this need not register the command, and
dropping the item is the intended configuration*; `None` means *this
item's command is mandatory and its absence is a bug*. See
[`Item::Command::capability`] for the whole rule and
`SHELL_FRAMEWORK.md` §7 for why the two cases must stay
distinguishable.

⚠ **Read by the merge and by nothing else.** It is deliberately not a
question the ribbon renderer, a panel or an application ever asks: the
one rule §7 keeps is that a capability's presence is expressed by
registering its command, and a second reader of this field would be a
second place that knows.
