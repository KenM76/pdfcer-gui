//! # `dialogs::embed` — the confirmation before font programs go into a
//! document
//!
//! `tools.embed_fonts`, wired 2026-08-28. The last of the three commands the
//! font-folder work unblocked.
//!
//! ## ★★★ A window with no settings, and that is the design
//!
//! Every other dialog in this shell collects something. This one collects
//! **nothing** — there is no option to set, because the only configuration an
//! embed has is *which folders*, and that lives in Settings where it persists.
//! What is left is a question with two answers, and the whole window is the
//! evidence for answering it.
//!
//! That shape is chosen because of what an embed is. It puts font **programs**
//! — actual outlines — permanently into a document, changes its size, and can
//! bear on a PDF/A claim. A one-click ribbon verb would be dishonest about
//! that; a form would imply choices that do not exist.
//!
//! ## ★★ The preview is `embed_preview`, and it is the SAME computation the
//! commit runs
//!
//! `EditSession::embed_preview(&request)` is `&self` and side-effect-free, and
//! `embed_fonts` calls it internally before mutating — the engine's own words
//! for the shape are *"the same value is returned by the preview query and by
//! the committing call, produced by the same function, so a front end cannot
//! show one thing and do another."*
//!
//! ★ That is the property `preview_font_resources` had to be fixed to have
//! twelve hours earlier, and the reason it mattered there applies here: a
//! preview and a commit that compute the same answer separately eventually
//! disagree, and the disagreement is silent.
//!
//! ## ★ The plan is computed ONCE, when the window opens
//!
//! Not per frame. Building it scans every configured font folder — reading and
//! parsing every font file in each — which measured **3,359 face names** on an
//! ordinary Windows font directory, and then reads each matched donor's bytes
//! into memory. A window that redid that sixty times a second would be
//! unusable, and nothing it depends on can change while it is open: the
//! document is not editable behind this window, and the folder list is not
//! either.
//!
//! ## Rule 4
//!
//! Nothing here marks the canvas, and the disclosures this window shows are the
//! *pre*-commit half. What the embed actually did lands in the disclosure line
//! through `app::actions::fonts`.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::embed as t;

/// The window body's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "embed.body";
/// The Embed button — the one control that changes anything.
// ui-text-exempt: trace region name, never displayed
pub const REGION_EMBED: &str = "embed.commit";

/// The window, and the plan it is showing.
pub struct EmbedDialog {
    /// What the engine says would happen, computed once. See the header.
    plan: pdfcer_core::font_embed_missing::EmbedPlan,
    /// The request the plan came from, carried so the commit sends the
    /// identical one.
    ///
    /// ★★ Carried rather than rebuilt. Rebuilding would re-scan the folders and
    /// could resolve a *different* donor — a file added to a folder while the
    /// window was open — so the operator would confirm one thing and commit
    /// another. The request is the operand; the plan is its consequence; they
    /// travel together.
    request: pdfcer_core::font_embed_missing::EmbedRequest,
    /// The blocked rows worth drawing, by index into `plan.blocked`.
    ///
    /// ★★ NOT the whole list. Under `EmbedSelection::AllMissing` the engine
    /// reports *"every other font in the document, including the ones that are
    /// simply already embedded"* as blocked — which on an ordinary drawing is
    /// most of them. A window that listed those would put twenty rows of *"it
    /// is already embedded"* in front of the two the operator can act on.
    ///
    /// ★ Everything else is kept, including `ProgramDeclaredButUnreadable`,
    /// which the engine's own `missing_program` flag also excludes. That one is
    /// not a font the operator has to find a file for, and it **is** a finding
    /// about their document — so it is drawn and not counted, which is exactly
    /// the distinction `missing_program` exists to let a shell draw.
    shown: Vec<usize>,
    /// Anything the folder scan skipped, so a missing donor is explicable.
    skipped: Vec<String>,
    embed_requested: bool,
    close_requested: bool,
}

impl EmbedDialog {
    /// Build the plan and open, or answer `None` when there is nothing to show.
    ///
    /// ★ `None` for a document with no missing fonts is deliberate: opening a
    /// window to say *"there is nothing to do"* is a modal an operator has to
    /// dismiss to learn they did not need it. The disclosure line says it
    /// instead — see [`open_for`]'s caller.
    #[must_use]
    pub fn open(doc: &OpenDoc, folders: &[std::path::PathBuf]) -> Option<Self> {
        // ★★★ `true`: pdfcer's own standard-14 faces may answer when the
        // operator's folders cannot. `OPERATOR_REQUESTS.md` O47, answered
        // "yes" on 2026-08-28.
        //
        // Safe to leave on because it is the LAST rung: `resolve_for_embedding`
        // reaches the bundled table only after an exact name match and a
        // family equivalence have both failed, so a machine with fonts
        // configured never sees it. It is a floor, not a preference -- which is
        // also why it needed no checkbox of its own beside O50's.
        let library = crate::app::fonts::Library::scan_with(folders, true);
        // ★ Every font whose program is absent, which is what the operator
        // means by "embed the fonts". `EmbedSelection::AllMissing` is the
        // engine's own spelling of it, so this shell is not deciding what
        // "missing" means.
        let mut request = pdfcer_core::font_embed_missing::EmbedRequest::all_missing();

        // ★★ TWO passes, and the first one exists because `supplied` is keyed
        // by `/BaseFont` *exactly as the file spells it* — subset tag included.
        // Only the engine knows which spellings a document carries, so the
        // shell cannot build the donor map without asking first. The probe is a
        // pure query and the request it takes supplies nothing, which makes
        // every missing font come back as `NoSourceFont` — a blocked row naming
        // precisely the string this loop needs.
        let probe = doc.session.embed_preview(&request);
        for blocked in &probe.blocked {
            if !blocked.missing_program {
                continue;
            }
            let Some(base_font) = blocked.base_font.as_deref() else {
                continue;
            };
            let Some(donor) = library.donor_for(base_font) else {
                continue;
            };
            request.supplied.insert(
                base_font.to_owned(),
                pdfcer_core::font_embed_missing::SuppliedFont::new(
                    donor.program.to_vec(),
                    donor.face_name.to_owned(),
                    donor.source(),
                    if donor.matched == crate::app::fonts::Match::Bundled {
                        // ★★★ `Bundled`, reported as itself and not folded into
                        // `Alias`.
                        //
                        // The engine's three rungs are three materially
                        // different acts and it says so: an alias is a
                        // documented family equivalence reached on the
                        // operator's OWN machine, and bundled means *"nothing
                        // on the operator's machine was consulted."* Collapsing
                        // them would tell somebody that the Arial on their disk
                        // answered when what answered was a face compiled into
                        // pdfcer.
                        pdfcer_core::font_embed_missing::FontMatch::Bundled
                    } else if donor.matched.is_inferred() {
                        // ★★★ `Alias` for both inferred rungs, and reporting
                        // either as `Exact` would be a CORRECTNESS defect
                        // rather than a cosmetic one.
                        //
                        // `FontMatch::is_substitute` is what the engine's
                        // symbolic guard turns on, and its reason is exact: a
                        // symbolic font's codes mean what its own program says
                        // they mean, so a stand-in draws a different repertoire
                        // rather than a different style. Claiming `Exact` for
                        // the Arial that answered a document's `Helvetica`
                        // would walk it straight past that guard.
                        pdfcer_core::font_embed_missing::FontMatch::Alias
                    } else {
                        pdfcer_core::font_embed_missing::FontMatch::Exact
                    },
                ),
            );
        }

        // Re-plan with the donors in hand: this is the plan the operator sees
        // and the one the commit will run.
        let plan = doc.session.embed_preview(&request);
        let shown: Vec<usize> = plan
            .blocked
            .iter()
            .enumerate()
            .filter(|(_, b)| {
                !matches!(
                    b.blocker,
                    pdfcer_core::font_embed_missing::EmbedBlocker::AlreadyEmbedded
                )
            })
            .map(|(index, _)| index)
            .collect();
        if plan.targets.is_empty() && shown.is_empty() {
            return None;
        }
        // ★★ The window's own plan counts, traced when it opens - see
        // `dialogs::unembed`'s equivalent for the argument. A window with
        // `targets=0` is correct for a document nothing on the machine can
        // answer for, and is indistinguishable from a broken button without
        // this line.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "embed-fonts-opened targets={} blocked={} shown={} supplied={}",
                plan.targets.len(),
                plan.blocked.len(),
                shown.len(),
                request.supplied.len()
            )
        });
        Some(Self {
            plan,
            request,
            shown,
            skipped: library.skipped,
            embed_requested: false,
            close_requested: false,
        })
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "embed-fonts", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(560.0, 560.0),
            egui::vec2(380.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.embed_requested) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "embed-fonts-requested targets={} blocked={} shown={} supplied={}",
                    self.plan.targets.len(),
                    self.plan.blocked.len(),
                    self.shown.len(),
                    self.request.supplied.len()
                )
            });
            actions.push(Action::EmbedFonts {
                request: Box::new(self.request.clone()),
            });
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        // ★★ The PDF/A line FIRST when there is one, above the lists. It is the
        // only sentence here about the document as a whole rather than about
        // one font, and an operator scanning a list of twenty faces should meet
        // it before the list rather than under it.
        if let Some(line) = t::pdfa_line(&self.plan.pdfa) {
            ui.label(line);
            ui.add_space(8.0);
        }

        egui::ScrollArea::vertical()
            .max_height((ui.available_height() - 56.0).max(160.0))
            .show(ui, |ui| {
                if !self.plan.targets.is_empty() {
                    ui.label(t::will_embed(self.plan.targets.len()));
                    for target in &self.plan.targets {
                        // The document's own spelling first — that is the name
                        // the operator saw in the Fonts panel and in whatever
                        // told them a font was missing. The donor's advertised
                        // name is only a fallback for a font with no
                        // `/BaseFont` at all.
                        let face = target.base_font.as_deref().unwrap_or(&target.face_name);
                        ui.small(t::embed_row(face, &target.source, rung(target.matched)));
                    }
                    ui.small(t::size_ceiling(self.plan.bytes_added_uncompressed()));
                    ui.add_space(8.0);
                }

                // ★★★ The END-STATE number, and its wording is GATED.
                //
                // `unexplained_missing`'s own docs are explicit that a window
                // saying *"each one is listed below"* is making a claim it
                // cannot keep under a named selection, and that the failure is
                // invisible to any test that only ever sends `AllMissing` —
                // which is the only selection this window sends today. So the
                // gate is written now, while the reason for it is legible,
                // rather than when a second selection arrives and nobody
                // remembers why the sentence was worded that way.
                let unexplained = self.plan.unexplained_missing();
                if unexplained > 0 {
                    ui.label(t::still_missing_partly_unexplained(
                        self.plan.missing_after(),
                        unexplained,
                    ));
                    ui.add_space(8.0);
                } else if let Some(line) = t::still_missing(self.plan.missing_after()) {
                    ui.label(line);
                    ui.add_space(8.0);
                }

                if !self.shown.is_empty() {
                    ui.label(t::cannot_embed(self.shown.len()));
                    for &index in &self.shown {
                        let blocked = &self.plan.blocked[index];
                        let face = blocked.base_font.as_deref().unwrap_or_default();
                        ui.small(t::blocked_row(face, &blocked.blocker));
                    }
                    ui.add_space(8.0);
                }
                if !self.plan.unmatched.is_empty() {
                    ui.small(t::unmatched(&self.plan.unmatched));
                    ui.add_space(8.0);
                }
                // ★ The folder scan's skips, last. They explain a missing donor
                // — *"none of your font folders holds it"* is the blocker, and
                // *"that file is 40 MB"* is why — so they belong after the
                // blockers they account for rather than before them.
                for note in &self.skipped {
                    ui.small(note);
                }
            });

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            // ★ Greyed when there is nothing to embed, with the reason on
            // hover: R9's temporarily-unavailable case. The window still opens
            // in that state, because the BLOCKED list is then the entire answer
            // the operator came for — *"here is what is missing and here is
            // what each one needs"* is worth a window even when the button is
            // dead.
            let can = !self.plan.targets.is_empty();
            let embed = ui.add_enabled(can, egui::Button::new(t::embed_button()));
            crate::diag::ui_rect_visible(REGION_EMBED, embed.rect, ui.clip_rect());
            let embed = if can {
                embed
            } else {
                embed.on_disabled_hover_text(t::nothing_to_embed())
            };
            if embed.clicked() {
                self.embed_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// The engine's provenance, back in this shell's own terms.
///
/// ★★ The two enums exist because the crate boundary is load-bearing —
/// `pdfcer-core` must not depend on `pdfcer-render`, so neither can name the
/// other's type and *"a shell converts between them in one line."* This is the
/// return leg of that conversion, and it is exhaustive rather than
/// wildcard-defaulted: `FontMatch` is `#[non_exhaustive]`, and a fourth rung
/// arriving must fail to compile here rather than quietly render as the row for
/// the most reassuring of the three.
fn rung(matched: pdfcer_core::font_embed_missing::FontMatch) -> crate::app::fonts::Match {
    use pdfcer_core::font_embed_missing::FontMatch;
    match matched {
        FontMatch::Exact => crate::app::fonts::Match::Exact,
        FontMatch::Alias => crate::app::fonts::Match::Alias,
        FontMatch::Bundled => crate::app::fonts::Match::Bundled,
        // ★ The catch-all reports the LOUDEST row, not the quietest. A rung
        // this build cannot name is one it cannot vouch for, and the honest
        // rendering of "pdfcer chose this and I do not know how" is the sentence
        // that says it is a stand-in.
        _ => crate::app::fonts::Match::Bundled,
    }
}

/// Open it for the current document, or answer `None`.
#[must_use]
pub fn open_for(status: &Status, folders: &[std::path::PathBuf]) -> Option<EmbedDialog> {
    match status {
        Status::Open(doc) => EmbedDialog::open(doc, folders),
        _ => None,
    }
}
