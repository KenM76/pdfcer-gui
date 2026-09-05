//! # `native_clipboard::win32` — the Windows half, and every `unsafe` in the
//! # crate
//!
//! See the crate docs for why this exists at all. This file is the whole of the
//! platform work: eleven imported functions across three libraries, two RAII
//! types, one private entry point, and no state that outlives a call.
//!
//! ## ★ Why the declarations are hand-written rather than a crate
//!
//! `crates/native-window/src/win32.rs` sets the precedent and states the
//! reason: *"four symbols is not worth a dependency"*, and `tools/ui-verify`
//! already declares its own `user32` externs in this repository for exactly
//! that argument. Eleven symbols does not change it. Both `clipboard-win` and
//! `windows` are already linked into `pdfcer-gui.exe`, so this is not a
//! question of build cost — it is that the ownership contract in
//! [`Staged::surrender`] is the entire content of this crate, and describing it
//! at second hand through another library's newtypes makes it harder to check,
//! not easier.
//!
//! Each declaration below is copied from the SDK signature. Each call site
//! carries a `SAFETY` comment naming the invariant it upholds.
//!
//! ## ★★ Three libraries, named explicitly
//!
//! `native_window::win32` names none, because all four of its symbols are in
//! `user32`, which the toolchain links by default. This file's symbols are
//! split across **three**: the clipboard is `user32`, the moveable memory it
//! takes is `kernel32`, and the metafile is `gdi32`. `native_window::clipboard`
//! records that the first build of the two-library version *"failed to link
//! with eight `LNK2019 unresolved external symbol` errors"*, so they are
//! declared per library. Being explicit is also the honest form: a reader can
//! see which DLL each call crosses into, and a symbol that moves libraries in a
//! future SDK fails at link time here rather than at run time somewhere else.
//!
//! ## ★★★ The two invariants this file exists to hold
//!
//! **1. `SetClipboardData` takes ownership on success.** A handle it accepts
//! belongs to the clipboard; freeing it afterwards is a double free, and
//! Windows will corrupt or fault on the next paste. A handle it refuses is
//! still ours; not freeing it is a leak. [`Staged`] holds both directions in
//! one place: it owns every handle it created, `Drop` frees it, and
//! [`Staged::surrender`] clears that ownership **only** when the clipboard
//! actually took it. No code path — success, error, or panic — has to remember
//! which case it is in.
//!
//! **2. An opened clipboard must be closed on every path.** It is a
//! system-wide, per-window-station lock; a process that leaves it open blocks
//! every other program on the desktop from copying or pasting. [`OpenGuard`]
//! closes it in `Drop`, so an `?` early return, a `return Err(...)` in the
//! middle of the loop, and an unwinding panic all close it. There is no
//! `CloseClipboard` call anywhere else in this crate, deliberately — a second
//! one would be a second thing to keep correct.

use std::ffi::c_void;

use crate::{CF_ENHMETAFILE, Entry, PlaceError, Slot};

/// An opaque OS handle: an `HGLOBAL`, an `HENHMETAFILE` or an `HWND` depending
/// on which call it crosses. Untyped for the same reason the SDK's own
/// `HANDLE` is: the distinction that matters here is **who owns it**, and that
/// is tracked by [`Staged`] rather than by the type.
type Handle = *mut c_void;

/// `GMEM_MOVEABLE` — the only allocation flag the clipboard accepts.
///
/// A `GMEM_FIXED` block handed to `SetClipboardData` is not refused; it is
/// accepted and then freed by a mechanism that assumes a moveable handle. This
/// constant is not a preference.
const GMEM_MOVEABLE: u32 = 0x0002;

/// How many times to ask for the clipboard before giving up.
///
/// Windows serialises clipboard access across the whole desktop, so a refusal
/// is normally *"another program is mid-copy"* and is over in milliseconds.
/// Ten attempts is `clipboard-win`'s own convention for the same wait.
const OPEN_ATTEMPTS: u32 = 10;

/// How long to wait between attempts.
///
/// ★ Ten milliseconds rather than the fifty a background tool would use,
/// because this runs on a GUI thread: the worst case is a tenth of a second of
/// unresponsiveness, which is below the threshold at which a window is
/// perceived to have stalled. Half a second would not be.
const OPEN_RETRY: std::time::Duration = std::time::Duration::from_millis(10);

#[link(name = "user32")]
unsafe extern "system" {
    /// Open the clipboard for this task. A null owner means "this task",
    /// which is what a caller with no window handle to hand over uses.
    fn OpenClipboard(owner: Handle) -> i32;
    /// Close it. Required on every path — see the module header.
    fn CloseClipboard() -> i32;
    /// Discard the clipboard's contents and give this task ownership of it.
    /// **This is the destructive step**, and everything that can fail is done
    /// before it.
    fn EmptyClipboard() -> i32;
    /// Place one handle under one format id. Returns the handle on success and
    /// null on failure; **on success the clipboard owns the handle.**
    fn SetClipboardData(format: u32, mem: Handle) -> Handle;
    /// Register a clipboard format by name, returning its id, or 0 on failure.
    ///
    /// Registration is process-independent and idempotent: a name already
    /// registered by another program returns that same id, which is exactly
    /// what makes `"PNG"` mean the same thing to Office and to this process.
    fn RegisterClipboardFormatW(name: *const u16) -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> Handle;
    fn GlobalFree(mem: Handle) -> Handle;
    fn GlobalLock(mem: Handle) -> *mut c_void;
    fn GlobalUnlock(mem: Handle) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    /// Build a memory metafile from [MS-EMF] bytes, returning an
    /// `HENHMETAFILE` or null.
    ///
    /// ★★ Note the argument order: **size first, then the pointer**. The
    /// reverse would compile — both are integers to the linker — and would
    /// hand GDI a length of whatever the pointer's low 32 bits happen to be.
    fn SetEnhMetaFileBits(size: u32, bits: *const u8) -> Handle;
    /// Delete a metafile handle this process still owns.
    fn DeleteEnhMetaFile(hemf: Handle) -> i32;
}

/// ★★★ **A handle that has been created but not yet given away.**
///
/// The type that holds invariant 1 from the module header. It owns whatever it
/// created; `Drop` frees it; [`Self::surrender`] hands it to the clipboard and
/// clears the ownership **only if the clipboard took it**.
///
/// ⇒ Written as an owning type rather than as careful sequencing, because
/// careful sequencing has three paths — success, error, panic — and every one
/// of them would have to remember which handles were still ours. This has one.
struct Staged {
    /// The `HGLOBAL` or `HENHMETAFILE`. Never null: both constructors refuse.
    handle: Handle,
    /// Which free function this handle needs. An `HGLOBAL` passed to
    /// `DeleteEnhMetaFile` is not refused — GDI would look it up in a table it
    /// is not in — so the two are never confusable at run time here.
    kind: Kind,
    /// The clipboard format id to place it under.
    format: u32,
    /// The format's name, carried purely so a failure can name it.
    name: &'static str,
    /// Set by [`Self::surrender`] when — and only when — `SetClipboardData`
    /// returned non-null. While it is `false` this value owns the handle.
    surrendered: bool,
}

/// Which of the two free functions a [`Staged`] handle needs.
#[derive(Clone, Copy)]
enum Kind {
    /// `GlobalFree`.
    Global,
    /// `DeleteEnhMetaFile`.
    EnhMetaFile,
}

impl Staged {
    /// Copy `bytes` into moveable global memory, ready to place under `format`.
    fn global(name: &'static str, format: u32, bytes: &[u8]) -> Result<Self, PlaceError> {
        // ★ A zero-byte allocation is refused rather than attempted.
        // `GlobalAlloc(GMEM_MOVEABLE, 0)` returns a handle that cannot be
        // locked, and `SetClipboardData` would accept it — producing a format
        // that is advertised on the clipboard and yields nothing when read.
        if bytes.is_empty() {
            return Err(PlaceError::Stage(name));
        }
        // SAFETY: no pointers are passed in; failure is a null return, which
        // is checked immediately below. The handle is unowned by anything else
        // until this function returns it inside `Self`, whose `Drop` frees it.
        let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes.len()) };
        if handle.is_null() {
            return Err(PlaceError::Stage(name));
        }
        // ★ Constructed BEFORE the lock, so that every failure below is a
        // plain `return` and the handle is freed by this value's `Drop`. An
        // explicit `GlobalFree` on each error path is the version of this
        // function that eventually grows a path without one.
        let staged = Self {
            handle,
            kind: Kind::Global,
            format,
            name,
            surrendered: false,
        };
        // SAFETY: `handle` came from `GlobalAlloc` immediately above and has
        // never been locked, so the lock count goes 0 → 1. A null return means
        // the lock failed and nothing is mapped.
        let dst = unsafe { GlobalLock(staged.handle) };
        if dst.is_null() {
            return Err(PlaceError::Stage(name));
        }
        // SAFETY: `dst` addresses exactly `bytes.len()` bytes — that is the
        // size just allocated, and `GlobalLock` returns a pointer to the whole
        // block. The two regions cannot overlap: one is a fresh allocation
        // this thread has just been handed. `u8` has no alignment requirement.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst.cast::<u8>(), bytes.len()) };
        // SAFETY: releasing the lock taken above, on the same handle, exactly
        // once. The return value is the remaining lock count and is not an
        // error indicator for a count that has reached zero.
        unsafe { GlobalUnlock(staged.handle) };
        Ok(staged)
    }

    /// Turn [MS-EMF] bytes into a GDI metafile handle, ready to place under
    /// `CF_ENHMETAFILE`.
    ///
    /// ★★ This is where a malformed metafile is caught, and catching it *here*
    /// — before the clipboard is opened — is the whole reason staging is
    /// separate from placement. `SetEnhMetaFileBits` parses the record
    /// structure and returns null for bits it will not accept, so a bad
    /// metafile becomes a refusal that leaves the operator's clipboard
    /// untouched rather than a GDI failure half-way through replacing it.
    fn metafile(name: &'static str, bytes: &[u8]) -> Result<Self, PlaceError> {
        let Ok(size) = u32::try_from(bytes.len()) else {
            // An [MS-EMF] metafile records its own byte count in a `u32`, so a
            // buffer larger than `u32::MAX` is not a metafile Windows could
            // have produced or could read. Refused rather than truncated.
            return Err(PlaceError::Stage(name));
        };
        if size == 0 {
            return Err(PlaceError::Stage(name));
        }
        // SAFETY: `bytes` is a live slice of exactly `size` bytes for the
        // duration of the call, and `SetEnhMetaFileBits` COPIES out of it — it
        // does not retain the pointer, so the borrow need not outlive this
        // line. Argument order is (size, pointer), per the declaration above.
        // A null return means GDI rejected the bits and no handle was created.
        let handle = unsafe { SetEnhMetaFileBits(size, bytes.as_ptr()) };
        if handle.is_null() {
            return Err(PlaceError::Stage(name));
        }
        Ok(Self {
            handle,
            kind: Kind::EnhMetaFile,
            format: CF_ENHMETAFILE,
            name,
            surrendered: false,
        })
    }

    /// ★★★ **Hand the handle to the clipboard**, and stop owning it if that
    /// worked.
    ///
    /// Returns whether the clipboard took it. The `surrendered` write is the
    /// single line that decides between a leak and a double free, and it
    /// happens in the same expression as the call whose result it depends on —
    /// there is no path between them on which something else could run.
    ///
    /// # Preconditions
    ///
    /// The clipboard must be open and owned by this thread ([`OpenGuard`]), and
    /// `EmptyClipboard` must already have been called. `SetClipboardData`
    /// requires both, and Win32 answers a violation with a plain failure rather
    /// than an explanation.
    #[must_use]
    fn surrender(&mut self) -> bool {
        // SAFETY: the caller holds an `OpenGuard`, so the clipboard is open and
        // owned by this thread — `SetClipboardData`'s only precondition. The
        // handle is non-null (both constructors refuse null), is of the kind
        // `self.format` expects (an `HGLOBAL` for every format except
        // `CF_ENHMETAFILE`, whose handle can only have come from
        // `Self::metafile`), and is still owned by this value: `surrendered` is
        // false, and it is set to true immediately below, in the same
        // expression, if and only if the clipboard took ownership. After that
        // point NOTHING in this crate frees it — see `Drop`.
        let taken = !unsafe { SetClipboardData(self.format, self.handle) }.is_null();
        self.surrendered = taken;
        taken
    }
}

impl Drop for Staged {
    fn drop(&mut self) {
        if self.surrendered {
            // ★★★ THE CLIPBOARD OWNS IT. Freeing here would be a double free,
            // and the symptom would appear in another process, on a later
            // paste, with nothing pointing back to this line.
            return;
        }
        match self.kind {
            // SAFETY: the handle came from `GlobalAlloc` in `Self::global`,
            // has been unlocked (lock count 0), and has not been given to the
            // clipboard — `surrendered` is false, checked immediately above.
            // This value is being dropped, so nothing can use it afterwards.
            Kind::Global => unsafe {
                GlobalFree(self.handle);
            },
            // SAFETY: the handle came from `SetEnhMetaFileBits` in
            // `Self::metafile` and has not been given to the clipboard, by the
            // same check. `DeleteEnhMetaFile` is the matching destructor for a
            // memory metafile.
            Kind::EnhMetaFile => unsafe {
                DeleteEnhMetaFile(self.handle);
            },
        }
    }
}

/// ★★★ **The open clipboard, closed by `Drop` on every path including a
/// panic.**
///
/// The type that holds invariant 2 from the module header. There is exactly one
/// `CloseClipboard` call in this crate and it is inside this `Drop`.
///
/// ⚠ Why a guard rather than a close at the end of [`place`]: the clipboard is
/// a **desktop-wide lock**. A process that returns early, or unwinds, without
/// closing it leaves every other program on the machine unable to copy or paste
/// until this process exits — a failure the operator would experience as
/// *"Windows' clipboard has stopped working"*, with nothing connecting it to a
/// PDF viewer. That is not a class of bug worth being careful about; it is one
/// worth making unrepresentable.
struct OpenGuard;

impl OpenGuard {
    /// Ask for the clipboard, retrying while another process holds it.
    ///
    /// `None` means the budget ran out, which is a transient and reportable
    /// condition rather than an error in this program.
    fn acquire() -> Option<Self> {
        for attempt in 0..OPEN_ATTEMPTS {
            // SAFETY: a null owner means "this task", which is what a caller
            // with no window handle uses. No pointer is dereferenced. On
            // success this thread holds the clipboard until `CloseClipboard`,
            // which is `Drop`'s only job.
            if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
                return Some(Self);
            }
            // ★ No sleep after the LAST attempt: the budget is a wait for
            // somebody else to finish, and there is nothing left to wait for
            // once this loop is going to give up anyway.
            if attempt + 1 < OPEN_ATTEMPTS {
                std::thread::sleep(OPEN_RETRY);
            }
        }
        None
    }
}

impl Drop for OpenGuard {
    fn drop(&mut self) {
        // SAFETY: this value exists only when `OpenClipboard` succeeded on
        // this thread, it is not `Clone` or `Copy`, and it is dropped exactly
        // once — so this closes a clipboard this thread opened, once.
        unsafe { CloseClipboard() };
    }
}

/// Register a clipboard format name, or `None`.
fn register(name: &str) -> Option<u32> {
    // ★ UTF-16 with an explicit terminator. `RegisterClipboardFormatW` reads
    // until a NUL; a `Vec<u16>` built from `encode_utf16` alone has none, and
    // the call would read past the end of the allocation.
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `wide` is a NUL-terminated UTF-16 buffer that outlives the call,
    // which copies the name into the system's registration table and retains
    // no pointer. A zero return means the registration failed.
    let id = unsafe { RegisterClipboardFormatW(wide.as_ptr()) };
    (id != 0).then_some(id)
}

/// See [`crate::place`], whose documentation carries the staging argument.
pub(crate) fn place(entries: &[Entry<'_>]) -> Result<Vec<&'static str>, PlaceError> {
    if entries.is_empty() {
        return Err(PlaceError::Nothing);
    }

    // ---- STAGE: everything that can fail, before anything is destroyed -----
    //
    // ★★★ On any `?` below, `staged` is dropped and every handle created so
    // far is freed. The clipboard has not been opened, let alone emptied, so
    // the operator still has whatever they copied last. This is the property
    // that makes the transaction all-or-nothing in the direction that matters.
    let mut staged: Vec<Staged> = Vec::with_capacity(entries.len());
    for entry in entries {
        staged.push(match entry.slot {
            Slot::Registered => {
                let id = register(entry.name).ok_or(PlaceError::Register(entry.name))?;
                Staged::global(entry.name, id, entry.bytes)?
            }
            Slot::Predefined(id) => Staged::global(entry.name, id, entry.bytes)?,
            Slot::EnhMetaFile => Staged::metafile(entry.name, entry.bytes)?,
        });
    }

    // ---- PLACE: open, empty, hand over, close ------------------------------
    //
    // ★ `_guard` is declared AFTER `staged`, so it drops FIRST (locals drop in
    // reverse declaration order). The clipboard is therefore closed before any
    // un-surrendered handle is freed, which is the order Win32 wants: a handle
    // the clipboard refused is ours again the moment `SetClipboardData`
    // returned null, and freeing it after the close cannot race the clipboard's
    // own cleanup of the handles it DID take.
    let _guard = OpenGuard::acquire().ok_or(PlaceError::Open)?;
    // SAFETY: the clipboard is open and owned by this thread (the guard). This
    // discards the previous contents — the one destructive step, and the reason
    // every fallible operation is already done by this line.
    if unsafe { EmptyClipboard() } == 0 {
        // Nothing was destroyed: `EmptyClipboard` either empties or fails. The
        // clipboard still holds what it held, so this is reported as an open
        // failure rather than as a partial placement.
        return Err(PlaceError::Open);
    }

    let mut placed = Vec::with_capacity(staged.len());
    for item in &mut staged {
        if !item.surrender() {
            // ⚠ The one non-atomic exit, and it is stated rather than hidden:
            // entries before this one are on the clipboard. Win32 offers no
            // rollback for an emptied clipboard, so the honest thing is to name
            // the format that failed and let the caller say so.
            return Err(PlaceError::Set(item.name));
        }
        placed.push(item.name);
    }
    Ok(placed)
}
