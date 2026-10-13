//! Model handle (`mb_model`): the neural motion planner plus its skeleton.

use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::Arc;

use crate::ffi as sys;

use crate::agent::Agent;
use crate::command::Command;
use crate::error::{simple_error, write_error_buffer, Result};
use crate::library::Library;
use crate::motion::Motion;
use crate::options::RuntimeOptions;
use crate::style::Style;

/// A loaded motion-bricks model bundle. Cheap to clone; every clone refers to
/// the same native handle, freed when the last clone drops.
#[derive(Clone)]
pub struct Model {
    pub(crate) library: Arc<Library>,
    raw: Arc<RawModel>,
}

struct RawModel {
    library: Arc<Library>,
    raw: *mut sys::mb_model,
}

impl Drop for RawModel {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { (self.library.symbols.mb_model_free)(self.raw) }
        }
    }
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model").field("joints", &self.raw.raw).finish()
    }
}

/// One skeleton joint: name, parent index (`-1` for the root) and the
/// neutral (rest-pose) position in metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Joint {
    /// Joint name from the bundle's manifest.
    pub name: String,
    /// Parent joint index, `-1` for the root joint.
    pub parent: i32,
    /// Neutral (rest-pose) position in metres.
    pub position: [f32; 3],
}

impl Model {
    /// Load a model bundle from `bundle_directory` (containing the GGUF
    /// files and manifest). `options` may be `None` for library defaults.
    pub fn load(
        library: &Arc<Library>,
        bundle_directory: impl AsRef<Path>,
        options: Option<&RuntimeOptions>,
    ) -> Result<Model> {
        let path = bundle_directory.as_ref();
        let c_path = CString::new(
            path.to_str()
                .ok_or_else(|| simple_error("load model", "bundle path is not valid UTF-8"))?,
        )
        .map_err(|_| simple_error("load model", "bundle path contains NUL"))?;

        let mut buffer = library.buffer();
        let mut raw: *mut sys::mb_model = std::ptr::null_mut();
        let options_pointer = match options {
            Some(value) => value.as_raw(),
            None => std::ptr::null(),
        };
        let status = unsafe {
            (library.symbols.mb_model_load)(
                c_path.as_ptr(),
                options_pointer,
                &mut raw,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        library.check("load model", status, &buffer, || Model {
            library: Arc::clone(library),
            raw: Arc::new(RawModel {
                library: Arc::clone(library),
                raw,
            }),
        })
    }

    pub(crate) fn library(&self) -> &Arc<Library> {
        &self.library
    }

    pub(crate) fn raw(&self) -> *const sys::mb_model {
        self.raw.raw
    }

    pub(crate) fn same_handle(&self, other: &Model) -> bool {
        Arc::ptr_eq(&self.raw, &other.raw)
    }

    /// Total neural parameter count across the bundle's GGUF files.
    pub fn parameter_count(&self) -> Result<u64> {
        let mut buffer = self.library.buffer();
        let mut count: u64 = 0;
        let status = unsafe {
            (self.library.symbols.mb_model_get_parameter_count)(
                self.raw.raw,
                &mut count,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get parameter count", status, &buffer, || count)
    }

    /// Number of skeleton joints (34 for G1).
    pub fn joint_count(&self) -> Result<u32> {
        let mut buffer = self.library.buffer();
        let mut count: u32 = 0;
        let status = unsafe {
            (self.library.symbols.mb_model_get_joint_count)(
                self.raw.raw,
                &mut count,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get joint count", status, &buffer, || count)
    }

    /// Name of joint `index` (borrowed from the library and copied).
    pub fn joint_name(&self, index: u32) -> Result<String> {
        let mut buffer = self.library.buffer();
        let mut pointer: *const std::os::raw::c_char = std::ptr::null();
        let status = unsafe {
            (self.library.symbols.mb_model_get_joint_name)(
                self.raw.raw,
                index,
                &mut pointer,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get joint name", status, &buffer, || unsafe {
            CStr::from_ptr(pointer).to_string_lossy().into_owned()
        })
    }

    /// Parent index of joint `index` (`-1` for the root joint).
    pub fn joint_parent(&self, index: u32) -> Result<i32> {
        let mut buffer = self.library.buffer();
        let mut parent: i32 = 0;
        let status = unsafe {
            (self.library.symbols.mb_model_get_joint_parent)(
                self.raw.raw,
                index,
                &mut parent,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.check("get joint parent", status, &buffer, || parent)
    }

    /// Neutral (rest-pose) position of joint `index` as `[x, y, z]` metres.
    pub fn neutral_joint_position(&self, index: u32) -> Result<[f32; 3]> {
        let mut buffer = self.library.buffer();
        let (mut x, mut y, mut z): (f32, f32, f32) = (0.0, 0.0, 0.0);
        let status = unsafe {
            (self.library.symbols.mb_model_get_neutral_joint_position)(
                self.raw.raw,
                index,
                &mut x,
                &mut y,
                &mut z,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library
            .check("get neutral joint position", status, &buffer, || [x, y, z])
    }

    /// The full skeleton, mirroring the Go binding's `Model.Skeleton`.
    pub fn skeleton(&self) -> Result<Vec<Joint>> {
        let count = self.joint_count()?;
        let mut joints = Vec::with_capacity(count as usize);
        for index in 0..count {
            joints.push(Joint {
                name: self.joint_name(index)?,
                parent: self.joint_parent(index)?,
                position: self.neutral_joint_position(index)?,
            });
        }
        Ok(joints)
    }

    /// Load a `.mbstyle` style file through this model.
    pub fn load_style(&self, style_path: impl AsRef<Path>) -> Result<Style> {
        Style::load(self, style_path)
    }

    /// Create an animation agent bound to this model.
    pub fn agent(&self) -> Result<Agent> {
        Agent::create(self)
    }

    /// Plan independent robots through one shared native model call.
    ///
    /// `agents` and `commands` are paired by index; every agent must belong
    /// to this model and every command must belong to the same library.
    /// Count must be 1..=64. On failure no motion is returned and the native
    /// outputs are all freed.
    pub fn plan_batch(&self, agents: &[&Agent], commands: &[&Command]) -> Result<Vec<Motion>> {
        if agents.len() != commands.len() {
            return Err(simple_error(
                "plan motion batch",
                "agent and command slices must have equal length",
            ));
        }
        if agents.is_empty() || agents.len() > 64 {
            return Err(simple_error(
                "plan motion batch",
                "batch requires 1..=64 entries",
            ));
        }
        let agent_pointers: Vec<*const sys::mb_agent> = agents
            .iter()
            .map(|agent| {
                if !agent.model.same_handle(self) {
                    return Err(simple_error(
                        "plan motion batch",
                        "batch contains an agent from a different model",
                    ));
                }
                Ok(agent.raw())
            })
            .collect::<Result<Vec<_>>>()?;
        let command_pointers: Vec<*const sys::mb_command> = commands
            .iter()
            .map(|command| {
                if !Arc::ptr_eq(command.library(), &self.library) {
                    return Err(simple_error(
                        "plan motion batch",
                        "batch contains a command from a different library",
                    ));
                }
                Ok(command.raw())
            })
            .collect::<Result<Vec<_>>>()?;

        let mut buffer = self.library.buffer();
        let mut outputs: Vec<*mut sys::mb_motion> = vec![std::ptr::null_mut(); agents.len()];
        let status = unsafe {
            (self.library.symbols.mb_agent_plan_batch)(
                agent_pointers.as_ptr(),
                command_pointers.as_ptr(),
                agents.len() as u64,
                outputs.as_mut_ptr(),
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            )
        };
        self.library.translate("plan motion batch", status, &buffer)?;
        // Success: every output is a live handle we now own. Motion::read
        // copies the data, so each native handle is freed immediately.
        let mut motions = Vec::with_capacity(outputs.len());
        for (index, raw) in outputs.iter().enumerate() {
            match Motion::read(&self.library, *raw) {
                Ok(motion) => motions.push(motion),
                Err(error) => {
                    // Free this motion and all remaining, unread handles.
                    for raw in &outputs[index..] {
                        if !raw.is_null() {
                            unsafe { (self.library.symbols.mb_motion_free)(*raw) };
                        }
                    }
                    return Err(error);
                }
            }
            unsafe { (self.library.symbols.mb_motion_free)(*raw) };
        }
        Ok(motions)
    }
}
