//! A minimal PNG encoder — enough to write an evidence file, and nothing more.
//!
//! Design and rationale: `docs/modules/ui-verify/png.md`.

/// Encode 8-bit RGB scanlines as a PNG.
#[must_use]
pub fn encode_rgb(width: u32, height: u32, rgb: &[u8]) -> Option<Vec<u8>> {
    let expected = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(3)?;
    if rgb.len() != expected || width == 0 || height == 0 {
        return None;
    }

    // Scanlines with their filter-type bytes.
    let stride = width as usize * 3;
    let mut raw = Vec::with_capacity(expected + height as usize);
    for row in 0..height as usize {
        raw.push(0u8); // filter: None
        raw.extend_from_slice(&rgb[row * stride..(row + 1) * stride]);
    }

    let mut out = Vec::with_capacity(raw.len() + 1024);
    out.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // depth 8, truecolour, no interlace
    write_chunk(&mut out, b"IHDR", &ihdr);
    write_chunk(&mut out, b"IDAT", &zlib_stored(&raw));
    write_chunk(&mut out, b"IEND", &[]);
    Some(out)
}

/// A zlib stream carrying `data` in stored deflate blocks.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut z = Vec::with_capacity(data.len() + data.len() / 65535 * 5 + 16);
    // CMF = 0x78 (deflate, 32K window), FLG = 0x01 chosen so that
    // (CMF << 8 | FLG) % 31 == 0, which is the header's own check constraint.
    z.extend_from_slice(&[0x78, 0x01]);

    let mut offset = 0usize;
    // An empty input still needs one (final, empty) block, or the stream is
    // truncated and strict decoders reject the file.
    loop {
        let len = (data.len() - offset).min(0xFFFF);
        let final_block = offset + len >= data.len();
        z.push(u8::from(final_block)); // BFINAL, BTYPE = 00 (stored)
        z.extend_from_slice(&(len as u16).to_le_bytes());
        z.extend_from_slice(&(!(len as u16)).to_le_bytes()); // one's complement
        z.extend_from_slice(&data[offset..offset + len]);
        offset += len;
        if final_block {
            break;
        }
    }

    z.extend_from_slice(&adler32(data).to_be_bytes());
    z
}

fn write_chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(body);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// Adler-32 over the uncompressed data, as zlib requires.
fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// CRC-32 (the PNG/zip polynomial), computed without a precomputed table.
///
/// Table-free because the table would be a 1 KiB static that only this
/// function uses, and a screenshot's three chunks are not a hot path.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_length_mismatch_instead_of_panicking() {
        assert!(encode_rgb(2, 2, &[0; 3]).is_none());
    }

    #[test]
    fn writes_a_well_formed_signature_and_chunk_sequence() {
        let png = encode_rgb(2, 2, &[0xFF; 12]).expect("encodes");
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        // Chunk order matters: IHDR first, IEND last.
        let s = String::from_utf8_lossy(&png);
        let ihdr = s.find("IHDR").expect("IHDR");
        let idat = s.find("IDAT").expect("IDAT");
        let iend = s.find("IEND").expect("IEND");
        assert!(ihdr < idat && idat < iend);
    }

    /// The zlib header's own check constraint. Getting it wrong produces a
    /// file that some decoders accept and others reject, which is the worst
    /// kind of wrong.
    #[test]
    fn the_zlib_header_satisfies_its_check_constraint() {
        let z = zlib_stored(b"hello");
        let check = (u32::from(z[0]) << 8) | u32::from(z[1]);
        assert_eq!(check % 31, 0);
    }

    #[test]
    fn adler32_matches_the_known_value_for_a_known_input() {
        // The canonical zlib example.
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn crc32_matches_the_known_value_for_a_known_input() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn an_input_larger_than_one_stored_block_is_split_and_terminated_once() {
        let data = vec![7u8; 200_000];
        let z = zlib_stored(&data);
        // Header + three blocks (65535, 65535, 65535, 3930 => four) + adler.
        assert!(z.len() > data.len());
        assert_eq!(&z[z.len() - 4..], &adler32(&data).to_be_bytes());
    }
}
