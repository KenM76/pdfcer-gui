//! # `outward` — handing a link's address to Windows
//!
//! The one place this program asks the operating system to open something a
//! document names. It runs `rundll32 url.dll,FileProtocolHandler <target>`,
//! never `cmd`, so no character in the target is read by a shell. A target
//! carrying a quote or a control character is refused before anything runs.
//!
//! `PDFCER_DIAG_NO_LAUNCH` set to anything makes [`open`] trace
//! `external-open kind=… target=…` and return without launching, so the
//! driven harness can follow a link without a browser appearing.
//!
//! Design and rationale: `DESIGNS.md` § O320.

/// What [`open`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opened {
    /// Windows was asked to open it.
    Launched,
    /// The harness seam is set; nothing ran.
    Suppressed,
}

/// The schemes a browser or mail program answers; anything else is named in
/// the prompt as not a web page.
const WEB_SCHEMES: [&str; 4] = ["http", "https", "mailto", "ftp"];

/// The scheme of `uri`, lower-cased, or `None` when it has none.
#[must_use]
pub fn scheme(uri: &str) -> Option<String> {
    let (scheme, _) = uri.split_once(':')?;
    let valid = scheme
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    valid.then(|| scheme.to_ascii_lowercase())
}

/// Whether `uri` is an address a browser or mail program answers.
#[must_use]
pub fn is_web(uri: &str) -> bool {
    scheme(uri).is_some_and(|s| WEB_SCHEMES.contains(&s.as_str()))
}

/// The site an answer about `uri` is remembered under: the host, without
/// user, password or port, for an address with one; otherwise the scheme and
/// the address up to any query, so `mailto:a@b.com` is its own site.
#[must_use]
pub fn site(uri: &str) -> String {
    let uri = uri.trim();
    if let Some((_, rest)) = uri.split_once("://") {
        let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
        let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
        let host = if host.starts_with('[') {
            host.split_inclusive(']').next().unwrap_or(host)
        } else {
            host.split(':').next().unwrap_or(host)
        };
        return host.to_ascii_lowercase();
    }
    uri.split(['?', '#'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Whether `target` may be handed to Windows: not empty, no quote, no
/// control character.
#[must_use]
pub fn handable(target: &str) -> bool {
    !target.trim().is_empty() && !target.chars().any(|c| c == '"' || c.is_control())
}

/// **Ask Windows to open `target`**, a web address or a file path. `kind`
/// is the trace's word for what it is (`web`, `file`).
///
/// # Errors
/// A sentence for the status line: the target was refused as unsafe to hand
/// over, or the program could not be started.
pub fn open(kind: &str, target: &str) -> Result<Opened, String> {
    if !handable(target) {
        return Err(crate::text::outward::not_handable().to_owned());
    }
    // ui-text-exempt: an environment variable name, never displayed.
    if std::env::var_os("PDFCER_DIAG_NO_LAUNCH").is_some() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("external-open kind={kind} target={target}")
        });
        return Ok(Opened::Suppressed);
    }
    launch(target).map(|()| Opened::Launched)
}

/// **The file a link's file specification names**, as a path on this machine.
///
/// §7.11.2's form `/C/dir/file.pdf` is drive `C:`; a Windows path or a
/// `file:///` address is taken as written; anything else is relative and
/// resolved against `folder`, the linking document's folder, as Acrobat does.
/// With no folder (an unsaved document) a relative name stays relative.
#[must_use]
pub fn resolve_file(spec: &str, folder: Option<&std::path::Path>) -> std::path::PathBuf {
    let spec = spec.trim();
    let spec = spec
        .strip_prefix("file:///") // ui-text-exempt: a URL scheme prefix, never displayed.
        .unwrap_or(spec);
    let bytes = spec.as_bytes();
    // `/C/…` or a bare `/C`: a one-letter first component is a drive.
    if bytes.len() >= 2
        && bytes[0] == b'/'
        && bytes[1].is_ascii_alphabetic()
        && (bytes.len() == 2 || bytes[2] == b'/')
    {
        let rest = spec.get(3..).unwrap_or_default();
        return std::path::PathBuf::from(format!("{}:/{rest}", char::from(bytes[1])));
    }
    let path = std::path::Path::new(spec);
    let windows_absolute = (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        || spec.starts_with("\\\\");
    if path.is_absolute() || windows_absolute {
        return path.to_path_buf();
    }
    folder.map_or_else(|| path.to_path_buf(), |f| f.join(path))
}

/// Whether `path` names a PDF, which this program opens itself.
#[must_use]
pub fn is_pdf(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf")) // ui-text-exempt: a file extension
}

#[cfg(windows)]
fn launch(target: &str) -> Result<(), String> {
    // ui-text-exempt: program and entry-point names, never displayed.
    std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", target])
        .spawn()
        .map(drop)
        .map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn launch(_target: &str) -> Result<(), String> {
    Err(crate::text::outward::no_launcher().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_site_is_the_bare_host() {
        assert_eq!(
            site("https://User:pw@Example.COM:8443/a?b#c"),
            "example.com"
        );
        assert_eq!(site("http://example.com"), "example.com");
        assert_eq!(
            site("mailto:Someone@Example.com?subject=x"),
            "mailto:someone@example.com"
        );
    }

    #[test]
    fn web_schemes_are_told_from_others() {
        assert!(is_web("HTTPS://example.com"));
        assert!(is_web("mailto:a@b.c"));
        assert!(!is_web("file:///C:/x.exe"));
        assert!(!is_web("no scheme here"));
        assert_eq!(
            scheme("ms-settings:display").as_deref(),
            Some("ms-settings")
        );
    }

    #[test]
    fn a_quote_or_control_character_is_never_handed_over() {
        assert!(handable("https://example.com/a b"));
        assert!(!handable("https://example.com/\" & calc"));
        assert!(!handable("https://example.com/\n"));
        assert!(!handable("  "));
    }

    #[test]
    fn a_file_specification_resolves_against_the_documents_folder() {
        let folder = std::path::Path::new("D:/drawings");
        assert_eq!(
            resolve_file("other.pdf", Some(folder)),
            folder.join("other.pdf")
        );
        assert_eq!(
            resolve_file("../sheets/a.pdf", Some(folder)),
            folder.join("../sheets/a.pdf")
        );
        assert_eq!(
            resolve_file("/C/plans/b.pdf", Some(folder)),
            std::path::PathBuf::from("C:/plans/b.pdf")
        );
        assert_eq!(
            resolve_file("E:\\x\\c.dwg", Some(folder)),
            std::path::PathBuf::from("E:\\x\\c.dwg")
        );
        assert_eq!(
            resolve_file("file:///F:/d.pdf", Some(folder)),
            std::path::PathBuf::from("F:/d.pdf")
        );
        assert_eq!(
            resolve_file("\\\\server\\share\\e.pdf", Some(folder)),
            std::path::PathBuf::from("\\\\server\\share\\e.pdf")
        );
        assert_eq!(
            resolve_file("rel.pdf", None),
            std::path::PathBuf::from("rel.pdf")
        );
    }

    #[test]
    fn a_pdf_is_known_by_its_extension_in_any_case() {
        assert!(is_pdf(std::path::Path::new("a/B.PDF")));
        assert!(is_pdf(std::path::Path::new("c.pdf")));
        assert!(!is_pdf(std::path::Path::new("payload.exe")));
        assert!(!is_pdf(std::path::Path::new("pdf")));
    }
}
