//! Unix `crypt(3)` wrapper for Perl `crypt` (DES / system hashing).

/// Hash `plaintext` with `salt` using the platform libc `crypt`.
/// On non-Unix targets, returns an empty string.
pub fn perl_crypt(plaintext: &str, salt: &str) -> String {
    #[cfg(unix)]
    {
        use std::ffi::{CStr, CString};

        extern "C" {
            fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char;
        }

        unsafe {
            let key = match CString::new(plaintext.as_bytes()) {
                Ok(s) => s,
                Err(_) => return String::new(),
            };
            let salt = match CString::new(salt.as_bytes()) {
                Ok(s) => s,
                Err(_) => return String::new(),
            };
            let ptr = crypt(key.as_ptr(), salt.as_ptr());
            if ptr.is_null() {
                return String::new();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (plaintext, salt);
        String::new()
    }
}
