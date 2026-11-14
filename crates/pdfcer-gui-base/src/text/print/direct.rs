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
     instead of through the driver. Only plain jobs go this way: a job that \
     needs pdfcer's own layout prints through the driver as usual."
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
    /// Several pages per sheet or a booklet.
    #[must_use]
    pub const fn impose() -> &'static str {
        "Several pages per sheet"
    }
    /// Poster.
    #[must_use]
    pub const fn poster() -> &'static str {
        "Poster"
    }
    /// A percentage or shrink-only scale.
    #[must_use]
    pub const fn scale() -> &'static str {
        "This scale"
    }
    /// A paper other than the printer's own.
    #[must_use]
    pub const fn paper() -> &'static str {
        "Choosing the paper"
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
    /// Reverse order.
    #[must_use]
    pub const fn reverse() -> &'static str {
        "Reverse order"
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
        format!("{head} The printer lays the pages out.")
    } else {
        format!(
            "{head} The printer lays the pages out, and said it ignored: {}.",
            ignored.join(", ")
        )
    }
}
