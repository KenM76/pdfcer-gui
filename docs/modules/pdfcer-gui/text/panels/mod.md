# `text::panels` — every string the dock's panels show

One area of the catalog described in [`crate::text`]'s header. It covers
the panel bodies in [`crate::panels`]. The three document-structure
panels are in this file; Comments, Fonts, Objects and Properties each have
their own module, and Forms and Pages have their own areas one level up.

| Module | Panel |
|---|---|
| `mod.rs` (this file) | the three **document-structure** panels — Bookmarks, Layers, Signatures — plus [`byte_size`], which several areas share |
| [`attachments`] | the Attachments panel — the files a document carries inside itself, and the four verbs opposite them |
| [`comments`] | the Comments panel — every annotation on the document, what each one is, and the five disclosures a row can carry |
| [`face`] | the **face chooser**, drawn on two surfaces from one module, and the standard-14 disclosure it owes |
| [`fonts`] | the Fonts panel's inventory report |
| [`objects`] | the Objects panel, and the wording of every [`crate::panels::objects::summary::ObjectSummary`] fact |
| [`properties`] | the Properties panel |

## Almost every sentence here is salvaged verbatim, and that is the point

These strings came across from the old shell's `ui_text.rs` (7,912 lines,
1,193 entries) **with their doc comments**, because the doc comment is
usually the record of a defect the wording was changed to fix. Three
examples, all of which are below:

- [`signature_leaves_tail`] is worded as *under-protection* rather than
  as damage, because ISO 32000-1 §12.8.1 makes whole-file coverage a
  `should` — the document is conforming, and an operator told "invalid"
  about a legal file has been misled just as surely as one told nothing.
- [`layers_session_only_note`] exists because a panel of tickboxes over a
  document is, by every other application's convention, an editor — and
  this one is not. Its doc comment carries the reasoning the wording holds.
- [`fonts::font_verdict_removable`] and its four siblings are **two
  words each**, because a full sentence there clips at the dock's edge and
  takes the byte size with it.

⚠ **Rewriting any of those from fresh words re-derives a decision already
paid for**, and the rewrite has no access to the screenshot or the defect
that bought the current wording. Amend a string only with a reason, and
write the reason into its doc comment.

## The three panels in this file share a posture

**Each leads with the shape of what it is about to tell you**, because a
panel headed "Signatures" listing byte counts is the single likeliest
place in this application for an operator to take away more than was said.
The Signatures panel's opener is [`crate::text::trust::panel_intro`],
beside the rest of the trust copy. The Layers panel opens by saying that a
toggle changes what you see and not the document, and that nothing it does
is saved. The Bookmarks panel says when its own reader gave up.

That ordering is not stylistic. A caveat below a list arrives after the
operator has already drawn a conclusion.

## Conventions, restated from [`crate::text`] because they bind here

- **Sentence case, no trailing period on labels; full sentences with
  punctuation for prose.**
- **Name the thing and what the operator can do about it.**
- **Never state a capability the build does not have** — and never state
  the ABSENCE of one either, which is the half that rots. A denial is a
  claim about the engine with a shelf life of hours; see the note above
  [`signatures_none`].
