//! # `app::clock` — the one place this shell reads a wall clock
//!
//! One function, one caller today, and a module header out of proportion to
//! both — because *"what time is it"* is a question with more wrong answers
//! than it looks, and because the crate below this one has **deliberately
//! refused to answer it**.
//!
//! ## ★★★ Why the engine will not do this and the shell must
//!
//! `pdfcer-core`'s `MarkupNote::modified` takes a PDF date string **from the
//! caller** and its own note says why:
//!
//! > pdfcer will not supply the timestamp, and you should not expect it to.
//!
//! Two reasons, both of which land on this side of the boundary rather than
//! disappearing:
//!
//! 1. **Determinism.** A library that reads a clock cannot be tested by
//!    comparing bytes, and `pdfcer-core`'s whole authoring test suite is byte
//!    comparison. One `SystemTime::now()` inside it would make every
//!    annotation fixture unrepeatable.
//! 2. **Rule 4.** A timestamp is a *claim about when something happened*. If
//!    pdfcer invented one, the file would assert a fact pdfcer made up. Taking
//!    it from the caller makes the claim the caller's.
//!
//! ⇒ **The shell is the caller and the shell is a program a person is sitting
//! in front of.** *"When did I write this comment"* has a true answer here and
//! does not down there. So this module exists, and the obligation it inherits
//! is that the answer must be **true or absent** — never plausible.
//!
//! ## ★★ Why UTC, when the operator is in a time zone
//!
//! Because §7.9.4 permits `Z` and this crate has no way to learn the local
//! offset without a dependency. The three options were:
//!
//! | | |
//! |---|---|
//! | **UTC with `Z`** | correct, unambiguous, reads four hours ahead of his clock in the summer |
//! | local time with no offset | §7.9.4 permits it and it means *"unknown zone"* — a reader in another country cannot order two comments |
//! | local time labelled as UTC | **a lie in the file**, and the one option that is out of the question |
//!
//! ⇒ The third is the tempting one and is the reason this table is written
//! down: it produces the string that looks right to the person who typed the
//! comment, and it is wrong in a way nobody would ever notice until two
//! reviewers in two countries compared notes.
//!
//! ★ The day a timezone crate is worth adding, this function is the only place
//! that changes, and `Z` is a correct value it will be replacing rather than a
//! bug it will be fixing.
//!
//! ## ★ The calendar arithmetic is Hinnant's, not a guess
//!
//! `days_from_civil`'s inverse — the standard days-to-civil algorithm, shifted
//! to a March-based year so the leap day falls at the end and the leap rule
//! needs no special case. It is exact for every date the `i64` range holds and
//! is the same algorithm C++20's `<chrono>` specifies.
//!
//! It is written out rather than approximated because *"good enough for a
//! timestamp"* is how a file ends up claiming 30 February. The engine
//! explicitly does **not** check the calendar — *"accepting a caller's nonsense
//! date is their claim about their own document"* — so nothing downstream will
//! catch an error made here.

use std::time::{SystemTime, UNIX_EPOCH};

/// **Now, as a PDF date string in UTC** — `D:YYYYMMDDHHmmSSZ`.
///
/// `None` if the system clock is before the Unix epoch, which is the one
/// failure this can have and is not a case to paper over: a machine whose
/// clock says 1969 would otherwise write a comment dated during the Apollo
/// programme, and **no timestamp is better than a false one**. The caller
/// omits `/M` entirely on `None`, which §12.5.6.4 permits.
///
/// # Examples
///
/// ```
/// let stamp = pdfcer_gui::app::clock::pdf_date_utc().expect("a sane clock");
/// assert!(stamp.starts_with("D:20"));
/// assert!(stamp.ends_with('Z'));
/// assert_eq!(stamp.len(), 17);
/// ```
#[must_use]
pub fn pdf_date_utc() -> Option<String> {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(format_pdf_date(secs))
}

/// **A moment as `YYYY-MM-DD`, UTC** — the calendar date and nothing else.
///
/// Added 2026-09-05 for [`crate::trust`], which has to print the modification
/// time of the operator's Acrobat trust store so an anchor set that silently
/// went stale is visible rather than merely old.
///
/// # Why a second formatter rather than trimming [`pdf_date_utc`]
///
/// Because they answer different questions and the difference is in the
/// **type**, not the string. `pdf_date_utc` is a PDF date literal — it exists to
/// be written into a document, it carries the `D:` prefix and the `Z` suffix
/// that §7.9.4 requires, and it reads the clock itself because the only moment
/// it can mean is *now*. This one formats a moment the **caller** supplies, for
/// a human to read, and reads no clock at all.
///
/// ★ **Date only, to the day.** The question it answers is *"is this anchor set
/// current?"* — a question about weeks and months, since AATL refreshes are not
/// a daily event — and a timestamp to the second would suggest a precision
/// about staleness that nothing here has.
///
/// Both spellings go through the same [`civil_from_days`], so the two can never
/// disagree about what day a given instant falls on.
///
/// # Examples
///
/// ```
/// assert_eq!(pdfcer_gui::app::clock::iso_date_utc(0), "1970-01-01");
/// // 2024-05-27T00:00:00Z, the date on this operator's own trust store.
/// assert_eq!(pdfcer_gui::app::clock::iso_date_utc(1_716_768_000), "2024-05-27");
/// ```
#[must_use]
pub fn iso_date_utc(unix_secs: u64) -> String {
    let (year, month, day) = civil_from_days(unix_secs / 86_400);
    format!("{year:04}-{month:02}-{day:02}")
}

/// The pure half, so the formatting can be tested without a clock.
///
/// ★ Split out for exactly that reason and for no other. A function that reads
/// the clock and formats it in one body is a function whose formatting can only
/// be tested by asserting on today's date — which is a test that passes for a
/// year and then starts failing at a month boundary for reasons nobody
/// remembers.
#[must_use]
fn format_pdf_date(unix_secs: u64) -> String {
    // Whole days since the epoch, and the seconds within today. Unsigned
    // throughout, so no floor-division subtlety: `unix_secs` is a count of
    // seconds since 1970 and cannot be negative by construction.
    let days = unix_secs / 86_400;
    let rem = unix_secs % 86_400;
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (year, month, day) = civil_from_days(days);
    format!("D:{year:04}{month:02}{day:02}{hour:02}{minute:02}{second:02}Z")
}

/// Days since 1970-01-01 → `(year, month, day)`.
///
/// Howard Hinnant's `civil_from_days`, the algorithm C++20 `<chrono>`
/// specifies. The shift to a **March-based** year is the whole trick: with
/// March as month 1, the leap day lands on the last day of the year, so the
/// day-of-year formula is a single linear expression and the leap rule needs
/// no branch at all.
///
/// ★ The magic numbers are the algorithm's own and are not tunable:
/// `719_468` is the day count from 0000-03-01 to 1970-01-01, `146_097` is the
/// days in a 400-year Gregorian cycle, and `153` and `2` are the coefficients
/// of the linear month-length pattern March..February. Changing any of them
/// does not make it approximate — it makes it wrong.
#[must_use]
fn civil_from_days(days_since_epoch: u64) -> (u64, u64, u64) {
    let z = days_since_epoch + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097; // day of era, 0..=146_096
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // 0..=399
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // 0..=365, March-based
    let mp = (5 * doy + 2) / 153; // 0..=11, March = 0
    let d = doy - (153 * mp + 2) / 5 + 1; // 1..=31
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // March-based back to Jan = 1
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **Known instants, formatted exactly.**
    ///
    /// Four dates chosen for what each one would break: the epoch itself, a
    /// leap day, the day after a leap day, and a century year that is **not** a
    /// leap year. The last is the one a hand-rolled calendar gets wrong — 1900
    /// and 2100 are divisible by four and are common years — and it is why the
    /// algorithm is Hinnant's rather than `days / 365`.
    #[test]
    fn known_instants_format_exactly() {
        assert_eq!(format_pdf_date(0), "D:19700101000000Z");
        // 2024-02-29T12:24:56Z — a leap day.
        //
        // ★ The expected string here was written as `123456` and the code was
        // right: the instant is 12:24:56, not 12:34:56. Worth leaving the note
        // because it is the failure mode this test is FOR — the seconds
        // arithmetic and the calendar arithmetic are independent, and a test
        // whose expectation is derived from the same head that wrote the code
        // checks neither. These four are from an independent implementation.
        assert_eq!(format_pdf_date(1_709_209_496), "D:20240229122456Z");
        // 2024-03-01T00:00:00Z — the day after it.
        assert_eq!(format_pdf_date(1_709_251_200), "D:20240301000000Z");
        // 2100-03-01T00:00:00Z. If 2100 were treated as a leap year this
        // would come out as 2100-02-29.
        assert_eq!(format_pdf_date(4_107_542_400), "D:21000301000000Z");
    }

    /// ★★★ **The string this shell writes is one the ENGINE accepts.**
    ///
    /// Not a re-implementation of `MarkupNote::validate` — the real one, called
    /// on the real output. A format that drifted from what the engine parses
    /// would be refused at author time with `MarkupDateMalformed`, and the
    /// operator would meet it as *"my comment did not save"*.
    ///
    /// ⇒ This is the assertion that makes the two sides of the boundary agree
    /// by test rather than by both files claiming to follow §7.9.4.
    #[test]
    fn the_engine_accepts_what_this_module_writes() {
        let stamp = format_pdf_date(1_756_382_400);
        let note = pdfcer_core::edit::MarkupNote::new("x").at(&stamp);
        assert!(
            note.validate().is_ok(),
            "the engine refused a date this shell would write: {stamp}"
        );
    }

    /// ★ A live clock produces a plausible, well-shaped answer.
    ///
    /// Deliberately weak — it asserts the SHAPE and a lower bound on the year,
    /// never the value. A test that asserted today's date would be a test that
    /// starts failing tomorrow, and the formatting itself is pinned above by
    /// instants that do not move.
    #[test]
    fn the_live_clock_is_shaped_like_a_pdf_date() {
        let stamp = pdf_date_utc().expect("the system clock is after 1970");
        assert_eq!(stamp.len(), 17, "{stamp}");
        assert!(stamp.starts_with("D:20"), "{stamp}");
        assert!(stamp.ends_with('Z'), "{stamp}");
        assert!(
            stamp[2..16].bytes().all(|b| b.is_ascii_digit()),
            "every component is digits: {stamp}"
        );
    }
}
