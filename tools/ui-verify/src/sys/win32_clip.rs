//! Writing the clipboard as another program would, so a check can paste
//! content pdfcer did not copy.
//!
//! Contract: [`set_clipboard`] replaces the clipboard with the given formats
//! and marks them as excluded from Windows clipboard history and cloud sync,
//! so a driven check never shows up in the operator's Win+V list. A check that
//! writes the clipboard takes a [`snapshot`] first and hands it to [`restore`]
//! afterwards: the clipboard is the operator's, and a harness that leaves it
//! changed has changed his machine. Only memory-block formats are carried by
//! a snapshot; Windows synthesises the bitmap and metafile handles from them.

use super::win32::with_clipboard;
use windows_sys::Win32::Foundation::GlobalFree;
use windows_sys::Win32::System::DataExchange::{
    EmptyClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardSequenceNumber,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};

/// `CF_UNICODETEXT`.
pub const CF_UNICODETEXT: u32 = 13;
/// `CF_DIB`.
pub const CF_DIB: u32 = 8;
/// `CF_DIBV5`.
pub const CF_DIBV5: u32 = 17;

/// Formats whose clipboard handle is not a memory block, or is private to its
/// owner, and so cannot be copied by [`snapshot`]: bitmap, metafile picture,
/// palette, enhanced metafile, the owner-display and display formats, and the
/// private and GDI-object ranges.
fn is_memory_block(format: u32) -> bool {
    !matches!(format, 2 | 3 | 9 | 14 | 0x80..=0x8E | 0x200..=0x3FF)
}

/// The clipboard's change counter: it differs whenever any program has written
/// the clipboard since it was last read.
#[must_use]
pub fn clipboard_sequence() -> u32 {
    // SAFETY: takes no arguments and touches no memory of ours.
    unsafe { GetClipboardSequenceNumber() }
}

/// The id of the registered clipboard format `name`, registering it if new.
#[must_use]
pub fn register_format(name: &str) -> u32 {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `wide` is a live NUL-terminated UTF-16 string.
    unsafe { RegisterClipboardFormatW(wide.as_ptr()) }
}

/// Copy `bytes` into a new movable memory block, or `None` if allocation fails.
fn global_from(bytes: &[u8]) -> Option<*mut core::ffi::c_void> {
    // SAFETY: a movable block of at least one byte; checked for null below.
    let block = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1)) };
    if block.is_null() {
        return None;
    }
    // SAFETY: `block` is a live movable block of at least `bytes.len()` bytes.
    let ptr = unsafe { GlobalLock(block) }.cast::<u8>();
    if ptr.is_null() {
        // SAFETY: the block was allocated above and never handed out.
        unsafe { GlobalFree(block) };
        return None;
    }
    // SAFETY: `ptr` addresses at least `bytes.len()` writable bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        GlobalUnlock(block);
    }
    Some(block)
}

/// Put one format on the open clipboard. The system owns the block on success.
fn put(format: u32, bytes: &[u8]) -> bool {
    let Some(block) = global_from(bytes) else {
        return false;
    };
    // SAFETY: the clipboard is open and emptied by this task; `block` is a
    // movable memory block, which is what SetClipboardData takes.
    let set = unsafe { SetClipboardData(format, block) };
    if set.is_null() {
        // SAFETY: ownership did not pass to the system, so it is still ours.
        unsafe { GlobalFree(block) };
        return false;
    }
    true
}

/// The three registered formats that keep an entry out of clipboard history,
/// cloud sync and clipboard monitors.
fn privacy_formats() -> [u32; 3] {
    [
        register_format("ExcludeClipboardContentFromMonitorProcessing"), // ui-text-exempt: Windows format name
        register_format("CanIncludeInClipboardHistory"), // ui-text-exempt: Windows format name
        register_format("CanUploadToCloudClipboard"),    // ui-text-exempt: Windows format name
    ]
}

/// Empty the open clipboard, put the privacy markers, then `items` other than
/// markers. Whether every item was placed.
fn replace_open(items: &[(u32, Vec<u8>)]) -> bool {
    // SAFETY: the clipboard is open and owned by this task.
    if unsafe { EmptyClipboard() } == 0 {
        return false;
    }
    let [exclude, history, cloud] = privacy_formats();
    let zero = 0u32.to_le_bytes();
    let marked = put(exclude, &[0]) & put(history, &zero) & put(cloud, &zero);
    items
        .iter()
        .filter(|(format, _)| ![exclude, history, cloud].contains(format))
        .fold(marked, |ok, (format, bytes)| put(*format, bytes) & ok)
}

/// Replace the clipboard with `items`, each `(format, bytes)`, kept out of
/// clipboard history and cloud sync. Whether every item was placed.
pub fn set_clipboard(items: &[(u32, Vec<u8>)]) -> bool {
    with_clipboard(|| replace_open(items)).unwrap_or(false)
}

/// The bytes of `format` on the clipboard, or `None` when it is absent or not
/// a memory block.
#[must_use]
pub fn clipboard_bytes(format: u32) -> Option<Vec<u8>> {
    if !is_memory_block(format) {
        return None;
    }
    with_clipboard(|| read_open(format)).flatten()
}

/// Read `format` from the already-open clipboard.
fn read_open(format: u32) -> Option<Vec<u8>> {
    // SAFETY: the clipboard is open; null means the format is absent.
    let handle = unsafe { GetClipboardData(format) };
    if handle.is_null() {
        return None;
    }
    // SAFETY: a memory-block format's handle is an HGLOBAL; its size and lock
    // are valid until the clipboard closes.
    let size = unsafe { GlobalSize(handle) };
    let ptr = unsafe { GlobalLock(handle) }.cast::<u8>();
    if ptr.is_null() {
        return None;
    }
    // SAFETY: `size` bytes are readable at `ptr` while locked.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, size) }.to_vec();
    // SAFETY: paired with the lock above.
    unsafe { GlobalUnlock(handle) };
    Some(bytes)
}

/// Every memory-block format on the clipboard, with its bytes, for [`restore`].
#[must_use]
pub fn snapshot() -> Vec<(u32, Vec<u8>)> {
    with_clipboard(|| {
        let mut out = Vec::new();
        let mut format = 0;
        loop {
            // SAFETY: the clipboard is open; 0 ends the walk.
            format = unsafe { EnumClipboardFormats(format) };
            if format == 0 || out.len() > 64 {
                break;
            }
            if is_memory_block(format)
                && let Some(bytes) = read_open(format)
            {
                out.push((format, bytes));
            }
        }
        out
    })
    .unwrap_or_default()
}

/// Put back what [`snapshot`] took, kept out of clipboard history so the
/// operator's own entry is not listed twice. An empty snapshot leaves only
/// the markers.
pub fn restore(saved: &[(u32, Vec<u8>)]) -> bool {
    with_clipboard(|| replace_open(saved)).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Writes a private format and reads it back, then restores the clipboard.
    #[test]
    #[ignore = "writes the system clipboard; run by hand"]
    fn a_written_format_reads_back_and_the_clipboard_is_restored() {
        let saved = snapshot();
        let format = register_format("pdfcer.ui-verify.selftest"); // ui-text-exempt: format name
        assert!(set_clipboard(&[(format, b"round trip".to_vec())]));
        assert_eq!(clipboard_bytes(format).as_deref(), Some(&b"round trip"[..]));
        assert_ne!(clipboard_bytes(CF_DIB).as_deref(), Some(&b"round trip"[..]));
        assert!(restore(&saved));
    }
}
