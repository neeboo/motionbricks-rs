//! Error type and status codes for the safe motionbricks API.

use std::ffi::{c_char, CStr};

/// Native `mb_status` codes (`motionbricks.h`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Status(pub u32);

impl Status {
    /// Success.
    pub const OK: Status = Status(crate::ffi::MB_OK);
    /// A caller-supplied value was rejected.
    pub const INVALID_ARGUMENT: Status = Status(crate::ffi::MB_INVALID_ARGUMENT);
    /// Allocation failed inside the library.
    pub const OUT_OF_MEMORY: Status = Status(crate::ffi::MB_OUT_OF_MEMORY);
    /// A file could not be read or written.
    pub const IO_ERROR: Status = Status(crate::ffi::MB_IO_ERROR);
    /// A file was readable but not a valid bundle/style/format.
    pub const INVALID_FORMAT: Status = Status(crate::ffi::MB_INVALID_FORMAT);
    /// The bundle was built for a different model or ABI generation.
    pub const INCOMPATIBLE_MODEL: Status = Status(crate::ffi::MB_INCOMPATIBLE_MODEL);
    /// The requested device backend is not present in this build.
    pub const BACKEND_UNAVAILABLE: Status = Status(crate::ffi::MB_BACKEND_UNAVAILABLE);
    /// Inference ran and failed.
    pub const COMPUTE_FAILED: Status = Status(crate::ffi::MB_COMPUTE_FAILED);
    /// The entry point exists but this build does not implement it.
    pub const NOT_IMPLEMENTED: Status = Status(crate::ffi::MB_NOT_IMPLEMENTED);
    /// An unexpected library-side fault.
    pub const INTERNAL_ERROR: Status = Status(crate::ffi::MB_INTERNAL_ERROR);

    /// True for [`Status::OK`].
    pub fn is_ok(self) -> bool {
        self == Status::OK
    }
}

impl From<u32> for Status {
    fn from(value: u32) -> Self {
        Status(value)
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match *self {
            Status::OK => "ok",
            Status::INVALID_ARGUMENT => "invalid argument",
            Status::OUT_OF_MEMORY => "out of memory",
            Status::IO_ERROR => "io error",
            Status::INVALID_FORMAT => "invalid format",
            Status::INCOMPATIBLE_MODEL => "incompatible model",
            Status::BACKEND_UNAVAILABLE => "backend unavailable",
            Status::COMPUTE_FAILED => "compute failed",
            Status::NOT_IMPLEMENTED => "not implemented",
            Status::INTERNAL_ERROR => "internal error",
            Status(other) => return write!(f, "unknown status {other}"),
        };
        f.write_str(name)
    }
}

/// A failed native call: the numeric status plus the message the library
/// wrote into the error buffer (falling back to `mb_status_string`).
#[derive(Debug, Clone)]
pub struct Error {
    /// Numeric status code returned by the native call.
    pub status: Status,
    /// Operation context plus the library's error-buffer message.
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.status)
    }
}

impl std::error::Error for Error {}

/// Convenience alias: every fallible API returns `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn error_buffer() -> [u8; crate::ffi::MB_ERROR_CAPACITY as usize] {
    [0u8; crate::ffi::MB_ERROR_CAPACITY as usize]
}

/// Interpret a status + filled error buffer as an [`Error`], prefixing the
/// failed operation (mirrors the Go binding's `check`).
pub(crate) fn take_error(
    status_string: impl Fn(u32) -> String,
    operation: &str,
    status: u32,
    buffer: &[u8],
) -> Error {
    let native = CStr::from_bytes_until_nul(buffer)
        .ok()
        .map(|message| message.to_string_lossy().into_owned())
        .filter(|message| !message.is_empty());
    let message = native.unwrap_or_else(|| status_string(status));
    Error {
        status: Status(status),
        message: format!("{operation}: {message}"),
    }
}

pub(crate) fn simple_error(operation: &str, detail: impl Into<String>) -> Error {
    Error {
        status: Status::INVALID_ARGUMENT,
        message: format!("{operation}: {}", detail.into()),
    }
}

pub(crate) unsafe fn write_error_buffer(buffer: &mut [u8]) -> *mut c_char {
    buffer.as_mut_ptr().cast::<c_char>()
}
