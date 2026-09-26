//! # `text::export_form` — the words form-data export says
//!
//!
//! ## The sentence this module exists for
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
//! It is a **disclosure**, not a warning, and the wording keeps that
//! distinction. pdfcer did something correct and is saying what it did. A
//! sentence shaped as an alarm would invite the operator to undo a protection
//! they did not ask for and should keep.
//!
//! ## Why the counts are stated at all
//!
//! Because an export is a file the operator cannot see from here. *"Written"*
//! alone is true of a zero-field export and of a four-hundred-field one, and
//! the number is the only thing that distinguishes "it worked" from "it worked
//! on nothing". `export_dxf`'s own outcome sentences make the same argument.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/export_form.md`.

/// The save dialog's title bar.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Export form data — type .fdf, .xfdf or .csv"
}

/// The import dialog's title bar.
#[must_use]
pub const fn import_dialog_title() -> &'static str {
    "Import form data — .fdf, .xfdf or .csv"
}

/// **What an import did, and what it could not find.**
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

/// The bytes were read and are not form data pdfcer can parse.
#[must_use]
pub fn import_unparseable(detail: &str) -> String {
    format!("That file is not form data pdfcer can read: {detail}")
}

/// The engine refused the import outright.
#[must_use]
pub fn import_refused(detail: &str) -> String {
    format!("pdfcer would not import into this document: {detail}")
}

/// The open document carries no `/AcroForm` at all.
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

/// **Values were rewritten so a spreadsheet will not execute them.**
#[must_use]
pub fn neutralised(count: usize, fields: &[String]) -> String {
    let names = name_list(fields);
    format!(
        "{count} value(s) started with a character a spreadsheet reads as a formula, so pdfcer \
         put a quote in front of them: {names}."
    )
}

/// The field names, bounded.
fn name_list(fields: &[String]) -> String {
    if fields.len() <= MAX_NAMED_FIELDS {
        return fields.join(", ");
    }
    let shown = fields[..MAX_NAMED_FIELDS].join(", ");
    let rest = fields.len() - MAX_NAMED_FIELDS;
    format!("{shown} and {rest} more")
}

/// How many field names the neutralisation sentence lists before eliding.
const MAX_NAMED_FIELDS: usize = 4;

/// Where the file went.
#[must_use]
pub fn written_to(path: &str) -> String {
    format!("Written to {path}")
}

/// The write failed, with the operating system's own reason.
#[must_use]
pub fn export_failed(detail: &str) -> String {
    format!("The form data could not be written: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The neutralisation sentence says what was done, not that something
    /// went wrong.**
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
    #[test]
    fn the_two_empty_states_are_told_apart() {
        assert_ne!(no_form(), no_fields());
        assert!(no_fields().contains("no fields"));
    }
}
