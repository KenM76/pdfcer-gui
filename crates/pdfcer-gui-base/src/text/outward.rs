//! # `text::outward` — every word said when a document reaches outside itself
//!
//! The Security Warning a web link raises, the status lines that follow an
//! answer, and the words of Security ▸ Trust. The prompt's sentences are
//! Acrobat's own, so an operator who knows that prompt knows this one.

/// The web-link prompt's title.
#[must_use]
pub const fn web_title() -> &'static str {
    "Security Warning"
}

/// The web-link prompt's question, naming the address in full.
#[must_use]
pub fn web_question(uri: &str) -> String {
    format!(
        "The document is trying to connect to:\n\n{uri}\n\nIf you trust this site, choose \
         Allow. If you do not trust this site, choose Block."
    )
}

/// Said under the address when it is not a web address.
#[must_use]
pub fn web_not_a_site(scheme: &str) -> String {
    format!("This is not a web page: it asks Windows to open a \"{scheme}:\" address.")
}

/// The prompt's remember checkbox.
#[must_use]
pub const fn web_remember() -> &'static str {
    "Remember this action for this site for all PDF documents"
}

/// Hover on the remember checkbox.
#[must_use]
pub const fn web_remember_hover() -> &'static str {
    "Your answer is kept for this site. Change it under Security > Trust > Web sites."
}

/// The prompt's allowing button.
#[must_use]
pub const fn web_allow() -> &'static str {
    "Allow"
}

/// The prompt's refusing button.
#[must_use]
pub const fn web_block() -> &'static str {
    "Block"
}

/// Status line after a link opened.
#[must_use]
pub fn web_opened(uri: &str) -> String {
    format!("Opened {uri} in your web browser.")
}

/// Status line after a link was blocked by the operator's remembered answer
/// or blanket setting, saying where to change it.
#[must_use]
pub fn web_refused(host: &str) -> String {
    format!(
        "The link to {host} was not opened, as you chose earlier. Change this under Security > \
         Trust > Web sites."
    )
}

/// Status line after the operator chose Block in the prompt.
#[must_use]
pub fn web_blocked(host: &str) -> String {
    format!("The link to {host} was not opened.")
}

/// Status line when Windows would not open the address.
#[must_use]
pub fn web_failed(uri: &str, why: &str) -> String {
    format!("Windows could not open {uri}: {why}")
}

/// Status line for a link whose `/URI` could not be read.
#[must_use]
pub const fn web_unreadable() -> &'static str {
    "This link is meant to open a web address, but the address in the file cannot be read."
}

/// Why an address was not handed to Windows at all.
#[must_use]
pub const fn not_handable() -> &'static str {
    "The address contains a quote mark or a control character, so it was not handed to Windows."
}

/// Said off Windows, where there is nothing to hand an address to.
#[must_use]
pub const fn no_launcher() -> &'static str {
    "Opening addresses outside pdfcer is available on Windows only."
}
