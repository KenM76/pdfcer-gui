//! The Win32 calls, declared by hand. Each handle or allocation is owned by
//! exactly one value whose `Drop` releases it.

use std::ffi::c_void;
use std::fs::File;
use std::io;
use std::os::windows::io::{FromRawHandle, RawHandle};

type Handle = *mut c_void;

const INVALID_HANDLE_VALUE: Handle = -1isize as Handle;
const PIPE_ACCESS_DUPLEX: u32 = 0x0000_0003;
const FILE_FLAG_FIRST_PIPE_INSTANCE: u32 = 0x0008_0000;
/// Byte type, byte read mode and blocking mode are all zero.
const PIPE_BYTE_WAIT: u32 = 0;
const PIPE_REJECT_REMOTE_CLIENTS: u32 = 0x0000_0008;
const PIPE_UNLIMITED_INSTANCES: u32 = 255;
const BUFFER: u32 = 64 * 1024;
const ERROR_PIPE_CONNECTED: i32 = 535;
const ERROR_FILE_NOT_FOUND: i32 = 2;
const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_USER_CLASS: u32 = 1;
const SDDL_REVISION_1: u32 = 1;

#[repr(C)]
struct SecurityAttributes {
    length: u32,
    descriptor: *mut c_void,
    inherit: i32,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateNamedPipeW(
        name: *const u16,
        open_mode: u32,
        pipe_mode: u32,
        max_instances: u32,
        out_buffer: u32,
        in_buffer: u32,
        default_timeout: u32,
        security: *const SecurityAttributes,
    ) -> Handle;
    fn ConnectNamedPipe(pipe: Handle, overlapped: *mut c_void) -> i32;
    fn DisconnectNamedPipe(pipe: Handle) -> i32;
    fn WaitNamedPipeW(name: *const u16, timeout_ms: u32) -> i32;
    fn FlushFileBuffers(file: Handle) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetCurrentProcess() -> Handle;
    fn LocalFree(mem: *mut c_void) -> *mut c_void;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(process: Handle, access: u32, token: *mut Handle) -> i32;
    fn GetTokenInformation(
        token: Handle,
        class: u32,
        info: *mut c_void,
        length: u32,
        returned: *mut u32,
    ) -> i32;
    fn ConvertSidToStringSidW(sid: *mut c_void, out: *mut *mut u16) -> i32;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        sddl: *const u16,
        revision: u32,
        descriptor: *mut *mut c_void,
        size: *mut u32,
    ) -> i32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// A `LocalAlloc` allocation the OS handed back.
struct LocalMem(*mut c_void);

impl Drop for LocalMem {
    fn drop(&mut self) {
        // SAFETY: allocated by the OS with LocalAlloc; freed once, here.
        unsafe { LocalFree(self.0) };
    }
}

struct Token(Handle);

impl Drop for Token {
    fn drop(&mut self) {
        // SAFETY: a token handle this value opened and owns.
        unsafe { CloseHandle(self.0) };
    }
}

/// The current process user's SID as an `S-1-5-…` string.
fn current_user_sid() -> io::Result<String> {
    let mut raw: Handle = std::ptr::null_mut();
    // SAFETY: the process pseudo-handle needs no closing; `raw` is an out-pointer.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = Token(raw);
    // TOKEN_USER is a SID pointer and a u32, followed by the SID it points
    // into. 256 bytes holds any SID (at most 68 bytes); u64 for alignment.
    let mut buf = [0u64; 32];
    let mut len = 0u32;
    // SAFETY: `buf` is writable for the length passed.
    let ok = unsafe {
        GetTokenInformation(
            token.0,
            TOKEN_USER_CLASS,
            buf.as_mut_ptr().cast(),
            u32::try_from(std::mem::size_of_val(&buf)).unwrap_or(u32::MAX),
            &mut len,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: TOKEN_USER's first field is the SID pointer, pointing into `buf`.
    let sid = unsafe { *buf.as_ptr().cast::<*mut c_void>() };
    let mut text: *mut u16 = std::ptr::null_mut();
    // SAFETY: `sid` is valid while `buf` lives; `text` is an out-pointer.
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let owned = LocalMem(text.cast());
    let mut n = 0usize;
    // SAFETY: the OS returned a NUL-terminated wide string, alive until `owned` drops.
    let s = unsafe {
        while *text.add(n) != 0 {
            n += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(text, n))
    };
    drop(owned);
    Ok(s)
}

/// A security descriptor granting the current user, and only them, all access.
fn current_user_only() -> io::Result<LocalMem> {
    let sddl = wide(&format!("D:P(A;;GA;;;{})", current_user_sid()?));
    let mut sd: *mut c_void = std::ptr::null_mut();
    // SAFETY: `sddl` is NUL-terminated; `sd` is an out-pointer; size is optional.
    let ok = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut sd,
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(LocalMem(sd))
}

/// Whether a server holds `name`, asked without connecting to it.
pub(crate) fn exists(name: &str) -> bool {
    let wname = wide(name);
    // SAFETY: `wname` is NUL-terminated and outlives the call.
    if unsafe { WaitNamedPipeW(wname.as_ptr(), 1) } != 0 {
        return true;
    }
    // Every instance busy (a timeout) still means a server is there.
    io::Error::last_os_error().raw_os_error() != Some(ERROR_FILE_NOT_FOUND)
}

/// One pipe instance; the `File` owns and closes the handle.
#[derive(Debug)]
pub(crate) struct Instance {
    file: File,
    handle: Handle,
}

// SAFETY: a pipe handle may be used from any thread; `handle` is only a copy
// of the one `file` owns.
unsafe impl Send for Instance {}

impl Instance {
    pub(crate) fn create(name: &str, first: bool) -> io::Result<Self> {
        let descriptor = current_user_only()?;
        let attrs = SecurityAttributes {
            length: u32::try_from(std::mem::size_of::<SecurityAttributes>()).unwrap_or(0),
            descriptor: descriptor.0,
            inherit: 0,
        };
        let open_mode = PIPE_ACCESS_DUPLEX
            | if first {
                FILE_FLAG_FIRST_PIPE_INSTANCE
            } else {
                0
            };
        let wname = wide(name);
        // SAFETY: every pointer is valid for the call; the descriptor outlives it.
        let handle = unsafe {
            CreateNamedPipeW(
                wname.as_ptr(),
                open_mode,
                PIPE_BYTE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                BUFFER,
                BUFFER,
                0,
                &attrs,
            )
        };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: a fresh handle we own; the File closes it exactly once.
        let file = unsafe { File::from_raw_handle(handle as RawHandle) };
        Ok(Self { file, handle })
    }

    pub(crate) fn connect(&self) -> io::Result<()> {
        // SAFETY: our live pipe handle; synchronous, no overlapped.
        if unsafe { ConnectNamedPipe(self.handle, std::ptr::null_mut()) } != 0 {
            return Ok(());
        }
        let err = io::Error::last_os_error();
        // A client that connected between create and connect is connected.
        if err.raw_os_error() == Some(ERROR_PIPE_CONNECTED) {
            Ok(())
        } else {
            Err(err)
        }
    }

    pub(crate) fn file(&mut self) -> &mut File {
        &mut self.file
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // Flush first so the client reads everything written before the close.
        // SAFETY: our live handle; the File closes it after this returns.
        unsafe {
            FlushFileBuffers(self.handle);
            DisconnectNamedPipe(self.handle);
        }
    }
}
