//! # `text::export_form` — the words form-data export says
//!
//! `file.export_form_data`, wired 2026-08-27. Its verb is
//! [`crate::app::actions::export::form_data`], whose header carries the design;
//! this file carries the copy.
//!
//! ## ★★★ The sentence this module exists for
//!
//! [`neutralised`]. Everything else here is a count and a path.
//!
//! `formcsv::to_csv` rewrites any value beginning `=`, `+`, `-` or `@` so a
//! spreadsheet does not execute it as a formula when the file is opened. That
//! is the right thing to do — it is a real and well-documented injection route
//! — and doing it **silently** would leave an operator believing their exported
//! data is byte-identical to what the form holds. It is not.
//!
//! Rule 4's half that survives: *inferences the operator cannot see still owe
//! an off-canvas report.* A neutralised value looks completely ordinary in the
//! CSV; nothing about the file says a character was added. So the count is
//! stated and the fields are named.
//!
//! ★ It is a **disclosure**, not a warning, and the wording keeps that
//! distinction. pdfcer did something correct and is saying what it did. A
//! sentence shaped as an alarm would invite the operator to undo a protection
//! they did not ask for and should keep.
//!
//! ## ★ Why the counts are stated at all
//!
//! Because an export is a file the operator cannot see from here. *"Written"*
//! alone is true of a zero-field export and of a four-hundred-field one, and
//! the number is the only thing that distinguishes "it worked" from "it worked
//! on nothing". `export_dxf`'s own outcome sentences make the same argument.

/// The save dialog's title bar.
///
/// ★ It names all three formats, because the dialog is where the format is
/// **chosen** — by the extension — and a title saying only "Export form data"
/// would leave an operator who wants CSV with no way to know they may ask for
/// it. The one place this can be said is the one window they are looking at.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Export form data — type .fdf, .xfdf or .csv"
}

/// The import dialog's title bar.
#[must_use]
pub const fn import_dialog_title() -> &'static str {
    "Import form data — .fdf, .xfdf or .csv"
}

/// ★★★ **What an import did, and what it could not find.**
///
/// The two numbers are not decoration and the second is the important one: a
/// data file may legitimately name a **superset** of this document's fields —
/// that is the ordinary case when one FDF fills a family of related forms — and
/// `import_form_data` counts those and skips them rather than failing.
///
/// So an operator who imports forty values into a thirty-field form gets
/// thirty filled and ten skipped, and **nothing anywhere else would tell
/// them**. A sentence saying only "imported" would be true and would hide the
/// ten fields they thought they were setting.
///
/// ★ `skipped` is mentioned only when it is non-zero. The overwhelming case is
/// a file that matches, and a bar that narrated "0 skipped" would be adding a
/// number to be ignored.
#[must_use]
pub fn imported(applied: usize, skipped: usize) -> String {
    if skipped == 0 {
        format!("Imported {applied} field value(s).")
    } else {
        format!(
            "Imported {applied} field value(s). {skipped} name(s) in the file are not fields in \
             this document and were left alone."
        )
    }
}

/// The file could not be read from disk.
#[must_use]
pub fn import_unreadable(detail: &str) -> String {
    format!("That file could not be read: {detail}")
}

/// ★★ The bytes were read and are not form data pdfcer can parse.
///
/// Distinct from [`import_unreadable`], and the distinction is the operator's
/// next move: an unreadable file is a permissions or a path problem, and an
/// unparseable one means they picked the wrong file or the format is one pdfcer
/// does not read. The remedies share nothing.
#[must_use]
pub fn import_unparseable(detail: &str) -> String {
    format!("That file is not form data pdfcer can read: {detail}")
}

/// The engine refused the import outright.
///
/// ★ Its own sentence rather than folding into [`import_unparseable`], because
/// this is a refusal about the **document** — no form, a certification that
/// forbids filling, an encrypted file — rather than about the data file. An
/// operator told their data file was bad when their document is certified would
/// go and re-export it, twice.
#[must_use]
pub fn import_refused(detail: &str) -> String {
    format!("pdfcer would not import into this document: {detail}")
}

/// The open document carries no `/AcroForm` at all.
///
/// ★ Distinct from [`no_fields`], and the two are not pedantry: a document with
/// no form has nothing to export and never will until fields are added, while a
/// document with an empty form is one somebody has already started. The remedy
/// differs, so the sentence does.
#[must_use]
pub const fn no_form() -> &'static str {
    "This document has no form, so there are no values to export."
}

/// There is an `/AcroForm` and it holds no fields.
#[must_use]
pub const fn no_fields() -> &'static str {
    "This document's form has no fields in it yet, so there is nothing to export."
}

/// FDF written.
///
/// ★ The format is named in the operator's terms — *"the format Acrobat
/// uses"* — because `FDF` is an acronym that tells somebody who does not
/// already know it precisely nothing, and the reason to pick it over the other
/// two is exactly that other software reads it.
#[must_use]
pub fn wrote_fdf(fields: usize) -> String {
    format!("Exported {fields} field value(s) as FDF, the format Acrobat reads.")
}

/// XFDF written.
#[must_use]
pub fn wrote_xfdf(fields: usize) -> String {
    format!("Exported {fields} field value(s) as XFDF, the XML form of the same data.")
}

/// CSV written.
#[must_use]
pub fn wrote_csv(fields: usize) -> String {
    format!("Exported {fields} field value(s) as CSV, for a spreadsheet.")
}

/// ★★★ **Values were rewritten so a spreadsheet will not execute them.**
///
/// See the module header. The three things this sentence has to carry:
///
/// **How many**, because one is a curiosity and forty is a form somebody has
/// been putting expressions into on purpose.
///
/// **Which**, because the operator may need to check the value survived
/// intelligibly — a part number `-40C` is a legitimate value that a spreadsheet
/// would otherwise read as arithmetic, and its owner should know it now reads
/// with a leading quote.
///
/// **What was done**, in the passive voice of a thing pdfcer did rather than a
/// thing that went wrong. It is a protection, and an operator who reads it as
/// an error will go looking for a way to switch it off.
///
/// ★ The field list is **elided in the middle** past a few names. A status line
/// is one line; naming four hundred fields would push everything else off it,
/// and the first and last names are what an operator scans to recognise the
/// group.
#[must_use]
pub fn neutralised(count: usize, fields: &[String]) -> String {
    let names = name_list(fields);
    format!(
        "{count} value(s) started with a character a spreadsheet reads as a formula, so pdfcer \
         put a quote in front of them: {names}."
    )
}

/// The field names, bounded.
///
/// ★ It keeps the FIRST few and says how many were dropped, rather than
/// sampling from the middle or the end. A form's field names share a prefix —
/// `Revision.Row0.Date`, `Revision.Row1.Date` — so the opening names are what
/// identify the group, and an operator who recognises the prefix does not need
/// the rest.
fn name_list(fields: &[String]) -> String {
    if fields.len() <= MAX_NAMED_FIELDS {
        return fields.join(", ");
    }
    let shown = fields[..MAX_NAMED_FIELDS].join(", ");
    let rest = fields.len() - MAX_NAMED_FIELDS;
    format!("{shown} and {rest} more")
}

/// How many field names the neutralisation sentence lists before eliding.
///
/// Four. Enough to recognise a group — a revision table's four columns are the
/// commonest case this fires on — and few enough that the sentence still fits a
/// status line beside the count that precedes it.
const MAX_NAMED_FIELDS: usize = 4;

/// Where the file went.
///
/// ★ Its own sentence rather than a clause on the format line, because the two
/// answer different questions and an operator scanning for *"where is it?"*
/// should not have to read past *"what is it?"*.
#[must_use]
pub fn written_to(path: &str) -> String {
    format!("Written to {path}")
}

/// The write failed, with the operating system's own reason.
///
/// ★ The OS string is passed through rather than re-worded, for
/// `export_dxf::export_failed`'s reason: *"access is denied"* and *"the device
/// is not ready"* are different problems with different remedies, and a
/// generic *"could not write the file"* throws away the only part an operator
/// can act on.
#[must_use]
pub fn export_failed(detail: &str) -> String {
    format!("The form data could not be written: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **The neutralisation sentence says what was done, not that something
    /// went wrong.**
    ///
    /// The failure this guards is a rewording toward alarm. pdfcer performed a
    /// protection the operator did not ask for and should keep; a sentence
    /// containing "error", "failed" or "warning" would invite them to go
    /// looking for the switch that turns it off.
    #[test]
    fn the_neutralisation_disclosure_reads_as_an_act_not_an_alarm() {
        let line = neutralised(3, &["A".to_owned(), "B".to_owned()]);
        for alarm in ["error", "failed", "warning", "danger", "unsafe"] {
            assert!(
                !line.to_lowercase().contains(alarm),
                "the disclosure reads as an alarm: {line}"
            );
        }
        assert!(line.contains('3'), "it must say how many: {line}");
        assert!(line.contains('A'), "and which: {line}");
    }

    /// **A long field list is elided rather than allowed to run off the bar.**
    ///
    /// ★ Asserted against a real shape rather than a token: a form whose every
    /// field is formula-shaped is a revision table with forty rows, and that is
    /// the case that would otherwise push the count off the line.
    #[test]
    fn a_long_field_list_is_bounded() {
        let many: Vec<String> = (0..40).map(|i| format!("Revision.Row{i}.Date")).collect();
        let line = neutralised(many.len(), &many);
        assert!(line.len() < 240, "too long for a status line: {line}");
        assert!(
            line.contains("Revision.Row0.Date"),
            "the first name must survive, so the group is recognisable: {line}"
        );
    }

    /// **"No form" and "an empty form" are different sentences.**
    ///
    /// They describe states with different remedies — add a form, or add fields
    /// to the one you have — and a single sentence covering both would be
    /// vague about the only thing the operator needs.
    #[test]
    fn the_two_empty_states_are_told_apart() {
        assert_ne!(no_form(), no_fields());
        assert!(no_fields().contains("no fields"));
    }
}
