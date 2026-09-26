# `pdfcer-gui/app/actions/prefs`

**The verbs whose subject is a PREFERENCE, not a document.**

Its own module under **R2**, not because [`apply`] ran out of room but
because a preference verb answers to a different set of rules than every
other arm there.

# What makes this a family rather than a size-driven cut

Four properties, and every member has all four. A verb that has three of
them belongs somewhere else.

1. **It needs no open document.** Every other arm in [`apply`] acts
   on `Status::Open(doc)`, and the guard's *"no document: silently drop"* is
   the right answer for all of them. It is the wrong answer here, and
   wrong in a way that is almost impossible to report: the Find bar is
   reachable with nothing open, so a preference set there would stick most
   of the time and vanish the rest. A defect that works most of the time is
   the worst kind there is.

2. **It changes nothing an undo could reach.** No `EditSession`, no epoch
   bump, no raster invalidation, no [`funnel::vector_edit`]. The
   four-step protocol that every document change goes through has nothing
   to do here, which is why these arms `return` rather than falling into
   it.

3. **The live half has already been applied by the surface that raised
   it.** This is the property a reader most often gets backwards, so it is
   stated once here rather than twice in two variant docs. The *live* value
   and the *persisted* value have different owners:

   | Preference | Live owner | Why the live half cannot wait a frame |
   |---|---|---|
   | Find's *Zoom* tick | [`crate::find::FindState`] | the very next *Next* would obey the old answer |
   | Pages previews + limit | [`crate::panels::pages::thumbnails::ThumbnailCache`] | the very next render would obey the old answer |

   What neither surface can reach is `PdfcerApp::prefs` and the file behind
   it. **That is the only thing these actions are for.** They are a
   write-through, not an apply.

4. **It writes the file immediately**, and the failure is swallowed. The
   rule `app::actions::view`'s `smart_select` states and this module
   inherits: *one discrete operator decision is one write, now*. Losing a
   preference across a restart does not justify a modal in front of
   somebody who is in the middle of searching.

# Why the operand is carried and never re-read

The arm runs **after** the frame that raised it. The widget that was ticked
may not exist any more — a panel can close, a dock tab can change, the Find
bar can be dismissed — so an arm that went looking for the control to ask
what it was set to would be reading a surface that has already gone. Every
action in this crate carries a complete statement of intent for that
reason; here it is not a style rule but the difference between a preference
that sticks and one that sticks when the panel happens to still be open.

[`PrefAction::PagePreviews`](prefs::PrefAction::PagePreviews) takes this one
step further and carries
**both** of its two values even when only one changed, because
`Prefs::save` is a whole-file write. See its own doc.

# Where the other direction lives

Nowhere near here, deliberately. The read-back happens **once, at
construction**, in `crate::app::PdfcerApp::new`, which seeds the live
owners from the file. A preference that were re-read per frame would let
the file win an argument the operator had already had with the control.
