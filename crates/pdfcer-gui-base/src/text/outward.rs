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

/// The Launch File prompt's title.
#[must_use]
pub const fn launch_title() -> &'static str {
    "Launch File"
}

/// The Launch File prompt's warning, naming the file in full.
#[must_use]
pub fn launch_question(file: &str) -> String {
    format!(
        "The file and its viewing application are set to be launched by this PDF file:\n\n\
         {file}\n\nThe file may contain programs, macros, or viruses that could potentially harm \
         your computer. Only open the file if you are sure it is safe. If this file was placed by \
         a trusted person or program, click Open to view the file."
    )
}

/// Said under the file when it is a PDF, which opens here rather than in another program.
#[must_use]
pub const fn launch_is_pdf() -> &'static str {
    "This is a PDF: it opens here, in a tab of its own."
}

/// The Launch File prompt's checkbox.
#[must_use]
pub const fn launch_dont_ask() -> &'static str {
    "Do not show this message again"
}

/// Hover on the Launch File checkbox.
#[must_use]
pub const fn launch_dont_ask_hover() -> &'static str {
    "With Open, files a link names open without asking from now on. Turn asking back on under \
     Security > Trust > Ask before opening other files."
}

/// The Launch File prompt's opening button.
#[must_use]
pub const fn launch_open() -> &'static str {
    "Open"
}

/// The Launch File prompt's refusing button.
#[must_use]
pub const fn launch_refuse() -> &'static str {
    "Do Not Open"
}

/// Status line after a file was handed to the program Windows opens it with.
#[must_use]
pub fn file_opened(file: &str) -> String {
    format!("Opened {file} in the program Windows uses for it.")
}

/// Status line when Windows would not open the file.
#[must_use]
pub fn file_failed(file: &str, why: &str) -> String {
    format!("Windows could not open {file}: {why}")
}

/// Status line after the operator chose Do Not Open.
#[must_use]
pub fn file_not_opened(file: &str) -> String {
    format!("{file} was not opened.")
}

/// Status line when the file a link names is not there.
#[must_use]
pub fn file_missing(file: &str) -> String {
    format!("This link opens {file}, and there is no such file.")
}

/// Status line when the linked PDF opened but the place it names is not in it.
#[must_use]
pub fn file_place_missing(file: &str, place: &str) -> String {
    format!("Opened {file}, which has no {place}; showing its first page.")
}

/// A page number, 1-based, for [`file_place_missing`]'s hole.
#[must_use]
pub fn place_page(page: usize) -> String {
    format!("page {page}")
}

/// A destination name, for [`file_place_missing`]'s hole.
#[must_use]
pub fn place_named(name: &str) -> String {
    format!("destination named \"{name}\"")
}

/// The 3D prompt's window title.
#[must_use]
pub const fn model_title() -> &'static str {
    "3D Content"
}

/// The 3D prompt's message: Acrobat's document message bar.
#[must_use]
pub const fn model_disabled() -> &'static str {
    "3D content has been disabled. Enable this feature if you trust this document."
}

/// The 3D prompt's default button.
#[must_use]
pub const fn model_trust_once() -> &'static str {
    "Trust this document one time only"
}

/// Hover on [`model_trust_once`].
#[must_use]
pub const fn model_trust_once_hover() -> &'static str {
    "Shows this document's 3D models until the document is closed."
}

/// The 3D prompt's lasting button.
#[must_use]
pub const fn model_trust_always() -> &'static str {
    "Trust this document always"
}

/// Hover on [`model_trust_always`].
#[must_use]
pub const fn model_trust_always_hover() -> &'static str {
    "Shows this document's 3D models whenever it is open, while Play 3D content is off."
}

/// The 3D prompt's refusing button.
#[must_use]
pub const fn model_refuse() -> &'static str {
    "Cancel"
}

/// Status line after the 3D prompt was cancelled.
#[must_use]
pub const fn model_not_shown() -> &'static str {
    "The 3D model was not shown: Play 3D content is off and this document is not trusted."
}

/// Security ▸ Trust ▸ Web sites…'s window title.
#[must_use]
pub const fn sites_title() -> &'static str {
    "Web Sites"
}

/// The web-link mode that asks.
#[must_use]
pub const fn sites_mode_ask() -> &'static str {
    "Ask before opening a web link, unless the site is listed below"
}

/// The web-link mode that allows all.
#[must_use]
pub const fn sites_mode_allow() -> &'static str {
    "Allow all web sites"
}

/// The web-link mode that blocks all.
#[must_use]
pub const fn sites_mode_block() -> &'static str {
    "Block all web sites"
}

/// The heading over the remembered sites.
#[must_use]
pub const fn sites_heading() -> &'static str {
    "Remembered sites"
}

/// Said under the heading while the mode is not Ask.
#[must_use]
pub const fn sites_unused() -> &'static str {
    "These answers are kept but not used while every site is allowed or blocked."
}

/// Said in place of an empty list.
#[must_use]
pub const fn sites_none() -> &'static str {
    "No site has a remembered answer."
}

/// A site row's Allow choice.
#[must_use]
pub const fn sites_allow() -> &'static str {
    "Allow"
}

/// A site row's Block choice.
#[must_use]
pub const fn sites_block() -> &'static str {
    "Block"
}

/// A site row's remove button.
#[must_use]
pub const fn sites_forget() -> &'static str {
    "Forget"
}

/// The window's accept button.
#[must_use]
pub const fn sites_ok() -> &'static str {
    "OK"
}

/// The window's cancel button.
#[must_use]
pub const fn sites_cancel() -> &'static str {
    "Cancel"
}
