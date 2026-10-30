//! # `printspooler::network` — printing to a network printer by its IPP address
//!
//! The printer receives the PDF itself and lays it out (RFC 8011 Print-Job),
//! so nothing of the Print dialog's sheet composition applies. [`query`]
//! asks the printer what it is and what it offers; [`send`] checks the
//! request against that answer, sends, and returns the receipt. Every
//! exchange runs on the caller's thread and may take up to [`TIMEOUT`]; the
//! dialog calls both from a worker.
//!
//! `ipp://` goes over the engine's plain-HTTP transport; `ipps://` needs
//! TLS, which `pdfcer-fetch` supplies when the `ipps` feature is on.

use std::time::Duration;

use pdfcer_print::finishing::{Edge, Finishing, Fold, Staple, ipp_finishings};
use pdfcer_print::ipp::{self, IppJob, IppResponse, Sides};

/// The connect and per-read bound for one exchange; a send allows five
/// times this for the upload.
pub const TIMEOUT: Duration = Duration::from_secs(60);

const STAPLES: [Staple; 9] = [
    Staple::TopLeft,
    Staple::TopRight,
    Staple::BottomLeft,
    Staple::BottomRight,
    Staple::DualLeft,
    Staple::DualRight,
    Staple::DualTop,
    Staple::DualBottom,
    Staple::Saddle,
];
const EDGES: [Edge; 4] = [Edge::Left, Edge::Top, Edge::Right, Edge::Bottom];
const FOLDS: [Fold; 12] = [
    Fold::Half,
    Fold::Letter,
    Fold::Z,
    Fold::HalfZ,
    Fold::Gate,
    Fold::DoubleGate,
    Fold::LeftGate,
    Fold::RightGate,
    Fold::Accordion,
    Fold::Parallel,
    Fold::Poster,
    Fold::EngineeringZ,
];

/// What a network printer said about itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetPrinter {
    /// `printer-make-and-model`, when reported.
    pub model: Option<String>,
    /// The document formats it lists; empty when it lists none.
    pub formats: Vec<String>,
    pub two_sided: bool,
    pub monochrome: bool,
    pub staples: Vec<Staple>,
    pub punch: Vec<Edge>,
    pub folds: Vec<Fold>,
    /// `output-bin-supported` keywords.
    pub bins: Vec<String>,
    /// `finishings-supported`, or `None` when the printer does not say.
    finishings: Option<Vec<i32>>,
}

impl NetPrinter {
    /// Whether the printer takes PDF: it says so, or lists no formats at all.
    #[must_use]
    pub fn accepts_pdf(&self) -> bool {
        self.formats.is_empty() || self.formats.iter().any(|f| f == "application/pdf")
    }

    fn from_response(r: &IppResponse) -> Self {
        let strings = |name: &str| -> Vec<String> {
            r.printer_strings(name)
                .into_iter()
                .map(str::to_owned)
                .collect()
        };
        let finishings = r.printer_ints("finishings-supported");
        let offered = |f: &Finishing| {
            finishings.is_some() && ipp_finishings(f, finishings.as_deref()).is_ok()
        };
        let with = |set: &dyn Fn(&mut Finishing)| {
            let mut f = Finishing::default();
            set(&mut f);
            f
        };
        Self {
            model: strings("printer-make-and-model").into_iter().next(),
            formats: strings("document-format-supported"),
            two_sided: strings("sides-supported")
                .iter()
                .any(|s| s.starts_with("two-sided")),
            monochrome: strings("print-color-mode-supported")
                .iter()
                .any(|s| s == "monochrome"),
            staples: STAPLES
                .into_iter()
                .filter(|&s| offered(&with(&|f| f.staple = Some(s))))
                .collect(),
            punch: EDGES
                .into_iter()
                .filter(|&e| offered(&with(&|f| f.punch = Some(e))))
                .collect(),
            folds: FOLDS
                .into_iter()
                .filter(|&d| offered(&with(&|f| f.fold = Some(d))))
                .collect(),
            bins: strings("output-bin-supported"),
            finishings,
        }
    }
}

/// What to print and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetJob {
    pub copies: u16,
    /// `None` leaves the printer's default.
    pub sides: Option<Sides>,
    /// 1-based inclusive; empty for every page.
    pub ranges: Vec<(u32, u32)>,
    pub monochrome: bool,
    pub finishing: Finishing,
}

/// What the printer said when it took the job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetReceipt {
    pub job_id: Option<i32>,
    /// Attributes the printer reported it ignored or substituted.
    pub ignored: Vec<String>,
    pub bytes: usize,
}

/// Ask the printer at `uri` what it is and what it offers.
///
/// # Errors
///
/// The address is not an IPP address, the printer cannot be reached, or it
/// answered with an error status; the sentence names which.
pub fn query(uri: &str) -> Result<NetPrinter, String> {
    let r = exchange(uri, &ipp::get_printer_attributes(uri, &user(), 1), TIMEOUT)?;
    Ok(NetPrinter::from_response(&r))
}

/// Check `job` against `printer`'s answer and send `pdf` as `name`.
///
/// # Errors
///
/// The printer does not take PDF, refuses a finishing or bin it did not
/// list, cannot be reached, or rejects the job. Nothing is sent when the
/// check fails.
pub fn send(
    uri: &str,
    name: &str,
    pdf: &[u8],
    job: &NetJob,
    printer: &NetPrinter,
) -> Result<NetReceipt, String> {
    if !printer.accepts_pdf() {
        return Err(crate::text::print::network::no_pdf(&printer.formats));
    }
    let mut out = IppJob::new(uri, &user(), name);
    out.copies = job.copies.max(1);
    out.sides = job.sides;
    out.page_ranges.clone_from(&job.ranges);
    out.monochrome = job.monochrome;
    out.finishings = ipp_finishings(&job.finishing, printer.finishings.as_deref())
        .map_err(|refused| crate::text::print::network::refused_finishing(&refused))?;
    if let Some(bin) = &job.finishing.output_bin {
        if !printer.bins.iter().any(|b| b == bin) {
            return Err(crate::text::print::network::refused_finishing(
                std::slice::from_ref(bin),
            ));
        }
        out.output_bin = Some(bin.clone());
    }
    let r = exchange(uri, &out.print_job(2, pdf), TIMEOUT * 5)?;
    let receipt = NetReceipt {
        job_id: r.job_id(),
        ignored: r.unsupported().into_iter().map(str::to_owned).collect(),
        bytes: pdf.len(),
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        let finishings: Vec<String> = out.finishings.iter().map(|f| f.to_string()).collect();
        format!(
            "netprint-sent job_id={} bytes={} copies={} finishings={} bin={} ignored={}",
            receipt
                .job_id
                .map_or("none".to_owned(), |id| id.to_string()),
            receipt.bytes,
            out.copies,
            if finishings.is_empty() {
                "none".to_owned()
            } else {
                finishings.join(",")
            },
            out.output_bin.as_deref().unwrap_or("none"),
            receipt.ignored.len()
        )
    });
    Ok(receipt)
}

/// The requesting user: `USERNAME` or `USER`, else `pdfcer`.
fn user() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "pdfcer".to_owned()) // ui-text-exempt: an IPP user name, never displayed
}

/// One request to `uri`, decoded; an error status is an `Err` naming it.
fn exchange(uri: &str, message: &[u8], timeout: Duration) -> Result<IppResponse, String> {
    let url = ipp::http_url(uri).ok_or_else(|| crate::text::print::network::not_ipp(uri))?;
    let raw = post(&url, message, timeout)?;
    let r = ipp::parse_response(&raw).map_err(|e| e.to_string())?;
    if r.is_success() {
        return Ok(r);
    }
    Err(crate::text::print::network::printer_refused(
        ipp::status_name(r.status),
        r.status_message(),
    ))
}

fn post(url: &str, message: &[u8], timeout: Duration) -> Result<Vec<u8>, String> {
    if url.to_ascii_lowercase().starts_with("https://") {
        return post_tls(url, message, timeout);
    }
    ipp::post_ipp_http(url, message, timeout).map_err(|e| e.to_string())
}

#[cfg(feature = "ipps")]
fn post_tls(url: &str, message: &[u8], timeout: Duration) -> Result<Vec<u8>, String> {
    pdfcer_fetch::post_ipp(url, message, timeout).map_err(|e| e.to_string())
}

#[cfg(not(feature = "ipps"))]
fn post_tls(url: &str, _message: &[u8], _timeout: Duration) -> Result<Vec<u8>, String> {
    Err(crate::text::print::network::no_tls(url))
}

/// `text` as 1-based inclusive page ranges within `count` pages: `""` is
/// every page; otherwise comma-separated numbers and `a-b` spans.
///
/// # Errors
///
/// The part that is not a page number of this document.
pub fn parse_ranges(text: &str, count: u32) -> Result<Vec<(u32, u32)>, String> {
    let mut out = Vec::new();
    for part in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let bad = || crate::text::print::network::bad_range(part, count);
        let (a, b) = match part.split_once('-') {
            Some((a, b)) => (a.trim().parse::<u32>(), b.trim().parse::<u32>()),
            None => (part.parse::<u32>(), part.parse::<u32>()),
        };
        let (Ok(a), Ok(b)) = (a, b) else {
            return Err(bad());
        };
        if a == 0 || b < a || b > count {
            return Err(bad());
        }
        out.push((a, b));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_print::ipp::{IppAttribute, IppGroup, IppValue, group, tag};

    fn answer(attrs: Vec<IppAttribute>) -> IppResponse {
        let bytes = ipp::encode_request(0, 1, &[IppGroup::new(group::PRINTER, attrs)], b"");
        ipp::parse_response(&bytes).unwrap()
    }

    /// The offer is read from `finishings-supported` through the engine's own check.
    #[test]
    fn the_offer_follows_finishings_supported() {
        let r = answer(vec![
            IppAttribute::many(
                "finishings-supported",
                vec![IppValue::Enum(3), IppValue::Enum(20), IppValue::Enum(74)],
            ),
            IppAttribute::new("output-bin-supported", IppValue::keyword("top")),
            IppAttribute::new(
                "document-format-supported",
                IppValue::string(tag::MIME, "application/pdf"),
            ),
        ]);
        let p = NetPrinter::from_response(&r);
        assert_eq!(p.staples, vec![Staple::TopLeft]);
        assert_eq!(p.punch, vec![Edge::Left]);
        assert!(p.folds.is_empty());
        assert_eq!(p.bins, vec!["top".to_owned()]);
        assert!(p.accepts_pdf());
    }

    /// A printer that lists no finishings offers none, rather than all.
    #[test]
    fn no_list_offers_nothing() {
        let p = NetPrinter::from_response(&answer(Vec::new()));
        assert!(p.staples.is_empty() && p.punch.is_empty() && p.folds.is_empty());
    }

    /// A printer listing formats without PDF does not take it.
    #[test]
    fn a_printer_without_pdf_is_refused() {
        let r = answer(vec![IppAttribute::new(
            "document-format-supported",
            IppValue::string(tag::MIME, "image/pwg-raster"),
        )]);
        assert!(!NetPrinter::from_response(&r).accepts_pdf());
    }

    #[test]
    fn ranges_parse_within_the_document() {
        assert_eq!(parse_ranges("", 4), Ok(Vec::new()));
        assert_eq!(parse_ranges("1-2, 4", 4), Ok(vec![(1, 2), (4, 4)]));
        assert!(parse_ranges("0", 4).is_err());
        assert!(parse_ranges("3-5", 4).is_err());
        assert!(parse_ranges("x", 4).is_err());
    }
}
