# `text::status::refused` — **the one sentence for an edit the ENGINE
refused and could not be asked why**

One string, in a file of its own, for [`super`]'s stated reason and
[`super::formdelete`]'s exact precedent: a catalog area is keyed by the
consumer it serves, this one's consumer is
[`crate::app::status::decline`]'s [`Declined::EditRefused`] arm alone, and
**R2** (no `.rs` file over 1,500 lines) is what forced the split while the
subject boundary is what decided where it fell. `super`'s `mod.rs` stands at
1,479 lines, so a paragraph added there would be a paragraph added to a file
two dozen lines from the ceiling.

## ★★★ Why the sentence lives HERE and not with a surface

[`crate::app::status::decline::Declined::line`] reaches out of
[`crate::text::status`] five times — to [`crate::text::tool`], to
`text::forms::groups`, to `text::panels::bookmarks` — and every one of those
reaches is the same rule applied: **a string lives with the surface that
owns its subject.** The Points tool's refusal belongs to the tool; a
bookmark drag's belongs to the bookmarks panel.

This sentence has no such surface. Its subject is not text editing, not
rotation, not form fields: it is *an edit — any edit — that
`crate::app::actions::funnel` asked for and the engine declined*, arriving
from any of ~78 call sites, about whichever verb the operator happened to
invoke. The only surface that owns it is the status bar's `⊗` slot itself,
and the catalog area whose consumer is `crate::app::status` is this one.

⇒ So the reach-across precedent argues **against** reaching across here. It
stays in `text::status`, in a file of its own, exactly as `formdelete` did.

[`Declined::EditRefused`]: crate::app::status::decline
