//! Runtime options handle (`mb_runtime_options`).

use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;

use crate::ffi as sys;

use crate::error::{simple_error, write_error_buffer, Result};
use crate::library::Library;

/// Opaque runtime options passed to [`crate::Model::load`].
///
/// Options only need to live until the model load returns; the C library
/// copies what it needs. Freeing happens automatically on drop.
pub struct RuntimeOptions {
    pub(crate) library: Arc<Library>,
    raw: *mut sys::mb_runtime_options,
}

impl RuntimeOptions {
    pub(crate) fn as_raw(&self) -> *const sys::mb_runtime_options {
        self.raw
    }
}

// Safety: the C library requires calls sharing an options handle to be
// serialized; we expose &mut/&self methods only, and the raw pointer makes
// the type !Send/!Sync by default, which matches that contract.
impl Drop for RuntimeOptions {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { (self.library.symbols.mb_runtime_options_free)(self.raw) }
        }
    }
}

impl RuntimeOptions {
    /// Create an empty options handle (library defaults: device AUTO).
    pub fn new(library: Arc<Library>) -> Result<RuntimeOptions> {
        let mut buffer = library.buffer();
        let mut raw: *mut sys::mb_runtime_options = std::ptr::null_mut();
        let status = unsafe {
            (library.symbols.mb_runtime_options_create)(
                &mut raw,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        library.check("create runtime options", status, &buffer, || RuntimeOptions {
            library: Arc::clone(&library),
            raw,
        })
    }

    /// Select the execution device.
    pub fn set_device(&self, device: Device) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_runtime_options_set_device)(
                self.raw,
                device.to_raw(),
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library
            .check("select device", status, &buffer, || ())
    }

    /// Query the selected device.
    pub fn device(&self) -> Result<Device> {
        let mut buffer = self.library.buffer();
        let mut device: sys::MbDevice = 0;
        let status = unsafe {
            (self.library.symbols.mb_runtime_options_get_device)(
                self.raw,
                &mut device,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library
            .check("get device", status, &buffer, || Device::from_raw(device))
    }

    /// Set the CPU thread count used by the backend.
    pub fn set_threads(&self, threads: u32) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_runtime_options_set_threads)(
                self.raw,
                threads,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library
            .check("set threads", status, &buffer, || ())
    }

    /// Query the CPU thread count.
    pub fn threads(&self) -> Result<u32> {
        let mut buffer = self.library.buffer();
        let mut threads: u32 = 0;
        let status = unsafe {
            (self.library.symbols.mb_runtime_options_get_threads)(
                self.raw,
                &mut threads,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get threads", status, &buffer, || threads)
    }

    /// Override backend discovery before the first model load in this
    /// process. `None` clears the override.
    pub fn set_backend_directory(&self, directory: Option<&Path>) -> Result<()> {
        let mut buffer = self.library.buffer();
        let c_directory = match directory {
            Some(path) => Some(CString::new(
                path.to_str()
                    .ok_or_else(|| simple_error("set backend directory", "path is not valid UTF-8"))?,
            )
            .map_err(|_| simple_error("set backend directory", "path contains NUL"))?),
            None => None,
        };
        let pointer = c_directory
            .as_ref()
            .map(|value| value.as_ptr())
            .unwrap_or(std::ptr::null());
        let status = unsafe {
            (self.library.symbols.mb_runtime_options_set_backend_directory)(
                self.raw,
                pointer,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library
            .check("set backend directory", status, &buffer, || ())
    }

    /// Query the backend directory override (`None` when unset).
    pub fn backend_directory(&self) -> Result<Option<String>> {
        let mut buffer = self.library.buffer();
        let mut pointer: *const std::os::raw::c_char = std::ptr::null();
        let status = unsafe {
            (self.library.symbols.mb_runtime_options_get_backend_directory)(
                self.raw,
                &mut pointer,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get backend directory", status, &buffer, || {
            if pointer.is_null() {
                None
            } else {
                Some(unsafe { std::ffi::CStr::from_ptr(pointer) }.to_string_lossy().into_owned())
            }
        })
    }
}

/// Execution device (`mb_device`). The numeric values match the C header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Device {
    /// Library default resolution.
    Auto,
    /// CPU-only execution.
    Cpu,
    /// Vulkan backend execution.
    Vulkan,
}

impl Device {
    /// The matching `MB_DEVICE_*` constant.
    pub const fn to_raw(self) -> sys::MbDevice {
        match self {
            Device::Auto => sys::MB_DEVICE_AUTO,
            Device::Cpu => sys::MB_DEVICE_CPU,
            Device::Vulkan => sys::MB_DEVICE_VULKAN,
        }
    }

    /// Interpret a raw `MB_DEVICE_*` value (unknown values read as Auto).
    pub const fn from_raw(value: sys::MbDevice) -> Device {
        match value {
            sys::MB_DEVICE_CPU => Device::Cpu,
            sys::MB_DEVICE_VULKAN => Device::Vulkan,
            _ => Device::Auto,
        }
    }
}
