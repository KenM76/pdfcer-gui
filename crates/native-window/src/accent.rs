//! The operator's Windows accent colour, read from the per-user registry
//! value the Settings app writes (`Explorer\Accent\AccentColorMenu`).

use std::ffi::c_void;

type Hkey = *mut c_void;

/// `HKEY_CURRENT_USER`, as `winreg.h` defines it: a sign-extended constant.
const HKEY_CURRENT_USER: Hkey = 0x8000_0001_u32 as i32 as isize as Hkey;
/// `RRF_RT_REG_DWORD`: accept only a `REG_DWORD`.
const RRF_RT_REG_DWORD: u32 = 0x0000_0010;

#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegGetValueW(
        key: Hkey,
        sub_key: *const u16,
        value: *const u16,
        flags: u32,
        kind: *mut u32,
        data: *mut c_void,
        data_len: *mut u32,
    ) -> i32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// The accent as `[r, g, b]`, or `None` when the value is absent (Windows 10
/// before 1809, or a policy that removed it).
#[must_use]
pub fn system_accent() -> Option<[u8; 3]> {
    let sub_key = wide(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent");
    let value = wide("AccentColorMenu");
    let mut data: u32 = 0;
    let mut len = u32::try_from(std::mem::size_of::<u32>()).ok()?;
    // SAFETY: both strings are NUL-terminated and outlive the call; `data` is
    // a live `u32` and `len` says so; `RRF_RT_REG_DWORD` makes the call refuse
    // any value that is not exactly four bytes.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&raw mut data).cast(),
            &raw mut len,
        )
    };
    // The DWORD is laid out 0xAABBGGRR.
    let [r, g, b, _] = data.to_le_bytes();
    (status == 0).then_some([r, g, b])
}
