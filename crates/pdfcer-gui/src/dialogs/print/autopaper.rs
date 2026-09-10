//! # `dialogs::print::autopaper` — pick the sheet from the pages
//!
//! Operator request **O167**, 2026-09-10: *"we also need the option to auto
//! select paper size based on the page sizes in the pdf."*
//!
//! ## What this module is, and what it deliberately is not
//!
//! It is **arithmetic over two lists**: every sheet the driver enumerated
//! ([`super::spooler::PaperForm`]) and every page of the job measured at its
//! rotated extent. It answers one question — *which enumerated sheet should
//! this job be asked for?* — and returns that answer together with everything
//! the disclosure line needs in order to say what happened and why.
//!
//! It is **not** a policy about mixed page sizes, not a call into the spooler,
//! and not a `Ui`. It opens no device context, reads no `DEVMODE` and cannot
//! print. That is on purpose: this is the half of the feature a unit test can
//! drive, and this project's standing lesson is that a verb's unit test cannot
//! see the chain in front of it — so the arithmetic is asserted here and the
//! chain from the combo entry to the spooled `dmPaperSize` is asserted by
//! driving the binary (`tools/ui-verify/src/checks/print_paper.rs`).
//!
//! ## ★ The rule, stated once
//!
//! > **The chosen sheet is the smallest enumerated sheet that contains every
//! > page in the job, with each page free to lie either way round on it. If no
//! > enumerated sheet contains them all, the largest enumerated sheet is
//! > chosen and the shortfall is reported.**
//!
//! Three parts of that sentence are load-bearing and each is argued below.
//!
//! ### "every page", not "the first page"
//!
//! A `DEVMODE` names **one** sheet and a job has many pages. Choosing from
//! page 1 would put a 40-page set of A3 details onto A4 because the cover
//! sheet happened to be A4, and nothing on screen would say why every drawing
//! came out at 71 %. Choosing the sheet that holds the *largest* page means
//! the biggest drawing is right and the smaller ones are scaled down onto a
//! bigger sheet — visibly generous rather than invisibly cropped.
//!
//! Windows' own answer for a genuinely mixed set is the **choose tray by sheet
//! size** flag, which this dialog already exposes beside the paper control
//! (`spooler::DeviceSettings::pick_tray_by_page_size`). A device with more
//! than one roll or tray will then feed each page its own sheet, and the
//! `dmPaperSize` this module picks is what the rest lands on. The disclosure
//! says so when the job is mixed, because an operator with a mixed set needs
//! to know that one flag is the difference.
//!
//! ### "either way round"
//!
//! `dmPaperSize` names a **physical piece of paper**; which way the image is
//! laid on it is `dmOrientation`, a separate field with its own control in
//! this dialog. A3 and "A3 landscape" are not two sheets. So the fit test
//! tries the page both ways round, and it is not a compromise — a fit test
//! that respected page orientation would refuse an A3 sheet for a landscape
//! A3 drawing, which is the commonest CAD export there is.
//!
//! ### "the largest, and reported"
//!
//! When nothing fits — an A0 site plan on an office printer whose largest
//! sheet is A3 — there is no honest choice that makes the drawing come out
//! right. The alternatives were to fall back to saying nothing about paper
//! (which prints on whatever the device is standing on, chosen by nobody), or
//! to pick the biggest sheet the device has and **say that the page is bigger
//! than it**. The second is chosen: the operator asked pdfcer to match the
//! pages, and the closest available match plus a sentence naming the shortfall
//! is a better answer than a silent no-op. See
//! [`crate::text::print::paper_auto_too_big`].
//!
//! ## Tolerance, and why it is 2 pt
//!
//! Producers do not emit exact ISO sizes. A4 is 595.276 pt and is written
//! `595.32`, `595.3`, `595` and `595.2756` by four different exporters; a
//! SolidWorks sheet set carries the drawing frame's size rather than the
//! standard's. A fit test with no tolerance would refuse an A4 sheet for a
//! 595.4 pt page and step up to A3, which is a whole size wrong for two
//! hundredths of a millimetre.
//!
//! 2 pt is 0.7 mm. It is comfortably larger than every rounding divergence
//! measured in the producer-quirk notes, and comfortably smaller than the gap
//! between any two ISO or ANSI sizes (the closest pair in ordinary use is
//! Letter at 612 pt wide and A4 at 595.3 — 16.7 pt apart, eight times the
//! tolerance). It cannot promote a page to the next size up and it cannot let
//! a genuinely oversized page pass.
//!
//! ## What "smallest" means when two sheets are the same size
//!
//! Drivers routinely enumerate the same physical sheet twice under different
//! names — `"A4"` and `"A4 210 x 297 mm"`, or a borderless twin. The tie is
//! broken by **the driver's own enumeration order**, first wins, which makes
//! the choice deterministic for a given device and matches what the operator
//! sees at the top of the list. Nothing better is available: pdfcer has no way
//! to know which of two identically-sized forms the device would rather have.

use super::spooler::{PaperChoice, PaperForm};

/// How close two lengths must be, in points, to count as the same size.
///
/// See the module header for why this is 2 pt and not zero.
const FIT_TOLERANCE_PT: f64 = 2.0;

/// **What auto selection decided**, in a form both the plan and the disclosure
/// can read.
///
/// One value carries the outcome *and* the evidence for it, so the sentence
/// under the combo can never describe a different decision from the one the
/// job was planned with — the same pairing argument
/// `crate::panels::docprops::offered_reading` makes for a label and a policy.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum AutoPaper {
    /// Auto selection is not the operator's choice, so nothing was computed.
    NotChosen,
    /// Auto is chosen but there is nothing to choose from or nothing to
    /// measure: the driver enumerated no sheets, or the document has no pages.
    ///
    /// Collapsed into one variant on purpose. Both mean *"pdfcer has no basis
    /// for a choice"*, both resolve to saying nothing about paper, and the
    /// combo is not drawn at all when the form list is empty — so the only way
    /// to reach this in a shipped build is a document with no pages, which
    /// cannot print either.
    NoBasis,
    /// A sheet was chosen and every page fits on it.
    Matched(Match),
    /// A sheet was chosen — the largest available — and the biggest page in
    /// the job does **not** fit on it.
    TooBig(Match),
}

/// The chosen sheet, and the measurements that chose it.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Match {
    /// The `dmPaperSize` value to request.
    pub(super) id: u16,
    /// The driver's own name for it. Never an identity — two forms may share
    /// a name — but it is what the operator sees in the list.
    pub(super) name: String,
    /// The physical sheet, in points.
    pub(super) sheet_pt: (f64, f64),
    /// The largest page in the job, in points, at its rotated extent. This is
    /// the page the choice was made for.
    pub(super) largest_page_pt: (f64, f64),
    /// `true` when the job does not have a single page size throughout.
    ///
    /// Drives the extra sentence about the tray flag; see the module header.
    pub(super) mixed: bool,
}

impl AutoPaper {
    /// The concrete paper choice this outcome resolves to.
    ///
    /// [`AutoPaper::NotChosen`] cannot legitimately be asked — the caller only
    /// resolves when the operator picked auto — but it answers
    /// [`PaperChoice::DeviceDefault`] rather than panicking, for the reason
    /// [`choose`]'s unreachable arm gives: a print dialog that unwraps is a
    /// print dialog that can take the application down mid-job.
    pub(super) fn resolved(&self) -> PaperChoice {
        match self {
            Self::Matched(m) | Self::TooBig(m) => PaperChoice::Form(m.id),
            Self::NotChosen | Self::NoBasis => PaperChoice::DeviceDefault,
        }
    }
}

/// **A stable one-word token for what the operator chose**, for the trace.
///
/// # ⚠ Why this exists rather than `{:?}` on the enum
///
/// A machine reads it. This project has already shipped a driven check that
/// reported the opposite of the truth because it was parsing a `Debug` tuple,
/// and `PaperChoice::Form(9)` contains a space in no rendering but does carry
/// punctuation a naive `key=value` split will mangle. These three tokens
/// contain no whitespace, no punctuation and no numbers, and their spelling is
/// pinned by a test.
pub(super) fn pick_token(choice: PaperChoice) -> &'static str {
    match choice {
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        PaperChoice::DeviceDefault => "device",
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        PaperChoice::Form(_) => "form",
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        PaperChoice::AutoFromPages => "auto",
    }
}

/// **A stable one-word token for how the choice came out**, for the trace.
///
/// The companion to [`pick_token`], and the field that makes a driven check
/// able to tell "auto matched A4" from "auto was chosen and found nothing".
/// Both leave `pick=auto`; only this field separates them.
pub(super) fn outcome_token(outcome: &AutoPaper) -> &'static str {
    match outcome {
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        AutoPaper::NotChosen => "off",
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        AutoPaper::NoBasis => "nobasis",
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        AutoPaper::Matched(_) => "matched",
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        AutoPaper::TooBig(_) => "toobig",
    }
}

/// **A point-pair as one whitespace-free token**, for the trace: `"595.28x841.89"`.
///
/// # ⚠ Why this exists rather than `{:?}` on the tuple
///
/// `sheet=` was `{:?}` until 2026-09-10 and printed `Some((1190.4, 841.68))`.
/// That survived only because `tools/ui-verify`'s splitter tracks bracket
/// depth; every consumer still had to do string surgery to get a number out,
/// and this project has already shipped a driven check that **reported the
/// opposite of the truth** because it was parsing a `Debug` tuple. A field a
/// machine reads gets a spelling chosen for the machine.
///
/// Two decisions inside it:
///
/// - **Two decimal places, always.** Fixed rather than `{}` so the spelling is
///   deterministic — `1190.4` and `1190.40` are the same number and two
///   different tokens, and a check that string-compares a before and an after
///   would see a change that did not happen. 0.01 pt is 3.5 micron, two orders
///   below the 2 pt fit tolerance, so nothing is lost by rounding here.
/// - **`none` for absent**, not `None`: lower-case, no punctuation, and it
///   reads the same as the other absent-value tokens on the same line.
pub(super) fn size_token(size: Option<(f64, f64)>) -> String {
    match size {
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        None => "none".to_owned(),
        Some((w, h)) => format!("{w:.2}x{h:.2}"),
    }
}

/// **The largest page the auto decision measured**, as a [`size_token`].
///
/// ★ This is the field that makes O167 checkable from outside the process,
/// and it is worth saying why the other three are not enough. `pick=auto` says
/// the operator chose the policy; `auto=matched` says the decision ran;
/// `paper=Form(8)` says it was turned into a request. **None of them says the
/// sheet has anything to do with this document.** A build that resolved auto
/// to the first form in the driver's list would emit all three, correctly, and
/// be completely wrong — and the operator's words were *"based on the page
/// sizes in the pdf"*.
///
/// With this beside `sheet=`, a driven check can assert the actual invariant:
/// `matched` means the largest page fits the chosen sheet either way round,
/// and `toobig` means it does not. That is the rule the module header states,
/// measured against a real driver's geometry rather than against the fixture
/// list a unit test supplies.
pub(super) fn largest_token(outcome: &AutoPaper) -> String {
    match outcome {
        AutoPaper::Matched(m) | AutoPaper::TooBig(m) => size_token(Some(m.largest_page_pt)),
        AutoPaper::NotChosen | AutoPaper::NoBasis => size_token(None),
    }
}

/// **Whether the job has more than one page size**, as a stable token.
///
/// `off` rather than `no` when auto was never chosen, because "this job is not
/// mixed" and "nobody asked" are different answers and a check that read the
/// first for the second would be asserting a property of a decision that never
/// happened. The same three-state care as `outcome_token`, one field along.
pub(super) fn mixed_token(outcome: &AutoPaper) -> &'static str {
    match outcome {
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        AutoPaper::Matched(m) | AutoPaper::TooBig(m) if m.mixed => "yes",
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        AutoPaper::Matched(_) | AutoPaper::TooBig(_) => "no",
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        AutoPaper::NotChosen | AutoPaper::NoBasis => "off",
    }
}

/// Does `page` lie on `sheet`, either way round, within tolerance?
///
/// Both orientations are tried because `dmPaperSize` names a physical sheet
/// and `dmOrientation` is a separate field — see the module header.
fn fits(page: (f64, f64), sheet: (f64, f64)) -> bool {
    let (pw, ph) = page;
    let (sw, sh) = sheet;
    (pw <= sw + FIT_TOLERANCE_PT && ph <= sh + FIT_TOLERANCE_PT)
        || (pw <= sh + FIT_TOLERANCE_PT && ph <= sw + FIT_TOLERANCE_PT)
}

/// Sheet area in square points, used only for ordering.
///
/// Area rather than either edge, because the ordering has to be total: two
/// sheets can each be wider than the other on one axis (a 200 x 400 roll cut
/// and a 300 x 300 square), and "smallest" has to mean something for that
/// pair. Area is the measure of how much paper is consumed, which is what the
/// operator is choosing between.
fn area(sheet: (f64, f64)) -> f64 {
    sheet.0 * sheet.1
}

/// **The whole decision**: which sheet this job should be asked for.
///
/// `page_sizes` are the pages of the job at their **rotated** extents — the
/// same measurement the preview and the canvas use, so all three agree by
/// construction. Passing raw `/MediaBox` sizes here would choose portrait
/// sheets for `/Rotate 90` landscape drawings.
///
/// Returns [`AutoPaper::NoBasis`] rather than an `Option`, so that every caller
/// has to name what it does about the no-basis case rather than reaching for
/// `unwrap_or_default`.
pub(super) fn choose(forms: &[PaperForm], page_sizes: &[(f64, f64)]) -> AutoPaper {
    let Some(&first) = page_sizes.first() else {
        return AutoPaper::NoBasis;
    };
    if forms.is_empty() {
        return AutoPaper::NoBasis;
    }

    // The job is mixed when any page differs from the first by more than the
    // fit tolerance on either axis, **either way round**. A set of landscape
    // and portrait A4 is not mixed in the sense that matters here: one sheet
    // serves it, and the tray flag has nothing to add.
    let mixed = page_sizes.iter().any(|&p| {
        let same =
            (p.0 - first.0).abs() <= FIT_TOLERANCE_PT && (p.1 - first.1).abs() <= FIT_TOLERANCE_PT;
        let turned =
            (p.0 - first.1).abs() <= FIT_TOLERANCE_PT && (p.1 - first.0).abs() <= FIT_TOLERANCE_PT;
        !(same || turned)
    });

    // ★ The page reported as "largest" is the one covering the most paper, and
    // it is REPORTED rather than used to choose. Note that it is not always
    // the page that constrains the choice: a 100 x 2000 roll page beats a
    // 900 x 900 square on the axis that matters even though the square covers
    // more paper. The sheet is therefore chosen by testing *every* page
    // against *every* sheet, below, and this value exists only for the
    // sentence under the combo.
    let largest_page_pt = page_sizes
        .iter()
        .copied()
        .max_by(|a, b| area(*a).total_cmp(&area(*b)))
        .unwrap_or(first);

    // Smallest sheet that holds every page. `min_by` keeps the FIRST of equal
    // elements, which is the driver's own enumeration order — the documented
    // tie-break.
    let all_fit = |sheet: (f64, f64)| page_sizes.iter().all(|&page| fits(page, sheet));
    if let Some(form) = forms
        .iter()
        .filter(|form| all_fit(form.size_pt))
        .min_by(|a, b| area(a.size_pt).total_cmp(&area(b.size_pt)))
    {
        return AutoPaper::Matched(Match {
            id: form.id,
            name: form.name.clone(),
            sheet_pt: form.size_pt,
            largest_page_pt,
            mixed,
        });
    }

    // Nothing holds them all. The largest sheet the device has, and a sentence
    // — see the module header for why this beats falling silent.
    //
    // The `else` arm cannot be reached: `forms` was checked non-empty above
    // and `max_by` over a non-empty iterator returns `Some`. It answers
    // `NoBasis` rather than panicking because a print dialog that unwraps is a
    // print dialog that can take the application down mid-job.
    let Some(form) = forms
        .iter()
        .max_by(|a, b| area(a.size_pt).total_cmp(&area(b.size_pt)))
    else {
        return AutoPaper::NoBasis;
    };
    AutoPaper::TooBig(Match {
        id: form.id,
        name: form.name.clone(),
        sheet_pt: form.size_pt,
        largest_page_pt,
        mixed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ordinary ISO and ANSI sheets, in the order a driver tends to list them
    /// — smallest first is NOT guaranteed by any driver, so the list here is
    /// deliberately out of order.
    fn forms() -> Vec<PaperForm> {
        vec![
            PaperForm {
                id: 8,
                name: "A3".to_owned(),
                size_pt: (841.89, 1190.55),
            },
            PaperForm {
                id: 1,
                name: "Letter".to_owned(),
                size_pt: (612.0, 792.0),
            },
            PaperForm {
                id: 9,
                name: "A4".to_owned(),
                size_pt: (595.276, 841.89),
            },
        ]
    }

    fn id_of(outcome: &AutoPaper) -> Option<u16> {
        match outcome {
            AutoPaper::Matched(m) | AutoPaper::TooBig(m) => Some(m.id),
            AutoPaper::NotChosen | AutoPaper::NoBasis => None,
        }
    }

    /// **An A4 document picks A4, not the first entry and not the biggest.**
    ///
    /// The list is deliberately ordered A3, Letter, A4 so that a `first()`
    /// would answer A3 and a `max` would answer A3 as well — only the stated
    /// rule answers A4.
    #[test]
    fn the_smallest_sheet_that_holds_the_page_wins() {
        let pages = vec![(595.276, 841.89); 4];
        let outcome = choose(&forms(), &pages);
        assert!(matches!(outcome, AutoPaper::Matched(_)), "{outcome:?}");
        assert_eq!(id_of(&outcome), Some(9));
    }

    /// **A landscape page picks the same sheet as the portrait one.**
    ///
    /// The commonest CAD export there is. A fit test that respected page
    /// orientation would step this up to A3 — a whole size wrong, on every
    /// drawing, silently.
    #[test]
    fn a_turned_page_lies_on_the_same_sheet() {
        let pages = vec![(841.89, 595.276)];
        assert_eq!(id_of(&choose(&forms(), &pages)), Some(9));
    }

    /// **A mixed set is served by the sheet that holds the biggest page.**
    ///
    /// And it is flagged as mixed, because that is the case where the tray
    /// flag is the difference between a right job and a scaled one.
    #[test]
    fn a_mixed_set_takes_the_sheet_that_holds_them_all() {
        let pages = vec![(595.276, 841.89), (841.89, 1190.55), (612.0, 792.0)];
        let outcome = choose(&forms(), &pages);
        assert_eq!(id_of(&outcome), Some(8), "{outcome:?}");
        match outcome {
            AutoPaper::Matched(m) => assert!(m.mixed, "three different sizes is a mixed job"),
            other => panic!("expected a match, got {other:?}"),
        }
    }

    /// **Portrait and landscape of one size is NOT a mixed job.**
    ///
    /// One sheet serves it and the tray flag has nothing to add, so the extra
    /// sentence would be noise. This is the case the `turned` clause in
    /// [`choose`] exists for, and without it every rotated page in an
    /// otherwise uniform set would trip the mixed sentence.
    #[test]
    fn turning_a_page_does_not_make_a_job_mixed() {
        let pages = vec![(595.276, 841.89), (841.89, 595.276)];
        match choose(&forms(), &pages) {
            AutoPaper::Matched(m) => assert!(!m.mixed),
            other => panic!("expected a match, got {other:?}"),
        }
    }

    /// **A page nothing holds gets the largest sheet AND the shortfall said.**
    ///
    /// An A0 site plan on an office printer. The verdict is `TooBig`, not
    /// `Matched`, which is what makes the disclosure name the page size rather
    /// than claim a fit that will not happen.
    #[test]
    fn a_page_too_big_for_every_sheet_takes_the_biggest_and_says_so() {
        let a0 = (2383.94, 3370.39);
        let outcome = choose(&forms(), &[a0]);
        assert!(matches!(outcome, AutoPaper::TooBig(_)), "{outcome:?}");
        assert_eq!(id_of(&outcome), Some(8), "A3 is the largest sheet listed");
        match outcome {
            AutoPaper::TooBig(m) => assert_eq!(m.largest_page_pt, a0),
            other => panic!("expected TooBig, got {other:?}"),
        }
    }

    /// **Producer rounding does not promote a page a whole size.**
    ///
    /// ⚠ The input is chosen to be awkward rather than convenient: 595.4 pt is
    /// *wider than A4* by a tenth of a millimetre, which is what a real
    /// exporter writes and what a zero-tolerance fit test refuses. Both the
    /// slightly-over and slightly-under cases are tried, because a tolerance
    /// applied on one side only is a tolerance that was never tested.
    #[test]
    fn a_tenth_of_a_millimetre_of_rounding_is_not_a_size_change() {
        for page in [(595.4, 842.1), (595.0, 841.5)] {
            assert_eq!(
                id_of(&choose(&forms(), &[page])),
                Some(9),
                "{page:?} is A4 to within a driver's rounding"
            );
        }
    }

    /// **A page genuinely bigger than the sheet is not waved through.**
    ///
    /// The other half of the tolerance claim, and the one that matters: a
    /// tolerance wide enough to accept a real oversize would silently crop
    /// every drawing. 10 pt over A4 on the short edge is 3.5 mm — small,
    /// visible, and refused.
    ///
    /// ★ The answer is A3, not Letter, and the reason is worth keeping: Letter
    /// is **wider** than this page (612 vs 605) and **shorter** than it
    /// (792 vs 842), so it does not hold it either way round. A fit test that
    /// compared one axis, or compared areas, would have answered Letter — and
    /// the drawing would have come off the machine with 50 mm missing from the
    /// bottom. This test was written expecting Letter and the code was right;
    /// the expectation is recorded here because the same mistake is the
    /// obvious one to make when this function is next changed.
    #[test]
    fn the_tolerance_does_not_swallow_a_real_oversize() {
        assert_eq!(
            id_of(&choose(&forms(), &[(605.0, 842.0)])),
            Some(8),
            "10 pt over A4 must step up, and Letter is too SHORT to be the step"
        );
    }

    /// **No forms and no pages both answer "no basis", not a guess.**
    ///
    /// The alternative — defaulting to Letter, or to the first form — would be
    /// pdfcer inventing a sheet nobody chose and then reporting it as a match.
    #[test]
    fn nothing_to_choose_from_is_answered_honestly() {
        assert_eq!(choose(&[], &[(595.0, 842.0)]), AutoPaper::NoBasis);
        assert_eq!(choose(&forms(), &[]), AutoPaper::NoBasis);
    }

    /// **Every outcome resolves to a paper the engine can act on.**
    ///
    /// ★ The property asserted is the one that matters downstream:
    /// `AutoFromPages` must never survive the resolution. A build where it did
    /// would hand the spooler a variant it maps to `DeviceDefault` anyway — so
    /// the job would print on the device's own sheet while the sentence under
    /// the combo claimed a match, which is precisely the silent divergence
    /// this whole feature is a disclosure about.
    #[test]
    fn no_outcome_resolves_to_the_auto_variant_itself() {
        let a_match = Match {
            id: 9,
            name: "A4".to_owned(),
            sheet_pt: (595.276, 841.89),
            largest_page_pt: (595.276, 841.89),
            mixed: false,
        };
        for outcome in [
            AutoPaper::NotChosen,
            AutoPaper::NoBasis,
            AutoPaper::Matched(a_match.clone()),
            AutoPaper::TooBig(a_match),
        ] {
            assert_ne!(
                outcome.resolved(),
                PaperChoice::AutoFromPages,
                "{outcome:?} resolved to the unresolvable variant"
            );
        }
        assert_eq!(AutoPaper::NoBasis.resolved(), PaperChoice::DeviceDefault);
    }

    /// **A size token survives the trip out and back**, which is the only
    /// property of it that matters.
    ///
    /// `size_token` is unlike [`pick_token`] and [`outcome_token`]: it is not
    /// a fixed vocabulary, it is a measurement written for another process to
    /// read. So this parses it the way `tools/ui-verify` parses it — split on
    /// `x`, two `f64`s — rather than asserting a literal, because a test that
    /// asserted `"595.28x841.89"` would pass on a spelling no consumer could
    /// read back, and that is precisely the failure this token replaced.
    ///
    /// ★ The tolerance is one hundredth of a point, which is the rounding the
    /// two-decimal format applies on purpose. 0.01 pt is 3.5 micron; the fit
    /// tolerance this number is compared against is [`FIT_TOLERANCE_PT`], two
    /// hundred times larger.
    #[test]
    fn a_size_token_reads_back_as_the_number_it_was_written_from() {
        // Deliberately awkward: an exact ISO size with more precision than the
        // token keeps, a whole number that must not lose its decimals, and a
        // value that rounds UP at the second place.
        for (w, h) in [(595.276, 841.89), (1190.0, 1684.0), (612.345_6, 792.0)] {
            let token = size_token(Some((w, h)));
            assert!(
                !token.contains(char::is_whitespace),
                "{token:?} carries whitespace, which truncates the field and every field after it"
            );
            let (left, right) = token
                .split_once('x')
                .unwrap_or_else(|| panic!("{token:?} has no `x` separator"));
            let back: (f64, f64) = (
                left.parse()
                    .unwrap_or_else(|_| panic!("{left:?} is not a number")),
                right
                    .parse()
                    .unwrap_or_else(|_| panic!("{right:?} is not a number")),
            );
            assert!(
                (back.0 - w).abs() <= 0.01 && (back.1 - h).abs() <= 0.01,
                "{token:?} read back as {back:?}, which is not ({w}, {h})"
            );
        }
        assert_eq!(size_token(None), "none");
    }

    /// **`largest=` and `mixed=` say `none`/`off` when auto was never chosen**,
    /// rather than a value that reads like an answer.
    ///
    /// The distinction is the same one [`outcome_token`] makes: *"this job is
    /// not mixed"* and *"nobody asked"* are different facts, and a driven check
    /// that read the first for the second would be asserting a property of a
    /// decision that never ran. A `false` in that slot would be indistinguishable
    /// from a real measurement.
    #[test]
    fn an_unchosen_auto_reports_absence_rather_than_an_answer() {
        for outcome in [AutoPaper::NotChosen, AutoPaper::NoBasis] {
            assert_eq!(largest_token(&outcome), "none", "{outcome:?}");
            assert_eq!(mixed_token(&outcome), "off", "{outcome:?}");
        }

        let single = Match {
            id: 9,
            name: "A4".to_owned(),
            sheet_pt: (595.276, 841.89),
            largest_page_pt: (595.276, 841.89),
            mixed: false,
        };
        let many = Match {
            mixed: true,
            ..single.clone()
        };
        assert_eq!(mixed_token(&AutoPaper::Matched(single.clone())), "no");
        assert_eq!(mixed_token(&AutoPaper::TooBig(many.clone())), "yes");
        // ★ And the page it reports is the page the choice was made FOR, not
        // the sheet it chose — the two are equal in this fixture on purpose,
        // so the assertion below uses a Match where they differ.
        let bigger = Match {
            largest_page_pt: (1190.0, 1684.0),
            ..single
        };
        assert_eq!(largest_token(&AutoPaper::TooBig(bigger)), "1190.00x1684.00");
    }

    /// **The trace tokens are distinct, and contain nothing a parser splits
    /// on.**
    ///
    /// A trace line is `event key=value ...` split on whitespace, so a token
    /// carrying a space would silently truncate the field and every field
    /// after it. Two tokens that collided would be worse: the check would read
    /// a value, believe it, and report a verdict about the wrong state.
    #[test]
    fn the_trace_tokens_are_distinct_and_parseable() {
        let a_match = Match {
            id: 9,
            name: "A4".to_owned(),
            sheet_pt: (595.276, 841.89),
            largest_page_pt: (595.276, 841.89),
            mixed: false,
        };
        let outcomes = [
            outcome_token(&AutoPaper::NotChosen),
            outcome_token(&AutoPaper::NoBasis),
            outcome_token(&AutoPaper::Matched(a_match.clone())),
            outcome_token(&AutoPaper::TooBig(a_match)),
        ];
        let picks = [
            pick_token(PaperChoice::DeviceDefault),
            pick_token(PaperChoice::Form(9)),
            pick_token(PaperChoice::AutoFromPages),
        ];
        for token in outcomes.iter().chain(picks.iter()) {
            assert!(
                !token.is_empty() && token.chars().all(|c| c.is_ascii_lowercase()),
                "{token:?} is not a bare lowercase word"
            );
        }
        let mut seen: Vec<&str> = outcomes.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), outcomes.len(), "two outcomes share a token");
        let mut seen: Vec<&str> = picks.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), picks.len(), "two picks share a token");
    }

    /// **Two forms of the same size resolve to the first the driver listed.**
    ///
    /// Drivers really do enumerate `"A4"` and a borderless twin. The tie-break
    /// is documented as enumeration order, and it is asserted here so that a
    /// later change from `min_by` to `min_by_key` — which does not promise
    /// which of the equal elements it keeps — cannot silently reverse it.
    #[test]
    fn equal_sheets_break_the_tie_on_the_drivers_own_order() {
        let forms = vec![
            PaperForm {
                id: 9,
                name: "A4".to_owned(),
                size_pt: (595.276, 841.89),
            },
            PaperForm {
                id: 260,
                name: "A4 (borderless)".to_owned(),
                size_pt: (595.276, 841.89),
            },
        ];
        assert_eq!(id_of(&choose(&forms, &[(595.276, 841.89)])), Some(9));
    }
}
