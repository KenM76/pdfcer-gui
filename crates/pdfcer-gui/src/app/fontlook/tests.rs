use super::*;
use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;

/// One line: "BOLD" in Helvetica-Bold 12 pt, "PLAIN" in Helvetica 12 pt and
/// "SMALL" in Helvetica 9 pt.
fn page() -> OpenDoc {
    let content = "BT /F2 12 Tf 10 20 Td (BOLD) Tj ET BT /F1 12 Tf 60 20 Td (PLAIN) Tj ET BT /F1 9 Tf 120 20 Td (SMALL) Tj ET";
    let bodies = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 40] /Contents 4 0 R /Resources << /Font << /F1 5 0 R /F2 6 0 R >> >> >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".to_owned(),
    ];
    let mut buf = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in bodies.iter().enumerate() {
        offsets.push(buf.len());
        buf.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = buf.len();
    buf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", bodies.len() + 1).as_bytes(),
    );
    for o in offsets {
        buf.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    buf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            bodies.len() + 1
        )
        .as_bytes(),
    );
    let doc = Document::from_bytes(buf).expect("parses");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    OpenDoc::new("look.pdf".into(), EditSession::new(doc), pages)
}

fn run_of(doc: &OpenDoc, text: &str) -> usize {
    let page = doc.provenance_page_text(0).expect("text");
    page.runs
        .iter()
        .position(|r| r.text.contains(text))
        .unwrap_or_else(|| panic!("no run reads {text}"))
}

fn span(doc: &OpenDoc, first: &str, last: &str) -> Look {
    let (a, b) = (run_of(doc, first), run_of(doc, last));
    look(
        doc,
        Subject {
            page: 0,
            at: TextPosition::new(a.min(b), 0),
            to: Some(TextPosition::new(a.max(b), usize::MAX)),
            caret: None,
        },
    )
    .expect("letters with provenance")
}

/// A selection inside one run shows its face and size; across two faces the
/// face box goes blank and the shared size stays; across two sizes the size
/// goes blank and the shared face stays.
#[test]
fn a_mixed_selection_blanks_only_what_differs() {
    let doc = page();
    let bold = span(&doc, "BOLD", "BOLD");
    assert_eq!(bold.face, Some(Shown::One("Helvetica-Bold".to_owned())));
    assert_eq!(bold.size, Some(Shown::One(12.0)));
    let faces = span(&doc, "BOLD", "PLAIN");
    assert_eq!(faces.face, Some(Shown::Mixed));
    assert_eq!(faces.size, Some(Shown::One(12.0)));
    let sizes = span(&doc, "PLAIN", "SMALL");
    assert_eq!(sizes.face, Some(Shown::One("Helvetica".to_owned())));
    assert_eq!(sizes.size, Some(Shown::Mixed));
}

/// A caret reads the one letter at it, so a caret in the small run shows
/// 9 pt whatever the first run says.
#[test]
fn a_caret_reads_its_own_run() {
    let doc = page();
    let small = run_of(&doc, "SMALL");
    let at = look(
        &doc,
        Subject {
            page: 0,
            at: TextPosition::new(small, 2),
            to: None,
            caret: Some(3),
        },
    )
    .expect("a letter");
    assert_eq!(at.size, Some(Shown::One(9.0)));
    assert_eq!(at.face, Some(Shown::One("Helvetica".to_owned())));
}
