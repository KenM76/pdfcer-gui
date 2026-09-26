# `app::actions::attachments` — the three verbs whose subject is a whole
FILE living inside the document

A sub-enum rather than three variants of [`super::action::Action`].
`super`'s declaration of `action` states the rule that puts them here —
*"the next family of variants to **grow** is the one that will have to
become a sub-enum"* — and a family of three verbs has grown before it is
written.

## What makes these a family, and it is not "they are all about
attachments"

A subject label would be the weak answer, and this enum has a structural one
that no other family in the vocabulary shares:

> **Every verb here operates on bytes that are in no page's content stream,
> and two of the three are file-system operations that happen to touch a
> PDF.**

Three consequences follow from that one property, and the whole module is
shaped by them:

1. **Every one of them opens a native file dialog, and therefore must be an
   `Action`.** `super::write`'s header states the rule in the sharpest form
   this project has written it: *"A native file dialog must not open inside
   a layout pass. It is a modal OS window that blocks the thread, so opening
   one from a widget's `clicked()` branch leaves egui part-way through a
   frame that will not finish until the operator has answered."*
   [`AttachmentAction::Attach`] and [`AttachmentAction::SaveCopy`] are
   `Action`s for **both** reasons — the funnel's invariant *and* the
   dialog's timing — where `WriteAction`'s three are `Action`s for the
   second alone.
2. **Nothing they do is visible on the canvas.** A document-level
   attachment (§7.11.4.1 route 2) is reached from the catalogue's
   `/Names /EmbeddedFiles` and appears in no rendering, so *every* one of
   these verbs owes a sentence to `crate::app::status`. Contrast
   `super::bookmarks`, where a rename is deliberately silent because the row
   the operator is looking at now reads the new name. There is no such row
   here: the panel is the only witness, and the panel is where the operator
   already is, so the disclosure carries the part the panel cannot show —
   what happened to the *file*.
3. **The operand cannot be an index and cannot be an `ObjId` either.** See
   [`AttachmentRef`], which is the interesting type in this module.

## The one refusal this module surfaces, and the three it does not

`EditSession::attach_file` refuses four ways. Three of them —
`DocumentEncrypted`, the certification gate, and
`ObjectCreationWouldExposeHiddenObjects` — are properties of the **document**
that every authoring verb in this shell shares, and this shell's settled
answer for them is [`super::apply::vector_edit`]'s trace. That is not
neglect: they are conditions no control on this panel created and none can
clear, and four sentences competing for the one status slot would evict the
disclosures the successful verbs owe.

`AttachmentTreeUnsupported` is surfaced, and the argument is at
[`crate::text::panels::attachments::attach_refused_multi_node_tree`]. In one
line: it is unreachable from any other surface, and the press otherwise
produces **nothing at all** — no row, no message — which an operator cannot
distinguish from a broken button.

## What is deliberately NOT here

**Editing a description.** `attach_file` takes it at attach time and
`pdfcer-core` has no verb that changes one afterwards, so there is no
variant, no field and no control. R9: an absent capability renders nothing.
The panel *says* so — see `attach_description_note` — because a control that
cannot exist is different from a limit that must not be discovered by
trying.

**Removing a page-level file attachment.** `detach_file` addresses the
`/EmbeddedFiles` name tree and answers `AttachmentNotFound` for a
`/FileAttachment` annotation **by name**, precisely so a shell can tell the
two apart; those are removed with `delete_annotation`, which is
`super::annot::AnnotAction::Delete`'s business and reached from the Comments
panel. Wiring a second route to it from here would give one act two
implementations, and the second would be the one that forgot the page
invalidation.
