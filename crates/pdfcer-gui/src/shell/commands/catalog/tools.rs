//! # `shell::commands::catalog::tools` — the Tools tab — what runs across files, or is configured once
//!
//!
//! ## The split is per TAB, and the reason it was refused before is gone
//!
//! [`super`]'s header argued against exactly this cut:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/commands/catalog/tools.md`.

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
        //
        // `tools.split_files` is not registered: `pages.split`'s window splits
        // the open document, and splitting files chosen on disk needs a file
        // list it does not have. R9: an unavailable capability renders
        // nothing. Its reason is in `manifest::PLANNED`.
        command("tools.font_folders", t::tools_font_folders(), 710).with_icon("font-folders"),
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
        command("tools.ink_picker", t::tools_ink_picker(), 721)
            .with_icon("ink-picker")
            .enabled_when("doc.pages"),
    ]
}
