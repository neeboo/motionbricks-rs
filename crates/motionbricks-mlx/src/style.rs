//! Style handle (`mb_style`): a named, speed-parameterised motion style.

use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;

use crate::ffi as sys;

use crate::error::{simple_error, write_error_buffer, Result};
use crate::library::Library;
use crate::model::Model;

/// A motion style loaded from a `.mbstyle` file. Cheap to clone; every clone
/// refers to the same native handle, and the handle is freed when the last
/// clone drops.
#[derive(Clone)]
pub struct Style {
    pub(crate) library: Arc<Library>,
    // The style is loaded through a model; keep the model alive while any
    // style handle exists so the native pointer can never dangle.
    #[allow(dead_code)]
    model: Model,
    raw: Arc<RawStyle>,
    /// Style name, read once at load time.
    pub name: String,
    /// Configured walking speed in metres per second, read at load time.
    pub speed: f32,
}

struct RawStyle {
    library: Arc<Library>,
    raw: *mut sys::mb_style,
}

impl Drop for RawStyle {
    fn drop(&mut self) {
        // Safety: `library` keeps the dynamic library (and therefore the
        // symbols) alive for as long as this handle exists.
        if !self.raw.is_null() {
            unsafe { (self.library.symbols.mb_style_free)(self.raw) }
        }
    }
}

impl std::fmt::Debug for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Style")
            .field("name", &self.name)
            .field("speed", &self.speed)
            .finish()
    }
}

impl Style {
    pub(crate) fn load(model: &Model, style_path: impl AsRef<Path>) -> Result<Style> {
        let library = model.library();
        let path = style_path.as_ref();
        let c_path = CString::new(
            path.to_str()
                .ok_or_else(|| simple_error("load style", "path is not valid UTF-8"))?,
        )
        .map_err(|_| simple_error("load style", "path contains NUL"))?;

        let mut buffer = library.buffer();
        let mut raw: *mut sys::mb_style = std::ptr::null_mut();
        let status = unsafe {
            (library.symbols.mb_style_load)(
                model.raw(),
                c_path.as_ptr(),
                &mut raw,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        library.check("load style", status, &buffer, || raw)?;

        let mut name_buffer = library.buffer();
        let mut name_pointer: *const std::os::raw::c_char = std::ptr::null();
        let status = unsafe {
            (library.symbols.mb_style_get_name)(
                raw,
                &mut name_pointer,
                write_error_buffer(&mut name_buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        let name_result: Result<String> = library.check("get style name", status, &name_buffer, || unsafe {
            std::ffi::CStr::from_ptr(name_pointer).to_string_lossy().into_owned()
        });

        let mut speed_buffer = library.buffer();
        let mut speed: f32 = 0.0;
        let status = unsafe {
            (library.symbols.mb_style_get_speed)(
                raw,
                &mut speed,
                write_error_buffer(&mut speed_buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        let speed_result: Result<f32> = library.check("get style speed", status, &speed_buffer, || speed);

        // On metadata failure, free before propagating.
        let (name, speed) = match (name_result, speed_result) {
            (Ok(name), Ok(speed)) => (name, speed),
            (Err(error), _) | (_, Err(error)) => {
                unsafe { (library.symbols.mb_style_free)(raw) };
                return Err(error);
            }
        };

        Ok(Style {
            library: Arc::clone(library),
            model: model.clone(),
            raw: Arc::new(RawStyle {
                library: Arc::clone(library),
                raw,
            }),
            name,
            speed,
        })
    }

    pub(crate) fn raw(&self) -> *const sys::mb_style {
        self.raw.raw
    }

    /// Update the style's speed in metres per second (live native call).
    pub fn set_speed(&self, metres_per_second: f32) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_style_set_speed)(
                self.raw.raw,
                metres_per_second,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("set style speed", status, &buffer, || ())
    }

    /// Query the style's speed (live native call, unlike the cached
    /// [`Style::speed`] field).
    pub fn get_speed(&self) -> Result<f32> {
        let mut buffer = self.library.buffer();
        let mut speed: f32 = 0.0;
        let status = unsafe {
            (self.library.symbols.mb_style_get_speed)(
                self.raw.raw,
                &mut speed,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get style speed", status, &buffer, || speed)
    }
}
