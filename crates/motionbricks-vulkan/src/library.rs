//! Runtime loading of the motionbricks dynamic library.

use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::Arc;

use crate::ffi as sys;

use crate::error::{error_buffer, take_error, Error, Result};
use crate::{options::RuntimeOptions, model::Model, Device};

/// An opened motionbricks dynamic library with every entry point resolved.
///
/// Create through [`Library::open`]. The value is cheap to share: all handle
/// types keep an `Arc<Library>` internally. The underlying library is
/// deliberately never unloaded for the lifetime of the process.
#[derive(Debug)] // Symbols is Copy + Debug
pub struct Library {
    pub(crate) symbols: sys::Symbols,
}

impl Library {
    /// dlopen the library at `path`, resolve all `mb_*` symbols eagerly and
    /// verify the reported ABI version equals 1.
    pub fn open(path: impl AsRef<Path>) -> Result<Arc<Library>> {
        let path = path.as_ref();
        let c_path = CString::new(
            path.to_str()
                .ok_or_else(|| Error {
                    status: crate::error::Status::INVALID_ARGUMENT,
                    message: format!("library path is not valid UTF-8: {}", path.display()),
                })?,
        )
        .map_err(|_| Error {
            status: crate::error::Status::INVALID_ARGUMENT,
            message: format!("library path contains NUL: {}", path.display()),
        })?;
        let loaded = sys::open(&c_path).map_err(|error| Error {
            status: crate::error::Status::INVALID_FORMAT,
            message: error.to_string(),
        })?;
        Ok(Arc::new(Library {
            symbols: loaded.symbols,
        }))
    }

    /// The ABI version reported by the library (checked to be
    /// [`sys::MB_ABI_VERSION`] at open time).
    pub fn abi_version(&self) -> u32 {
        unsafe { (self.symbols.mb_abi_version)() }
    }

    /// Human-readable text for a status code (borrowed from the library and
    /// copied before returning).
    pub fn status_string(&self, status: crate::error::Status) -> String {
        unsafe {
            let pointer = (self.symbols.mb_status_string)(status.0);
            if pointer.is_null() {
                return status.to_string();
            }
            CStr::from_ptr(pointer).to_string_lossy().into_owned()
        }
    }

    /// Convenience mirroring the Go binding's `LoadModel`: create one-shot
    /// runtime options with `device`, load the bundle, free the options.
    pub fn load_model(self: &Arc<Self>, bundle_directory: impl AsRef<Path>, device: Device) -> Result<Model> {
        let options = RuntimeOptions::new(Arc::clone(self))?;
        options.set_device(device)?;
        Model::load(self, bundle_directory, Some(&options))
    }

    /// Invoke a status-returning native call and translate failures.
    pub(crate) fn check<T>(
        &self,
        operation: &str,
        status: sys::MbStatus,
        buffer: &[u8],
        ok: impl FnOnce() -> T,
    ) -> Result<T> {
        if status == sys::MB_OK {
            return Ok(ok());
        }
        Err(take_error(
            |code| self.status_string(crate::error::Status(code)),
            operation,
            status,
            buffer,
        ))
    }

    /// Translate a bare status into `Err` (no success payload).
    pub(crate) fn translate(&self, operation: &str, status: sys::MbStatus, buffer: &[u8]) -> Result<()> {
        self.check(operation, status, buffer, || ())
    }

    pub(crate) fn buffer(&self) -> [u8; sys::MB_ERROR_CAPACITY as usize] {
        error_buffer()
    }
}
