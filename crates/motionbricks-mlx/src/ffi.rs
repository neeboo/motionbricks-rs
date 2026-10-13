//! Raw FFI bindings for the motionbricks C ABI (version 1).
//!
//! Mirrors `include/motionbricks/motionbricks.h` one-to-one. The dynamic
//! library is loaded at runtime through `libloading`, exactly like the
//! reference PureGo binding: there is no build-time link step and no
//! struct layouts cross the ABI — every handle is opaque.
//!
//! The [`Symbols`] struct holds one function pointer per C entry point.
//! Load it with [`open`] (which also verifies [`MB_ABI_VERSION`]).
//!
//! # Safety contract (from the C header)
//!
//! - Every status-returning function takes an error buffer
//!   `(char * error, uint64_t error_capacity)`; on failure the buffer holds a
//!   NUL-terminated message and out-parameters are left untouched (or NULL).
//! - Motion buffers returned by `mb_motion_get_*` are **borrowed**: valid
//!   until `mb_motion_free` on that handle.
//! - Callers must serialize calls sharing a model, agent, style or command.
//! - The loaded dynamic library is intentionally never unloaded; unloading a
//!   library whose function pointers may still be referenced is unsound.

// This module mirrors the C header verbatim: the symbol names are the
// documentation, so the crate-wide missing_docs lint is relaxed here.
#![allow(missing_docs)]

use std::os::raw::c_char;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

/// ABI version this crate is generated against (`MB_ABI_VERSION`).
pub const MB_ABI_VERSION: u32 = 1;

/// `mb_status` codes.
pub type MbStatus = u32;
pub const MB_OK: MbStatus = 0;
pub const MB_INVALID_ARGUMENT: MbStatus = 1;
pub const MB_OUT_OF_MEMORY: MbStatus = 2;
pub const MB_IO_ERROR: MbStatus = 3;
pub const MB_INVALID_FORMAT: MbStatus = 4;
pub const MB_INCOMPATIBLE_MODEL: MbStatus = 5;
pub const MB_BACKEND_UNAVAILABLE: MbStatus = 6;
pub const MB_COMPUTE_FAILED: MbStatus = 7;
pub const MB_NOT_IMPLEMENTED: MbStatus = 8;
pub const MB_INTERNAL_ERROR: MbStatus = 9;

/// `mb_device` codes.
pub type MbDevice = u32;
pub const MB_DEVICE_AUTO: MbDevice = 0;
pub const MB_DEVICE_CPU: MbDevice = 1;
pub const MB_DEVICE_VULKAN: MbDevice = 2;

/// Opaque handles: no struct layout ever crosses the ABI.
#[repr(C)]
pub struct mb_runtime_options {
    _private: [u8; 0],
}
#[repr(C)]
pub struct mb_model {
    _private: [u8; 0],
}
#[repr(C)]
pub struct mb_style {
    _private: [u8; 0],
}
#[repr(C)]
pub struct mb_command {
    _private: [u8; 0],
}
#[repr(C)]
pub struct mb_agent {
    _private: [u8; 0],
}
#[repr(C)]
pub struct mb_motion {
    _private: [u8; 0],
}

/// Error buffer size used by the reference bindings (Go: `errorBufferSize`).
pub const MB_ERROR_CAPACITY: u64 = 1024;

// ---------------------------------------------------------------------------
// Function pointer types, in header order.
// ---------------------------------------------------------------------------

pub type FnAbiVersion = unsafe extern "C" fn() -> u32;
pub type FnStatusString = unsafe extern "C" fn(status: MbStatus) -> *const c_char;

pub type FnRuntimeOptionsCreate =
    unsafe extern "C" fn(output: *mut *mut mb_runtime_options, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnRuntimeOptionsFree = unsafe extern "C" fn(value: *mut mb_runtime_options);
pub type FnRuntimeOptionsSetDevice =
    unsafe extern "C" fn(value: *mut mb_runtime_options, device: MbDevice, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnRuntimeOptionsGetDevice =
    unsafe extern "C" fn(value: *const mb_runtime_options, output: *mut MbDevice, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnRuntimeOptionsSetThreads =
    unsafe extern "C" fn(value: *mut mb_runtime_options, threads: u32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnRuntimeOptionsGetThreads =
    unsafe extern "C" fn(value: *const mb_runtime_options, output: *mut u32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnRuntimeOptionsSetBackendDirectory =
    unsafe extern "C" fn(value: *mut mb_runtime_options, directory: *const c_char, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnRuntimeOptionsGetBackendDirectory =
    unsafe extern "C" fn(value: *const mb_runtime_options, output: *mut *const c_char, error: *mut c_char, error_capacity: u64) -> MbStatus;

pub type FnCommandCreate =
    unsafe extern "C" fn(output: *mut *mut mb_command, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandFree = unsafe extern "C" fn(value: *mut mb_command);
pub type FnCommandSetStyle =
    unsafe extern "C" fn(value: *mut mb_command, style: *const mb_style, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandGetStyle =
    unsafe extern "C" fn(value: *const mb_command, output: *mut *const mb_style, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandSetVec3 =
    unsafe extern "C" fn(value: *mut mb_command, x: f32, y: f32, z: f32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandGetVec3 =
    unsafe extern "C" fn(value: *const mb_command, x: *mut f32, y: *mut f32, z: *mut f32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandSetTargetSpeed =
    unsafe extern "C" fn(value: *mut mb_command, metres_per_second: f32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandGetTargetSpeed =
    unsafe extern "C" fn(value: *const mb_command, output: *mut f32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandSetWorldTarget = unsafe extern "C" fn(
    value: *mut mb_command,
    x: f32,
    y: f32,
    z: f32,
    heading_radians: f32,
    enabled: u32,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;
pub type FnCommandGetWorldTarget = unsafe extern "C" fn(
    value: *const mb_command,
    x: *mut f32,
    y: *mut f32,
    z: *mut f32,
    heading_radians: *mut f32,
    enabled: *mut u32,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;
pub type FnCommandSetSeed =
    unsafe extern "C" fn(value: *mut mb_command, seed: u64, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandGetSeed =
    unsafe extern "C" fn(value: *const mb_command, output: *mut u64, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandSetSamplingArgmax =
    unsafe extern "C" fn(value: *mut mb_command, enabled: u32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnCommandGetSamplingArgmax =
    unsafe extern "C" fn(value: *const mb_command, output: *mut u32, error: *mut c_char, error_capacity: u64) -> MbStatus;

pub type FnModelLoad = unsafe extern "C" fn(
    bundle_directory: *const c_char,
    options: *const mb_runtime_options,
    output: *mut *mut mb_model,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;
pub type FnModelFree = unsafe extern "C" fn(value: *mut mb_model);
pub type FnModelGetParameterCount =
    unsafe extern "C" fn(value: *const mb_model, output: *mut u64, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnModelGetJointCount =
    unsafe extern "C" fn(value: *const mb_model, output: *mut u32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnModelGetJointName = unsafe extern "C" fn(
    value: *const mb_model,
    joint: u32,
    output: *mut *const c_char,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;
pub type FnModelGetJointParent =
    unsafe extern "C" fn(value: *const mb_model, joint: u32, output: *mut i32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnModelGetNeutralJointPosition = unsafe extern "C" fn(
    value: *const mb_model,
    joint: u32,
    x: *mut f32,
    y: *mut f32,
    z: *mut f32,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;

pub type FnStyleLoad = unsafe extern "C" fn(
    model: *const mb_model,
    style_path: *const c_char,
    output: *mut *mut mb_style,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;
pub type FnStyleFree = unsafe extern "C" fn(value: *mut mb_style);
pub type FnStyleGetName =
    unsafe extern "C" fn(value: *const mb_style, output: *mut *const c_char, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnStyleSetSpeed =
    unsafe extern "C" fn(value: *mut mb_style, metres_per_second: f32, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnStyleGetSpeed =
    unsafe extern "C" fn(value: *const mb_style, output: *mut f32, error: *mut c_char, error_capacity: u64) -> MbStatus;

pub type FnAgentCreate =
    unsafe extern "C" fn(model: *const mb_model, output: *mut *mut mb_agent, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnAgentFree = unsafe extern "C" fn(value: *mut mb_agent);
pub type FnAgentReset =
    unsafe extern "C" fn(value: *mut mb_agent, initial_style: *const mb_style, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnAgentSetContext = unsafe extern "C" fn(
    value: *mut mb_agent,
    root_xyz: *const f32,
    local_rotation_xyzw: *const f32,
    frames: u64,
    joints: u64,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;
pub type FnAgentPlan = unsafe extern "C" fn(
    value: *mut mb_agent,
    command: *const mb_command,
    output: *mut *mut mb_motion,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;
pub type FnAgentPlanBatch = unsafe extern "C" fn(
    agents: *const *const mb_agent,
    commands: *const *const mb_command,
    count: u64,
    outputs: *mut *mut mb_motion,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;
pub type FnAgentAdvance =
    unsafe extern "C" fn(value: *mut mb_agent, frames: u32, error: *mut c_char, error_capacity: u64) -> MbStatus;

pub type FnMotionFree = unsafe extern "C" fn(value: *mut mb_motion);
pub type FnMotionGetCount =
    unsafe extern "C" fn(value: *const mb_motion, output: *mut u64, error: *mut c_char, error_capacity: u64) -> MbStatus;
pub type FnMotionGetBuffer = unsafe extern "C" fn(
    value: *const mb_motion,
    output: *mut *const f32,
    values: *mut u64,
    error: *mut c_char,
    error_capacity: u64,
) -> MbStatus;

// ---------------------------------------------------------------------------
// Symbol table.
// ---------------------------------------------------------------------------

/// One function pointer per C entry point, resolved once at load time.
///
/// Obtain through [`open`]. Keeping a `Symbols` alive after dropping the
/// `libloading::Library` is safe by construction here because [`open`]
/// intentionally leaks the library handle (the C library is never unloaded).
#[derive(Debug, Clone, Copy)]
pub struct Symbols {
    pub mb_abi_version: FnAbiVersion,
    pub mb_status_string: FnStatusString,

    pub mb_runtime_options_create: FnRuntimeOptionsCreate,
    pub mb_runtime_options_free: FnRuntimeOptionsFree,
    pub mb_runtime_options_set_device: FnRuntimeOptionsSetDevice,
    pub mb_runtime_options_get_device: FnRuntimeOptionsGetDevice,
    pub mb_runtime_options_set_threads: FnRuntimeOptionsSetThreads,
    pub mb_runtime_options_get_threads: FnRuntimeOptionsGetThreads,
    pub mb_runtime_options_set_backend_directory: FnRuntimeOptionsSetBackendDirectory,
    pub mb_runtime_options_get_backend_directory: FnRuntimeOptionsGetBackendDirectory,

    pub mb_command_create: FnCommandCreate,
    pub mb_command_free: FnCommandFree,
    pub mb_command_set_style: FnCommandSetStyle,
    pub mb_command_get_style: FnCommandGetStyle,
    pub mb_command_set_movement_direction: FnCommandSetVec3,
    pub mb_command_get_movement_direction: FnCommandGetVec3,
    pub mb_command_set_facing_direction: FnCommandSetVec3,
    pub mb_command_get_facing_direction: FnCommandGetVec3,
    pub mb_command_set_target_speed: FnCommandSetTargetSpeed,
    pub mb_command_get_target_speed: FnCommandGetTargetSpeed,
    pub mb_command_set_world_target: FnCommandSetWorldTarget,
    pub mb_command_get_world_target: FnCommandGetWorldTarget,
    pub mb_command_set_seed: FnCommandSetSeed,
    pub mb_command_get_seed: FnCommandGetSeed,
    pub mb_command_set_sampling_argmax: FnCommandSetSamplingArgmax,
    pub mb_command_get_sampling_argmax: FnCommandGetSamplingArgmax,

    pub mb_model_load: FnModelLoad,
    pub mb_model_free: FnModelFree,
    pub mb_model_get_parameter_count: FnModelGetParameterCount,
    pub mb_model_get_joint_count: FnModelGetJointCount,
    pub mb_model_get_joint_name: FnModelGetJointName,
    pub mb_model_get_joint_parent: FnModelGetJointParent,
    pub mb_model_get_neutral_joint_position: FnModelGetNeutralJointPosition,

    pub mb_style_load: FnStyleLoad,
    pub mb_style_free: FnStyleFree,
    pub mb_style_get_name: FnStyleGetName,
    pub mb_style_set_speed: FnStyleSetSpeed,
    pub mb_style_get_speed: FnStyleGetSpeed,

    pub mb_agent_create: FnAgentCreate,
    pub mb_agent_free: FnAgentFree,
    pub mb_agent_reset: FnAgentReset,
    pub mb_agent_set_context: FnAgentSetContext,
    pub mb_agent_plan: FnAgentPlan,
    pub mb_agent_plan_batch: FnAgentPlanBatch,
    pub mb_agent_advance: FnAgentAdvance,

    pub mb_motion_free: FnMotionFree,
    pub mb_motion_get_frame_count: FnMotionGetCount,
    pub mb_motion_get_joint_count: FnMotionGetCount,
    pub mb_motion_get_root_translations: FnMotionGetBuffer,
    pub mb_motion_get_local_rotations_xyzw: FnMotionGetBuffer,
    pub mb_motion_get_target_frame_count: FnMotionGetCount,
    pub mb_motion_get_target_root_translations: FnMotionGetBuffer,
    pub mb_motion_get_target_local_rotations_xyzw: FnMotionGetBuffer,
}

/// Result of [`open`]: the resolved symbol table plus the ABI version
/// reported by the library (checked to equal [`MB_ABI_VERSION`]).
pub struct Loaded {
    pub symbols: Symbols,
}

/// Load a motionbricks dynamic library and resolve every symbol.
///
/// The library is dlopen'd (or LoadLibrary'd on Windows) at `path`, all
/// `mb_*` entry points are resolved eagerly (so a mismatched/older library
/// fails here rather than mid-plan), and the reported ABI version is checked
/// against [`MB_ABI_VERSION`].
///
/// The library is deliberately never unloaded: see the module docs.
///
/// # Errors
///
/// Returns the `libloading` error if the file cannot be opened or any
/// symbol is missing, and an [`AbiMismatch`] error if the library reports a
/// different ABI version.
pub fn open(path: &std::ffi::CStr) -> Result<Loaded, OpenError> {
    // libloading on unix accepts an OsStr; pass the C string bytes through
    // `CString` so the trailing NUL and the platform encoding are both right.
    let library = unsafe {
        libloading::Library::new(std::ffi::OsStr::from_bytes(path.to_bytes_with_nul()))
    }
    .map_err(OpenError::Load)?;
    let symbols = Symbols::load(&library).map_err(OpenError::Load)?;
    // Leak the handle: fn pointers must outlive any drop.
    std::mem::forget(library);
    let version = unsafe { (symbols.mb_abi_version)() };
    if version != MB_ABI_VERSION {
        return Err(OpenError::AbiMismatch {
            expected: MB_ABI_VERSION,
            found: version,
        });
    }
    Ok(Loaded { symbols })
}

/// Failure modes of [`open`].
#[derive(Debug)]
pub enum OpenError {
    /// dlopen/LoadLibrary or symbol resolution failed.
    Load(libloading::Error),
    /// The library reported an unsupported ABI version.
    AbiMismatch { expected: u32, found: u32 },
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::Load(error) => write!(f, "cannot load motionbricks library: {error}"),
            OpenError::AbiMismatch { expected, found } => {
                write!(f, "motionbricks ABI version {found} is unsupported (expected {expected})")
            }
        }
    }
}

impl std::error::Error for OpenError {}

impl Symbols {
    fn load(library: &libloading::Library) -> Result<Symbols, libloading::Error> {
        // `library.get` requires a trailing NUL byte, hence `concat!(name, "\0")`.
        macro_rules! c_symbol {
            ($name:literal) => {
                unsafe { *library.get(concat!($name, "\0").as_bytes())? }
            };
        }
        Ok(Symbols {
            mb_abi_version: c_symbol!("mb_abi_version"),
            mb_status_string: c_symbol!("mb_status_string"),
            mb_runtime_options_create: c_symbol!("mb_runtime_options_create"),
            mb_runtime_options_free: c_symbol!("mb_runtime_options_free"),
            mb_runtime_options_set_device: c_symbol!("mb_runtime_options_set_device"),
            mb_runtime_options_get_device: c_symbol!("mb_runtime_options_get_device"),
            mb_runtime_options_set_threads: c_symbol!("mb_runtime_options_set_threads"),
            mb_runtime_options_get_threads: c_symbol!("mb_runtime_options_get_threads"),
            mb_runtime_options_set_backend_directory: c_symbol!("mb_runtime_options_set_backend_directory"),
            mb_runtime_options_get_backend_directory: c_symbol!("mb_runtime_options_get_backend_directory"),
            mb_command_create: c_symbol!("mb_command_create"),
            mb_command_free: c_symbol!("mb_command_free"),
            mb_command_set_style: c_symbol!("mb_command_set_style"),
            mb_command_get_style: c_symbol!("mb_command_get_style"),
            mb_command_set_movement_direction: c_symbol!("mb_command_set_movement_direction"),
            mb_command_get_movement_direction: c_symbol!("mb_command_get_movement_direction"),
            mb_command_set_facing_direction: c_symbol!("mb_command_set_facing_direction"),
            mb_command_get_facing_direction: c_symbol!("mb_command_get_facing_direction"),
            mb_command_set_target_speed: c_symbol!("mb_command_set_target_speed"),
            mb_command_get_target_speed: c_symbol!("mb_command_get_target_speed"),
            mb_command_set_world_target: c_symbol!("mb_command_set_world_target"),
            mb_command_get_world_target: c_symbol!("mb_command_get_world_target"),
            mb_command_set_seed: c_symbol!("mb_command_set_seed"),
            mb_command_get_seed: c_symbol!("mb_command_get_seed"),
            mb_command_set_sampling_argmax: c_symbol!("mb_command_set_sampling_argmax"),
            mb_command_get_sampling_argmax: c_symbol!("mb_command_get_sampling_argmax"),
            mb_model_load: c_symbol!("mb_model_load"),
            mb_model_free: c_symbol!("mb_model_free"),
            mb_model_get_parameter_count: c_symbol!("mb_model_get_parameter_count"),
            mb_model_get_joint_count: c_symbol!("mb_model_get_joint_count"),
            mb_model_get_joint_name: c_symbol!("mb_model_get_joint_name"),
            mb_model_get_joint_parent: c_symbol!("mb_model_get_joint_parent"),
            mb_model_get_neutral_joint_position: c_symbol!("mb_model_get_neutral_joint_position"),
            mb_style_load: c_symbol!("mb_style_load"),
            mb_style_free: c_symbol!("mb_style_free"),
            mb_style_get_name: c_symbol!("mb_style_get_name"),
            mb_style_set_speed: c_symbol!("mb_style_set_speed"),
            mb_style_get_speed: c_symbol!("mb_style_get_speed"),
            mb_agent_create: c_symbol!("mb_agent_create"),
            mb_agent_free: c_symbol!("mb_agent_free"),
            mb_agent_reset: c_symbol!("mb_agent_reset"),
            mb_agent_set_context: c_symbol!("mb_agent_set_context"),
            mb_agent_plan: c_symbol!("mb_agent_plan"),
            mb_agent_plan_batch: c_symbol!("mb_agent_plan_batch"),
            mb_agent_advance: c_symbol!("mb_agent_advance"),
            mb_motion_free: c_symbol!("mb_motion_free"),
            mb_motion_get_frame_count: c_symbol!("mb_motion_get_frame_count"),
            mb_motion_get_joint_count: c_symbol!("mb_motion_get_joint_count"),
            mb_motion_get_root_translations: c_symbol!("mb_motion_get_root_translations"),
            mb_motion_get_local_rotations_xyzw: c_symbol!("mb_motion_get_local_rotations_xyzw"),
            mb_motion_get_target_frame_count: c_symbol!("mb_motion_get_target_frame_count"),
            mb_motion_get_target_root_translations: c_symbol!("mb_motion_get_target_root_translations"),
            mb_motion_get_target_local_rotations_xyzw: c_symbol!("mb_motion_get_target_local_rotations_xyzw"),
        })
    }
}
