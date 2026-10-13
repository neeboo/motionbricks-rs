//! Agent handle (`mb_agent`): stateful animation planner for one robot.

use crate::ffi as sys;

use crate::error::{simple_error, write_error_buffer, Result};
use crate::model::Model;
use crate::motion::Motion;
use crate::style::Style;

/// One stateful robot. An agent keeps animation state between plans
/// (`reset`, `advance`, `set_context`) and must be used serially.
pub struct Agent {
    pub(crate) model: Model,
    raw: *mut sys::mb_agent,
}

impl Drop for Agent {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { (self.model.library.symbols.mb_agent_free)(self.raw) }
        }
    }
}

impl std::fmt::Debug for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Agent").finish()
    }
}

impl Agent {
    pub(crate) fn create(model: &Model) -> Result<Agent> {
        let mut buffer = model.library.buffer();
        let mut raw: *mut sys::mb_agent = std::ptr::null_mut();
        let status = unsafe {
            (model.library.symbols.mb_agent_create)(
                model.raw(),
                &mut raw,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        model.library.check("create agent", status, &buffer, || Agent {
            model: model.clone(),
            raw,
        })
    }

    pub(crate) fn raw(&self) -> *const sys::mb_agent {
        self.raw
    }

    /// (Re-)initialise the agent's animation state from `initial_style`.
    pub fn reset(&self, initial_style: &Style) -> Result<()> {
        let mut buffer = self.model.library.buffer();
        let status = unsafe {
            (self.model.library.symbols.mb_agent_reset)(
                self.raw,
                initial_style.raw(),
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.model.library.check("reset agent", status, &buffer, || ())
    }

    /// Advance the internal animation clock by `frames` frames without
    /// planning (seeds continuity between separately planned segments).
    pub fn advance(&self, frames: u32) -> Result<()> {
        let mut buffer = self.model.library.buffer();
        let status = unsafe {
            (self.model.library.symbols.mb_agent_advance)(
                self.raw,
                frames,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.model
            .library
            .check("advance agent", status, &buffer, || ())
    }

    /// Seed the animation state from an existing pose sequence: `roots` is
    /// row-major `[frames][3]` and `rotations` row-major
    /// `[frames][joints][4]` (XYZW).
    pub fn set_context(&self, roots: &[f32], rotations: &[f32], frames: u64, joints: u64) -> Result<()> {
        if roots.len() as u64 != frames * 3 || rotations.len() as u64 != frames * joints * 4 {
            return Err(simple_error(
                "set agent context",
                "context shape mismatch: expected roots frames*3 and rotations frames*joints*4",
            ));
        }
        if roots.is_empty() || rotations.is_empty() {
            return Err(simple_error("set agent context", "context buffers must not be empty"));
        }
        let mut buffer = self.model.library.buffer();
        let status = unsafe {
            (self.model.library.symbols.mb_agent_set_context)(
                self.raw,
                roots.as_ptr(),
                rotations.as_ptr(),
                frames,
                joints,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.model
            .library
            .check("set agent context", status, &buffer, || ())
    }

    /// Plan one motion segment. The returned [`Motion`] owns its data (the
    /// native handle is freed before return), so it may outlive this agent.
    pub fn plan(&self, command: &crate::command::Command) -> Result<Motion> {
        let mut buffer = self.model.library.buffer();
        let mut raw: *mut sys::mb_motion = std::ptr::null_mut();
        let status = unsafe {
            (self.model.library.symbols.mb_agent_plan)(
                self.raw,
                command.raw(),
                &mut raw,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.model.library.check("plan motion", status, &buffer, || ())?;
        // Read (copy) then free, mirroring the Go binding: callers own data.
        let motion = Motion::read(&self.model.library, raw);
        unsafe { (self.model.library.symbols.mb_motion_free)(raw) };
        motion
    }
}
