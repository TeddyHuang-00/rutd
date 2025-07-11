use std::ffi::OsStr;

/// Validates UTF-8 input for completion functions.
/// Returns None if the input is not valid UTF-8, which should result in no completions.
pub fn validate_utf8_or_empty(input: &OsStr) -> Option<&str> {
    input.to_str()
}