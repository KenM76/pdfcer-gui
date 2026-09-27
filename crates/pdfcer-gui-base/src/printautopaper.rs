//! # `printautopaper` — pick the sheet from the pages
//!
//! Operator request **O167**, 2026-09-10: *"we also need the option to auto
//! select paper size based on the page sizes in the pdf."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printautopaper.md`.

use crate::printspooler::{PaperChoice, PaperForm};
use crate::text::print as t;

/// How close two lengths must be, in points, to count as the same size.
///
/// See the module header for why this is 2 pt and not zero.
const FIT_TOLERANCE_PT: f64 = 2.0;

/// **What auto selection decided**, in a form both the plan and the disclosure
/// can read.
#[derive(Debug, Clone, PartialEq)]
pub enum AutoPaper {
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
pub struct Match {
    /// The `dmPaperSize` value to request.
    pub id: u16,
    /// The driver's own name for it. Never an identity — two forms may share
    /// a name — but it is what the operator sees in the list.
    pub name: String,
    /// The physical sheet, in points.
    pub sheet_pt: (f64, f64),
    /// The largest page in the job, in points, at its rotated extent. This is
    /// the page the choice was made for.
    pub largest_page_pt: (f64, f64),
    /// `true` when the job does not have a single page size throughout.
    ///
    /// Drives the extra sentence about the tray flag; see the module header.
    pub mixed: bool,
}

impl AutoPaper {
    /// The concrete paper choice this outcome resolves to.
    pub fn resolved(&self) -> PaperChoice {
        match self {
            Self::Matched(m) | Self::TooBig(m) => PaperChoice::Form(m.id),
            Self::NotChosen | Self::NoBasis => PaperChoice::DeviceDefault,
        }
    }
}

/// **A stable one-word token for what the operator chose**, for the trace.
pub fn pick_token(choice: PaperChoice) -> &'static str {
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
pub fn outcome_token(outcome: &AutoPaper) -> &'static str {
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
pub fn size_token(size: Option<(f64, f64)>) -> String {
    match size {
        // ui-text-exempt: a diagnostic token, never displayed in the UI.
        None => "none".to_owned(),
        Some((w, h)) => format!("{w:.2}x{h:.2}"),
    }
}

/// **The largest page the auto decision measured**, as a [`size_token`].
pub fn largest_token(outcome: &AutoPaper) -> String {
    match outcome {
        AutoPaper::Matched(m) | AutoPaper::TooBig(m) => size_token(Some(m.largest_page_pt)),
        AutoPaper::NotChosen | AutoPaper::NoBasis => size_token(None),
    }
}

/// **Whether the job has more than one page size**, as a stable token.
pub fn mixed_token(outcome: &AutoPaper) -> &'static str {
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
fn area(sheet: (f64, f64)) -> f64 {
    sheet.0 * sheet.1
}

/// **The whole decision**: which sheet this job should be asked for.
pub fn choose(forms: &[PaperForm], page_sizes: &[(f64, f64)]) -> AutoPaper {
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

impl AutoPaper {
    /// **The disclosure sentence for auto paper selection**, for the paper tab.
    pub fn line(&self) -> String {
        match self {
            AutoPaper::Matched(m) => t::paper_auto_matched(&m.name, m.sheet_pt, m.largest_page_pt),
            AutoPaper::TooBig(m) => t::paper_auto_too_big(&m.name, m.sheet_pt, m.largest_page_pt),
            AutoPaper::NoBasis | AutoPaper::NotChosen => t::paper_auto_no_basis().to_owned(),
        }
    }

    /// Does this job have more than one page size?
    pub fn is_mixed(&self) -> bool {
        match self {
            AutoPaper::Matched(m) | AutoPaper::TooBig(m) => m.mixed,
            AutoPaper::NoBasis | AutoPaper::NotChosen => false,
        }
    }
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
    #[test]
    fn the_smallest_sheet_that_holds_the_page_wins() {
        let pages = vec![(595.276, 841.89); 4];
        let outcome = choose(&forms(), &pages);
        assert!(matches!(outcome, AutoPaper::Matched(_)), "{outcome:?}");
        assert_eq!(id_of(&outcome), Some(9));
    }

    /// **A landscape page picks the same sheet as the portrait one.**
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
    #[test]
    fn turning_a_page_does_not_make_a_job_mixed() {
        let pages = vec![(595.276, 841.89), (841.89, 595.276)];
        match choose(&forms(), &pages) {
            AutoPaper::Matched(m) => assert!(!m.mixed),
            other => panic!("expected a match, got {other:?}"),
        }
    }

    /// **A page nothing holds gets the largest sheet AND the shortfall said.**
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
        // And the page it reports is the page the choice was made FOR, not
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
