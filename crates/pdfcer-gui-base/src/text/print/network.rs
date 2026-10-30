//! # `text::print::network` — the words for printing to a network printer

/// The window title.
#[must_use]
pub const fn title() -> &'static str {
    "Send to network printer"
}

/// Above the address field.
#[must_use]
pub const fn address_label() -> &'static str {
    "Printer address"
}

/// Inside the empty address field.
#[must_use]
pub const fn address_hint() -> &'static str {
    "ipp://printer.local/ipp/print"
}

/// Asks the printer what it is.
#[must_use]
pub const fn connect() -> &'static str {
    "Connect"
}

/// While a request is out.
#[must_use]
pub const fn waiting() -> &'static str {
    "Waiting for the printer…"
}

/// The printer answered.
#[must_use]
pub fn connected(model: Option<&str>) -> String {
    model.map_or_else(|| "Connected.".to_owned(), |m| format!("Connected: {m}"))
}

/// What the printer does with the document.
#[must_use]
pub const fn lays_out_note() -> &'static str {
    "The printer receives the PDF and lays it out itself; its own settings decide paper, scaling and which comments print."
}

/// Above the page-range field.
#[must_use]
pub const fn pages_label() -> &'static str {
    "Pages (blank for all)"
}

/// Above the copies field.
#[must_use]
pub const fn copies_label() -> &'static str {
    "Copies"
}

/// Two-sided printing.
#[must_use]
pub const fn two_sided() -> &'static str {
    "Two-sided"
}

/// Monochrome printing.
#[must_use]
pub const fn grey() -> &'static str {
    "In grey"
}

/// Above the fold choice.
#[must_use]
pub const fn fold_label() -> &'static str {
    "Fold"
}

/// Closes the window.
#[must_use]
pub const fn cancel() -> &'static str {
    "Close"
}

/// Sends the document.
#[must_use]
pub const fn send() -> &'static str {
    "Send"
}

/// The job was taken.
#[must_use]
/// `ignored` lists the settings the printer ignored or substituted.
pub fn sent(job_id: Option<i32>, bytes: usize, ignored: &[String]) -> String {
    let kb = bytes.div_ceil(1024);
    let head = match job_id {
        Some(id) => format!("Sent ({kb} KB) as job {id}."),
        None => format!("Sent ({kb} KB)."),
    };
    if ignored.is_empty() {
        head
    } else {
        format!(
            "{head} The printer ignored or substituted: {}",
            ignored.join(", ")
        )
    }
}

/// One fold.
#[must_use]
pub const fn fold(f: pdfcer_print::finishing::Fold) -> &'static str {
    use pdfcer_print::finishing::Fold;
    match f {
        Fold::Half => "In half",
        Fold::Letter => "Letter (tri-fold)",
        Fold::Z => "Z-fold",
        Fold::HalfZ => "Half, then Z",
        Fold::Gate => "Gate",
        Fold::DoubleGate => "Double gate",
        Fold::LeftGate => "Left gate",
        Fold::RightGate => "Right gate",
        Fold::Accordion => "Accordion",
        Fold::Parallel => "Parallel",
        Fold::Poster => "Poster",
        Fold::EngineeringZ => "Engineering Z-fold",
        _ => "Other",
    }
}

/// The address is not one this window can reach.
#[must_use]
pub fn not_ipp(uri: &str) -> String {
    format!("{uri} is not a printer address. It starts with ipp://, ipps://, http:// or https://.")
}

/// An `ipps://` address in a build without TLS.
#[must_use]
pub fn no_tls(url: &str) -> String {
    format!(
        "{url} needs a secure connection, which this build cannot make. Use the printer's ipp:// address."
    )
}

/// The printer answered with an error status.
#[must_use]
pub fn printer_refused(status: &str, message: Option<&str>) -> String {
    match message {
        Some(m) => format!("The printer refused: {status} ({m})."),
        None => format!("The printer refused: {status}."),
    }
}

/// The printer does not take PDF.
#[must_use]
pub fn no_pdf<S: AsRef<str>>(formats: &[S]) -> String {
    let formats: Vec<&str> = formats.iter().map(AsRef::as_ref).collect();
    format!(
        "This printer does not accept PDF (it takes {}). Print through a Windows printer instead.",
        formats.join(", ")
    )
}

/// A finishing or tray the printer did not list.
#[must_use]
pub fn refused_finishing<S: AsRef<str>>(what: &[S]) -> String {
    let what: Vec<&str> = what.iter().map(AsRef::as_ref).collect();
    format!(
        "The printer does not offer {}. Nothing was sent.",
        what.join("; ")
    )
}

/// A page range that is not in this document.
#[must_use]
pub fn bad_range(part: &str, count: u32) -> String {
    format!("\"{part}\" is not a page or span of pages between 1 and {count}.")
}
