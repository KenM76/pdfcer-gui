# `app::layers` — **which optional-content groups are hidden, and whose
answer that is**

Everything here is about the **override relationship** between the operator
and the document's own `/D` optional-content configuration (ISO 32000-1
§8.11.4.3); nothing else in [`crate::app::state::OpenDoc`] participates in
it, which is why the subject lives in a file of its own.

## The rule the whole file exists to hold

`OpenDoc::layers.hidden` is `Option<BTreeSet<ObjId>>`, and the `Option` is
**three states, not two**:

| value | meaning |
|---|---|
| `None` | **obey the document.** Whatever `/D` says is off, is off |
| `Some(set)` | the operator's **complete** answer, replacing `/D` entirely |
| `Some(∅)` | the operator has explicitly revealed **everything**, including layers the document turns off |

The last two are the pair that gets collapsed. `reset_layers` restores the
document's configuration; `set_hidden_layers(BTreeSet::new())` shows every
layer the producer had deliberately hidden — a watermark, a plot-only
border, a set of construction lines. They are different acts with different
results, and core API trap T-12.9 is why: `pdfcer_render::LayerVisibility`
replaces the document's configuration rather than merging with it.

## Why the override *replaces* rather than merges

Because a merge has no expressible way to say *"show a layer the document
turns off"*. A caller therefore starts from [`OpenDoc::hidden_layers`],
which is the **complete current answer** — the override if there is one,
and the document's own resolution otherwise — and hands back a complete new
one. Handing in only the groups the operator touched would reveal every
layer the document had turned off, on the first click of any layer control.

## Why `hidden_layers` is computed and not cached

It is read when a control is clicked, not per frame, and a cached copy
would be one more thing to invalidate on an edit that adds a layer. The
generation counter beside it is the cheap thing that *is* kept, and it is
what makes a page texture stale.
