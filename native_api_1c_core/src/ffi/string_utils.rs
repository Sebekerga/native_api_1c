use std::slice::from_raw_parts;

/// Helper function to convert pointer to UTF-16 string to Rust slice
/// # Arguments
/// * `s` - pointer to UTF-16 string
/// # Returns
/// `&[u16]` - slice of UTF-16 characters
/// # Safety
/// This function is unsafe because it takes a raw pointer and dereferences it
pub unsafe fn get_str<'a>(s: *const u16) -> &'a [u16] {
    unsafe fn strlen(s: *const u16) -> usize {
        let mut i = 0;
        while *s.add(i) != 0 {
            i += 1;
        }
        i + 1
    }

    let len = strlen(s);
    from_raw_parts(s, len)
}

/// Helper function to convert Rust string to UTF-16 string
/// # Arguments
/// * `s` - Rust string
/// # Returns
/// `Vec<u16>` - UTF-16 string without null terminator
#[cfg(target_family = "unix")]
pub fn os_string_nil(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// Helper function to convert Rust string to UTF-16 string
/// # Arguments
/// * `s` - Rust string
/// # Returns
/// `Vec<u16>` - UTF-16 string with null terminator
#[cfg(target_family = "windows")]
pub fn os_string_nil(s: &str) -> Vec<u16> {
    let os_str = std::ffi::OsStr::new(s);
    std::os::windows::prelude::OsStrExt::encode_wide(os_str)
        .chain(Some(0).into_iter())
        .collect()
}

/// Helper function to convert Rust string to UTF-16 string
/// # Arguments
/// * `s` - Rust string
/// # Returns
/// `Vec<u16>` - UTF-16 string without null terminator
#[cfg(target_family = "unix")]
pub fn os_string(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// Helper function to convert Rust string to UTF-16 string
/// # Arguments
/// * `s` - Rust string
/// # Returns
/// `Vec<u16>` - UTF-16 string with null terminator
#[cfg(target_family = "windows")]
pub fn os_string(s: &str) -> Vec<u16> {
    let os_str = std::ffi::OsStr::new(s);
    std::os::windows::prelude::OsStrExt::encode_wide(os_str).collect()
}

/// Helper function to convert UTF-16 string to Rust string
/// # Arguments
/// * `s` - UTF-16 string
/// # Returns
/// `String` - Rust string
pub fn from_os_string(s: &[u16]) -> String {
    String::from_utf16_lossy(s)
        .trim_end_matches(char::from(0))
        .to_string()
}
