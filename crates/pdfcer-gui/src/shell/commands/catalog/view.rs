//! # `shell::commands::catalog::view` — the View tab — what is on screen and how the page is laid out
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
        // ★ **Page display — a radio, not four toggles.**
        //
        // Exactly one is active at a time, and which one is published as a
        // `selected:` condition by `PdfcerApp::conditions` so the active
        // position renders pressed (`egui_shell::ribbon::selected_condition`).
        // Four independent toggles would admit states that mean nothing —
        // "facing, but also single" — and would leave the ribbon reconstructing
        // which of them is on.
        //
        // All four are gated on `doc.pages` rather than `doc.open`: an
        // arrangement of pages is meaningless without pages, and a document
        // with `/Count 0` is legal.
        //
        // The tokens are contiguous (200-203) because they are one control.
        //
        // ★ **All four carry an icon or none would.** These are the positions
        // of one radio, and a radio whose positions are three glyphs and one
        // bare word does not read as a radio — the eye groups by shape before
        // it reads. The four glyphs are drawn as a set for the same reason:
        // left-to-right says how many pages are across, a cut bottom edge
        // says whether they keep coming, and those two axes are the whole
        // control (see `crate::icons::Icon::PageSingle`).
        command("view.page_single", t::view_page_single(), 200)
            .with_icon("page-single")
            .enabled_when("doc.pages"),
        command("view.page_continuous", t::view_page_continuous(), 201)
            .with_icon("page-continuous")
            .enabled_when("doc.pages"),
        command("view.page_facing", t::view_page_facing(), 202)
            .with_icon("page-facing")
            .enabled_when("doc.pages"),
        command(
            "view.page_facing_continuous",
            t::view_page_facing_continuous(),
            203,
        )
        .with_icon("page-facing-continuous")
        .enabled_when("doc.pages"),
        // The Render group is settings, not actions. They are available
        // with no document open because they are what the *next* document
        // will be drawn with, and a setting you can only change while
        // something is open is a setting you cannot prepare.
        //
        // ★ **No icons on any of the five, and that is a decision about the
        // whole group rather than five separate omissions.**
        //
        // Their labels ARE the control's content: "Strategy", "Raster scale",
        // "Settle delay", "Thin lines", "Antialias" each name a parameter
        // whose value is the thing an operator came here to read. None of the
        // five has an industry-conventional glyph — there is no picture of a
        // settle delay that anybody has already learned — so every candidate
        // would be art invented here, decoding to nothing the word beside it
        // did not already say.
        //
        // This is the reasoning the icon ui-spec §3.2 applied to Actual size
        // ("a numeral read at a glance is clearer than any glyph substitute
        // could be… both add a decode step a bare percentage does not need"),
        // applied to a group instead of a control. The 2026-08-14 icon pass
        // considered each of the five and refused each: an invented glyph on
        // a settings knob is decoration, and decoration on a ribbon costs the
        // legibility of the glyphs that mean something.
        //
        // ★ **All five were UNREGISTERED on 2026-08-17**, and tokens 210-214
        // are retired rather than reused — a token is an operator's saved
        // keybinding, and handing 211 to something else would silently rebind
        // whatever they had put on it.
        //
        // Three of the five had nothing behind them: there is no
        // tiled-progressive path in this shell, and `RenderOptions` has neither
        // a thin-lines nor an antialiasing field (`interpret.rs` sets
        // `anti_alias: true` as a literal). The other two were real and became
        // **settings** — Settings ▸ Drawing the page — because a value an
        // operator sets once and forgets is not an activity, which is what P2
        // says a ribbon tab picks.
        //
        // R8 is the rule that makes this a deletion rather than a hidden
        // control: *registering a command is the only way the GUI may learn
        // that a capability exists*. Three of these named no capability, and
        // the other two are no longer reached by a command at all.
        // `crate::app::prefs`' header carries the evidence per verdict.
        // ★ **No icon, on the icon ui-spec's own explicit instruction.**
        //
        // §3.2 is a whole section devoted to this one control: "Recommend
        // leaving `zoom_100_button()` as plain text ('100%'), not iconified…
        // a magnifier-with-'1' badge or a '1:1' pictograph both add a decode
        // step a bare percentage does not need… Flagged explicitly so the
        // engineer does not feel obligated to force an icon here against the
        // better outcome."
        //
        // The 2026-08-14 pass gave the other four Zoom entries glyphs and
        // deliberately left this one alone. A spec that anticipated being
        // overruled by a completeness drive, and argued against it in
        // advance, is the strongest kind of recorded decision there is.
        command("view.zoom_actual", t::view_zoom_actual(), 220).enabled_when("doc.pages"),
        // Zoom to selection is gated on `selection.bounds`, not on
        // `selection.any` — the two differ, and the difference is the
        // command's whole failure mode. A selection can exist and resolve to
        // no box (it names an object on another page, or one an edit has
        // renumbered), and the honest answer there is a greyed control, not
        // a press that silently frames nothing.
        //
        // Its glyph is a diagonal PAIR of corner brackets closing on an
        // object, not the four `fit-page` uses. Four here would differ from
        // Fit page — two buttons away in this same group — only by a small
        // rect in the middle, which is the same-group collision the icon
        // ui-spec §2.1 calls its one ❌-grade risk.
        command("view.zoom_selection", t::view_zoom_selection(), 223)
            .with_icon("zoom-selection")
            .enabled_when("selection.bounds"),
        // Arming, not acting: this changes what the next drag means. It
        // renders pressed while armed through the `selected:` convention,
        // and the canvas disarms it on release.
        //
        // `zoom-region` is the fourth member of the icon ui-spec §3.1
        // magnifier family, whose grammar is that the lens names the member:
        // empty is Find, a minus is zoom out, a plus is zoom in, a box is
        // "magnify the box you drag".
        command("view.zoom_region", t::view_zoom_region(), 224)
            .with_icon("zoom-region")
            .enabled_when("doc.pages"),
        // ★★ **The two pointer tools that make the canvas predictable**, added
        // 2026-08-19 on the operator's report:
        //
        // > *"The selector should be predictable like other programs. It seems a
        // > lot of ideas are getting invented instead of just using the … most
        // > common method expected."*
        //
        // He is right, and `view.tool_select` had been **deliberately absent** —
        // the comment that used to sit here read *"There is deliberately no
        // `view.tool_select` beside them"*, on the argument that Select is the
        // default you return to rather than a thing you pick. That argument is
        // sound and it produced an unusable surface: with no Select control
        // there was no *row of tools*, so the Hand and the Text tool read as two
        // unrelated toggles rather than as members of a set, and there was
        // nowhere for a third and fourth to join. A tool palette is the most
        // conventional object in this product class; not having one is the
        // invention.
        // ★★ **The object clipboard, 2026-08-19** — the operator's report:
        // *"also the standard copy/paste and I didn't try cut so possibly that
        // one too aren't implemented."* They were not.
        //
        // ★ Scoped to **markup and comments**, because that is what the engine
        // can express: `annot_author::spec_from_dict` reads one and `add_markup`
        // writes one back. Page content cannot be pasted — 157 verbs in
        // `edit.rs` and none inserts content, checked 2026-08-19 — so a copy of
        // a path would be offering a paste that could never happen. The labels
        // say "comment or markup" rather than "object" for exactly that reason.
        //
        // `enabled_when("doc.pages")` rather than a selection condition: what is
        // selected changes every click, and a control that greys and un-greys
        // under the pointer is harder to aim at than one that answers in a
        // sentence when pressed. The refusals are `canvas::clipboard::Refusal`,
        // on the status row, which is the same posture the six resize refusals
        // take.
        // ★★★ TWO conditions, so `edit.cut` is greyed over something the
        // clipboard cannot carry — asked for by `pdfcer-core` by name, because a
        // cut of an uncarryable thing is a deletion wearing a clipboard's
        // clothes.
        //
        // `Enable::Custom` rather than a string, because the predicate language
        // is one name with an optional `!` and *"anything richer belongs in
        // Custom, because a grammar in a string is a parser and a parser is a
        // thing that has its own bugs"*. Two names ANDed is exactly that case.
        //
        // ★ `selection.cut_permitted` defaults to TRUE, so this is `doc.pages`
        // on every ordinary document and every ordinary selection. It clears
        // only for a redaction mark and its two unreachable siblings. See
        // `canvas::cutgate`.
        //
        // ★★ Greying does NOT make the refusal redundant. A chord is dispatched
        // through the keymap without consulting enablement, so `Ctrl+X` reaches
        // the handler whatever the ribbon shows — which is why
        // `dispatch::clipboard` still names the reason on the status row, and
        // why that sentence carries the SUBTYPE the greyed button cannot.
        command("edit.cut", t::edit_cut(), 403)
            .with_icon("cut")
            .with_enable(egui_shell::commands::Enable::Custom(std::sync::Arc::new(
                |c| c.is_set("doc.pages") && c.is_set("selection.cut_permitted"),
            ))),
        command("edit.copy", t::edit_copy(), 404)
            .with_icon("copy")
            .enabled_when("doc.pages"),
        command("edit.paste", t::edit_paste(), 405)
            .with_icon("paste")
            .enabled_when("doc.pages"),
        // ★★ `edit.paste_duplicate` — Ken, 2026-08-29, O58. Registered as its
        // own command rather than read as a modifier inside `edit.paste`,
        // because a command is the unit this shell can bind, place, menu and
        // withhold; a modifier read inside a handler is reachable only from the
        // keyboard. `app::dispatch::clipboard`'s header carries the argument.
        //
        // ★ Same `doc.pages` gate and same icon as its sibling. A second paste
        // glyph would be a distinction the operator has to learn for no gain —
        // the two are told apart by their labels and by the chord in the
        // tooltip, which is how Word and Acrobat tell their paste variants
        // apart too.
        command("edit.paste_duplicate", t::edit_paste_duplicate(), 407)
            .with_icon("paste")
            .enabled_when("doc.pages"),
        command("view.tool_select", t::view_tool_select(), 252)
            .with_icon("cursor")
            .enabled_when("doc.pages"),
        command("view.tool_node", t::view_tool_node(), 253)
            .with_icon("cursor-node")
            .enabled_when("doc.pages"),
        command("view.tool_hand", t::view_tool_hand(), 225)
            .with_icon("hand")
            .enabled_when("doc.pages"),
        // ★ **The text tool** — 2026-08-14, and it closes two things at once.
        //
        // Beside `view.tool_hand` because View ▸ Navigate is where the *other*
        // pointer-tool toggle already lives, and because View is the one tab
        // every mode is shown. Both halves of that matter: a tool is a mode the
        // page is in rather than an action taken on it (which is why Navigate is
        // its own group and not a fourth button in Zoom), and a command lives on
        // exactly one tab — so a text tool on the **Edit** tab would be
        // unreachable from Read and Review, which is the shape of mistake the
        // operator has already had to correct twice (`edit.form_fill` →
        // `view.panel_forms`, `edit.copy_page_text` → `file.copy_page_text`).
        //
        // What it closes:
        //
        // 1. `canvas::textsel::takes_the_press` gave a press its text meaning
        //    only for the select tool in a mode that cannot select content, so
        //    Read ✓, Review ✓, **Edit ✗** — a reviewer could sweep text and an
        //    editor could not.
        // 2. The three `markup.*` text-markup commands are drawn on the Markup
        //    tab in Edit and could **never enable** there, because
        //    `selection.text` was never true. That is a live tension with
        //    `RIBBON_IA.md` P3 — greying is for *temporarily* unavailable — and
        //    it was not fixable by hiding them, because the Markup tab is in both
        //    Review and Edit and a command has one tab.
        //
        // ★ **The reference applications disagree here and Inkscape won.**
        // Acrobat and SolidWorks resolve text-versus-object *contextually*
        // inside one tool; only Inkscape uses a separate Text tool. The full
        // argument is at `crate::canvas::tool::CanvasTool::Text` and is not
        // restated here — in one line, an object marquee over *vector content*
        // is a surface Acrobat does not have at all, so its contextual answer is
        // not an answer to this conflict.
        //
        // **It arms a tool; it authors nothing**, so it takes no capability and
        // `retire_forbidden` permits it in every mode. It renders **pressed**
        // while armed through the same `selected:` convention `view.tool_hand`
        // documents, published from `PdfcerApp::conditions` — the step that was
        // once forgotten for the measure tools and shipped a tool that armed
        // without looking armed.
        //
        // `text-select` is a new glyph rather than a reuse of `add-text`: that
        // one is this I-beam **plus a badge**, and the badge is the difference
        // between creating text and selecting it. `doc.pages`, like every other
        // entry in this group — a pointer tool with no page under it has nothing
        // to point at.
        command("view.tool_text", t::view_tool_text(), 226)
            .with_icon("text-select")
            .enabled_when("doc.pages"),
        command("view.zoom_fit_page", t::view_zoom_fit_page(), 221)
            .with_icon("fit-page")
            .enabled_when("doc.pages"),
        command("view.zoom_fit_width", t::view_zoom_fit_width(), 222)
            .with_icon("fit-width")
            .enabled_when("doc.pages"),
        // ★ O29, "Adobe has fit height, so add that too." 227 is the next
        // free token in the View band; 223-226 are the zoom verbs and the two
        // pointer tools.
        command("view.zoom_fit_height", t::view_zoom_fit_height(), 227)
            .with_icon("fit-height")
            .enabled_when("doc.pages"),
        command("view.show_annotations", t::view_show_annotations(), 230)
            .with_icon("comment")
            .enabled_when("doc.pages"),
        // ★★★ `OPERATOR_REQUESTS.md` O70 — *"we should have a checkbox in
        // navigate for a Smart-Selector option"*. 258 is the next free token in
        // the View band — 257 is `view.close_other_documents`, which is at the
        // bottom of this file where a "next free" scan of the top of it does
        // not look. The uniqueness test caught it in one run, which is what
        // that test is for.
        //
        // ★ **`enabled_when("mode.edit_content")`, not `doc.pages`.** The
        // substitution it controls only happens where content is selectable at
        // all, so offering it in Read would be a control that reports a state
        // it does not currently govern — and R9 reserves greying for the
        // temporarily unavailable, which this is: it comes back in Edit.
        command("view.smart_select", t::view_smart_select(), 258)
            .with_icon("show-points")
            .enabled_when("mode.edit_content"),
        command("view.show_points", t::view_show_points(), 231)
            .with_icon("show-points")
            .enabled_when("doc.pages"),
        // ★ **The three chrome toggles**, and all three render pressed while
        // they are on, through the `selected:` convention `view.tool_hand`
        // documents and the page-display radio uses.
        //
        // They *can*, where the hand tool and the region zoom still cannot,
        // and the difference is worth naming because it is the reason the
        // state lives where it does: a `selected:` condition is published from
        // `PdfcerApp::conditions`, which is handed `&self` and **no
        // `egui::Context`** — so a toggle whose state lives in `egui::Memory`
        // has no route to the ribbon. These three live on
        // `crate::viewer::ViewState`, which `conditions` can read, so no
        // second mechanism was needed.
        //
        // ★ **All three now carry a glyph, and the note that used to stand
        // here is retired rather than reworded.** It read: "No icons: there
        // is no ruler, grid or guide key in `crate::icons::catalog`, and
        // naming one would draw the catalogue's deliberate slashed mark for
        // an unknown key. A command with no icon renders as its label, which
        // is the right answer here… the control's name is a word, and the
        // word is what makes it findable."
        //
        // The first half was a true statement about the catalogue and is now
        // false: `rulers`, `grid` and `guides` exist, authored 2026-08-14.
        //
        // The second half was a **misreading of the ribbon**, and it is worth
        // saying so plainly because it is the reason three controls stayed
        // bare longer than they had to. In a band, `egui_shell`'s
        // `band::command_button` is called with `shows_label: true` always —
        // an icon there is drawn *beside* the label, never instead of it.
        // Only the QAT goes icon-only, and only these four ids are on it:
        // `file.open`, `file.save_copy`, `edit.undo`, `edit.redo`. So an icon
        // on a band control costs the word nothing; the choice was never
        // "glyph or findable name", and reasoning as though it were produced
        // a group of three bare words in a row of pictures.
        //
        // `doc.pages`, like the rest of the Display group: a ruler with no
        // page to measure and a grid with no paper to rule are both chrome
        // about nothing.
        //
        // The tokens are contiguous (232-234) because they are one row of the
        // specification.
        command("view.rulers", t::view_rulers(), 232)
            .with_icon("rulers")
            .enabled_when("doc.pages"),
        command("view.grid", t::view_grid(), 233)
            .with_icon("grid")
            .enabled_when("doc.pages"),
        command("view.guides", t::view_guides(), 234)
            .with_icon("guides")
            .enabled_when("doc.pages"),
        // The sidebar is the application's own furniture and toggles with
        // or without a document; the panels inside it need one to describe.
        // ★★★ `view.sidebar` was HERE until 2026-08-31 — O68's sweep.
        //
        // There is no sidebar rail in this build; there is a dock, and every
        // dock panel already has its own command. So the control had nothing
        // behind it and never would have: it was not deferred work, it was a
        // command for a concept this shell does not have. Drawn FIRST in
        // View ▸ Panels and enabled with nothing open, which made it the most
        // prominent dead control in the program.
        //
        // R8: absence is expressed by not registering. R9: it renders nothing.
        // ★★★ **`view.panel_tool` was retired on 2026-09-04** —
        // `OPERATOR_REQUESTS.md` O123, and token 247 goes with it and is never
        // reused.
        //
        // The panel it toggled no longer exists: its status is a permanent
        // strip the right dock reserves (`crate::app::toolstatus`), its live
        // controls are in `crate::panels::properties::tool`, and its
        // disclosure block is in `crate::panels::properties::disclose`. A
        // toggle for a surface that is always drawn would be a control with
        // nothing to toggle, which R9 forbids.
        //
        // ★ Recorded here rather than deleted silently, because the argument
        // for registering it was recorded here at length and a reader finding
        // a gap between 246 and 248 deserves to know it was an answer to a
        // real complaint rather than an oversight.
        command("view.panel_bookmarks", t::view_panel_bookmarks(), 241)
            .with_icon("bookmarks")
            .enabled_when("doc.open"),
        // ★ **This carried a recorded "no icon" decision, and the decision
        // has expired.** It read: "There is no `document` (or `pages`) key in
        // `crate::icons::catalog`, and naming one would draw the catalogue's
        // deliberate visible slashed mark for an unknown key on a control an
        // operator uses constantly. A command with no icon renders as its
        // label, which is a real answer and the right one here — the panel's
        // name is a word, and the word is what makes it findable."
        //
        // Both halves have been overtaken:
        //
        // * The premise is gone. `pages` was authored on 2026-08-14 (three
        //   sheets, front one whole and two behind showing only the edges
        //   that clear it — `crate::icons::Icon::Pages` records what it was
        //   drawn to stay distinguishable from). Naming it draws a glyph.
        // * The fallback argument was a misreading of the ribbon, the same
        //   one the Display group's note made: a band control draws its icon
        //   BESIDE its label, never instead of it. Nothing about the word was
        //   ever at stake.
        //
        // Left standing, that comment would have read as a live reason to
        // keep a bare button in a row of glyphs. A decision whose premise has
        // been removed is not a decision any more.
        //
        // `doc.open`, not `doc.pages`, unlike every other entry in this group:
        // the Pages panel's own body handles a `/Count 0` document and says so,
        // which is more useful than a greyed toggle that cannot explain why a
        // legal PDF has no pages.
        command("view.panel_pages", t::view_panel_pages(), 245)
            .with_icon("pages")
            .enabled_when("doc.open"),
        command("view.panel_layers", t::view_panel_layers(), 242)
            .with_icon("layers")
            .enabled_when("doc.open"),
        command("view.panel_signatures", t::view_panel_signatures(), 243)
            .with_icon("signatures")
            .enabled_when("doc.open"),
        command("view.panel_objects", t::view_panel_objects(), 244)
            .with_icon("edit-objects")
            .enabled_when("doc.pages"),
        // ★ Was `edit.form_fill`, token 430, until the operator answered the
        // question `crate::app::modes` had been carrying: Read should fill
        // forms, because that is what Acrobat Reader does in its default
        // view. Read is shown `file` and `view` alone, and P1 gives a
        // command exactly one tab — so the verb moved to a tab Read has,
        // which meant a new id and a token in the `view.` block.
        //
        // It keeps `doc.pages` rather than `doc.open`: an AcroForm's fields
        // carry page-relative rectangles, so a document with no pages has
        // nowhere for a field to be.
        //
        // `forms` is a page carrying two input boxes — not `form-field`,
        // which makes a field and belongs to Edit. That distinction is the
        // same line this command's placement draws: filling is not authoring.
        // It does not contradict the icon ui-spec §8.14's "no dedicated
        // toolbar icon" for form filling either; that ruling is about there
        // being no fill TOOL to arm, and this is a panel toggle.
        command("view.panel_forms", t::view_panel_forms(), 246)
            .with_icon("forms")
            .enabled_when("doc.pages"),
        // Read mode and full screen are the two commands `RIBBON_IA.md` §3
        // named as having "no ribbon control at all" on a tab literally
        // called View. They have controls now, so they have glyphs.
        command("view.read_mode", t::view_read_mode(), 250).with_icon("read-mode"),
        command("view.fullscreen", t::view_fullscreen(), 251).with_icon("fullscreen"),
        // ★ `view.floating_panels` (252) and `view.app_initiative` (253) were
        // UNREGISTERED on 2026-08-17, tokens retired rather than reused.
        //
        // Neither had anything behind it. `egui-shell`'s dock has no floating
        // mode at all — its only `floating` is `egui`'s scroll-bar style — so
        // the first governed a capability that does not exist.
        //
        // The second is the more interesting deletion, and worth keeping the
        // reasoning for. `view.app_initiative` was a three-position policy —
        // Never · Ask · Allowed — about whether pdfcer may float a surface over
        // the page **on its own initiative**. Its specified default was
        // **Never**, and *nothing in this build does that*: the default is
        // already true by construction. So the control existed to switch off a
        // behaviour pdfcer does not have, which is a control that cannot do
        // anything whichever way it is set.
        //
        // Building it would mean building the behaviour first, and the
        // behaviour is the thing the operator objected to. It goes back on the
        // list the day something wants to float unasked, and not before.
        command("view.reset_layout", t::view_reset_layout(), 254).with_icon("reset-layout"),
        // ★ **The two document-switching verbs**, registered 2026-08-19 with
        // the document tab strip.
        //
        // `enabled_when("docs.multiple")` and not `doc.open`: with one document
        // open there is nothing to switch to, and R9 reserves greying for
        // *temporarily* unavailable — which this is, exactly. Opening a second
        // document arms both, and the hover says what they would do.
        //
        // They exist as commands rather than as bare keyboard handling because
        // `R8` allows no other way for the shell to learn a capability is
        // present: the chords in the manifest resolve against this registry,
        // so a build without them would have Ctrl+Tab bound to nothing rather
        // than bound to something that silently does nothing.
        // ★★ **The chevron borrow ended 2026-09-04**, with art adopted from the
        // outside review of 2026-09-03, and this is the borrow whose cost is
        // easiest to state in one sentence: **"previous document" and "previous
        // page" drew the same picture.** `chevron-left` and `chevron-right` are
        // the PAGE navigation glyphs — the status bar's page stepper wears them,
        // and they are documented on their variants as *"Previous page"* and
        // *"Next page"* — so the two verbs an operator most needs to keep apart,
        // move within this file and move to another file, were one tile each.
        // Pressing the wrong one does not lose work, but it does lose your
        // place, and the operator has no way to tell in advance which it will
        // be.
        //
        // The borrow was also against a WRITTEN reservation rather than merely
        // against taste: `chevron-left.svg`'s own note keeps the bare
        // two-segment chevron for a STEP through a sequence. Switching documents
        // is a JUMP between files, and `back.svg` had already settled what a
        // jump gets — *"straight-with-shaft is the untaken slot"*. So the answer
        // was reserved in the set before these commands existed, and the borrow
        // spent a slot that was being held for exactly this.
        //
        // A page with a shafted arrow beside it. **The PAGE says what moves** —
        // a whole document, not a position within one — **and the SHAFT says how
        // far.** Distinct from [`crate::icons::Icon::Back`], the set's other
        // shafted left arrow, by that page: Back leaves a surface with nothing
        // else in frame, this one carries the thing being switched to.
        //
        // ★ The two are the SAME drawing mirrored about x=24, and that is the
        // convention every navigation pair in this set follows —
        // `chevron-right.svg`'s entire comment is "mirror of chevron-left.svg",
        // and `upload.svg`/`download.svg` are described as exact mirrors about
        // y=24. Mirror; never redraw the second one by hand.
        command("view.next_document", t::view_next_document(), 255)
            .with_icon("document-next")
            .enabled_when("docs.multiple"),
        command("view.previous_document", t::view_previous_document(), 256)
            .with_icon("document-previous")
            .enabled_when("docs.multiple"),
        // ★ **Close others**, 2026-08-20, with the document tab strip.
        //
        // Its operand depends on the route: from a tab's context menu it keeps
        // the tab that was right-clicked, from the ribbon it keeps the one on
        // screen. `crate::app::PdfcerApp::tab_menu_target` is how the first is
        // supplied and `unwrap_or(active_slot)` is how the second falls back —
        // and the tooltip says *"the one you opened this on"* rather than
        // naming either, because that sentence is true from both routes.
        //
        // ★ There is deliberately **no** `close_document` beside it. The
        // conventional tab menu has three rows — Close, Close others, Close to
        // the right — and a Close here would be a second command with
        // `file.close`'s label and `file.close`'s behaviour from the ribbon,
        // differing only in a parked operand. `no_two_commands_share_a_label`
        // and `every_menu_command_is_also_reachable_from_the_ribbon` both
        // caught the attempt, and between them they are right: closing the tab
        // you right-clicked is already the ✕ on that tab and a middle click on
        // it, which are the two gestures every operator reaches for first.
        //
        // # The icon refusal that stood here is DISCHARGED IN HALF — 2026-09-04
        //
        // It read: *"No icon. `catalog`'s coverage table calls a context-menu
        // row's glyph decoration: a menu is a list of words, read rather than
        // scanned, and a half-iconed menu is worse than none."*
        //
        // **That reasoning is still correct about the menu**, and it is kept
        // here rather than deleted for that reason. A tab's context menu is read
        // top to bottom as a short list of sentences; glyphs down its left edge
        // are decoration, and a menu where some rows have one and some do not is
        // worse than a menu where none do.
        //
        // ★ **What the refusal missed is that this command is on the ribbon
        // too.** `manifest::view` places it on View ▸ Window, between two iconed
        // neighbours, and it is there because
        // `every_menu_command_is_also_reachable_from_the_ribbon` holds that a
        // right-click-only command is undiscoverable. On a ribbon the un-iconed
        // row is the odd one out — the tile is blank where every tile beside it
        // carries a mark — which is the *same* defect the refusal was trying to
        // avoid, arrived at from the other side. The note was reasoning about
        // one of this command's two surfaces and generalising to both.
        //
        // ⇒ Recorded rather than quietly reversed: **a reason that was true of
        // the surface it was written about, and was applied to a surface nobody
        // re-checked.**
        //
        // Two square-on sheets: a complete one in front, untouched, and behind
        // it a second carrying a small ✕ at its top-right. The front sheet is
        // the one you keep; the mark is on the others. Which document is kept
        // depends on the route — the right-clicked tab, or the one on screen —
        // and the glyph says only "this one stays, those go", which is true from
        // both, exactly as the tooltip's *"the one you opened this on"* is.
        //
        // ★ **The distinction from [`crate::icons::Icon::Close`] is load-bearing
        // and getting it backwards closes the wrong documents.** `close` is a
        // bare full-frame ✕ and means *dismiss the thing in front of you*; this
        // means the opposite — the thing in front of you is the survivor. Scale
        // and placement carry that entirely: small mark, on the sheet BEHIND.
        // Distinct from [`crate::icons::Icon::Copy`], also two offset rects, by
        // the ✕, and from [`crate::icons::Icon::SaveCopy`] by having no shutter.
        command(
            "view.close_other_documents",
            t::view_close_other_documents(),
            257,
        )
        .with_icon("close-others")
        .enabled_when("docs.multiple"),
    ]
}
