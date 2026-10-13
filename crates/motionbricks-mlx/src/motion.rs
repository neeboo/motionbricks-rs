//! Owned motion data copied out of the library's borrowed buffers.

use crate::ffi as sys;

use crate::error::{simple_error, write_error_buffer, Result};
use crate::library::Library;

/// One planned motion. All buffers are owned copies: the native `mb_motion`
/// is freed before `plan`/`plan_batch` returns, so a `Motion` may outlive the
/// agent, command and even the library handle that produced it.
#[derive(Debug, Clone, PartialEq)]
pub struct Motion {
    /// Number of animation frames.
    pub frames: u64,
    /// Number of joints per frame (34 for G1).
    pub joints: u64,
    /// Root translations, row-major `[frames][3]` (x, y, z metres).
    pub roots: Vec<f32>,
    /// Local joint rotations, row-major `[frames][joints][4]` (x, y, z, w).
    pub rotations: Vec<f32>,
    /// The style-pose constraints actually supplied to the planner.
    pub targets: Option<Keyframes>,
}

impl Motion {
    pub(crate) fn read(library: &Library, raw: *mut sys::mb_motion) -> Result<Motion> {
        let mut buffer = library.buffer();
        let (mut frames, mut joints) = (0u64, 0u64);
        let mut roots_pointer: *const f32 = std::ptr::null();
        let mut roots_count: u64 = 0;
        let mut rotations_pointer: *const f32 = std::ptr::null();
        let mut rotations_count: u64 = 0;
        let mut target_frames: u64 = 0;
        let mut target_roots_pointer: *const f32 = std::ptr::null();
        let mut target_roots_count: u64 = 0;
        let mut target_rotations_pointer: *const f32 = std::ptr::null();
        let mut target_rotations_count: u64 = 0;

        unsafe {
            let mut status = (library.symbols.mb_motion_get_frame_count)(
                raw,
                &mut frames,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            );
            library.check("get frame count", status, &buffer, || ())?;
            status = (library.symbols.mb_motion_get_joint_count)(
                raw,
                &mut joints,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            );
            library.check("get joint count", status, &buffer, || ())?;
            status = (library.symbols.mb_motion_get_root_translations)(
                raw,
                &mut roots_pointer,
                &mut roots_count,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            );
            library.check("get roots", status, &buffer, || ())?;
            status = (library.symbols.mb_motion_get_local_rotations_xyzw)(
                raw,
                &mut rotations_pointer,
                &mut rotations_count,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            );
            library.check("get rotations", status, &buffer, || ())?;
            status = (library.symbols.mb_motion_get_target_frame_count)(
                raw,
                &mut target_frames,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            );
            library.check("get target frame count", status, &buffer, || ())?;
            status = (library.symbols.mb_motion_get_target_root_translations)(
                raw,
                &mut target_roots_pointer,
                &mut target_roots_count,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            );
            library.check("get target roots", status, &buffer, || ())?;
            status = (library.symbols.mb_motion_get_target_local_rotations_xyzw)(
                raw,
                &mut target_rotations_pointer,
                &mut target_rotations_count,
                write_error_buffer(&mut buffer),
                sys::MB_ERROR_CAPACITY,
            );
            library.check("get target rotations", status, &buffer, || ())?;

            if roots_count != frames * 3
                || rotations_count != frames * joints * 4
                || (target_frames > 0
                    && (target_roots_count != target_frames * 3
                        || target_rotations_count != target_frames * joints * 4))
            {
                return Err(simple_error("read motion", "native motion data has inconsistent dimensions"));
            }

            let mut motion = Motion {
                frames,
                joints,
                roots: copy_slice(roots_pointer, roots_count),
                rotations: copy_slice(rotations_pointer, rotations_count),
                targets: None,
            };
            if target_frames > 0 {
                motion.targets = Some(Keyframes {
                    frames: target_frames,
                    joints,
                    roots: copy_slice(target_roots_pointer, target_roots_count),
                    rotations: copy_slice(target_rotations_pointer, target_rotations_count),
                });
            }
            Ok(motion)
        }
    }

    /// Root translation of `frame` as `[x, y, z]`.
    pub fn root_at(&self, frame: usize) -> Option<[f32; 3]> {
        let base = frame.checked_mul(3)?;
        Some([
            *self.roots.get(base)?,
            *self.roots.get(base + 1)?,
            *self.roots.get(base + 2)?,
        ])
    }

    /// Local rotation of `joint` in `frame` as `[x, y, z, w]`.
    pub fn rotation_at(&self, frame: usize, joint: usize) -> Option<[f32; 4]> {
        let base = frame.checked_mul(self.joints as usize)?
            .checked_add(joint)?
            .checked_mul(4)?;
        Some([
            *self.rotations.get(base)?,
            *self.rotations.get(base + 1)?,
            *self.rotations.get(base + 2)?,
            *self.rotations.get(base + 3)?,
        ])
    }

    /// All roots as a slice of `[x, y, z]` rows (zero-copy view).
    pub fn root_rows(&self) -> &[[f32; 3]] {
        flat_rows(&self.roots)
    }

    /// All rotations as a slice of `[x, y, z, w]` rows (zero-copy view).
    pub fn rotation_rows(&self) -> &[[f32; 4]] {
        flat_rows(&self.rotations)
    }
}

/// The four placed style-pose constraints for one plan (targets of
/// [`Motion`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframes {
    /// Number of target frames (typically 4).
    pub frames: u64,
    /// Joints per frame.
    pub joints: u64,
    /// Row-major `[frames][3]`.
    pub roots: Vec<f32>,
    /// Row-major `[frames][joints][4]` (XYZW).
    pub rotations: Vec<f32>,
}

// A small zero-copy helper replacing a `bytemuck` dependency: the row types
// are `[f32; N]`, which are valid reinterpretations of the flat buffer.
fn flat_rows<const WIDTH: usize>(values: &[f32]) -> &[[f32; WIDTH]] {
    if !values.len().is_multiple_of(WIDTH) {
        return &[];
    }
    let pointer = values.as_ptr();
    let length = values.len() / WIDTH;
    unsafe { std::slice::from_raw_parts(pointer.cast::<[f32; WIDTH]>(), length) }
}

unsafe fn copy_slice(pointer: *const f32, count: u64) -> Vec<f32> {
    if pointer.is_null() || count == 0 {
        return Vec::new();
    }
    std::slice::from_raw_parts(pointer, count as usize).to_vec()
}
