//! Command handle (`mb_command`): one planning request, reusable across plans.

use std::sync::Arc;

use crate::ffi as sys;

use crate::error::{write_error_buffer, Result};
use crate::library::Library;
use crate::style::Style;

/// A planning command: style, movement/facing direction, optional world
/// target, seed and sampling mode. Cheap to construct; reusable between
/// plans (each plan resets its private RNG from the seed).
pub struct Command {
    pub(crate) library: Arc<Library>,
    raw: *mut sys::mb_command,
    // The native command stores the style pointer we pass; keep the style
    // alive for the command's whole life so the pointer can never dangle,
    // whatever the native retention semantics are.
    style: Option<Style>,
}

impl Drop for Command {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { (self.library.symbols.mb_command_free)(self.raw) }
        }
    }
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").field("seed_set", &true).finish()
    }
}

impl Command {
    /// Create an empty command against `library`.
    pub fn new(library: &Arc<Library>) -> Result<Command> {
        let mut buffer = library.buffer();
        let mut raw: *mut sys::mb_command = std::ptr::null_mut();
        let status = unsafe {
            (library.symbols.mb_command_create)(
                &mut raw,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        library.check("create command", status, &buffer, || Command {
            library: Arc::clone(library),
            raw,
            style: None,
        })
    }

    pub(crate) fn library(&self) -> &Arc<Library> {
        &self.library
    }

    pub(crate) fn raw(&self) -> *const sys::mb_command {
        self.raw
    }

    /// Set the style this command plans with. The style is retained by the
    /// command (the handle stays valid even if the caller drops their clone).
    pub fn set_style(&mut self, style: &Style) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_command_set_style)(
                self.raw,
                style.raw(),
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("set command style", status, &buffer, || {
            self.style = Some(style.clone());
        })
    }

    /// The style previously set with [`Command::set_style`], if any.
    pub fn style(&self) -> Option<&Style> {
        self.style.as_ref()
    }

    /// Unit movement direction; typically `(x, 0, z)` on the ground plane.
    pub fn set_movement_direction(&mut self, x: f32, y: f32, z: f32) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_command_set_movement_direction)(
                self.raw,
                x,
                y,
                z,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("set movement", status, &buffer, || ())
    }

    /// Query the movement direction previously set.
    pub fn movement_direction(&self) -> Result<[f32; 3]> {
        let mut buffer = self.library.buffer();
        let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
        let status = unsafe {
            (self.library.symbols.mb_command_get_movement_direction)(
                self.raw,
                &mut x,
                &mut y,
                &mut z,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get movement", status, &buffer, || [x, y, z])
    }

    /// Unit facing direction; typically `(x, 0, z)`.
    pub fn set_facing_direction(&mut self, x: f32, y: f32, z: f32) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_command_set_facing_direction)(
                self.raw,
                x,
                y,
                z,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("set facing", status, &buffer, || ())
    }

    /// Query the facing direction previously set.
    pub fn facing_direction(&self) -> Result<[f32; 3]> {
        let mut buffer = self.library.buffer();
        let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
        let status = unsafe {
            (self.library.symbols.mb_command_get_facing_direction)(
                self.raw,
                &mut x,
                &mut y,
                &mut z,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get facing", status, &buffer, || [x, y, z])
    }

    /// Target speed in metres per second; a negative value selects the
    /// style's configured speed.
    pub fn set_target_speed(&mut self, metres_per_second: f32) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_command_set_target_speed)(
                self.raw,
                metres_per_second,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("set speed", status, &buffer, || ())
    }

    /// Query the target speed (negative selects the style's speed).
    pub fn target_speed(&self) -> Result<f32> {
        let mut buffer = self.library.buffer();
        let mut speed: f32 = 0.0;
        let status = unsafe {
            (self.library.symbols.mb_command_get_target_speed)(
                self.raw,
                &mut speed,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get speed", status, &buffer, || speed)
    }

    /// Place the robot at a world position with a heading (radians).
    pub fn set_world_target(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        heading_radians: f32,
        enabled: bool,
    ) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_command_set_world_target)(
                self.raw,
                x,
                y,
                z,
                heading_radians,
                enabled as u32,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("set world target", status, &buffer, || ())
    }

    /// Query the world target as `(position, heading_radians, enabled)`.
    #[allow(clippy::type_complexity)]
    pub fn world_target(&self) -> Result<([f32; 3], f32, bool)> {
        let mut buffer = self.library.buffer();
        let (mut x, mut y, mut z, mut heading): (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 0.0);
        let mut enabled: u32 = 0;
        let status = unsafe {
            (self.library.symbols.mb_command_get_world_target)(
                self.raw,
                &mut x,
                &mut y,
                &mut z,
                &mut heading,
                &mut enabled,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library
            .check("get world target", status, &buffer, || ([x, y, z], heading, enabled != 0))
    }

    /// RNG seed; each plan resets its private RNG from this value, so equal
    /// inputs and seed reproduce the same motion.
    pub fn set_seed(&mut self, seed: u64) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_command_set_seed)(
                self.raw,
                seed,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("set seed", status, &buffer, || ())
    }

    /// Query the seed previously set.
    pub fn seed(&self) -> Result<u64> {
        let mut buffer = self.library.buffer();
        let mut seed: u64 = 0;
        let status = unsafe {
            (self.library.symbols.mb_command_get_seed)(
                self.raw,
                &mut seed,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get seed", status, &buffer, || seed)
    }

    /// Diagnostic deterministic sampling. Default `false`: upstream-style
    /// temperature-1 Gumbel sampling from the command seed.
    pub fn set_sampling_argmax(&mut self, enabled: bool) -> Result<()> {
        let mut buffer = self.library.buffer();
        let status = unsafe {
            (self.library.symbols.mb_command_set_sampling_argmax)(
                self.raw,
                enabled as u32,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("set sampling", status, &buffer, || ())
    }

    /// Query whether deterministic argmax sampling is enabled.
    pub fn sampling_argmax(&self) -> Result<bool> {
        let mut buffer = self.library.buffer();
        let mut enabled: u32 = 0;
        let status = unsafe {
            (self.library.symbols.mb_command_get_sampling_argmax)(
                self.raw,
                &mut enabled,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library
            .check("get sampling", status, &buffer, || enabled != 0)
    }
}
