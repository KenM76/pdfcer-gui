//! # `shell::commands::catalog::tools` — the Tools tab — what runs across files, or is configured once
//!
//! One band of [`super::all`]'s catalogue. Split out of [`super`] under **R2**
//! on 2026-08-28, when the Attachments command took that file to 1,495 of its
//! 1,500 lines and the next command registered would have broken the rule.
//!
//! ## ★★★ The split is per TAB, and the reason it was refused before is gone
//!
//! [`super`]'s header argued against exactly this cut:
//!
//! > a per-tab split would put the handler-token blocks in eight files where a
//! > collision between two of them is invisible.
//!
//! **That objection was already false when it was written.**
//! `super::super::tests::every_handler_token_is_unique` sweeps the whole
//! registry, and `every_handler_token_is_in_its_tabs_block` asserts each token
//! sits in its own tab's hundred. A collision is not invisible — it is a red
//! test, in either arrangement — so the argument that kept 120 commands in one
//! file rested on a property two tests had already taken over.
//!
//! ⇒ Recorded rather than quietly reversed, because it is the same shape this
//! project keeps finding: **a reason that was true when written, is checked by
//! nobody, and outlives what made it true.**
//!
//! ## What is here, and what is not
//!
//! The `Command` entries and the argument for each one's label, tooltip,
//! handler token, icon and enable predicate. **The prose is the point** — most
//! of this file is the record of decisions that would otherwise be re-litigated,
//! which is also why the byte count grew past a limit in the first place.
//!
//! Not here: the registration itself ([`super::super::register`]), the
//! command-id-to-behaviour mapping ([`super::super::mapping`]), and the
//! reachability register ([`super::super::reach`]).

use egui_shell::Command;

use super::command;
use crate::text::commands as t;

/// This band's commands, in ribbon order.
pub(super) fn band() -> Vec<Command> {
    vec![
        //
        // The batch commands and the font folders take their inputs from
        // disk, so they are available with nothing open. That is the whole
        // distinction between this tab and Pages, expressed as a predicate.
        // ===================================================================
        command("tools.merge_files", t::tools_merge_files(), 700).with_icon("combine"),
        // ★★★ `tools.split_files` was HERE until 2026-08-31 — O68, and it is
        // UNREGISTERED rather than implemented or greyed.
        //
        // The operator pressed it and nothing happened. It had no dispatch arm
        // and its blocker is real and names a missing capability in THIS
        // repository: a boundary chooser (every N pages / at bookmarks / an
        // explicit list), a destination directory and a name template.
        // `plan_split` was built to feed exactly that dialog — the engine's own
        // comment says *"nothing is written until you click Split"* — and the
        // dialog does not exist.
        //
        // R9 decides the rest: an unavailable capability renders NOTHING.
        // Greying is reserved for the *temporarily* unavailable and is always
        // explained on hover, and there is no honest hover sentence for "this
        // was never written". It returns with `pages.split`, which is the same
        // dialog with a different operand set. Both are in `manifest::PLANNED`.
        command("tools.font_folders", t::tools_font_folders(), 710).with_icon("font-folders"),
        // ★★ **The three-way `fonts` share ended 2026-09-04**, with art adopted
        // from the outside review of 2026-09-03.
        //
        // What the borrow was costing: [`crate::icons::Icon::Fonts`] belongs to
        // the Fonts PANEL, which reports and writes nothing, and both font
        // commands named it. So a read-only report and two commands that rewrite
        // the document's font programs were one picture drawn three times,
        // across two different tabs. **An icon is a claim**, and this one claimed
        // that looking at a font list and stripping font programs out of a file
        // were the same kind of act. The destructive half is the worse of the
        // two: `unembed_fonts`' own action module records that unembedding
        // "genuinely breaks that guarantee", and it was drawing the same tile as
        // the panel that merely lists faces.
        //
        // ★ **The pair is one drawing with one difference and may not be redrawn
        // separately.** Both are a capital A sealed inside a frame; the frame is
        // SOLID for embed and DASHED for unembed, and that dash is the entire
        // distinction between them. It renders — `icons::svg` parses
        // `stroke-dasharray` (`Shape::dash`, added 2026-09-04 naming this pair
        // and `measure-perimeter` as the reason). If a later pass ever
        // "simplifies" one of these two to a solid frame, the two commands
        // become one picture again and the destructive one is the one that
        // disappears into the other.
        //
        // Against `fonts` itself the cue is the frame: an A on an open baseline
        // rule reads as "a typeface, listed"; an A closed inside a box reads as
        // "the face is held inside this container", which is what embedding IS,
        // and a broken box reads as the container named with nothing left in it.
        // Deliberately not from the I-beam family (`add-text`, `text-select`)
        // and not `edit-text`'s pencil: those act on the WORDS, and these act on
        // the FACES the words are drawn in.
        command("tools.embed_fonts", t::tools_embed_fonts(), 711)
            .with_icon("embed-fonts")
            .enabled_when("doc.open"),
        command("tools.unembed_fonts", t::tools_unembed_fonts(), 712)
            .with_icon("unembed-fonts")
            .enabled_when("doc.open"),
        // ★ **The borrow of `tools` ended 2026-09-04.** What it was costing is
        // the set's standing argument in miniature: **an icon is a claim.** The
        // `tools` wrench says *adjust this*, and this command adjusts nothing —
        // it reports what the renderer had to substitute or leave out. It also
        // meant the tab's own identity glyph was doing duty as one command's
        // art, so the tile was indistinguishable from the tab that contained it.
        // Same argument that keeps `fonts` from being a pencil.
        //
        // A folded page carrying a measurement trace across it. Distinct from
        // [`crate::icons::Icon::Text`]'s folded page by the trace, which is the
        // only thing on it that is not typography; distinct from
        // [`crate::icons::Icon::Properties`] because Properties answers *what
        // the document records* and this answers *what the drawing cost*.
        command(
            "tools.render_diagnostics",
            t::tools_render_diagnostics(),
            720,
        )
        .with_icon("render-diagnostics")
        .enabled_when("doc.open"),
    ]
}
