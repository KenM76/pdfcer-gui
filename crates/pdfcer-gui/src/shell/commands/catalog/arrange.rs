//! # `shell::commands::catalog::arrange` — the Markup tab's **Arrange** group:
//! which mark is drawn on top
//!
//! One band of [`super::all`]'s catalogue, registered 2026-09-06. It is the
//! **fifth** file in this per-tab split and the first that is a *group* rather
//! than a tab, so the seam is worth stating: [`super::markup`] holds the
//! commands that **add** a mark to the page, and these four change the order the
//! marks already there are painted in. Same tab, different subject, and the
//! subject is the one that decides which file a registration lives in — that is
//! [`super`]'s own rule, applied one level down.
//!
//! ## ★★★ The ids are `markup.*` and the file is `arrange.rs`, deliberately
//!
//! `super::super::tests::every_handler_token_is_in_its_tabs_block` asserts that
//! a command's handler token sits inside the hundred belonging to its id's
//! prefix — `file.` is 100, `markup.` is 500 — and it **panics by name** for a
//! prefix it does not know. An `arrange.` prefix would therefore have needed a
//! tenth row in that table, in a file three other tracks are editing today, to
//! state a fact that is already true: **these are Markup-tab commands.**
//!
//! *Arrange* is the name of the **group** they sit in. `RIBBON_IA.md` §5 names
//! groups freely and ids by tab, and every other group in this build follows the
//! same rule — `markup.highlight` is in the Text markup group, `file.encrypt` in
//! Security. So the id prefix says which tab, the file name says which group,
//! and neither is an abbreviation of the other.
//!
//! ## ★★★ Tokens 560-563, out of the band's own run, and why not 508
//!
//! Tokens are never reused and the Markup block already holds 500-507 (shapes
//! and Finish), 510-513 (text markup), 520-522 (notes), 530-531 (the node verbs,
//! wired hours before these), and 540 (Comments). A **new decade** rather than
//! the next free number, so that a reader of a raw token in a trace can tell an
//! Arrange command from a shape at a glance — which is the whole reason the
//! hundred-blocks exist, applied within a block that is now five families deep.
//!
//! ## ★★★ No icons, and it is a recorded refusal rather than an omission
//!
//! [`super`]'s header states the rule this follows: **`None` is a real answer.**
//! *"Every icon is a drawing somebody has to make, and inventing a key for an
//! icon that does not exist would produce a missing-glyph box at run time — a
//! placeholder, arriving through the back door."*
//!
//! There is no front, back, arrange or stacking glyph in
//! `crate::icons::catalog::mapping`, and the two near-misses were both refused
//! for stated reasons:
//!
//! | candidate | why not |
//! |---|---|
//! | `chevron-up` / `chevron-down` | they already mean *move this up the PAGE LIST* — `pages.move_up`'s glyphs — and borrowing them here would tell an operator that Bring forward reorders the document |
//! | `show-points`, `edit-objects` | they depict a shape's anatomy, not its depth; a picture that describes the wrong operation is worse than a word |
//!
//! ⇒ So all four render as their words, which is what
//! `crate::icons::assets::PROVENANCE.md` makes the honest fallback: that
//! directory is the **operator's own art**, and the alternative to a reuse is to
//! ask him for a drawing, not to generate one. Four labelled controls in a
//! captioned group read correctly; the labels are the four words every drawing
//! program uses and they are the most findable thing about the feature.
//!
//! ## ★ `selection.markup_restylable`, and its name is now narrower than the
//! fact
//!
//! The predicate these four wait on means *a markup annotation is selected and
//! this mode may author markup* — one fused fact, published by `app::conditions`
//! since 2026-09-06 for the five Format ▸ Markup controls. It is **exactly** the
//! question these four ask, so reusing it is what stops two names for one fact.
//!
//! Its name says *restylable* because restyling was its only client when it was
//! coined that morning. Arranging is not restyling. Renaming it is a change to
//! `app::conditions`, `shell::commands`' `KNOWN` list and `manifest::format` —
//! three files with three other tracks in them today — so it is **recorded here
//! rather than done**: the name is a client's, the fact is general, and the next
//! session that touches that condition should widen the name to
//! `selection.markup_actionable` or similar.
//!
//! ★★ The **lock** is deliberately outside the predicate, which is what makes
//! these four live on a locked mark and refuse with a sentence
//! (`app::dispatch::arrange`). That is R9 read correctly: §12.5.3 bit 8 is a
//! fact about one annotation rather than about the build or the mode, so it
//! earns an explanation and not an absence.

use egui_shell::Command;

use super::command;
use crate::text::commands as t;

/// This group's commands, in ribbon order.
///
/// ★ **Front first, back last** — the order every reference application uses,
/// and it is not arbitrary: read top to bottom the four are a single axis from
/// nearest to furthest, so the list itself teaches what the words mean. Sorting
/// them any other way (the two ends together, then the two steps) would group
/// them by *how far* rather than by *which way*, and an operator scanning for
/// "send it back" would have to read all four.
pub(super) fn band() -> Vec<Command> {
    vec![
        command("markup.bring_to_front", t::markup_bring_to_front(), 560)
            .enabled_when("selection.markup_restylable"),
        command("markup.bring_forward", t::markup_bring_forward(), 561)
            .enabled_when("selection.markup_restylable"),
        command("markup.send_backward", t::markup_send_backward(), 562)
            .enabled_when("selection.markup_restylable"),
        command("markup.send_to_back", t::markup_send_to_back(), 563)
            .enabled_when("selection.markup_restylable"),
    ]
}
