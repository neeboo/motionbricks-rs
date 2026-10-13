//! Safe Rust bindings for the [motionbricks](https://example.invalid) C ABI
//! (version 1) — the Vulkan flavour of the shared binding.
//!
//! The dynamic library (`libmotionbricks.dylib` / `.so`) is loaded at
//! runtime with [`Library::open`]; no build-time linking and no C struct
//! layouts are involved. All native handles are reference-counted and freed
//! on drop; planned motions are plain owned data.
//!
//! # Vulkan deployment
//!
//! This flavour defaults to `MB_DEVICE_VULKAN` (see [`DEFAULT_DEVICE`]). The
//! native library discovers its compute backends through a backend
//! directory; point it at your deployment's directory with
//! [`RuntimeOptions::set_backend_directory`] before the first model load —
//! see the crate README.
//!
//! # Example
//!
//! ```no_run
//! use motionbricks_vulkan::{Command, Device, Library};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let library = Library::open("libmotionbricks.so")?;
//! let model = library.load_model("g1-f32", Device::Vulkan)?;
//! let style = model.load_style("styles/walk.mbstyle")?;
//!
//! let agent = model.agent()?;
//! agent.reset(&style)?;
//!
//! let mut command = Command::new(&library)?;
//! command.set_style(&style)?;
//! command.set_movement_direction(1.0, 0.0, 0.0)?;
//! command.set_facing_direction(0.0, 0.0, 1.0)?;
//! command.set_seed(1)?;
//!
//! let motion = agent.plan(&command)?;
//! println!("planned {} frames x {} joints", motion.frames, motion.joints);
//! # Ok(())
//! # }
//! ```
//!
//! # Threading
//!
//! The C ABI requires calls sharing a model, agent, style or command to be
//! serialized. Handle types contain raw pointers and are therefore `!Send`
//! and `!Sync`, which enforces that at compile time. Use
//! [`Model::plan_batch`] for native parallel planning of independent robots.
//!
//! # Feature coverage
//!
//! Complete coverage of `motionbricks.h` (agent/command planner API). The
//! SONIC, physics and stateless-inference headers are not bound yet.

#![deny(missing_docs)]
// Handles are deliberately !Send/!Sync (the C ABI requires serialised use),
// so the Arc wrappers are thread-sharing bookkeeping only, not a thread
// safety claim.
#![allow(clippy::arc_with_non_send_sync)]

// Raw FFI layer: one function pointer per C symbol, runtime-loaded.
//! The safe API is built entirely on this module; it is public for
//! advanced callers who need unwrapped access to the symbols.
pub mod ffi;

mod agent;
mod command;
mod error;
mod library;
mod model;
mod motion;
mod options;
mod style;

pub use agent::Agent;
pub use command::Command;
pub use error::{Error, Result, Status};
pub use library::Library;
pub use model::{Joint, Model};
pub use motion::{Keyframes, Motion};
pub use options::{Device, RuntimeOptions};

/// This flavour's default device: `Vulkan`. Pass it explicitly to
/// [`Library::load_model`], or override per call site (tests on machines
/// without a Vulkan backend should use [`Device::Cpu`]).
pub const DEFAULT_DEVICE: Device = Device::Vulkan;
