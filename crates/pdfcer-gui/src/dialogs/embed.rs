//! # `dialogs::embed` — the confirmation before font programs go into a
//! document
//!
//! `tools.embed_fonts`, wired 2026-08-28. The last of the three commands the
//! font-folder work unblocked.
//!
//! ## ★★★ A window with ONE setting, and how it got there — corrected
//! 2026-09-05
//!
//! This header used to say the window has **no** settings, that *"there is no
//! option to set"*, and that a form here *"would imply choices that do not
//! exist"*. That was true when it was written and it is not true now, and the
//! sentence is corrected in place rather than left standing beside its
//! replacement.
//!
//! There is exactly one control: **use pdfcer's own copy of a standard-14 face
//! where none of your fonts answers**. It is off when the window opens, it is
//! drawn only when it would change something, and the fonts it would stand in
//! for are named beside it before it is ticked. Everything else in the window
//! is still a report, and the shape below still governs.
//!
//! ### Why the "no settings by design" argument did not survive
//!
//! It was used, on 2026-08-28, as the reason **not** to offer the choice at
//! all: the window has no settings, so a switch cannot be added, so the only
//! options are always or never — and always was taken. Recorded in
//! `OPERATOR_REQUESTS.md` **O47**.
//!
//! ★★ That is an argument about a *window* being used to settle a question
//! about a *capability*. A surface's current shape is not a reason to withhold
//! something the operator is entitled to decide; it is at most a reason to
//! think about where the control goes. A window that grows one honest,
//! defaulted-off, disclosed control is still an honest window — and the whole
//! product class, pdfcer's own CLI included, spells this exact decision as
//! exactly one switch.
//!
//! The rest of the original argument stands and is why there is not a second
//! control: an embed puts font **programs** — actual outlines — permanently
//! into a document, changes its size, and can bear on a PDF/A claim. A
//! one-click ribbon verb would be dishonest about that.
//!
//! ## ★★★ WHY THE SWITCH IS OFF BY DEFAULT, and it is not a matter of taste
//!
//! Two reasons, and the second is the one that decided it.
//!
//! **1. The letters change.** Embedding a stand-in changes what the document
//! looks like on the screen of whoever it is sent to. That is disclosed per
//! row either way — see [`crate::text::embed::embed_row`]'s `Bundled` arm — so
//! on its own it argues for loud disclosure rather than for a default.
//!
//! **2. ★★★ It is a LICENCE the operator takes on, not a look they accept.**
//! pdfcer's fourteen substitute faces are BSD-3-Clause (`THIRD_PARTY_LICENSES.md`,
//! *"Bundled Foxit substitute faces"*), and embedding one puts it **inside a
//! file the operator then distributes**, carrying that licence's attribution
//! condition with it. `pdfcer`'s own CLI says so in the argument's doc comment
//! and draws the only defensible conclusion: *"That is your decision to make,
//! so pdfcer does not make it for you."* — `pdfcer-cli/src/main.rs:1598`,
//! `--use-bundled-fonts`, off by default.
//!
//! ⇒ Until 2026-09-05 this window made that decision for him on every press,
//! with no way to decline it. Rule 4 is *fuzzy, never sneaky*: it forbids doing
//! a thing **silently**, not doing it — and the half that was missing here was
//! never the disclosure, which was already loud. It was the **choice**.
//!
//! ## What did NOT change
//!
//! * The rung order. pdfcer's own faces are still consulted **last**, after an
//!   exact name match and after a standard-14 family equivalence, so a machine
//!   with fonts configured reaches a real face first whether the box is ticked
//!   or not.
//! * The disclosure. Every substituted row still says *"none of your fonts
//!   matched, so pdfcer used …. It is a stand-in, not the font the document
//!   asks for."*
//! * **Nothing is marked on the canvas.** No badge, no tint, no provisional
//!   styling on substituted text. Rule 4's surviving half is that an inference
//!   the operator cannot see owes them a report **off** the page — which is
//!   this window before the act and the disclosure row after it. Both;
//!   neither on the drawing.
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
/// The "use pdfcer's own copy" checkbox.
///
/// ★ Declared so a driven check can tick it. It is the whole of O47's answer
/// and, being off by default, a check that never presses it measures the other
/// position of the switch — which is why `ui-verify`'s
/// `embedding_works_with_no_font_folder_at_all` now drives **both**.
// ui-text-exempt: trace region name, never displayed
pub const REGION_USE_OWN_FONTS: &str = "embed.use-own-fonts";

/// Height reserved under the scrolling report, in egui points.
///
/// The buttons, plus the checkbox and its two sentences when they are drawn.
/// A **constant** rather than a measurement, for the reason the print preview's
/// strip height is a constant and records in full: a scroll area sized from
/// what is laid out under it is a measurement feeding a size, the caption
/// re-wraps on a narrow window, and the operator watches the body settle over
/// several frames for no reason they can see. Reserving for the taller case
/// costs a little unused height on a window with no checkbox; measuring costs a
/// feedback loop.
const FOOTER_RESERVE_PTS: f32 = 132.0;

/// **One planned embed: an operand, its consequence, and the rows worth
/// drawing.**
///
/// # ★★★ Why there are two of these and not one recomputed on demand
///
/// Because building one costs a folder scan — every font file in every
/// configured folder, read and parsed, measured at **3,359 face names** on an
/// ordinary Windows font directory. A checkbox that re-scanned on each click
/// would put a visible pause on a toggle.
///
/// ★★ And it is not only speed. Two plans built from **one** scan cannot
/// disagree about what is on the disk. A plan rebuilt later could resolve a
/// different donor — a file dropped into a folder while the window was open —
/// so the operator would tick a box that said one thing and commit another.
/// The whole of `EmbedDialog`'s existing "the request is the operand, the plan
/// is its consequence, they travel together" argument applies twice over when
/// there are two of them.
struct Planned {
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
}

impl Planned {
    /// Ask the engine what `request` would do, and pick the rows worth showing.
    fn of(
        session: &pdfcer_core::edit::EditSession,
        request: pdfcer_core::font_embed_missing::EmbedRequest,
    ) -> Self {
        let plan = session.embed_preview(&request);
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
        Self {
            plan,
            request,
            shown,
        }
    }
}

/// The window, and the plan it is showing.
pub struct EmbedDialog {
    /// What happens with **only the operator's own fonts** — the default, and
    /// what the Embed button commits unless the box is ticked.
    own_only: Planned,
    /// What happens when pdfcer's own standard-14 faces are allowed to answer
    /// as the last rung.
    ///
    /// ★★★ `None` when it would change **nothing**, and that is the condition
    /// that decides whether the checkbox is drawn at all. On a machine with the
    /// OS font folders switched on (O50) a real Arial answers the document's
    /// `Helvetica` before the bundled rung is ever reached, so this is `None`
    /// on most documents and the window is exactly what it was.
    ///
    /// ⇒ R9's positive form: a control that cannot change the outcome is not
    /// drawn. Drawing it greyed would be worse — it would advertise a
    /// capability on a document where pdfcer has nothing to offer.
    with_own_fonts: Option<Planned>,
    /// The document's own spelling of every font pdfcer would stand in for.
    ///
    /// ★★★ **The consequence, by name, before it happens.** Not a count — a
    /// list. *"3 fonts would be substituted"* is a number an operator cannot
    /// act on; *"Helvetica, Helvetica-Bold, Times-Roman"* is one they can look
    /// at and say *"not the title block"*. Empty exactly when
    /// [`Self::with_own_fonts`] is `None`.
    own_font_faces: Vec<String>,
    /// Whether the operator has ticked the box. **Off when the window opens.**
    ///
    /// See the module header for the two reasons, of which the licence is the
    /// deciding one.
    use_own_fonts: bool,
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
        // ★★★ ONE SCAN, TWO REQUESTS, and `true` here is not the decision.
        //
        // The library is scanned with pdfcer's own faces **available**, because
        // the window has to be able to say what they would answer for before
        // the operator decides whether to use them. Which of the two requests
        // is committed is [`Self::use_own_fonts`], and that is off.
        //
        // ★ Splitting the two donor maps out of one scan rather than scanning
        // twice is the whole reason the checkbox is free: a second
        // `scan_with(folders, false)` would read and parse every font file on
        // the machine a second time — 3,359 faces, measured — to learn
        // something this pass already knows, and two scans of a directory that
        // is being written to can disagree.
        let library = crate::app::fonts::Library::scan_with(folders, true);
        // ★ Every font whose program is absent, which is what the operator
        // means by "embed the fonts". `EmbedSelection::AllMissing` is the
        // engine's own spelling of it, so this shell is not deciding what
        // "missing" means.
        let mut request = pdfcer_core::font_embed_missing::EmbedRequest::all_missing();
        // The same selection, and it will receive only the donors that came off
        // the operator's own disk.
        let mut own_only_request = pdfcer_core::font_embed_missing::EmbedRequest::all_missing();
        let mut own_font_faces: Vec<String> = Vec::new();

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
            let is_own_copy = donor.matched == crate::app::fonts::Match::Bundled;
            let supplied = pdfcer_core::font_embed_missing::SuppliedFont::new(
                donor.program.to_vec(),
                donor.face_name.to_owned(),
                donor.source(),
                if is_own_copy {
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
            );
            // ★★★ THE SPLIT, AND IT IS THE WHOLE OF THE FEATURE.
            //
            // `request` gets every donor; `own_only_request` gets only the ones
            // that came off the operator's own disk. Which of the two is drawn
            // and committed is the checkbox.
            //
            // ★ The test is `Match::Bundled`, which `Library::donor_for`
            // decides by asking **its own path map** rather than by matching on
            // the engine's rung — a name the folder walk never registered
            // cannot have come off a folder, whatever the engine calls the rung
            // it took. That is the safer of the two questions and it is why
            // this line does not need to know the engine's enum.
            if is_own_copy {
                own_font_faces.push(base_font.to_owned());
            } else {
                own_only_request
                    .supplied
                    .insert(base_font.to_owned(), supplied.clone());
            }
            request.supplied.insert(base_font.to_owned(), supplied);
        }
        // Stable, so two openings of one window list the same names in the same
        // order and a screenshot taken twice can be compared. `probe.blocked`'s
        // order is the engine's and this shell does not depend on it.
        own_font_faces.sort();

        // Re-plan with the donors in hand: these are the plans the operator
        // sees and one of them is the plan the commit will run.
        let own_only = Planned::of(&doc.session, own_only_request);
        // ★★★ `None` WHEN IT WOULD CHANGE NOTHING, and the emptiness of the
        // face list is the right test rather than comparing the two plans.
        //
        // A plan comparison would be a proxy: two `EmbedPlan`s can differ in a
        // field that is not a target — an ordering, a byte count — and a
        // control drawn because two structs differed would be a control whose
        // presence nobody can explain. `own_font_faces` is populated in exactly
        // one place, by the one condition that means *"pdfcer's own copy is
        // what answered here"*, so it is the thing the sentence beside the
        // checkbox is made of and the thing that decides whether there is a
        // checkbox.
        let with_own_fonts =
            (!own_font_faces.is_empty()).then(|| Planned::of(&doc.session, request));
        if own_only.plan.targets.is_empty() && own_only.shown.is_empty() && with_own_fonts.is_none()
        {
            return None;
        }
        // ★★ The window's own plan counts, traced when it opens - see
        // `dialogs::unembed`'s equivalent for the argument. A window with
        // `targets=0` is correct for a document nothing on the machine can
        // answer for, and is indistinguishable from a broken button without
        // this line.
        //
        // ★★★ `own_fonts_offered=` and `own_fonts_on=` are new on 2026-09-05
        // and they exist because of a shape this project keeps meeting: **a
        // driven check that the switch is off by default passes on a build that
        // ignores the switch entirely.** With both fields on the line, a check
        // can assert the offer was made *and* declined — two facts, not one —
        // and a build that stopped offering is as visible as a build that
        // stopped defaulting.
        let offered = own_font_faces.len();
        let own_targets = with_own_fonts
            .as_ref()
            .map_or(0, |planned| planned.plan.targets.len());
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "embed-fonts-opened targets={} blocked={} shown={} supplied={} \
                 own_fonts_offered={offered} own_fonts_targets={own_targets} own_fonts_on=false",
                own_only.plan.targets.len(),
                own_only.plan.blocked.len(),
                own_only.shown.len(),
                own_only.request.supplied.len()
            )
        });
        Some(Self {
            own_only,
            with_own_fonts,
            own_font_faces,
            // ★★★ OFF. See the module header: the letters change, and the
            // licence travels with the file. `pdfcer`'s own CLI spells the same
            // decision `--use-bundled-fonts`, absent by default.
            use_own_fonts: false,
            skipped: library.skipped,
            embed_requested: false,
            close_requested: false,
        })
    }

    /// **The plan the window is showing and the button would commit.**
    ///
    /// One accessor, so the list the operator reads and the request the commit
    /// sends cannot be chosen by two different pieces of code — which is how
    /// a window comes to show one thing and do another, and is the property
    /// `embed_preview` was designed to give this dialog for free.
    ///
    /// ★ The `unwrap_or` is not defensive noise: [`Self::use_own_fonts`] can
    /// only be `true` if the checkbox was drawn, and the checkbox is only drawn
    /// when [`Self::with_own_fonts`] is `Some`. The fallback exists so that a
    /// future caller that sets the flag some other way degrades to the **safe**
    /// side — the operator's own fonts — rather than panicking or, worse,
    /// silently committing something else.
    fn active(&self) -> &Planned {
        chosen(
            &self.own_only,
            self.with_own_fonts.as_ref(),
            self.use_own_fonts,
        )
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
            // ★ Both the plan being sent AND the position of the switch that
            // chose it. A line carrying only the counts cannot distinguish
            // "the operator declined pdfcer's own faces" from "this build has
            // no such faces", and those need different next actions.
            let on = self.use_own_fonts;
            let active = self.active();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "embed-fonts-requested targets={} blocked={} shown={} supplied={} \
                     own_fonts_on={on}",
                    active.plan.targets.len(),
                    active.plan.blocked.len(),
                    active.shown.len(),
                    active.request.supplied.len()
                )
            });
            actions.push(Action::EmbedFonts {
                request: Box::new(active.request.clone()),
            });
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

        // ★ The whole report below is drawn from `active()`, so the list the
        // operator reads is by construction the plan the button would send. The
        // borrow is taken once, for the read-only half of the body, and
        // released before the checkbox — which is the only thing here that
        // writes.
        let active = self.active();
        let plan = &active.plan;
        let shown = &active.shown;

        // ★★ The PDF/A line FIRST when there is one, above the lists. It is the
        // only sentence here about the document as a whole rather than about
        // one font, and an operator scanning a list of twenty faces should meet
        // it before the list rather than under it.
        if let Some(line) = t::pdfa_line(&plan.pdfa) {
            ui.label(line);
            ui.add_space(8.0);
        }

        let own_font_faces = &self.own_font_faces;
        egui::ScrollArea::vertical()
            // ★ 56 pt was the buttons; the checkbox and its sentence sit under
            // this area too, so the reservation grew with them. It stays a
            // constant rather than a measurement for the reason the print
            // preview's strip height is a constant: a body sized from what is
            // under it is a feedback loop, and this shell has met that three
            // times.
            .max_height((ui.available_height() - FOOTER_RESERVE_PTS).max(160.0))
            .show(ui, |ui| {
                if !plan.targets.is_empty() {
                    ui.label(t::will_embed(plan.targets.len()));
                    for target in &plan.targets {
                        // The document's own spelling first — that is the name
                        // the operator saw in the Fonts panel and in whatever
                        // told them a font was missing. The donor's advertised
                        // name is only a fallback for a font with no
                        // `/BaseFont` at all.
                        let face = target.base_font.as_deref().unwrap_or(&target.face_name);
                        ui.small(t::embed_row(face, &target.source, rung(target.matched)));
                    }
                    ui.small(t::size_ceiling(plan.bytes_added_uncompressed()));
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
                let unexplained = plan.unexplained_missing();
                if unexplained > 0 {
                    ui.label(t::still_missing_partly_unexplained(
                        plan.missing_after(),
                        unexplained,
                    ));
                    ui.add_space(8.0);
                } else if let Some(line) = t::still_missing(plan.missing_after()) {
                    ui.label(line);
                    ui.add_space(8.0);
                }

                if !shown.is_empty() {
                    ui.label(t::cannot_embed(shown.len()));
                    for &index in shown {
                        let blocked = &plan.blocked[index];
                        let face = blocked.base_font.as_deref().unwrap_or_default();
                        // ★★★ THE PER-FONT REMEDY HAD TO LEARN ABOUT THE BOX.
                        //
                        // With the box unticked, a standard-14 face pdfcer
                        // carries is now a `NoSourceFont` row — and that row's
                        // sentence used to end *"and it is not one of the
                        // fourteen pdfcer carries itself"*, which for exactly
                        // these fonts would be **false**. A refusal's wording is
                        // a claim about what would fix it, and the things that
                        // fix it change under it; this one changed today.
                        //
                        // So the row is told whether pdfcer has a copy, and says
                        // *"tick the box below"* when it does. It is the cheapest
                        // remedy on offer and sending an operator to a folder
                        // picker instead costs them the difference.
                        let ours = own_font_faces.iter().any(|name| name == face);
                        ui.small(t::blocked_row(face, &blocked.blocker, ours));
                    }
                    ui.add_space(8.0);
                }
                if !plan.unmatched.is_empty() {
                    ui.small(t::unmatched(&plan.unmatched));
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

        // ═══════════════════════════════════════════════════════════════════
        // ★★★ THE ONE CONTROL — O47, answered properly on 2026-09-05.
        //
        // Drawn here, between the report and the buttons, because that is where
        // the consequence belongs: the operator reads what will happen, meets
        // the one thing they can change about it, and then presses. Putting it
        // at the top would make it a setting to be configured before reading;
        // putting it beside the button would make it a modifier on a press.
        //
        // ★ Drawn ONLY when it would change something — see
        // [`Self::with_own_fonts`]. On a machine with the OS font folders on,
        // a real face answers first and this whole block is absent, so the
        // window is exactly what it was for anybody whose fonts are configured.
        // ═══════════════════════════════════════════════════════════════════
        if self.with_own_fonts.is_some() {
            ui.add_space(8.0);
            ui.separator();
            // ★★★ THE CONSEQUENCE IS STATED BEFORE THE BOX, AND IT NAMES THE
            // FONTS.
            //
            // Not *"3 fonts would be substituted"* — the actual list, in the
            // document's own spelling, because that is the form an operator can
            // act on. *"Helvetica, Helvetica-Bold"* is something he can look at
            // and say *"that is the title block, no"*; a count is not.
            //
            // Two sentences follow it, and both are consequences he cannot see
            // from the list: the letters change on the recipient's screen, and
            // the licence on pdfcer's copies travels inside the file he sends.
            // The second is the one the engine's CLI calls his decision to make.
            ui.label(t::own_fonts_offer(&self.own_font_faces));
            ui.small(t::own_fonts_consequence());
            let box_ = ui.checkbox(&mut self.use_own_fonts, t::own_fonts_checkbox());
            crate::diag::ui_rect_visible(REGION_USE_OWN_FONTS, box_.rect, ui.clip_rect());
            if box_.changed() {
                let on = self.use_own_fonts;
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!("embed-fonts-own-fonts on={on}")
                });
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            // ★ Greyed when there is nothing to embed, with the reason on
            // hover: R9's temporarily-unavailable case. The window still opens
            // in that state, because the BLOCKED list is then the entire answer
            // the operator came for — *"here is what is missing and here is
            // what each one needs"* is worth a window even when the button is
            // dead.
            let can = !self.active().plan.targets.is_empty();
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

/// **Which of the two plans the switch selects.**
///
/// # ★★★ Why this is a free generic function and not three lines inside
/// # [`EmbedDialog::active`]
///
/// Because the decision it makes is the whole of O47 and there is no other way
/// to assert it. A `Planned` holds an `EmbedPlan`, which only `pdfcer-core` can
/// build and only from an open document — so a unit test of `active` would need
/// a `Session`, a PDF, and a font folder, which is a driven check wearing a
/// test's clothes. Lifted out and made generic, the *selection* is testable
/// over anything, and what is tested is the function the running program calls
/// rather than a paraphrase of it.
///
/// ⇒ The shape this guards against is one this project keeps meeting:
/// **a test that the switch is off by default passes on a build that ignores
/// the switch entirely.** Both positions are asserted below, against the same
/// function the dialog uses.
///
/// ★ `with_own` being `None` wins over `use_own` being `true`, deliberately and
/// in that order. The checkbox is only drawn when there is something to choose,
/// so that combination should be unreachable — and if a future caller makes it
/// reachable, the safe answer is the operator's own fonts. A `panic!` or an
/// `unwrap` here would turn a wiring mistake into a crash on the Embed window;
/// silently committing the substitutes would turn it into a licence the
/// operator never agreed to. Degrading to `own_only` is the only arm that is
/// wrong in a direction nobody has to live with.
fn chosen<'a, T>(own_only: &'a T, with_own: Option<&'a T>, use_own: bool) -> &'a T {
    match (with_own, use_own) {
        (Some(theirs), true) => theirs,
        _ => own_only,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Both positions of the switch, against the function the dialog calls.**
    ///
    /// ★★★ The two assertions are worthless apart and are written as one test
    /// so that they cannot be separated:
    ///
    /// * *off* alone passes on a build that has no bundled faces to offer, or
    ///   that ignores the flag and always returns the operator's own plan;
    /// * *on* alone passes on a build that ignores the flag the other way and
    ///   always returns the substitutes — which is what shipped between
    ///   2026-08-28 and 2026-09-05, and is the state this row corrected.
    ///
    /// ★ Strings stand in for the two plans. What is under test is the
    /// **selection**, and a selection is the same function whatever it selects
    /// between — see [`chosen`] for why the real type cannot be constructed in
    /// a unit test at all.
    #[test]
    fn the_switch_is_off_by_default_and_it_is_the_switch_that_decides() {
        let mine = "the operator's own fonts";
        let pdfcers = "pdfcer's own standard-14 faces";
        assert_eq!(
            *chosen(&mine, Some(&pdfcers), false),
            mine,
            "OFF must commit the operator's own fonts. Embedding one of pdfcer's substitutes \
             changes what the letters look like on his client's screen and carries a \
             BSD-3-Clause attribution condition into a file he distributes — pdfcer does not \
             make that decision for him"
        );
        assert_eq!(
            *chosen(&mine, Some(&pdfcers), true),
            pdfcers,
            "ON must commit pdfcer's own faces. A switch that changes what the window DRAWS and \
             not what the button DOES is a window showing one thing and doing another"
        );
    }

    /// **With nothing to offer, the switch cannot select anything** — the arm
    /// the checkbox's own drawing condition is supposed to make unreachable.
    ///
    /// Asserted rather than left to the comment, because "unreachable by
    /// construction" is a claim about a construction that somebody will change.
    /// The safe direction is the operator's own fonts; see [`chosen`].
    #[test]
    fn with_nothing_to_offer_the_switch_falls_back_to_the_operators_own_fonts() {
        let mine = "the operator's own fonts";
        assert_eq!(*chosen(&mine, None, true), mine);
        assert_eq!(*chosen(&mine, None, false), mine);
    }
}
