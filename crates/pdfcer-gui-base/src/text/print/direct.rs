//! # `text::print::direct` — the words for sending the PDF itself

/// The switch on the Pages tab.
#[must_use]
pub const fn switch() -> &'static str {
    "Send the PDF itself when the printer reads it"
}

/// Hover on the switch.
#[must_use]
pub const fn hover() -> &'static str {
    "Some network printers read PDF and lay it out themselves. With this on, \
     pdfcer finds the printer's network address from its Windows port, asks it \
     whether it reads PDF, and if it does sends the document straight to it \
     instead of through the driver. Several pages per sheet, a booklet, a \
     poster, a chosen paper, a percentage scale or reverse order are first \
     laid out by pdfcer as finished sheets. A job that needs pdfcer's own \
     drawing prints through the driver as usual."
}

/// While the printer is being asked.
#[must_use]
pub const fn checking() -> &'static str {
    "Asking the printer whether it reads PDF…"
}

/// The printer reads PDF and this job can go to it.
#[must_use]
pub fn ready(uri: &str, found_from: &str) -> String {
    format!("This job goes to the printer as the PDF, at {uri} (found from {found_from}).")
}

/// The printer reads PDF and this job goes to it as sheets pdfcer lays out
/// for a `w_mm` by `h_mm` paper.
#[must_use]
pub fn ready_composed(uri: &str, found_from: &str, w_mm: u32, h_mm: u32) -> String {
    format!(
        "This job goes to the printer as the PDF, at {uri} (found from {found_from}). pdfcer \
         lays out its sheets for {w_mm} x {h_mm} mm paper, so load that paper."
    )
}

/// The printer reads PDF and this job needs pdfcer's layout.
#[must_use]
pub fn needs_layout(reason: &str) -> String {
    format!("This job prints through the driver: {reason} needs pdfcer's own layout.")
}

/// The printer cannot be sent the PDF.
#[must_use]
pub fn unavailable(why: &str) -> String {
    format!("This job prints through the driver: {why}.")
}

/// The printer lists formats and PDF is not one.
#[must_use]
pub const fn no_pdf() -> &'static str {
    "the printer does not read PDF"
}

/// Why a job needs pdfcer's layout, one phrase each.
pub mod needs {
    /// Print as image.
    #[must_use]
    pub const fn image() -> &'static str {
        "Print as image"
    }
    /// Colour ▸ Separations.
    #[must_use]
    pub const fn separations() -> &'static str {
        "Separations"
    }
    /// Black only.
    #[must_use]
    pub const fn black_only() -> &'static str {
        "Black only"
    }
    /// A printer profile.
    #[must_use]
    pub const fn profile() -> &'static str {
        "the printer profile"
    }
    /// A moved page.
    #[must_use]
    pub const fn moved() -> &'static str {
        "A moved page"
    }
    /// Fixed line width.
    #[must_use]
    pub const fn lines() -> &'static str {
        "Fixed line width"
    }
    /// A print-only header or footer.
    #[must_use]
    pub const fn running() -> &'static str {
        "The header or footer"
    }
    /// Annotations left out or printed alone.
    #[must_use]
    pub const fn scope() -> &'static str {
        "Leaving out the annotations"
    }
    /// Staple, punch or tray chosen through the driver.
    #[must_use]
    pub const fn finishing() -> &'static str {
        "The staple, punch or tray choice"
    }
}

/// After a job went to the printer as the PDF.
#[must_use]
pub fn sent(uri: &str, job_id: Option<i32>, ignored: &[String]) -> String {
    let head = match job_id {
        Some(id) => format!("Sent the PDF itself to the printer at {uri}, as its job {id}."),
        None => format!("Sent the PDF itself to the printer at {uri}."),
    };
    if ignored.is_empty() {
        head
    } else {
        format!(
            "{head} The printer said it ignored: {}.",
            ignored.join(", ")
        )
    }
}

/// After the document itself was sent: the printer placed its pages.
#[must_use]
pub const fn printer_lays_out() -> &'static str {
    "The printer lays the pages out."
}

/// After sheets pdfcer composed were sent: how many sides, and the paper.
#[must_use]
pub fn composed(sides: usize, w_mm: u32, h_mm: u32) -> String {
    format!(
        "pdfcer laid out {sides} sheet side(s) for {w_mm} x {h_mm} mm paper, printed at actual \
         size, so that paper must be loaded."
    )
}

/// The job went through the driver instead: the engine's reason, and the
/// particulars.
#[must_use]
pub fn fell_back(reason: pdfcer_print::pdf_direct::FallbackReason, why: &str) -> String {
    use pdfcer_print::pdf_direct::FallbackReason as R;
    let head = match reason {
        R::DriverOnlyOption => "the job asks for something only the driver can do",
        R::NoNetworkAddress => "the printer's port names no network address",
        R::NoPdf => "the printer does not read PDF",
        R::Unreachable => "no printer answered at its address",
        R::CannotCompose => "its sheets could not be laid out as a PDF",
        _ => "the PDF could not be sent",
    };
    format!("Printed through the driver, not as the PDF: {head} ({why}).")
}
