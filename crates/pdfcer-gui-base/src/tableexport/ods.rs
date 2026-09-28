//! OpenDocument spreadsheet (`.ods`, LibreOffice Calc) bytes for a set of
//! sheets: one `table:table` per [`Sheet`], merges as spanned cells over
//! `table:covered-table-cell`s, header rows bold, numbers as `float` cells.
//!
//! Written by hand: the format is three small XML parts in a zip, and the
//! only ordering rule is that `mimetype` is the archive's first entry, stored
//! uncompressed and with no extra field (ODF 1.2 part 3, §3.3). The zip
//! writer is the minimum that rule needs, over `flate2`, which the engine
//! already links.

use std::fmt::Write as _;
use std::io::Write as _;

use super::{Grid, Sheet, number};

/// The media type, which is also the whole of the `mimetype` entry.
const MIME: &str = "application/vnd.oasis.opendocument.spreadsheet";

// ui-text-exempt: OpenDocument XML markup, never displayed
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
 <manifest:file-entry manifest:full-path="/" manifest:version="1.2" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>
"#;

/// The archive for `sheets`.
///
/// # Errors
/// An archive past zip's 4 GiB or 65,535-entry limits, which only a
/// content.xml of more than 4 GiB could reach.
pub fn bytes(sheets: &[Sheet<'_>]) -> Result<Vec<u8>, String> {
    let mut zip = Zip::default();
    zip.add("mimetype", MIME.as_bytes(), false)?;
    zip.add("META-INF/manifest.xml", MANIFEST.as_bytes(), true)?;
    zip.add("content.xml", content(sheets).as_bytes(), true)?;
    zip.finish()
}

/// A zip archive written front to back: local entries, then the central
/// directory. No extra fields, no data descriptors, a fixed 1980-01-01 time.
#[derive(Default)]
struct Zip {
    out: Vec<u8>,
    central: Vec<u8>,
    entries: u16,
}

/// DOS date for 1980-01-01, the earliest a zip can state.
const DOS_DATE: u16 = (1 << 5) | 1;

impl Zip {
    fn add(&mut self, name: &str, body: &[u8], deflate: bool) -> Result<(), String> {
        let too_big = || crate::text::export_tables::archive_too_large().to_owned();
        let mut crc = flate2::Crc::new();
        crc.update(body);
        let data = if deflate {
            let mut encoder =
                flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(body).map_err(|e| e.to_string())?;
            encoder.finish().map_err(|e| e.to_string())?
        } else {
            body.to_vec()
        };
        let method: u16 = if deflate { 8 } else { 0 };
        let offset = u32::try_from(self.out.len()).map_err(|_| too_big())?;
        let packed = u32::try_from(data.len()).map_err(|_| too_big())?;
        let size = u32::try_from(body.len()).map_err(|_| too_big())?;
        let name_len = u16::try_from(name.len()).map_err(|_| too_big())?;
        // Version needed, flags, method, time, date, CRC, sizes, name length,
        // extra length: shared by the local header and the central record.
        let mut common = Vec::with_capacity(26);
        for half in [20u16, 0, method, 0, DOS_DATE] {
            common.extend_from_slice(&half.to_le_bytes());
        }
        for word in [crc.sum(), packed, size] {
            common.extend_from_slice(&word.to_le_bytes());
        }
        common.extend_from_slice(&name_len.to_le_bytes());
        common.extend_from_slice(&0u16.to_le_bytes());

        self.out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        self.out.extend_from_slice(&common);
        self.out.extend_from_slice(name.as_bytes());
        self.out.extend_from_slice(&data);

        self.central
            .extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        self.central.extend_from_slice(&20u16.to_le_bytes()); // made by
        self.central.extend_from_slice(&common);
        // Comment length, disk number, internal and external attributes.
        for half in [0u16, 0, 0] {
            self.central.extend_from_slice(&half.to_le_bytes());
        }
        self.central.extend_from_slice(&0u32.to_le_bytes());
        self.central.extend_from_slice(&offset.to_le_bytes());
        self.central.extend_from_slice(name.as_bytes());
        self.entries = self.entries.checked_add(1).ok_or_else(too_big)?;
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<u8>, String> {
        let too_big = || crate::text::export_tables::archive_too_large().to_owned();
        let start = u32::try_from(self.out.len()).map_err(|_| too_big())?;
        let length = u32::try_from(self.central.len()).map_err(|_| too_big())?;
        self.out.extend_from_slice(&self.central);
        self.out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        for half in [0u16, 0, self.entries, self.entries] {
            self.out.extend_from_slice(&half.to_le_bytes());
        }
        self.out.extend_from_slice(&length.to_le_bytes());
        self.out.extend_from_slice(&start.to_le_bytes());
        self.out.extend_from_slice(&0u16.to_le_bytes());
        Ok(self.out)
    }
}

/// `content.xml`: the styles every sheet shares, then each sheet.
fn content(sheets: &[Sheet<'_>]) -> String {
    let mut out = String::from(
        // ui-text-exempt: OpenDocument XML markup, never displayed
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" office:version="1.2">
<office:automatic-styles>
<style:style style:name="body" style:family="table-cell"><style:table-cell-properties fo:wrap-option="wrap" style:vertical-align="top"/></style:style>
<style:style style:name="head" style:family="table-cell"><style:table-cell-properties fo:wrap-option="wrap" style:vertical-align="top"/><style:text-properties fo:font-weight="bold"/></style:style>
"#,
    );
    for (s, sheet) in sheets.iter().enumerate() {
        for (c, width) in column_widths(sheet.grid).iter().enumerate() {
            let _ = writeln!(
                out,
                // ui-text-exempt: OpenDocument XML markup, never displayed
                r#"<style:style style:name="c{s}-{c}" style:family="table-column"><style:table-column-properties style:column-width="{width:.2}cm"/></style:style>"#
            );
        }
    }
    out.push_str("</office:automatic-styles>\n<office:body><office:spreadsheet>\n");
    for (s, sheet) in sheets.iter().enumerate() {
        table(&mut out, s, sheet);
    }
    out.push_str("</office:spreadsheet></office:body></office:document-content>\n");
    out
}

/// One `table:table`.
fn table(out: &mut String, s: usize, sheet: &Sheet<'_>) {
    let grid = sheet.grid;
    // ui-text-exempt: OpenDocument XML markup, never displayed
    let _ = writeln!(out, r#"<table:table table:name="{}">"#, escape(&sheet.name));
    for c in 0..grid.cells.first().map_or(0, Vec::len) {
        // ui-text-exempt: OpenDocument XML markup, never displayed
        let _ = writeln!(out, r#"<table:table-column table:style-name="c{s}-{c}"/>"#);
    }
    let covered = covered(grid);
    for (r, row) in grid.cells.iter().enumerate() {
        let style = if r < grid.header_rows { "head" } else { "body" };
        out.push_str("<table:table-row>");
        for (c, text) in row.iter().enumerate() {
            if covered.contains(&(r, c)) {
                out.push_str("<table:covered-table-cell/>");
                continue;
            }
            // ui-text-exempt: OpenDocument XML markup, never displayed
            let _ = write!(out, r#"<table:table-cell table:style-name="{style}""#);
            if let Some(m) = grid.merges.iter().find(|m| m.row == r && m.col == c) {
                let _ = write!(
                    out,
                    // ui-text-exempt: OpenDocument XML markup, never displayed
                    r#" table:number-rows-spanned="{}" table:number-columns-spanned="{}""#,
                    m.rows, m.cols
                );
            }
            if text.is_empty() {
                out.push_str("/>");
                continue;
            }
            if let Some(value) = number(text) {
                // ui-text-exempt: OpenDocument XML markup, never displayed
                let _ = write!(out, r#" office:value-type="float" office:value="{value}""#);
            } else {
                // ui-text-exempt: OpenDocument XML markup, never displayed
                out.push_str(r#" office:value-type="string""#);
            }
            out.push('>');
            for line in text.split('\n') {
                out.push_str("<text:p>");
                paragraph(out, line);
                out.push_str("</text:p>");
            }
            out.push_str("</table:table-cell>");
        }
        out.push_str("</table:table-row>\n");
    }
    out.push_str("</table:table>\n");
}

/// Every position a merge covers other than its top-left.
fn covered(grid: &Grid) -> Vec<(usize, usize)> {
    grid.merges
        .iter()
        .flat_map(|m| {
            (m.row..m.row + m.rows)
                .flat_map(move |r| (m.col..m.col + m.cols).map(move |c| (r, c)))
                .filter(move |&(r, c)| (r, c) != (m.row, m.col))
        })
        .collect()
}

/// A width per column in centimetres, from its longest line: ODF has no
/// autofit, and the default width cuts most drawing-table cells short.
fn column_widths(grid: &Grid) -> Vec<f32> {
    let cols = grid.cells.first().map_or(0, Vec::len);
    (0..cols)
        .map(|c| {
            let chars = grid
                .cells
                .iter()
                .filter_map(|row| row.get(c))
                .flat_map(|text| text.split('\n'))
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0)
                .clamp(4, 60);
            #[allow(clippy::cast_precision_loss)] // at most 60
            let width = chars as f32 * 0.21 + 0.4;
            width
        })
        .collect()
}

/// One line of cell text. ODF collapses runs of spaces, so a run keeps its
/// first space and states the rest as `<text:s text:c="n"/>`; a leading
/// space is stated the same way.
fn paragraph(out: &mut String, line: &str) {
    let mut run = 0usize;
    let flush = |out: &mut String, run: usize, leading: bool| {
        let literal = usize::from(!leading && run > 0);
        if literal == 1 {
            out.push(' ');
        }
        match run - literal {
            0 => {}
            1 => out.push_str("<text:s/>"),
            n => {
                // ui-text-exempt: OpenDocument XML markup, never displayed
                let _ = write!(out, r#"<text:s text:c="{n}"/>"#);
            }
        }
    };
    let mut started = false;
    for ch in line.chars() {
        if ch == ' ' {
            run += 1;
            continue;
        }
        if run > 0 {
            flush(out, run, !started);
            run = 0;
        }
        started = true;
        if ch == '\t' {
            out.push_str("<text:tab/>");
        } else {
            push_escaped(out, ch);
        }
    }
    if run > 0 {
        flush(out, run, !started);
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        push_escaped(&mut out, ch);
    }
    out
}

/// XML-escaped, with the control characters XML 1.0 cannot carry dropped.
fn push_escaped(out: &mut String, ch: char) {
    match ch {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        // ui-text-exempt: OpenDocument XML markup, never displayed
        '"' => out.push_str("&quot;"),
        c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
        c => out.push(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tableexport::Merge;

    fn sample() -> Grid {
        Grid {
            cells: vec![
                vec!["Size".into(), String::new()],
                vec!["12.5".into(), "a  <b>".into()],
            ],
            merges: vec![Merge {
                row: 0,
                col: 0,
                rows: 1,
                cols: 2,
            }],
            header_rows: 1,
        }
    }

    #[test]
    fn a_merge_spans_and_covers_and_a_number_is_a_float() {
        let grid = sample();
        let xml = content(&[Sheet {
            name: "Page 1 table 1".into(),
            grid: &grid,
        }]);
        assert!(xml.contains(r#"table:number-columns-spanned="2""#));
        assert_eq!(xml.matches("<table:covered-table-cell/>").count(), 1);
        assert!(xml.contains(r#"office:value-type="float" office:value="12.5""#));
        assert!(xml.contains(r#"<text:p>a <text:s/>&lt;b&gt;</text:p>"#));
        assert!(xml.contains(r#"table:style-name="head""#));
    }

    #[test]
    fn the_archive_opens_with_an_uncompressed_mimetype() {
        let grid = sample();
        let bytes = bytes(&[Sheet {
            name: "s".into(),
            grid: &grid,
        }])
        .unwrap();
        // Local file header, then the name at 30, then the stored body.
        assert_eq!(&bytes[..4], b"PK\x03\x04");
        assert_eq!(&bytes[30..38], b"mimetype");
        assert_eq!(&bytes[38..38 + MIME.len()], MIME.as_bytes());
        // The end record names three entries and a central directory that
        // starts with a central-record signature.
        let end = &bytes[bytes.len() - 22..];
        assert_eq!(&end[..4], b"PK\x05\x06");
        assert_eq!(u16::from_le_bytes([end[10], end[11]]), 3);
        let start = u32::from_le_bytes([end[16], end[17], end[18], end[19]]) as usize;
        assert_eq!(&bytes[start..start + 4], b"PK\x01\x02");
    }

    #[test]
    fn a_deflated_part_inflates_back_to_its_crc() {
        use std::io::Read as _;
        let grid = sample();
        let sheets = [Sheet {
            name: "s".into(),
            grid: &grid,
        }];
        let bytes = bytes(&sheets).unwrap();
        let expected = content(&sheets);
        // Walk the local headers to content.xml.
        let mut at = 0;
        loop {
            assert_eq!(&bytes[at..at + 4], b"PK\x03\x04");
            let word = |o: usize| u32::from_le_bytes(bytes[at + o..at + o + 4].try_into().unwrap());
            let half =
                |o: usize| usize::from(u16::from_le_bytes([bytes[at + o], bytes[at + o + 1]]));
            let (crc, packed, name_len) = (word(14), word(18) as usize, half(26));
            let name = &bytes[at + 30..at + 30 + name_len];
            let data = &bytes[at + 30 + name_len..at + 30 + name_len + packed];
            if name == b"content.xml" {
                let mut inflated = String::new();
                flate2::read::DeflateDecoder::new(data)
                    .read_to_string(&mut inflated)
                    .unwrap();
                assert_eq!(inflated, expected);
                let mut check = flate2::Crc::new();
                check.update(inflated.as_bytes());
                assert_eq!(check.sum(), crc);
                return;
            }
            at += 30 + name_len + packed;
        }
    }
}
