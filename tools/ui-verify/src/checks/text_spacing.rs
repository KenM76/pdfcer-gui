//! `text_spacing` — **sweep text, open Properties › Spacing and position,
//! press Superscript, and the file changes.** Driven by
//! `text_render_mode::drive` with its own controls.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/text_spacing.md`.

use crate::checks::text_render_mode::{Restyle, drive};
use crate::checks::{Check, CheckContext};
use crate::report::CheckReport;

const SUPERSCRIPT: Restyle = Restyle {
    stem: "text_spacing",
    opener: "properties.text.spacing",
    entry: "properties.text.script.super",
    change: "script",
    control: "the spacing fold",
    pick: "Superscript",
    refusal: "The fixture's text is a simple font",
    done: "the swept text is now superscript, through `format_text`",
};

/// See the module documentation.
pub struct SuperscriptReachesTheDocument;

impl Check for SuperscriptReachesTheDocument {
    fn name(&self) -> &'static str {
        "text_spacing"
    }

    fn defect(&self) -> &'static str {
        "Properties › Spacing and position offers Superscript for the swept text, and \
         pressing it commits nothing to the document"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, &SUPERSCRIPT) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}
