//! The **View** tab — *what is on my screen, and how is the page laid
//! out?*
//!
//! `RIBBON_IA.md` §5.2, amended by `MODES_AND_PANELS.md`'s two new Window
//! settings. Six groups: Page display, Render, Zoom, Display, Panels,
//! Window.
//!
//! # The defect this tab exists to fix
//!
//! `RIBBON_IA.md` §3, on the shipped build:
//!
//! > **The View tab contains no view controls.** It has two groups:
//! > `Panels` and `Show`. There is no zoom, no page layout, no view
//! > rotation, no read mode, no full screen. Read mode and full screen
//! > have **no ribbon control at all** — they are keyboard-only (Ctrl+H,
//! > F11) on a tab literally named View. This is the single most confusing
//! > thing in the current ribbon.
//!
//! So this tab gains four groups and loses one command (`Fonts`, to File ▸
//! Document, because it describes the file rather than the screen).
//!
//! # Zoom here does not duplicate the status bar in spirit
//!
//! The status bar keeps the *continuous* controls a user reaches for
//! constantly: −/%/+ and the fit toggles. This tab mirrors the three
//! **named** zoom levels under P1a — actual size, fit page, fit width — so
//! that a user looking under View for zoom finds zoom. The two *targeted*
//! zooms that would have no status-bar home, zoom-to-selection and marquee
//! zoom-to-region, are **N** and therefore absent.
//!
//! Mirroring is legal because the status bar is not a tab; the same
//! amendment that lets the QAT carry Open lets the status bar carry Fit
//! page. What P1 forbids, and `egui-shell` still enforces, is one command
//! on two *tabs*.
//!
//! # Two documented conflicts, resolved here
//!
//! `RIBBON_IA.md` §5.2's table lists **`Thin lines` twice** — once under
//! Render and once under Display. One command cannot be on one tab twice;
//! `egui_shell::Shell::validate` refuses it by name. It is kept in
//! **Render**, because that is where the parameter acts (it is a
//! rasterization rule about minimum stroke width, not an overlay the
//! viewer draws) and because the Render group's contents were enumerated
//! explicitly, thin lines included, when this tab was commissioned. The
//! Display entry is treated as the duplicate.
//!
//! §5.2 also lists **`Comments`** among View ▸ Panels' panel toggles,
//! while §5.5 gives Markup a `Comments` group holding the Comments panel
//! and §7's migration map sends the existing control there explicitly
//! (`Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`). The migration
//! map is the more specific statement, so the command lives on **Markup**
//! and this tab's Panels group does not list it.
//!
//! # The Render group is an operator decision, and it is a stated trade
//!
//! pdfcer caches one whole-page texture and scales it with linear filtering
//! during the settle interval. Measured in use on a large drawing, that is
//! *smoother* to pan and zoom than the comparison product's progressive
//! tile rendering — no seams, no piece-by-piece fill-in — at the cost of a
//! full re-raster once motion stops.
//!
//! Those are two legitimate trades, not a better and a worse, and which
//! one wins depends on the sheet and the machine. So the strategy is a
//! **choice on this tab**, with whole-page as the default because it is
//! what measured better. `ZOOM_SETTLE` and the raster-scale multiplier are
//! constants in the shipped code and become the two knobs beside it.
//!
//! **Status note.** Three of the five Render entries and both new Window
//! settings are **N** in `RIBBON_IA.md`'s marking, and would be absent
//! under P3. They are present because the tab was commissioned with them
//! named individually and their defaults specified — see
//! [`super::DIRECTED`], which lists every such entry with the instruction
//! that put it there, so the exception is visible rather than inferred.

use super::{command, group, group_two_rows, icon_only};
use crate::text::ribbon;
use egui_shell::manifest::Tab;

/// The View tab.
pub(super) fn tab() -> Tab {
    Tab::new("view", ribbon::tab_view())
        .with_question(ribbon::question_view())
        .with_groups([
            // ---------------------------------------------------------------
            // Page display — a radio set of which exactly one is active.
            //
            // Single page stays the **default**: paging one drawing sheet
            // at a time is the right model for drafting review, and the
            // existing navigation is good. Continuous, facing and
            // facing-continuous are modes chosen here for the case where
            // the document is a 40-page specification rather than a sheet
            // set, and the choice persists per document so opening a
            // drawing set does not inherit a report's setting.
            //
            // ★ **All four are present as of Phase 4**, and the note that
            // used to sit here — *"the build behind them is larger than it
            // looks: the viewer holds a single page index, and the object
            // provider returns nothing for any page but the current one"* —
            // was right and is discharged. The page range turned out not to
            // be a field at all: `viewer::strip` computes which pages are
            // on screen from where they are laid out and where the viewport
            // is, and `view.page_index` keeps its single index, now meaning
            // *"the page the operator is looking at"* — derived from the
            // scroll position under a continuous mode. The provider still
            // serves one page, and it is still the right one, because
            // pressing on a page makes it current before the hit test runs.
            //
            // The order is the scale it describes: fewest pages on screen
            // first, most last. Under P1a a radio's positions are ordered by
            // what they do, not alphabetically.
            //
            // Each renders **pressed** while it is active, through the
            // `selected:` convention `view.tool_hand` already documents —
            // published by `PdfcerApp::conditions` from
            // `shell::commands::page_display_command`. Without that the group
            // would be four buttons with no indication of which one you are
            // in, which for a radio is the whole of the control.
            // ---------------------------------------------------------------
            // ★ **Two rows**, `OPERATOR_REQUESTS.md` O97. Four square icon
            // buttons in a row is a strip; as a 2 × 2 block they are half the
            // width and read as the single four-position choice they are —
            // which is also what Acrobat's own view controls look like.
            group_two_rows(
                "page_display",
                ribbon::group_view_page_display(),
                [
                    icon_only("view.page_single"),
                    icon_only("view.page_continuous"),
                    icon_only("view.page_facing"),
                    icon_only("view.page_facing_continuous"),
                ],
            ),
            // ---------------------------------------------------------------
            // ★ Render — DELETED 2026-08-17, and this comment is the record.
            //
            // It held five commissioned knobs — strategy, quality, settle,
            // thin lines, antialiasing — all registered, all drawn, all inert.
            // Checked against the engine on 2026-08-17:
            //
            //   strategy    no tiled-progressive path exists in this shell
            //   thin lines  `RenderOptions` has no such field
            //   antialias   `interpret.rs` sets `anti_alias: true` as a literal
            //   quality     REAL — moved to Settings ▸ Drawing the page
            //   settle      REAL — moved to Settings ▸ Drawing the page
            //
            // The two that were real are **settings**, and a settings window
            // now exists to hold them. `RIBBON_IA.md` §6 lists what deliberately
            // does not go on the ribbon and these belong on it: a value an
            // operator sets once and forgets is not an activity, which is what
            // P2 says a ribbon tab picks.
            //
            // The group is deleted rather than shipped empty, because an empty
            // captioned band is a caption offering nothing — the placeholder P3
            // forbids, and the same call made for Edit ▸ Clipboard on
            // 2026-08-14. `crate::app::prefs`' header carries the evidence for
            // each of the five verdicts.
            // ---------------------------------------------------------------
            // ---------------------------------------------------------------
            // Navigate — what a drag on the page does. **Two items**, since
            // 2026-08-14.
            //
            // Its own group rather than two more buttons in Zoom, because a
            // tool is a MODE the page is in and a zoom level is an action
            // taken on it: pressing Hand changes what every later drag
            // means, while pressing Fit page happens once and is over.
            //
            // ★ **`view.tool_text` joined it**, and this group is where it
            // belongs for two reasons that are each sufficient. It is the
            // group that already holds a pointer-tool toggle, so the two
            // controls that answer *"what does a drag on the page do?"* are
            // adjacent and read as the pair they are. And it is on **View**,
            // which is the one tab every mode is shown — the text tool has to
            // be reachable from Read, Review and Edit alike, and a command
            // lives on exactly one tab (P1). Putting it on the Edit tab
            // instead would have been the same mistake the operator has
            // already corrected twice, in `edit.form_fill` → `view.panel_forms`
            // and `edit.copy_page_text` → `file.copy_page_text`.
            //
            // ORDER: Hand, then Text. Hand first because it is the older
            // control, because it is the one an operator reaches for on a
            // drawing sheet, and because the two are not a radio — they are
            // two ways of *leaving* the select tool, and neither implies the
            // other. Both render pressed while armed, through the `selected:`
            // convention this group's Hand already documents.
            //
            // There is deliberately no `view.tool_select` beside them. The
            // select tool is what the canvas does when nothing else is
            // armed, so a button for it would be a control whose pressed
            // state is "normal" — and Hand and Text each render pressed while
            // active, so between them they already say which of the three you
            // are in. Space-to-pan needs no control at all: the canvas reads
            // the key itself.
            // ---------------------------------------------------------------
            group(
                "navigate",
                ribbon::group_view_navigate(),
                // ★ **The order is the order a tool palette is always in**:
                // the arrow, the white arrow, the type tool, the hand. Every
                // program in this class puts them in that sequence, which means
                // the operator's eye already knows where to go before they have
                // read a single label.
                [
                    icon_only("view.tool_select"),
                    // ★★★ **Withheld outside Edit, 2026-08-31 — O69.**
                    //
                    // The operator: *"I'm still not entirely clear how to
                    // reliably get to a point where I can edit nodes. It seems
                    // like I click on Edit → Edit Objects, but also have to
                    // click on View → and the node selector under Navigate."*
                    //
                    // The Points tool has always been gated on
                    // `Capabilities::edit_content` at the dispatch arm — a
                    // press in Read or Review armed nothing, traced one line
                    // and said nothing on screen. So the button was drawn,
                    // enabled, and inert in two of the three modes: P3's
                    // definition, and the exact silent decline this project
                    // keeps removing.
                    //
                    // ★★ It is the ONE authoring tool parked on a tab that
                    // every mode shows. Markup and Measure tools are gated the
                    // same way and are never wrong, because their whole TAB
                    // disappears outside the modes that can use them — the
                    // second state cannot be set while the first is wrong. View
                    // is in every mode by design, so it cannot do that, and
                    // this item has to carry the condition itself.
                    //
                    // ★ `shown_when`, not `enabled_when`, and R9 is the
                    // reason: greying is for a capability that is
                    // *temporarily* unavailable and is explained on hover.
                    // "This mode does not edit page content" is not temporary
                    // — it is what the mode *is*, and the mode selector two
                    // inches away already says so. The Tool rail
                    // (`panels::tool::idle`) has hidden this exact row outside
                    // Edit since it was written; this brings the ribbon into
                    // line with the answer the shell already had in the other
                    // place the control appears.
                    //
                    // The `A` chord is unaffected in Read for the better
                    // reason: `capability::offers_command` filters chords by
                    // tab visibility, not item visibility, so the chord still
                    // reaches the arm — and the arm still declines. That is
                    // the next thing to fix under O69, not this one.
                    icon_only("view.tool_node").shown_when("mode.edit_content"),
                    icon_only("view.tool_text"),
                    icon_only("view.tool_hand"),
                    // ★★★ **Smart select** — `OPERATOR_REQUESTS.md` O70, and
                    // the operator asked for it here by name: *"we should have
                    // a checkbox in navigate for a Smart-Selector option."*
                    //
                    // ★ It is a **toggle among tools**, and that is not a
                    // category error: it changes what the arrow at the head of
                    // this row selects when you click with it. Putting it in
                    // View ▸ Display beside the chrome switches would file it
                    // with things that change what is DRAWN, and this changes
                    // what a gesture MEANS.
                    //
                    // `shown_when` for `view.tool_node`'s reason, verbatim:
                    // greying is for the temporarily unavailable and "this mode
                    // does not edit page content" is what the mode IS. The
                    // command also carries `enabled_when("mode.edit_content")`
                    // so a keyboard route cannot reach it where the item is
                    // hidden.
                    icon_only("view.smart_select").shown_when("mode.edit_content"),
                ],
            ),
            // ---------------------------------------------------------------
            // Zoom — the three named levels, mirrored from the status bar
            // under P1a, plus the two that frame something rather than
            // stepping. Both of the latter were **N** until Phase 3.
            //
            // The order is deliberate: the three fixed levels first, because
            // they answer "show me the whole page" and are what a reader
            // reaches for; the two framing verbs after, because they answer
            // "show me THIS" and require the operator to have already chosen
            // a this.
            // ---------------------------------------------------------------
            group(
                "zoom",
                ribbon::group_view_zoom(),
                [
                    command("view.zoom_actual"),
                    command("view.zoom_fit_page"),
                    command("view.zoom_fit_width"),
                    command("view.zoom_fit_height"),
                    command("view.zoom_selection"),
                    command("view.zoom_region"),
                ],
            ),
            // ---------------------------------------------------------------
            // Display — what is drawn over and beside the page. Thin lines
            // lives in Render (see header).
            //
            // ★ **Rulers, grid and guides were N and are now built**, which
            // completes `RIBBON_IA.md` §5.2's Display row and the last unbuilt
            // line of `FEATURES.md`'s Phase 3. The note that used to sit here
            // is discharged rather than reworded, and the three entries are
            // removed from `super::PLANNED` — where `view.guides` carried the
            // condition it was waiting on, *"needs a per-document store to
            // survive a reopen"*, which `crate::canvas::guides` now is.
            //
            // ORDER: rulers, grid, guides — which is the specification's, and
            // is also a dependency order the operator can feel. A grid is read
            // against the ruler's numbers (both come from one tick ladder),
            // and a guide is *dragged out of* a ruler, so the control that
            // makes the other two make sense comes first.
            //
            // Each renders **pressed** while it is on, through the `selected:`
            // convention — published by `PdfcerApp::conditions` from
            // `shell::commands::chrome_command`. Independent toggles rather
            // than a radio: any combination of the three is meaningful, unlike
            // "facing, but also single".
            // ---------------------------------------------------------------
            group(
                "display",
                ribbon::group_view_display(),
                [
                    icon_only("view.show_annotations"),
                    icon_only("view.show_points"),
                    icon_only("view.rulers"),
                    icon_only("view.grid"),
                    icon_only("view.guides"),
                ],
            ),
            // ---------------------------------------------------------------
            // Panels.
            //
            // `Sidebar` is the rail toggle — page thumbnails and the
            // active tool's options — which is why there is no separate
            // `Pages` panel command: the thumbnails are the rail's first
            // pane, not an independently toggleable panel.
            //
            // ★★★ **Two sentences were struck here on 2026-08-28, and both had
            // been false for a while.** They read:
            //
            // > ~~`Forms` is likewise not a panel toggle today; the forms
            // > surface is reached from Edit ▸ Forms. Both are in PLANNED.~~
            //
            // `view.panel_forms` shipped and is a panel toggle like any other;
            // `registers.rs` records its removal from `PLANNED` in the same
            // commit. And `view.sidebar` is **not** in `PLANNED` either — it is
            // registered, drawn, and on the SCAFFOLDED list, which is a
            // different register entirely.
            //
            // ⇒ Neither claim could fail a test. `PLANNED` is asserted in both
            // directions against the manifest, so an id wrongly *listed* there
            // fails loudly — but a **comment** saying an id is in `PLANNED`
            // when it is not is prose, and prose is checked by readers. This is
            // the fourth site found in one audit; see
            // `shell::commands::reach`'s count assertion for the habit that
            // finds them.
            // ---------------------------------------------------------------
            group(
                "panels",
                ribbon::group_view_panels(),
                [
                    // ★★★ `view.sidebar` was the FIRST item here until
                    // 2026-08-31 — `OPERATOR_REQUESTS.md` O68's sweep. The
                    // command is unregistered, so this item would be dropped
                    // at load anyway (an item naming an unregistered command
                    // is not drawn, which is R8's mechanism); it is deleted
                    // rather than left to be silently dropped, because a
                    // manifest is the SPEC and a spec naming something that
                    // does not exist is a lie a reader has to discover.
                    //
                    // There is no sidebar rail in this build — there is a
                    // dock, and every dock panel below already has its own
                    // command. So the group loses nothing.
                    // ★ Pages is a panel like any other in this build. The
                    // note above described the OLD shell's sidebar rail, in
                    // which thumbnails were the rail's first pane rather than
                    // an independently toggleable panel; this build has no
                    // rail, so the panel needs — and now has — its own toggle.
                    command("view.panel_pages"),
                    command("view.panel_bookmarks"),
                    command("view.panel_layers"),
                    command("view.panel_signatures"),
                    command("view.panel_objects"),
                    // ★ Last, and on this tab at all, because the operator
                    // answered `crate::app::modes`' open question on
                    // 2026-08-14: Read fills forms. Read is shown `file` and
                    // `view` alone, so a fill verb the mode can reach had to
                    // live on one of the two — and the Forms panel is a
                    // panel, so it belongs beside the other panel toggles
                    // rather than in a group of its own.
                    //
                    // Last in the group rather than in panel-name order:
                    // this one is a *capability* the other five are not — it
                    // opens the only panel here that writes to the document
                    // — and the operator meets the read-only surfaces first.
                    // Edit ▸ Forms keeps the three authoring verbs, which is
                    // the line this move draws: filling is not authoring.
                    command("view.panel_forms"),
                ],
            ),
            // ---------------------------------------------------------------
            // Window — the shape of the application.
            //
            // Read mode and full screen are the two commands that exist
            // today with no control at all, which is the defect quoted in
            // the module header.
            //
            // The two settings after them retire `FEATURES.md`'s "nothing
            // floats over the canvas" as an absolute and replace it with a
            // pair of independent choices. The distinction between them is
            // the whole point:
            //
            //   Floating panels  Off · Allowed     default Allowed
            //     Whether the OPERATOR may tear a panel out. Off restores
            //     today's behaviour exactly.
            //
            //   App initiative   Never · Ask · Allowed   default NEVER
            //     Whether pdfcer may float a surface over the canvas ON ITS
            //     OWN — tool option boxes, transient property bars,
            //     notifications.
            //
            // The second carries the original complaint (an accept/reject
            // box that appeared over the drawing and moved on every zoom),
            // and its default of Never preserves that decision's outcome
            // as the shipped behaviour while making it a choice rather
            // than a law. A panel the operator deliberately tears out is
            // not the same thing as a box the application decides to
            // float, and one setting each is what keeps them separable.
            //
            // Both are per-operator rather than per-document.
            //
            // `Save workspace…` and `Load workspace ⌄` are **N** and sit
            // between App initiative and Reset layout when they land.
            // ---------------------------------------------------------------
            group(
                "window",
                ribbon::group_view_window(),
                [
                    // ★ **Which document am I looking at** comes first, before
                    // the two verbs that change the shape of the window.
                    //
                    // `RIBBON_IA.md` §3 gives this group the question *"what
                    // shape is the application in?"*, and "which of my open
                    // documents is in front of me" is the first and largest
                    // answer to it — larger than read mode and larger than
                    // full screen, because it changes what is on the page
                    // rather than what is around it.
                    //
                    // Previous before Next, in reading order, as every
                    // navigation pair in this manifest is.
                    command("view.previous_document"),
                    command("view.next_document"),
                    // ★ On the ribbon as well as on a tab's context menu, and
                    // the ribbon entry is not decoration: `menus`'
                    // `every_menu_command_is_also_reachable_from_the_ribbon`
                    // holds that *a command reachable by right-click alone is
                    // undiscoverable*. From here it keeps the document on
                    // screen; from a tab, the tab that was right-clicked.
                    command("view.close_other_documents"),
                    command("view.read_mode"),
                    command("view.fullscreen"),
                    // ★★ **Dock all panels**, immediately before Reset
                    // layout, and the order is the two-tier shape
                    // `MODES_AND_PANELS.md` singles out: the cheap remedy
                    // first, the destructive one after it. Docking every
                    // float costs the operator nothing they arranged;
                    // resetting costs them the arrangement. A menu whose
                    // gentler remedy is *below* the harsher one trains people
                    // to reach past it.
                    command("view.dock_all_panels"),
                    // ★★ The two auto-hide commands, 2026-09-05, immediately
                    // before Reset layout and after the float recovery — the
                    // same two-tier order that pair already established: the
                    // remedies that cost the operator nothing they arranged
                    // come before the one that costs them the arrangement.
                    // Hiding a strip is reversible by the control beside it;
                    // Reset layout is not reversible at all.
                    //
                    // ⚠ This takes View ▸ Window from six items to eight, so
                    // the group goes from 2 × 3 to 3 × 3 and `mockups/
                    // pdfcer-shell.html` was updated to match — the mock is the
                    // spec where the two disagree, EXCEPT where a capability
                    // shipped after the mock was drawn, which this is. Moved on
                    // the mock side, said here.
                    command("view.ribbon_auto_hide"),
                    command("view.rail_auto_hide"),
                    command("view.reset_layout"),
                ],
            ),
        ])
}
