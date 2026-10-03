//! What the clipboard becomes on the way to a page: bytes parsed as a
//! document, and bytes written where a verb that reads a file can find them.

use pdfcer_core::document::Document;
use pdfcer_core::page_tree::Page;
use std::path::PathBuf;

/// `bytes` parsed as a document, with its pages.
///
/// # Errors
///
/// The engine's sentence when the bytes do not parse or have no page tree.
pub fn opened(bytes: Vec<u8>) -> Result<(Document, Vec<Page>), String> {
    let doc = Document::from_bytes(bytes).map_err(|e| e.to_string())?;
    let pages = pdfcer_core::page_tree::pages(&doc).map_err(|e| e.to_string())?;
    Ok((doc, pages))
}

/// `bytes` written as `name` in pdfcer's folder under the system's temporary
/// folder, for a verb that reads its source from a file.
///
/// # Errors
///
/// The system's sentence when the folder or the file cannot be written.
pub fn scratch(name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    // ui-text-exempt: a folder name, never displayed
    // temp-path-exempt: the running program's folder, not a test's; callers name each file uniquely.
    let dir = std::env::temp_dir().join("pdfcer-gui");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = dir.join(name);
    std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
    Ok(file)
}
