//! # `shell::commands::catalog::format` — the Format contextual tab — what changes about the selection
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
        // The tab is visible when `selection.any` and the command inside it
        // is enabled by the same condition. That is not redundant: the tab
        // and its contents are evaluated independently, and a Format tab
        // that appeared with a greyed Delete would be the placeholder P3
        // forbids, arriving through a mismatch rather than a decision.
        // ===================================================================
        // ★★ `selection.actionable`, not `selection.any`, since 2026-08-28.
        // Both commands can act on a selected FORM FIELD, which is not in
        // `SelectionState` — see `app::conditions` for why that is a second
        // condition rather than a widening of the first.
        command("format.delete", t::format_delete(), 800)
            .with_icon("delete")
            .enabled_when("selection.actionable"),
        // ★ A second ROUTE to `file.properties`, not a second command that
        // opens the panel. Its arm raises `Action::Command("file.properties")`,
        // which is the mechanism that keeps one command's guards in one place.
        //
        // Registered as its own id because the shell enforces one command, one
        // tab — and the two placements answer different questions: File ▸
        // Document is "tell me about this file", Format is "tell me about the
        // thing I just clicked".
        //
        // The icon is `properties`, shared with `file.properties` under the
        // header's shared-key convention: same panel, same glyph, and the two
        // are never drawn together because Format is contextual.
        command("format.properties", t::format_properties(), 801)
            .with_icon("properties")
            .enabled_when("selection.actionable"),
        // ★★ **Greyed, not absent, when the selection is not inside a form.**
        //
        // R9 draws the line by *why* a thing is unavailable: a capability this
        // build does not have renders nothing at all, and a capability that is
        // **temporarily** unavailable is greyed with the reason on hover. Which
        // one this is turns on a fact about the operator's current selection,
        // not about the build — the command works, on the very next click, on
        // any document with a form. That is the greying case, and it reads the
        // same way `format.delete`'s `selection.any` does one line above.
        //
        // ★ And a greyed control is a hint, never an enforcement. `enabled_when`
        // greys a ribbon item and stops nothing: every non-ribbon route — the
        // context menu, a chord, a future script — reaches the dispatcher
        // without consulting it. The arm in `app::dispatch` therefore asks the
        // same question again and *says why* when the answer is no, which is
        // the ruling this project made after a blanket dispatcher guard was
        // written and two tests refused it for making `Ctrl+Z` on an empty
        // stack do nothing and say nothing.
        //
        // ★ The glyph is `pick-form-xobject`, **reused** rather than new, under
        // the same shared-key convention `format.properties` uses one line
        // above. It is not a near-miss reuse of the kind the header's refusal
        // table is full of: that glyph's entire subject *is* a form XObject —
        // it is the pick filter's form class — and this command's entire
        // subject is a form XObject. Two controls about one thing, drawn the
        // same, is the convention working rather than an economy.
        //
        // Drawing new art was never the alternative. `icons/assets/PROVENANCE.md`
        // declares that directory the **operator's own art**, which is what
        // exempts it from `check-shipped-assets`, and a machine-drawn SVG would
        // make that provenance note false — `file.ocr`'s refusal argues it at
        // length and the argument is unchanged here.
        command("format.select_form", t::format_select_form(), 802)
            .with_icon("pick-form-xobject")
            .enabled_when("selection.in_form"),
        // ★★★ **The "option" half of decision 076**, registered 2026-08-28
        // after `EDITABLE_SURFACES.md` found `EditSession::unshare_form`
        // implemented in the engine and named nowhere in this crate.
        //
        // `RIBBON_IA.md` §5.8 is what puts it here. That section's table gives
        // the **Vector object** row as `Stroke · Fill · Winding rule · Node
        // tools · Delete` and describes the tab's job in one line: it *"carries
        // what a user changes **while working**"*, as against the Properties
        // panel's complete property set. Giving a page its own copy of a shared
        // drawing is exactly a mid-gesture act — it is what an operator does in
        // the second between noticing a typo in the title block and typing over
        // it — so it belongs on the tab rather than in the panel, and it
        // belongs in the **selection** group beside the two commands that are
        // also about *the thing you just clicked and what encloses it*.
        //
        // It is not in §5.8's table, because that table was written on
        // 2026-08-12 and this verb did not exist in the engine until this
        // month. §5.8's amendment convention is followed: the placement is
        // argued from the section's stated principle rather than from a row.
        //
        // ★★ **`selection.in_form`, the same predicate as `select_form` one
        // line above, and that is the correct answer rather than a convenient
        // one.** `app::conditions` publishes it as *"something selected on this
        // page lives inside a form XObject"*, and this verb's operand is
        // derived from a **leaf** — the outermost form enclosing it. The
        // condition is therefore literally the question "is there an operand?",
        // asked in the one place both controls read it from.
        //
        // ⇒ It is also why the two commands must not be collapsed into one that
        // selects-and-unshares: after `format.select_form` there is no leaf any
        // more, so the predicate is false and this control correctly greys.
        // Two acts, two conditions, both honest about what they need.
        //
        // ★ Greyed rather than absent, on `format.select_form`'s R9 reading
        // exactly: the capability is present in this build and on this
        // document, and what is missing is the operand, which the next click
        // supplies. The tooltip explains it on hover, which is the half of R9
        // that makes greying legitimate rather than lazy.
        //
        // ★ The glyph is `pick-form-xobject`, shared with `format.select_form`
        // under the header's shared-key convention. The two commands are about
        // one structure — a form XObject — and a family sharing a glyph is how
        // a ribbon reads as grouped. Drawing new art was never the alternative:
        // `icons/assets/PROVENANCE.md` declares that directory the operator's
        // own art, and a machine-drawn SVG would make that note false.
        command("format.unshare_form", t::format_unshare_form(), 808)
            .with_icon("pick-form-xobject")
            .enabled_when("selection.in_form"),
        // -------------------------------------------------------------------
        // The Font group — `RIBBON_IA.md` §5.8's "Text run" row.
        //
        // ★★★ **All five are `enabled_when("selection.text")` and NOT
        // `selection.any`**, and getting that backwards would grey them in
        // exactly the state where they work.
        //
        // `EditSession::format_text` locates its operand by a pinned byte span
        // into a decoded content buffer, keyed on a **run** of the page's text
        // extraction. `selection.any` is the *object* selection — a paint-order
        // index — and nothing in either crate maps between the two index
        // spaces. So the swept range is the operand, and the swept range is
        // what `selection.text` reports.
        //
        // ★★ **Greyed rather than absent when there is no sweep**, which is R9
        // read carefully. The capability is present — this build has
        // `format_text`, this mode may edit content, this document is open —
        // and what is missing is the *operand*, which the next gesture
        // supplies. That is the textbook temporarily-unavailable case, it is
        // greyed, and it is explained on hover. **The explanation is the whole
        // point**: `text::commands`' own note above these five records why
        // each tooltip has to name the route to an operand, and it is the
        // surface that answers O37's *"nothing on screen tells you to press
        // T"*.
        //
        // Their **absence** is a different rule and lives in the manifest:
        // every item of the group carries `visible_when: "mode.edit_content"`,
        // so Read and Review — which cannot change page content at all — draw
        // no Font group rather than five permanently greyed controls.
        //
        // ★ Three of the five are drawn by an `Item::Custom` and have no
        // button of their own: a face chooser, a size field and a colour
        // swatch are not buttons. They are registered anyway, because a
        // registered command is how this shell learns a capability exists
        // (R8), because the a11y name and the reachability check both read the
        // registry, and because the custom renderer draws the registered label
        // rather than a second copy of it. See `manifest::CUSTOM_BACKED`.
        //
        // ★★★ **The refusal that covered all five was CORRECTED on 2026-09-04,
        // and "corrected" is the load-bearing word — it was not discharged.**
        //
        // What stood here from 2026-08-27 until then, verbatim:
        //
        // > **No icons on any of the five.** Word draws `B` and `I` as glyphs
        // > and this build has no such art; `icons/assets/PROVENANCE.md`
        // > declares that directory the operator's own work, which is what
        // > exempts it from `check-shipped-assets`, and a machine-drawn
        // > substitute would make that note false. Without an icon a `Small`
        // > item resolves to `Medium` (`egui_shell::ribbon::sizing::resolved`),
        // > so the labels are what render — "Bold" and "Italic", which are
        // > unambiguous where a home-made glyph would not be.
        //
        // ⇒ **That paragraph was two arguments wearing one sentence**, and only
        // one of them was ever the operator's. Splitting them is the whole
        // content of this pass:
        //
        // * *"this build has no such art"* is a statement about **supply**, and
        //   the operator has ruled on supply — repeatedly, and by name here. The
        //   standing ruling of 2026-08-06 is carried in `Icon::Back`'s doc
        //   comment: a missing glyph is **AUTHORED**, not worked around, because
        //   working around it *"spends the operator's affordance to protect the
        //   font stack; an icon costs one asset and keeps both."* On 2026-09-04
        //   he quoted it back at this very pair: *"if bold and italics have no
        //   art in the set, why weren't they made automatically as I have
        //   instructed to be done for anything that a glyph is missing for on
        //   multiple occasions?"* So the supply half is **corrected**, and the
        //   two commands below name `bold` and `italic`.
        // * *"a machine-drawn substitute would make PROVENANCE.md false"* was
        //   never a claim about supply and it is **untouched**. It is answered
        //   the way the 2026-09-04 icon batch answered it for eleven other
        //   refusals: the asking happened, and the art is drawn in the §3 style
        //   contract with its ruling embedded, exactly as
        //   `icons/assets/PROVENANCE.md` requires of every future asset.
        //
        // ★★ This is **not a new capability**. `format.bold` and
        // `format.italic` did on 2026-08-26 precisely what they do now; what
        // changed is that a control which was bare because nobody had drawn its
        // picture stopped being bare. Recording that distinction is the point of
        // the pass — see the coverage counters in `super::super`, which move as a
        // correction and say so.
        //
        // ★ The general lesson, and it is the second time in three days this
        // project has paid for it (`edit.select_all` was the first): **a refusal
        // whose reason is "no art exists" has an expiry date, and quoting it
        // does not make it the operator's ruling.** A refusal that names a WRONG
        // PICTURE — `view.zoom_actual`, argued against by name in the icon
        // ui-spec §3.2 — has no expiry date at all. The two look identical in a
        // coverage table and are opposites.
        //
        // ★★ **The other three still refuse, and for a reason that is NOT about
        // supply**, which is why they are not corrected alongside their two
        // neighbours. A face chooser, a size field and a colour swatch are drawn
        // by an `Item::Custom` — an `egui::ComboBox`, an `egui::DragValue` and
        // `Ui::color_edit_button_srgb` (see `app::fontband`) — and none of those
        // widgets has an icon slot to draw into. The refusal is **structural**:
        // there is nowhere to put a glyph, not nowhere to get one. Word agrees on
        // the first two and its own font-name and size boxes carry no icon; the
        // swatch's entire face IS the colour, and a glyph over it would cover the
        // one thing the control exists to report.
        command("format.font", t::format_font(), 803).enabled_when("selection.text"),
        command("format.font_size", t::format_font_size(), 804).enabled_when("selection.text"),
        // ★ `bold` — a capital B stroked at 4 rather than the set's 2.5, so the
        // picture says HEAVIER, which is the thing the label cannot. The asset's
        // own comment carries the weight argument and the two axes that keep it
        // apart from its neighbour below. Closest neighbour in the whole set at
        // 16 px: `page-single`, at 0.471 — against a floor of 0.15 and a set
        // minimum of 0.211.
        command("format.bold", t::format_bold(), 805)
            .with_icon("bold")
            .enabled_when("selection.text"),
        // ★ `italic` — a slanted capital I with OFFSET serifs, which is the cue
        // that keeps it clear of `text-select`'s bare centred I-beam (0.737 at
        // 16 px; its closest neighbour anywhere is `measure-angle` at 0.719, and
        // the pair `bold ~ italic` measures 0.820). The slant is exaggerated to
        // about 28° on purpose — see the asset for why a typographically honest
        // 12° reads as a rendering bug at 16 px.
        command("format.italic", t::format_italic(), 806)
            .with_icon("italic")
            .enabled_when("selection.text"),
        command("format.font_colour", t::format_font_colour(), 807).enabled_when("selection.text"),
        // -------------------------------------------------------------------
        // The Markup group — `RIBBON_IA.md` §5.8's "Markup annotation" row,
        // registered 2026-09-06 on the operator's *"getting full editing
        // working for the Markup tools."*
        //
        // ★★★ **The five reasons this row was `PLANNED` were all one claim, and
        // it had been false for eighteen days.** `manifest::format`'s header
        // said `EditSession` had no verb that modified an annotation and that
        // the canvas selection could not address one.
        // `EditSession::set_markup_style` shipped 2026-08-18;
        // `canvas::selection::annot::AnnotTarget` landed the same day; and
        // `panels::properties::markup` has raised `Action::SetMarkupStyle`
        // since 2026-08-19. The header is corrected there, at length, because
        // the mistake is worth more than the correction: **a blocker is a
        // measurement with a date**, and this one outlived what made it true by
        // the whole of the feature's cost.
        //
        // ★★ **All five are `enabled_when(MARKUP_RESTYLABLE)`, which is the
        // SAME condition the manifest gives them as `shown_when`** — deliberate,
        // and this file's header already argues that the duplication is not
        // redundant: the tab and its contents are evaluated independently, and
        // a Format tab that appeared holding five greyed controls would be the
        // placeholder P3 forbids, arriving through a mismatch rather than a
        // decision. Here it is stronger than that. The condition IS the
        // question "is there an operand of the right kind, in a mode that may
        // change it?", so a state where the item is drawn and the command is
        // greyed cannot exist by construction — which is exactly what should be
        // true of a group whose absence is the whole R9 story.
        //
        // ⇒ The one greyed state these controls do have is a **locked**
        // annotation (§12.5.3 Table 165 bit 8), and it is deliberately NOT in
        // this predicate. `enabled_when` greys with the command's own tooltip,
        // and the honest sentence for a locked mark is
        // `text::panels::properties::markup_locked` — the same string the
        // Properties panel shows, so the two surfaces cannot refuse for
        // different reasons. `app::markupband` draws that greying itself, which
        // it has to anyway: the shell evaluates no predicate, draws no greying
        // and shows no tooltip for an `Item::Custom`.
        //
        // ★ **No icons on any of the five, and the refusal is STRUCTURAL rather
        // than about supply** — which is the distinction this project has now
        // paid for twice (`edit.select_all`, then `format.bold`/`format.italic`
        // on 2026-09-04). A refusal reading "no art exists" has an expiry date
        // and the operator's standing ruling is that the art gets **drawn**.
        // This is the other kind: two colour swatches, two drag fields and a
        // combo box are drawn by `Ui::color_edit_button_srgb`,
        // `egui::DragValue` and `egui::ComboBox`, and **none of those widgets
        // has an icon slot**. There is nowhere to put a glyph, not nowhere to
        // get one, and no amount of drawing touches it. The three Font controls
        // beside them refuse on the identical ground.
        //
        // ★ And a swatch's entire face IS the colour it reports; a glyph over
        // it would cover the one thing the control exists to say.
        command("format.colour", t::format_colour(), 809).enabled_when(MARKUP_RESTYLABLE),
        command("format.fill", t::format_fill(), 810).enabled_when(MARKUP_RESTYLABLE),
        command("format.line_width", t::format_line_width(), 811).enabled_when(MARKUP_RESTYLABLE),
        command("format.opacity", t::format_opacity(), 812).enabled_when(MARKUP_RESTYLABLE),
        // ★ The eighth control of `RIBBON_IA.md` §5.8's Markup row, and the last
        // to arrive: it read **⛔ no engine verb exists** until `MarkupStyle::dash`
        // shipped on the afternoon of 2026-09-06. No icon, for the structural
        // reason above — a `ComboBox` has no icon slot — and `enabled_when` for
        // the same predicate as the five beside it, because a line style is a
        // markup restyle like every other.
        command("format.line_style", t::format_line_style(), 814).enabled_when(MARKUP_RESTYLABLE),
        command("format.arrowheads", t::format_arrowheads(), 813).enabled_when(MARKUP_RESTYLABLE),
    ]
}

/// **A markup annotation is selected, and this mode may author markup.**
///
/// Spelled once here because five registrations read it, and spelled as a
/// constant rather than as five literals for [`crate::shell::commands::FILE_RECENT`]'s
/// reason: a typo in one of five copies produces a permanently greyed control
/// and no error at all, because an unset condition and a false condition are
/// the same value.
///
/// ★ It is **not** shared with `manifest::format`'s `MARKUP_VISIBLE_WHEN`,
/// which holds the same string. That is the same deliberate de-aliasing
/// `manifest::SELECTION_ANY` records the cost of: while `SELECTION_ANY` read
/// `= format::VISIBLE_WHEN`, editing the Format tab's condition would have
/// silently retargeted the canvas context menu's Delete. Two readers, two
/// spellings, and the manifest side carries the full account of what the
/// condition means.
const MARKUP_RESTYLABLE: &str = "selection.markup_restylable"; // ui-text-exempt: a condition name, never displayed
