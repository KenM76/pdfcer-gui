//! Reading the clipboard: one memory-block format at a time, and the sequence
//! number that says whether anything has changed since a given moment.
//!
//! Contract: [`get`] copies the bytes out while the clipboard is open and
//! closes it before returning, so no handle outlives the call. Formats whose
//! handle is not a memory block (bitmap, metafile, palette) are never read.

use super::win32::{OpenGuard, register};
use crate::Format;
use std::ffi::c_void;

type Handle = *mut c_void;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetClipboardData(format: u32) -> Handle;
    fn GetClipboardSequenceNumber() -> u32;
    fn IsClipboardFormatAvailable(format: u32) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalSize(mem: Handle) -> usize;
    fn GlobalLock(mem: Handle) -> *mut c_void;
    fn GlobalUnlock(mem: Handle) -> i32;
}

/// The numeric id of `format`, or `None` for a name Windows will not register.
fn id(format: Format) -> Option<u32> {
    match format {
        Format::Predefined(id) => Some(id),
        Format::Registered(name) => register(name),
    }
}

/// Whether `id`'s handle is an `HGLOBAL`; GDI and private formats are not.
fn is_memory_block(id: u32) -> bool {
    !matches!(id, 2 | 3 | 9 | 14 | 0x80..=0x8E | 0x200..=0x3FF)
}

pub(crate) fn sequence() -> u32 {
    // SAFETY: takes no arguments and touches no memory of ours.
    unsafe { GetClipboardSequenceNumber() }
}

pub(crate) fn available(format: Format) -> bool {
    // SAFETY: a plain query; needs no open clipboard.
    id(format).is_some_and(|id| unsafe { IsClipboardFormatAvailable(id) } != 0)
}

pub(crate) fn get(format: Format) -> Option<Vec<u8>> {
    let id = id(format).filter(|id| is_memory_block(*id))?;
    if !available(format) {
        return None;
    }
    let _open = OpenGuard::acquire()?;
    // SAFETY: the clipboard is open (held by `_open`); null means absent.
    let handle = unsafe { GetClipboardData(id) };
    if handle.is_null() {
        return None;
    }
    // SAFETY: `id` is a memory-block format, so `handle` is an HGLOBAL owned
    // by the clipboard and valid until it closes, which `_open` defers.
    let size = unsafe { GlobalSize(handle) };
    // SAFETY: as above; paired with the unlock below.
    let ptr = unsafe { GlobalLock(handle) }.cast::<u8>();
    if ptr.is_null() {
        return None;
    }
    // SAFETY: `size` bytes are readable at `ptr` while the block is locked.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, size) }.to_vec();
    // SAFETY: paired with the lock above.
    unsafe { GlobalUnlock(handle) };
    Some(bytes)
}
