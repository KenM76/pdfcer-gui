//! # `native_window::clipboard` — **a picture other programs can paste**
//!
//! ## What this closes
//!
//! The operator, 2026-08-31 (`OPERATOR_REQUESTS.md` **O71**):
//!
//! > *"In read mode the regular pointer should also allow us to select images
//! > so we can copy and paste them as well as text outside of the pdfcergui."*
//!
//! *"Outside of pdfcer-gui"* is the whole requirement. pdfcer has had an internal
//! clipboard since 2026-08-20 and it is the right thing for pdfcer→pdfcer work —
//! it carries a `MarkupSpec`, an `ObjectClip`, structure a bitmap cannot. None
//! of it means anything to Word, Outlook or Paint, and those are where the
//! operator is going.
//!
//! ## ★★★ THE TRAP: putting an image on the clipboard KILLS `Ctrl+V`
//!
//! Not "degrades". Stops it arriving at all.
//!
//! `egui-winit` turns `Ctrl+V` into `egui::Event::Paste(contents)` **only when
//! the OS clipboard holds non-empty text**, and it returns before pushing a key
//! event either way. So with a clipboard holding a picture and no text, the
//! keystroke vanishes: no paste event, no key event, nothing. `canvas::clipboard`
//! already discovered this from the other direction and puts a marker string on
//! the OS clipboard purely so the chord survives.
//!
//! ⇒ Which is why this function takes **both** and writes them in **one**
//! clipboard transaction. Two calls — one for the picture, one for the text —
//! cannot work: `EmptyClipboard` is per-open and the second would erase the
//! first. A picture that arrived by making internal paste stop working would be
//! a trade nobody asked for.
//!
//! ## What is written, and why those two formats
//!
//! | format | for |
//! |---|---|
//! | `CF_DIB` | every Windows program that pastes a picture. Word, Outlook, Paint, Excel, LibreOffice |
//! | `CF_UNICODETEXT` | pdfcer's own `Ctrl+V`, per the trap above — and a plain-text fallback anywhere else |
//!
//! **No PNG format, deliberately, and it is a stated gap rather than an
//! oversight.** `CF_DIB` cannot carry transparency that consumers agree about,
//! so the caller composites onto white before calling here. Registering
//! `"PNG"` as well would preserve alpha for the few applications that prefer it
//! (modern Office, browsers) and needs a PNG encoder, which this crate does not
//! have and which is not worth a dependency for a case nobody has reported. It
//! is written down here so the next person meets a decision rather than a hole.
//!
//! ## ★★ Why the pixels arrive as BGRA and leave as BGRX
//!
//! `CF_DIB` is a `BITMAPINFOHEADER` followed by pixel data, and the header this
//! writes declares **32 bits per pixel, `BI_RGB`, negative height**:
//!
//! - **32-bit** so every row is `width × 4` bytes and is 4-byte aligned by
//!   construction. A 24-bit DIB needs each row padded to a multiple of four,
//!   and a caller who forgets is the classic diagonal-smear bug.
//! - **Negative height** means top-down, matching how every raster in this
//!   codebase is already laid out. A positive height would need the rows
//!   reversed here, and a bottom-up DIB written top-down is an image that
//!   pastes upside down — plausible-looking and completely wrong.
//! - The fourth byte is **unused**, not alpha. `BI_RGB` at 32bpp has no agreed
//!   alpha meaning: some consumers read it, most ignore it, and one that
//!   ignores it renders a composited-on-black picture where the caller expected
//!   white. Compositing before the call is the only version of this that looks
//!   the same everywhere.

use std::ffi::c_void;

/// `CF_DIB` — a device-independent bitmap, header first.
const CF_DIB: u32 = 8;
/// `CF_UNICODETEXT` — UTF-16, NUL-terminated.
const CF_UNICODETEXT: u32 = 13;
/// `GMEM_MOVEABLE`, which is what the clipboard requires.
const GMEM_MOVEABLE: u32 = 0x0002;
/// `BI_RGB` — uncompressed.
const BI_RGB: u32 = 0;

/// An opaque global-memory handle.
type Handle = *mut c_void;

/// `BITMAPINFOHEADER`, laid out exactly as `wingdi.h` declares it.
///
/// ★ `#[repr(C)]` is load-bearing, not decoration: this struct is read by the
/// operating system and by every program that pastes. Rust's default layout is
/// unspecified, and a reordered field here would be a picture that pastes as
/// noise on some future compiler with nothing in this repository to catch it.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pels_per_meter: i32,
    y_pels_per_meter: i32,
    clr_used: u32,
    clr_important: u32,
}

// ★★ **Two libraries, named explicitly**, where `win32.rs` names none.
//
// That module's four symbols are all in `user32`, which the Rust toolchain
// links by default on this target; `kernel32` it links too. The clipboard's
// eight are split across BOTH — the clipboard itself is `user32`, the moveable
// memory it takes is `kernel32` — and the first build of this file failed to
// link with eight `LNK2019 unresolved external symbol` errors.
//
// ⇒ So they are declared per library rather than in one block. Being explicit
// is also the honest form: a reader can see which DLL each call crosses into,
// and a symbol that moves libraries in a future SDK fails at link time here
// rather than at run time somewhere else.
#[link(name = "user32")]
unsafe extern "system" {
    fn OpenClipboard(owner: Handle) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(format: u32, mem: Handle) -> Handle;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> Handle;
    fn GlobalFree(mem: Handle) -> Handle;
    fn GlobalLock(mem: Handle) -> *mut c_void;
    fn GlobalUnlock(mem: Handle) -> i32;
}

/// **Put a picture and a string on the clipboard, in one transaction.**
///
/// `rgba` is `width × height × 4` bytes, **top-down**, in RGBA order with the
/// alpha already composited away by the caller (see the module header). `text`
/// is what a text-pasting program gets, and is also what keeps this
/// application's own `Ctrl+V` working.
///
/// Returns `false` if the clipboard could not be opened — which is a real and
/// transient condition, because the clipboard is a single system-wide resource
/// and another process may hold it — or if the pixel buffer's length does not
/// match its declared size. A caller that gets `false` has put nothing on the
/// clipboard and should say so rather than assume.
///
/// # ★ Why the size mismatch is a refusal rather than a truncation
///
/// Because a truncated DIB is a picture: the wrong one, at the wrong size, made
/// of whatever bytes followed. Refusing keeps the previous clipboard content,
/// which the operator can still paste, instead of replacing it with garbage.
#[must_use]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    reason = "a bitmap dimension is checked against i32::MAX before the cast, and a byte count for an image that fits in memory fits in u32" // ui-text-exempt: a lint justification, never displayed
)]
pub fn set_image_and_text(rgba: &[u8], width: u32, height: u32, text: &str) -> bool {
    let (Ok(w), Ok(h)) = (i32::try_from(width), i32::try_from(height)) else {
        return false;
    };
    if w <= 0 || h <= 0 {
        return false;
    }
    let Some(pixels) = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
    else {
        return false;
    };
    if rgba.len() != pixels {
        return false;
    }

    // ---- the DIB: header, then BGRX rows, top-down -----------------------
    let header = BitmapInfoHeader {
        size: core::mem::size_of::<BitmapInfoHeader>() as u32,
        width: w,
        // Negative: top-down. See the module header.
        height: -h,
        planes: 1,
        bit_count: 32,
        compression: BI_RGB,
        size_image: pixels as u32,
        ..BitmapInfoHeader::default()
    };
    let mut dib = Vec::with_capacity(core::mem::size_of::<BitmapInfoHeader>() + pixels);
    // SAFETY: reading a `#[repr(C)]` POD struct as its own bytes. No padding
    // is uninitialised — every field is written above.
    dib.extend_from_slice(unsafe {
        core::slice::from_raw_parts(
            std::ptr::from_ref(&header).cast::<u8>(),
            core::mem::size_of::<BitmapInfoHeader>(),
        )
    });
    for px in rgba.chunks_exact(4) {
        // RGBA in, BGRX out. The fourth byte is unused rather than alpha.
        dib.extend_from_slice(&[px[2], px[1], px[0], 0]);
    }

    // ---- the text: UTF-16, NUL-terminated --------------------------------
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

    // ---- one transaction --------------------------------------------------
    //
    // ★ Both handles are allocated BEFORE the clipboard is opened. The
    // clipboard is a system-wide lock; holding it across two allocations is
    // holding it longer than necessary, and an allocation failure inside the
    // open would mean deciding whether to publish half a transaction.
    let Some(dib_mem) = global_copy(&dib) else {
        return false;
    };
    let Some(text_mem) = global_copy(bytes_of_u16(&wide)) else {
        // SAFETY: `dib_mem` came from `GlobalAlloc` and ownership has not
        // passed to the clipboard, because the clipboard was never opened.
        unsafe { GlobalFree(dib_mem) };
        return false;
    };

    // SAFETY: a null owner means "this task", which is what a process without
    // a window handle to hand over uses. Failure is reported, not assumed.
    if unsafe { OpenClipboard(std::ptr::null_mut()) } == 0 {
        // SAFETY: neither handle reached the clipboard, so both are still ours.
        unsafe {
            GlobalFree(dib_mem);
            GlobalFree(text_mem);
        }
        return false;
    }
    // SAFETY: the clipboard is open and owned by this task.
    unsafe { EmptyClipboard() };
    // SAFETY: as above. On success the clipboard OWNS the handle and this
    // process must not free it — which is why there is no `GlobalFree` on the
    // success path, and why a leak here would be the *other* kind of bug.
    let dib_ok = !unsafe { SetClipboardData(CF_DIB, dib_mem) }.is_null();
    let text_ok = !unsafe { SetClipboardData(CF_UNICODETEXT, text_mem) }.is_null();
    // SAFETY: closing a clipboard this task opened.
    unsafe { CloseClipboard() };

    // A handle the clipboard refused is still ours to free.
    if !dib_ok {
        // SAFETY: `SetClipboardData` returned null, so ownership did not pass.
        unsafe { GlobalFree(dib_mem) };
    }
    if !text_ok {
        // SAFETY: as above.
        unsafe { GlobalFree(text_mem) };
    }
    dib_ok && text_ok
}

/// Copy a byte slice into moveable global memory the clipboard can take.
///
/// Returns `None` on an allocation failure, which a caller must treat as *"the
/// clipboard was not written"* rather than as an empty success.
fn global_copy(bytes: &[u8]) -> Option<Handle> {
    // SAFETY: no pointers in; failure is a null return.
    let mem = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes.len()) };
    if mem.is_null() {
        return None;
    }
    // SAFETY: `mem` came from `GlobalAlloc` above and is unlocked.
    let dst = unsafe { GlobalLock(mem) };
    if dst.is_null() {
        // SAFETY: the lock failed, so nothing is mapped and the handle is ours.
        unsafe { GlobalFree(mem) };
        return None;
    }
    // SAFETY: `dst` addresses `bytes.len()` bytes just allocated, and the two
    // regions cannot overlap — one is a fresh allocation.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst.cast::<u8>(), bytes.len()) };
    // SAFETY: unlocking the lock taken above.
    unsafe { GlobalUnlock(mem) };
    Some(mem)
}

/// A `u16` slice as bytes, for the UTF-16 clipboard payload.
fn bytes_of_u16(v: &[u16]) -> &[u8] {
    // SAFETY: every `u16` is two initialised bytes; the lifetime is tied to the
    // input and the length is doubled to match. `u8` has no alignment
    // requirement, so a `u16`-aligned pointer is trivially valid for it.
    unsafe { core::slice::from_raw_parts(v.as_ptr().cast::<u8>(), core::mem::size_of_val(v)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ **A buffer whose length does not match its declared size is refused**,
    /// and refusing is what keeps whatever the operator had on the clipboard.
    ///
    /// Asserted without touching the clipboard, because the check happens
    /// before any Win32 call — which is deliberate: a test that opened the real
    /// clipboard would fight every other program on the machine for it.
    #[test]
    fn a_mismatched_buffer_is_refused_before_anything_is_opened() {
        assert!(!set_image_and_text(&[0; 15], 2, 2, "x"), "15 bytes for 2x2");
        assert!(!set_image_and_text(&[], 0, 0, "x"), "no pixels at all");
        assert!(!set_image_and_text(&[0; 16], 2, 0, "x"), "zero height");
    }
}
