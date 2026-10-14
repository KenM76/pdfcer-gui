//! Unpacking an OCR add-on zip into a folder: the stored and deflated entries
//! of a plain (non-ZIP64) archive, each checked against its CRC-32 and size.
//!
//! Contract:
//! - Every entry lands under `into`; a name that is absolute, holds `..`, a
//!   drive or stream colon, or a NUL is refused before anything is written.
//! - An encrypted entry, a method other than stored or deflate, a ZIP64 or
//!   multi-disk archive, or a duplicate name is refused.
//! - On a refusal, files already written stay; the caller unpacks into a
//!   staging folder it deletes.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const EOCD_SIG: u32 = 0x0605_4b50;
const CDH_SIG: u32 = 0x0201_4b50;
const LFH_SIG: u32 = 0x0403_4b50;
const ZIP64_LOCATOR_SIG: u32 = 0x0706_4b50;
const EOCD_LEN: u64 = 22;
/// The end record plus the longest comment it may carry.
const TAIL_LEN: u64 = EOCD_LEN + 65_535;
/// More entries than any add-on holds; a ceiling against a hostile archive.
pub const MAX_ENTRIES: usize = 20_000;

/// Why an archive was refused.
#[derive(Debug)]
pub enum ZipError {
    /// Reading the archive or writing a file failed.
    Io(PathBuf, io::Error),
    /// No end-of-archive record: not a zip, or truncated.
    NotZip,
    /// A ZIP64 or multi-disk archive.
    Zip64,
    /// More than [`MAX_ENTRIES`] entries.
    TooManyEntries,
    /// The named entry is encrypted.
    Encrypted(String),
    /// The named entry uses this compression method.
    Method(String, u16),
    /// The named entry's path would land outside the folder.
    UnsafePath(String),
    /// The named entry's bytes do not match its recorded size or CRC-32, or
    /// its name repeats.
    Corrupt(String),
}

/// What [`unpack`] wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unpacked {
    /// Files written.
    pub files: usize,
    /// Their total size in bytes.
    pub bytes: u64,
}

struct Entry {
    name: String,
    flags: u16,
    method: u16,
    crc: u32,
    packed: u64,
    size: u64,
    offset: u64,
}

/// Unpack the archive at `zip` under `into`, which must exist.
///
/// # Errors
///
/// [`ZipError`] for the first entry or structure refused.
pub fn unpack(zip: &Path, into: &Path) -> Result<Unpacked, ZipError> {
    let io_err = |e| ZipError::Io(zip.to_path_buf(), e);
    let mut file = File::open(zip).map_err(io_err)?;
    let entries = central_directory(&mut file).map_err(|e| e.at(zip))?;
    let mut out = Unpacked { files: 0, bytes: 0 };
    for entry in &entries {
        let rel = safe_path(&entry.name).ok_or_else(|| ZipError::UnsafePath(entry.name.clone()))?;
        let target = into.join(&rel);
        if entry.name.ends_with('/') {
            std::fs::create_dir_all(&target).map_err(|e| ZipError::Io(target.clone(), e))?;
            continue;
        }
        if entry.flags & 1 != 0 {
            return Err(ZipError::Encrypted(entry.name.clone()));
        }
        if entry.method != 0 && entry.method != 8 {
            return Err(ZipError::Method(entry.name.clone(), entry.method));
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ZipError::Io(parent.to_path_buf(), e))?;
        }
        extract(&mut file, entry, &target).map_err(|e| e.at(zip))?;
        out.files += 1;
        out.bytes += entry.size;
    }
    Ok(out)
}

/// A read failure inside the archive, before the archive path is known.
enum Raw {
    Io(io::Error),
    Zip(ZipError),
}

impl Raw {
    fn at(self, zip: &Path) -> ZipError {
        match self {
            Self::Io(e) => ZipError::Io(zip.to_path_buf(), e),
            Self::Zip(z) => z,
        }
    }
}

impl From<io::Error> for Raw {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ZipError> for Raw {
    fn from(z: ZipError) -> Self {
        Self::Zip(z)
    }
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn central_directory(file: &mut File) -> Result<Vec<Entry>, Raw> {
    let len = file.seek(SeekFrom::End(0))?;
    if len < EOCD_LEN {
        return Err(ZipError::NotZip.into());
    }
    let tail_len = len.min(TAIL_LEN);
    file.seek(SeekFrom::Start(len - tail_len))?;
    let mut tail = vec![0; usize::try_from(tail_len).unwrap_or(0)];
    file.read_exact(&mut tail)?;
    let at = (0..=tail.len() - 22)
        .rev()
        .find(|&i| u32_at(&tail, i) == EOCD_SIG)
        .ok_or(ZipError::NotZip)?;
    let eocd = &tail[at..];
    let zip64_locator = at >= 20 && u32_at(&tail, at - 20) == ZIP64_LOCATOR_SIG;
    let (disk, cd_disk) = (u16_at(eocd, 4), u16_at(eocd, 6));
    let count = u16_at(eocd, 10);
    let (cd_size, cd_offset) = (u32_at(eocd, 12), u32_at(eocd, 16));
    if zip64_locator
        || disk != 0
        || cd_disk != 0
        || count == u16::MAX
        || cd_size == u32::MAX
        || cd_offset == u32::MAX
    {
        return Err(ZipError::Zip64.into());
    }
    if usize::from(count) > MAX_ENTRIES {
        return Err(ZipError::TooManyEntries.into());
    }
    if u64::from(cd_offset) + u64::from(cd_size) > len {
        return Err(ZipError::NotZip.into());
    }
    file.seek(SeekFrom::Start(u64::from(cd_offset)))?;
    let mut cd = vec![0; usize::try_from(cd_size).map_err(|_| ZipError::NotZip)?];
    file.read_exact(&mut cd)?;
    parse_entries(&cd, count)
}

fn parse_entries(cd: &[u8], count: u16) -> Result<Vec<Entry>, Raw> {
    let mut entries = Vec::with_capacity(usize::from(count));
    let mut at = 0;
    for _ in 0..count {
        if at + 46 > cd.len() || u32_at(cd, at) != CDH_SIG {
            return Err(ZipError::NotZip.into());
        }
        let h = &cd[at..];
        let (name_len, extra_len, comment_len) = (
            usize::from(u16_at(h, 28)),
            usize::from(u16_at(h, 30)),
            usize::from(u16_at(h, 32)),
        );
        let end = at + 46 + name_len + extra_len + comment_len;
        if end > cd.len() {
            return Err(ZipError::NotZip.into());
        }
        let name = String::from_utf8_lossy(&h[46..46 + name_len]).into_owned();
        let (packed, size, offset) = (u32_at(h, 20), u32_at(h, 24), u32_at(h, 42));
        if packed == u32::MAX || size == u32::MAX || offset == u32::MAX {
            return Err(ZipError::Zip64.into());
        }
        entries.push(Entry {
            name,
            flags: u16_at(h, 8),
            method: u16_at(h, 10),
            crc: u32_at(h, 16),
            packed: u64::from(packed),
            size: u64::from(size),
            offset: u64::from(offset),
        });
        at = end;
    }
    Ok(entries)
}

/// `name` as a relative path of plain components, or `None` when it could
/// land outside the folder it is unpacked into.
#[must_use]
pub fn safe_path(name: &str) -> Option<PathBuf> {
    let name = name.replace('\\', "/");
    if name.starts_with('/') || name.contains('\0') || name.contains(':') {
        return None;
    }
    let mut out = PathBuf::new();
    for part in name.split('/').filter(|p| !p.is_empty()) {
        if part == "." || part == ".." {
            return None;
        }
        out.push(part);
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

fn extract(file: &mut File, entry: &Entry, target: &Path) -> Result<(), Raw> {
    file.seek(SeekFrom::Start(entry.offset))?;
    let mut lfh = [0u8; 30];
    file.read_exact(&mut lfh)?;
    if u32_at(&lfh, 0) != LFH_SIG {
        return Err(ZipError::Corrupt(entry.name.clone()).into());
    }
    let skip = u64::from(u16_at(&lfh, 26)) + u64::from(u16_at(&lfh, 28));
    file.seek(SeekFrom::Current(i64::try_from(skip).unwrap_or(0)))?;
    let packed = (&mut *file).take(entry.packed);
    let mut reader: Box<dyn Read + '_> = if entry.method == 8 {
        Box::new(flate2::read::DeflateDecoder::new(packed))
    } else {
        Box::new(packed)
    };
    let mut out = match File::create_new(target) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            return Err(ZipError::Corrupt(entry.name.clone()).into());
        }
        Err(e) => return Err(ZipError::Io(target.to_path_buf(), e).into()),
    };
    let mut crc = flate2::Crc::new();
    let mut written = 0u64;
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|_| ZipError::Corrupt(entry.name.clone()))?;
        if n == 0 {
            break;
        }
        written += n as u64;
        if written > entry.size {
            return Err(ZipError::Corrupt(entry.name.clone()).into());
        }
        crc.update(&buf[..n]);
        out.write_all(&buf[..n])
            .map_err(|e| ZipError::Io(target.to_path_buf(), e))?;
    }
    if written != entry.size || crc.sum() != entry.crc {
        return Err(ZipError::Corrupt(entry.name.clone()).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("pdfcer-addonzip-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        dir
    }

    /// A zip of `(name, bytes, deflate)` entries, written the way every zip
    /// tool writes one: local headers, then the central directory, then the
    /// end record.
    fn zip(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut cd = Vec::new();
        for (name, bytes, deflate) in entries {
            let mut crc = flate2::Crc::new();
            crc.update(bytes);
            let packed = if *deflate {
                let mut e =
                    flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
                e.write_all(bytes).expect("deflate");
                e.finish().expect("deflate")
            } else {
                bytes.to_vec()
            };
            let method: u16 = if *deflate { 8 } else { 0 };
            let offset = out.len() as u32;
            let common = |v: &mut Vec<u8>| {
                v.extend_from_slice(&0u16.to_le_bytes()); // flags
                v.extend_from_slice(&method.to_le_bytes());
                v.extend_from_slice(&[0; 4]); // time, date
                v.extend_from_slice(&crc.sum().to_le_bytes());
                v.extend_from_slice(&(packed.len() as u32).to_le_bytes());
                v.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                v.extend_from_slice(&(name.len() as u16).to_le_bytes());
                v.extend_from_slice(&0u16.to_le_bytes()); // extra
            };
            out.extend_from_slice(&LFH_SIG.to_le_bytes());
            out.extend_from_slice(&20u16.to_le_bytes());
            common(&mut out);
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&packed);
            cd.extend_from_slice(&CDH_SIG.to_le_bytes());
            cd.extend_from_slice(&[20, 0, 20, 0]);
            common(&mut cd);
            cd.extend_from_slice(&[0; 6]); // comment length, disk, internal attrs
            cd.extend_from_slice(&[0; 4]); // external attrs
            cd.extend_from_slice(&offset.to_le_bytes());
            cd.extend_from_slice(name.as_bytes());
        }
        let cd_offset = out.len() as u32;
        out.extend_from_slice(&cd);
        out.extend_from_slice(&EOCD_SIG.to_le_bytes());
        out.extend_from_slice(&[0; 4]);
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(cd.len() as u32).to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }

    fn write_zip(dir: &Path, bytes: &[u8]) -> PathBuf {
        let path = dir.join("addon.zip");
        std::fs::write(&path, bytes).expect("zip");
        path
    }

    #[test]
    fn stored_and_deflated_entries_unpack_byte_for_byte() {
        let dir = scratch("ok");
        let big = b"pdfcer ".repeat(5000);
        let path = write_zip(
            &dir,
            &zip(&[
                ("paddle/", b"", false),
                ("paddle/dict.txt", b"a\nb\n", false),
                ("paddle/rec.onnx", &big, true),
            ]),
        );
        let into = dir.join("out");
        std::fs::create_dir_all(&into).expect("out");
        let got = unpack(&path, &into).expect("unpack");
        assert_eq!(
            got,
            Unpacked {
                files: 2,
                bytes: 4 + big.len() as u64
            }
        );
        assert_eq!(
            std::fs::read(into.join("paddle/rec.onnx")).expect("rec"),
            big
        );
        assert_eq!(
            std::fs::read(into.join("paddle/dict.txt")).expect("dict"),
            b"a\nb\n"
        );
    }

    #[test]
    fn a_name_climbing_out_of_the_folder_is_refused_before_writing() {
        let dir = scratch("climb");
        let path = write_zip(&dir, &zip(&[("../evil.txt", b"x", false)]));
        let into = dir.join("out");
        std::fs::create_dir_all(&into).expect("out");
        assert!(matches!(unpack(&path, &into), Err(ZipError::UnsafePath(_))));
        assert!(!dir.join("evil.txt").exists());
        for bad in ["/abs", "C:/x", "a/../../b", "a\\..\\b", "./"] {
            assert_eq!(safe_path(bad), None, "{bad}");
        }
        assert_eq!(
            safe_path("a\\b/c"),
            Some(Path::new("a").join("b").join("c"))
        );
    }

    #[test]
    fn a_damaged_entry_is_refused() {
        let dir = scratch("crc");
        let mut bytes = zip(&[("m/f.bin", b"0123456789", false)]);
        let at = bytes
            .windows(10)
            .position(|w| w == b"0123456789")
            .expect("data");
        bytes[at] = b'X';
        let path = write_zip(&dir, &bytes);
        let into = dir.join("out");
        std::fs::create_dir_all(&into).expect("out");
        assert!(matches!(unpack(&path, &into), Err(ZipError::Corrupt(_))));
    }

    #[test]
    fn a_file_that_is_not_a_zip_is_said_to_be_one() {
        let dir = scratch("notzip");
        let path = write_zip(&dir, b"%PDF-1.7 not an archive at all, just some bytes");
        assert!(matches!(unpack(&path, &dir), Err(ZipError::NotZip)));
    }
}
