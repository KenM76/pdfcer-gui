//! Reading the files Security ▸ Add validation evidence… embeds:
//! certificates, CRLs and OCSP responses, as DER or PEM.
//!
//! Contract: [`read`] turns one file into the blobs it holds, each tagged with
//! the `/DSS` array it belongs in. It does not validate the DER — the engine's
//! `add_validation_material` parses every blob before writing and names the
//! one it cannot store — so this module decides only *which kind* a blob is.
//!
//! - **PEM** is recognised by its `-----BEGIN` line, and the label decides the
//!   kind (`X509 CRL` → CRL, `OCSP RESPONSE` → OCSP, anything naming a
//!   certificate → certificate). One file may hold several blocks, as a chain
//!   exported from a browser does. A block with any other label is refused
//!   rather than guessed at.
//! - **DER** carries no label, so the extension decides: `.crl` → CRL,
//!   `.ocsp` / `.ors` / `.resp` → OCSP, everything else → certificate. A
//!   misfiled blob fails the engine's parse, and the refusal names the file.

use pdfcer_core::sign::ltv::MaterialKind;

const BEGIN: &str = "-----BEGIN "; // ui-text-exempt: PEM syntax (RFC 7468), never displayed
const DASHES: &str = "-----"; // ui-text-exempt: PEM syntax (RFC 7468), never displayed
const CRL: &str = "X509 CRL"; // ui-text-exempt: a PEM label (RFC 7468), matched not displayed
const OCSP: &str = "OCSP RESPONSE"; // ui-text-exempt: a PEM label, matched not displayed
const CERTS: [&str; 3] = ["CERTIFICATE", "X509 CERTIFICATE", "TRUSTED CERTIFICATE"]; // ui-text-exempt: PEM labels, matched not displayed

/// Why a file yielded nothing to embed. Displayed after the file's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unreadable {
    /// A PEM block's base64 body does not decode.
    BadBase64,
    /// A PEM block whose label is none of the three kinds.
    UnknownLabel(String),
    /// A `-----BEGIN` with no matching `-----END`.
    Unterminated,
    /// The file is empty.
    Empty,
}

/// The blobs one file holds, in file order.
///
/// # Errors
///
/// [`Unreadable`] when the file is empty or a PEM block cannot be read; no
/// partial result is returned, so one bad block refuses the whole file.
pub fn read(file_name: &str, bytes: &[u8]) -> Result<Vec<(MaterialKind, Vec<u8>)>, Unreadable> {
    if bytes.is_empty() {
        return Err(Unreadable::Empty);
    }
    let text = std::str::from_utf8(bytes).ok();
    match text {
        Some(text) if text.contains(BEGIN) => read_pem(text),
        _ => Ok(vec![(kind_from_extension(file_name), bytes.to_vec())]),
    }
}

/// The kind a label-less DER file is taken to be.
#[must_use]
pub fn kind_from_extension(file_name: &str) -> MaterialKind {
    let ext = std::path::Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("crl") => MaterialKind::Crl,
        Some("ocsp" | "ors" | "resp") => MaterialKind::Ocsp,
        _ => MaterialKind::Certificate,
    }
}

fn kind_from_label(label: &str) -> Option<MaterialKind> {
    match label {
        CRL => Some(MaterialKind::Crl),
        OCSP => Some(MaterialKind::Ocsp),
        _ if CERTS.contains(&label) => Some(MaterialKind::Certificate),
        _ => None,
    }
}

fn read_pem(text: &str) -> Result<Vec<(MaterialKind, Vec<u8>)>, Unreadable> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(BEGIN) {
        let after = &rest[start + BEGIN.len()..];
        let label_end = after.find(DASHES).ok_or(Unreadable::Unterminated)?;
        let label = after[..label_end].trim();
        let body_start = &after[label_end + DASHES.len()..];
        let end_marker = format!("{DASHES}END {label}{DASHES}"); // ui-text-exempt: PEM syntax, matched not displayed
        let body_end = body_start
            .find(&end_marker)
            .ok_or(Unreadable::Unterminated)?;
        let kind =
            kind_from_label(label).ok_or_else(|| Unreadable::UnknownLabel(label.to_owned()))?;
        let der = decode_base64(&body_start[..body_end]).ok_or(Unreadable::BadBase64)?;
        out.push((kind, der));
        rest = &body_start[body_end + end_marker.len()..];
    }
    Ok(out)
}

/// Standard-alphabet base64 (RFC 4648 §4), whitespace ignored, `=` padding
/// optional. `None` on any other character.
fn decode_base64(body: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(body.len() * 3 / 4);
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    for c in body.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            c if c.is_ascii_whitespace() => continue,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((acc >> bits) & 0xFF).ok()?);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decodes_the_rfc_vectors() {
        assert_eq!(decode_base64("Zm9vYmFy").unwrap(), b"foobar");
        assert_eq!(decode_base64("Zm9vYg==").unwrap(), b"foob");
        assert_eq!(decode_base64("Zm9v\r\nYmE=").unwrap(), b"fooba");
        assert!(decode_base64("Zm9v!").is_none());
    }

    #[test]
    fn a_pem_chain_yields_every_block_with_its_labelled_kind() {
        let pem = "-----BEGIN CERTIFICATE-----\nMAA=\n-----END CERTIFICATE-----\n\
                   -----BEGIN X509 CRL-----\nMAE=\n-----END X509 CRL-----\n";
        let got = read("chain.pem", pem.as_bytes()).unwrap();
        assert_eq!(
            got,
            vec![
                (MaterialKind::Certificate, vec![0x30, 0x00]),
                (MaterialKind::Crl, vec![0x30, 0x01]),
            ]
        );
    }

    #[test]
    fn a_pem_label_that_is_none_of_the_three_is_refused_not_guessed() {
        let pem = "-----BEGIN PRIVATE KEY-----\nMAA=\n-----END PRIVATE KEY-----\n";
        assert_eq!(
            read("key.pem", pem.as_bytes()),
            Err(Unreadable::UnknownLabel("PRIVATE KEY".to_owned()))
        );
    }

    #[test]
    fn der_takes_its_kind_from_the_extension() {
        let der = [0x30, 0x00];
        assert_eq!(read("a.CRL", &der).unwrap()[0].0, MaterialKind::Crl);
        assert_eq!(read("a.ocsp", &der).unwrap()[0].0, MaterialKind::Ocsp);
        assert_eq!(read("a.cer", &der).unwrap()[0].0, MaterialKind::Certificate);
        assert_eq!(read("a.cer", &[]), Err(Unreadable::Empty));
    }
}
