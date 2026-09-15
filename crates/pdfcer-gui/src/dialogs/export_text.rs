//! # `dialogs::export_text` — the words on the page, in a file anything can
//! read
//!
//!
//! > *"also the engine can export PDFs as text. we should have export/import
//! > for that."*
//!
//!
//! It read:
//!
//! > *"There is no import, and this window says nothing about one. The ask
//! > names two halves and only one of them exists. `pdfcer-core` offers no
//! > route from a text file back into a PDF — not a document builder, not a
//! > 'replace this page's text' verb, and not a way to hand `add_ocr_layer` a
//! > file instead of a recogniser's positioned words. … **No control was drawn
//! > that declines when pressed**, and nothing in this window implies a round
//! > trip. R9: a placeholder is worse than an absence, because an absence is
//! > honest and a placeholder is a promise."*
//!
//!
//! ⇒ [`crate::dialogs::import_text`] is the window R9 forbade drawing until
//! there was something behind it. **This window still says nothing about the
//! round trip**, and that is now a wording decision rather than an honesty one:
//! a window's job is its own act, and the pair is expressed by the two commands
//! sitting next to each other on File ▸ Export — which is how
//! `export_form_data` and `import_form_data` already say it.
//!
//! ## ★★ The default writes the CLIPBOARD's own bytes
//!
//!
//! Every control that departs from it — the page-marker separator, Windows line
//! endings, the byte-order mark — is **opt-in**, and every one of them is named
//! in the receipt afterwards. Two answers to *"what is the text of this
//! document"* in one program is worse than either, because both look like text.
//!
//! ## ★★★ What this window has to say before the press, and what it cannot
//!
//! The image export's rule: *"everything that could make this export empty has
//! already been said in the window."* That rule **cannot be met here**, and
//! saying so is the design.
//!
//! The thing that makes a text export empty is that the page is a *picture* of
//! its words — a scan, or a plot from a system that outlined its text. Nothing
//! about that is knowable from a page count, a page size or anything else this
//! window holds, and computing it would mean extracting the document to draw a
//! window that offers to extract the document.
//!
//! ⇒ So this window states the **standing** losses (layout, derived breaks,
//! style) where an operator can weigh them before pressing, and the *counted*
//! ones — the empty pages, the unreadable fonts, the scan refusal — arrive
//! afterwards, off-canvas, from `app::actions::export::text`. That is the same
//! two-part shape [`crate::text::export_text`]'s header sets out, and it is why
//! the losses are said twice in two different registers rather than once.
//!
//! ## ★ The page scope is `imageexport`'s, called rather than copied
//!
//! [`crate::app::actions::imageexport::PageScope`] and `resolve_pages`, which in
//! turn call `crate::dialogs::print::tabs::parse_page_range` — the print
//! dialog's parser, which the OCR window and the Insert-pages window already
//! share. **Five surfaces, one answer** to *"is `1,1` two exports of page one?"*
//! and *"does `5-3` mean anything?"*, and an operator who learned the syntax on
//! Print is entitled to it here.
//!
//! ⇒ The module it lives in is now misnamed — `imageexport` owns the page-scope
//! type for a text export — and that is **stated rather than fixed**. Moving it
//! is a rename across a file another track is editing, made in the same pass as
//! a new feature, which is exactly the diff `RIBBON_IA.md` records as
//! unreviewable. The trigger is armed: the next surface to need a page scope is
//! the one that moves it to a module named for what it is.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::exporttext::{LineEndings, PageSeparator, TextExportPlan};
use crate::app::actions::imageexport::{PageScope, resolve_pages};
use crate::app::state::{OpenDoc, Status};
use crate::text::export_text as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:export-text"; // ui-text-exempt: trace region name, never displayed
/// ★★ The region ONE page-scope radio publishes.
///
/// ★ These exist for `OPERATOR_REQUESTS.md` **O196**. Until the export windows
/// remembered anything, a driven check had nothing to assert about a radio
/// beyond *"it is drawn"*; now the question is which one is **selected on
/// open**, and that cannot be asked of a group rectangle.
///
/// The alternative — pressing the group's rectangle plus an offset — is the
/// check that presses the wrong control the day a hint gains a line, which
/// `dialogs::export_image::region_for_format` states in full.
#[must_use]
pub const fn region_for_scope(scope: PageScope) -> &'static str {
    match scope {
        // ui-text-exempt: trace region names, matched by tools/ui-verify and
        // never displayed.
        PageScope::AllPages => "export-text.pages.all",
        PageScope::CurrentPage => "export-text.pages.current",
        PageScope::Typed => "export-text.pages.typed",
    }
}
/// The region the page-scope radio GROUP publishes — all three together, plus
/// the range box.
pub const REGION_PAGES: &str = "export-text.pages"; // ui-text-exempt: trace region name, never displayed
/// ★★ The region ONE separator radio publishes. Same argument as
/// [`region_for_scope`], and the same reason: **which one is ticked on open**
/// is now a question worth asking.
#[must_use]
pub const fn region_for_separator(separator: PageSeparator) -> &'static str {
    match separator {
        // ui-text-exempt: trace region names, matched by tools/ui-verify and
        // never displayed.
        PageSeparator::FormFeed => "export-text.separator.form-feed",
        PageSeparator::Marker => "export-text.separator.marker",
    }
}
/// The region the Windows-line-endings checkbox publishes.
pub const REGION_LINE_ENDINGS: &str = "export-text.line-endings"; // ui-text-exempt: trace region name, never displayed
/// The region the byte-order-mark checkbox publishes.
pub const REGION_BOM: &str = "export-text.bom"; // ui-text-exempt: trace region name, never displayed
/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-text.export"; // ui-text-exempt: trace region name, never displayed

/// The Export-text window's live state.
pub struct ExportTextDialog {
    /// The page on screen when the window opened.
    ///
    /// **Frozen at open**, for the reason every page-scoped window here freezes
    /// it: an operator who opens this on page 7 and pages away behind it must
    /// not export page 9. The radio label says which page, so the choice stays
    /// checkable.
    page_index: usize,
    /// How many pages the document has, frozen with the above and for the same
    /// reason.
    page_count: usize,
    /// Which pages.
    ///
    /// # ★ Why this window opens on **every page** where the image window
    /// opens on *this page only*
    ///
    /// The shipped value lives in [`crate::app::prefs::ExportTextPrefs`] as of
    /// **O196** and the operator can now change it by exporting; the *argument*
    /// stays here, because it is an argument about this window's verb and the
    /// preferences file cites this file for it by name.
    ///
    /// The two verbs are asked for in different moods. A picture of a page is a
    /// picture of *a* page — fifty PNGs is a directory nobody wanted, and the
    /// image window's own multi-file naming rule exists because that scope is
    /// the unusual one. The text of a document is asked for to search it, to
    /// diff it, or to paste it into a specification, and all three want the
    /// whole thing. The verb one group over that already answers *"the text of
    /// THIS page"* is `file.copy_page_text`, and it is one keystroke.
    scope: PageScope,
    /// The typed range, kept across scope changes so switching to **Every
    /// page** and back does not lose what was typed.
    ///
    /// ⚠ **Not remembered between jobs**, and that is O196's one deliberate
    /// omission here: a typed range is a statement about *this document's* page
    /// numbering. See [`Self::open`].
    range_text: String,
    /// What goes between one page and the next.
    ///
    /// ★ The shipped default is the engine's own separator — see
    /// `PageSeparator::FormFeed`. Defaulting to the marker would make the
    /// departure from the clipboard's bytes the thing an operator has to notice
    /// and undo, and the argument for that is now on
    /// [`crate::app::prefs::ExportTextPrefs`] with the value it argues for.
    separator: PageSeparator,
    /// How lines end.
    line_endings: LineEndings,
    /// Whether the file opens with a UTF-8 byte-order mark.
    byte_order_mark: bool,
    /// Set by Export, consumed after the window's closure returns.
    export_requested: bool,
    /// Set by Cancel, consumed by [`Self::show`].
    close_requested: bool,
}

impl ExportTextDialog {
    /// Open the window for the document on screen, seeded from what the last
    /// text export asked for.
    ///
    ///
    /// > *"the export windows forget everything. every time I export a dxf I
    /// > have to set it up again."*
    ///
    /// All four of this window's answers were literals until that day. The
    /// membership rule — **a setting is remembered only if it would still be
    /// the right answer for a different document** — and the argument for the
    /// shipped value of each one live on
    /// [`crate::app::prefs::ExportTextPrefs`]. Read that first; this is only
    /// the seeding.
    ///
    /// ⚠ `range_text` is NOT seeded and is not a preference. A typed range is
    /// a statement about *this document's* page numbering, and restoring
    /// `12-40` onto a nine-page file would open the window in a state whose
    /// Export button is already dead for a reason the operator did not cause.
    #[must_use]
    pub fn open(doc: &OpenDoc, remembered: &crate::app::prefs::ExportTextPrefs) -> Self {
        let page_index = doc.view.page_index;
        let page_count = doc.pages.len();
        let dialog = Self {
            page_index,
            page_count,
            scope: remembered.scope,
            // Deliberately empty — see the note on this function.
            range_text: String::new(),
            separator: remembered.separator,
            line_endings: remembered.line_endings,
            byte_order_mark: remembered.byte_order_mark,
            export_requested: false,
            close_requested: false,
        };

        // ★★★ **Traced from the BUILT dialog, and the position of these lines
        // is the whole point of them.**
        //
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-text-open page={} pages={} scope={} separator={} endings={} bom={}",
                dialog.page_index,
                dialog.page_count,
                // ★ Stable lowercase tokens, never `{:?}`. This project's
                // standing lesson, and the preferences file's own `*_key`
                // functions are what produce them, so the token a check reads
                // here and the token on disk cannot drift.
                crate::app::prefs::exporting::page_scope_key_or(
                    dialog.scope,
                    crate::app::prefs::ExportTextPrefs::default().scope,
                ),
                crate::app::prefs::exporting::separator_key(dialog.separator),
                crate::app::prefs::exporting::line_endings_key(dialog.line_endings),
                u8::from(dialog.byte_order_mark),
            )
        });
        dialog
    }

    /// **This window's state, reduced to what a different document would still
    /// want** — the producing half of `OPERATOR_REQUESTS.md` **O196**.
    ///
    /// The membership rule and the argument for every inclusion and every
    /// omission live on [`crate::app::prefs::ExportTextPrefs`], which is the
    /// type this returns; this function is only the projection. It is one
    /// struct literal with **no `..Default::default()`**, so a field added to
    /// `ExportTextPrefs` is a compile error here rather than a preference
    /// written to disk as its own default and never actually remembered.
    fn habits(&self) -> crate::app::prefs::ExportTextPrefs {
        crate::app::prefs::ExportTextPrefs {
            scope: self.scope,
            separator: self.separator,
            line_endings: self.line_endings,
            byte_order_mark: self.byte_order_mark,
        }
    }

    /// Draw it. Returns `false` when it should close.
    ///
    /// Takes `&mut Prefs` for O196 alone: the Export press writes this window's
    /// habits to the preferences file before the action is pushed. See
    /// [`crate::dialogs::export_remembered`] for why it happens at the press
    /// and not at the close.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        actions: &mut Vec<Action>,
        prefs: &mut crate::app::prefs::Prefs,
    ) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "export-text", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 600.0),
            egui::vec2(360.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.export_requested)
            && let Some(plan) = self.plan()
        {
            // ★★★ O196, and the POSITION is the decision: the habits are
            // written when the operator presses Export, never when the window
            // closes. Closing without exporting is how a person says *"not
            // this"*. The argument is in
            // [`crate::dialogs::export_remembered`], stated once for all three
            // export windows.
            crate::dialogs::export_remembered::remember_text(self.habits(), prefs);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-text-requested pages={} separator={} endings={} bom={}",
                    plan.pages.len(),
                    // Tokens, never `{:?}`: the same reduction the preferences
                    // file performs, so a check reading this line and a check
                    // reading the file cannot disagree. A `{:?}` on a
                    // payload-carrying variant prints the payload too, and the
                    // failure message then quotes the truth while the assertion
                    // reports the opposite.
                    crate::app::prefs::exporting::separator_key(plan.separator),
                    crate::app::prefs::exporting::line_endings_key(plan.line_endings),
                    u8::from(plan.byte_order_mark)
                )
            });
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::Text { plan },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The pages this window is currently offering, or `None` when the typed
    /// range names none.
    ///
    /// Called twice per frame — once to grey the Export button, once on the
    /// press — and it is cheap: a parse of a short string. Computing it live is
    /// what keeps the button and the sentence beside the box from ever
    /// disagreeing.
    fn pages(&self) -> Option<Vec<usize>> {
        resolve_pages(
            self.scope,
            &self.range_text,
            self.page_count,
            self.page_index,
        )
    }

    /// The plan, or `None` when there is nothing to export.
    fn plan(&self) -> Option<TextExportPlan> {
        Some(TextExportPlan {
            pages: self.pages()?,
            separator: self.separator,
            line_endings: self.line_endings,
            byte_order_mark: self.byte_order_mark,
        })
    }

    /// The whole window body.
    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        let pages = self.pages_group(ui);
        ui.add_space(8.0);
        self.separator_group(ui);
        ui.add_space(8.0);
        self.file_group(ui);
        ui.add_space(8.0);
        self.losses_group(ui);
        ui.add_space(8.0);

        ui.separator();
        ui.horizontal(|ui| {
            // Disabled rather than absent when the typed range names no page —
            // P3's rule: a greyed control beside the sentence saying why teaches
            // what to change; a control that vanishes teaches that the window is
            // unpredictable.
            let response = ui.add_enabled(pages.is_some(), egui::Button::new(t::export_button()));
            crate::diag::ui_rect(REGION_EXPORT, response.rect);
            if response.clicked() {
                self.export_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// Which pages, and — live — whether the typed range names any.
    fn pages_group(&mut self, ui: &mut Ui) -> Option<Vec<usize>> {
        // No `.strong()` anywhere in this window — R84 / DEFECTS.md D11.
        ui.label(t::pages_heading());
        let start = ui.cursor();
        // ★ Each radio's OWN rectangle, for O196 — see [`region_for_scope`].
        let response = ui.radio_value(
            &mut self.scope,
            PageScope::AllPages,
            t::pages_all(self.page_count),
        );
        crate::diag::ui_rect(region_for_scope(PageScope::AllPages), response.rect);
        let response = ui.radio_value(
            &mut self.scope,
            PageScope::CurrentPage,
            t::pages_current(self.page_index.saturating_add(1)),
        );
        crate::diag::ui_rect(region_for_scope(PageScope::CurrentPage), response.rect);
        ui.horizontal(|ui| {
            let response = ui.radio_value(&mut self.scope, PageScope::Typed, t::pages_range());
            crate::diag::ui_rect(region_for_scope(PageScope::Typed), response.rect);
            // Typing in the box selects the radio. Without it an operator types
            // a range, presses Export and gets every page — the classic shape of
            // this control getting it wrong, and one the print dialog already
            // avoids.
            if ui.text_edit_singleline(&mut self.range_text).changed() {
                self.scope = PageScope::Typed;
            }
        });
        ui.weak(t::pages_range_hint());

        let pages = self.pages();
        // Drawn only for the typed case: the other two scopes can fail solely on
        // an empty document, which the command's own `doc.pages` predicate
        // already excludes, and a sentence explaining an unreachable state
        // trains the operator to skip the bar.
        if pages.is_none() && self.scope == PageScope::Typed {
            ui.label(t::pages_range_invalid(self.page_count));
        }
        crate::diag::ui_rect(REGION_PAGES, start.union(ui.cursor()));
        pages
    }

    /// What goes between one page and the next.
    ///
    /// Two radios rather than a checkbox, because the operator is choosing
    /// between **two things that go in the file**, one of which is the standard
    /// character and one of which is prose pdfcer wrote. A checkbox labelled
    /// "add page markers" would make the second look like a formatting
    /// preference rather than like added content, which is precisely the
    /// distinction rule 4 exists to keep visible.
    fn separator_group(&mut self, ui: &mut Ui) {
        ui.label(t::separator_heading());
        // ★ Each radio's OWN rectangle, for O196 — see
        // [`region_for_separator`].
        let response = ui.radio_value(
            &mut self.separator,
            PageSeparator::FormFeed,
            t::separator_form_feed(),
        );
        crate::diag::ui_rect(region_for_separator(PageSeparator::FormFeed), response.rect);
        ui.weak(t::separator_form_feed_hint());
        let response = ui.radio_value(
            &mut self.separator,
            PageSeparator::Marker,
            t::separator_marker(),
        );
        crate::diag::ui_rect(region_for_separator(PageSeparator::Marker), response.rect);
        // ★ Not `weak`. This is the one hint in the window that says pdfcer will
        // put words of its own into the operator's file, and a quiet grey line
        // is the thing an operator skips.
        ui.label(t::separator_marker_hint());
    }

    /// How the bytes are written.
    fn file_group(&mut self, ui: &mut Ui) {
        ui.label(t::file_heading());
        // ★ The encoding is a STATEMENT, not a control. There is no code-page
        // option and there will not be one: a CAD drawing carries degree signs
        // and diameter marks, and offering an encoding that cannot represent
        // them is offering a way to lose them silently.
        ui.weak(t::encoding_line());
        let response = ui.checkbox(&mut self.byte_order_mark, t::bom());
        crate::diag::ui_rect(REGION_BOM, response.rect);
        ui.weak(t::bom_hint());

        // Read and written through the enum rather than mirrored into a local
        // `bool` — `dialogs::export_dxf`'s rule for `DxfText`: a shadow copy is
        // how a window comes to show one thing and write another.
        let mut windows = matches!(self.line_endings, LineEndings::Windows);
        let response = ui.checkbox(&mut windows, t::line_endings_windows());
        crate::diag::ui_rect(REGION_LINE_ENDINGS, response.rect);
        if response.changed() {
            self.line_endings = if windows {
                LineEndings::Windows
            } else {
                LineEndings::AsExtracted
            };
        }
        ui.weak(t::line_endings_hint());
    }

    /// The standing losses — true of every text export of every document, and
    /// therefore sayable before the press.
    ///
    /// ★ Drawn as ordinary labels rather than `weak`, and above the buttons
    /// rather than below them. This is the paragraph that decides whether the
    /// operator should be doing this at all — a drafter who needs the title
    /// block's *layout* wants the DXF export or the image export, and this is
    /// where they find that out.
    fn losses_group(&mut self, ui: &mut Ui) {
        ui.label(t::loses_heading());
        ui.label(t::loses_layout());
        ui.label(t::loses_breaks());
        ui.label(t::loses_style());
    }
}

/// Open the window for `status`, or decline.
///
/// `doc.pages` non-empty is the same guard the command's own predicate applies,
/// asserted twice for the reason the DXF window asserts it twice: the predicate
/// greys the control and this refuses the open, and a keymap or a restored
/// layout can reach a command without going through the ribbon at all.
#[must_use]
pub fn open_for(
    status: &Status,
    remembered: &crate::app::prefs::ExportTextPrefs,
) -> Option<ExportTextDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => Some(ExportTextDialog::open(doc, remembered)),
        _ => None,
    }
}
