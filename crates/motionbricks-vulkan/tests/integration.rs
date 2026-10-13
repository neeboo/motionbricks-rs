//! Integration tests against a real `libmotionbricks` dynamic library and
//! model bundle. Skipped (with a message) when the artifacts are absent.
//!
//! Artifact discovery: environment overrides first, then the sibling
//! motion-bricks.cpp build layout used during development:
//!
//! - `MOTIONBRICKS_LIBRARY`  e.g. `.../build/bench/libmotionbricks.dylib`
//! - `MOTIONBRICKS_BUNDLE`   e.g. `.../generated/g1-f32`
//! - `MOTIONBRICKS_STYLE`    e.g. `.../generated/styles/walk.mbstyle`

use std::path::{Path, PathBuf};
use std::sync::Arc;

use motionbricks_vulkan::{Command, Device, Library, Model, Motion, RuntimeOptions};

struct Artifacts {
    library: PathBuf,
    bundle: PathBuf,
    style: PathBuf,
}

fn default_repo() -> Option<PathBuf> {
    // tests/ -> repo root -> sibling motion-bricks-cpp checkout
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Some(root.parent()?.join("motion-bricks-cpp").join("repo"))
}

fn artifacts() -> Option<Artifacts> {
    let repo = default_repo();
    let library = std::env::var_os("MOTIONBRICKS_LIBRARY")
        .map(PathBuf::from)
        .or_else(|| {
            let repo = repo.as_ref()?;
            let dylib = repo.join("build/bench/libmotionbricks.dylib");
            if dylib.exists() {
                Some(dylib)
            } else {
                Some(repo.join("build/bench/libmotionbricks.so"))
            }
        })?;
    let bundle = std::env::var_os("MOTIONBRICKS_BUNDLE")
        .map(PathBuf::from)
        .or_else(|| repo.as_ref().map(|r| r.join("generated/g1-f32")))?;
    let style = std::env::var_os("MOTIONBRICKS_STYLE")
        .map(PathBuf::from)
        .or_else(|| repo.as_ref().map(|r| r.join("generated/styles/walk.mbstyle")))?;
    if !(library.exists() && bundle.exists() && style.exists()) {
        return None;
    }
    Some(Artifacts {
        library,
        bundle,
        style,
    })
}

fn open_library() -> Option<Arc<Library>> {
    let artifacts = match artifacts() {
        Some(value) => value,
        None => {
            eprintln!(
                "skipping: no libmotionbricks library/bundle/style found \
                 (set MOTIONBRICKS_LIBRARY, MOTIONBRICKS_BUNDLE, MOTIONBRICKS_STYLE)"
            );
            return None;
        }
    };
    let library = Library::open(&artifacts.library).expect("open library");
    Some(library)
}

fn load_model(library: &Arc<Library>) -> Model {
    let artifacts = artifacts().expect("artifacts");
    // Pin the CPU device: deterministic and always available in dev builds.
    library
        .load_model(&artifacts.bundle, Device::Cpu)
        .expect("load model")
}

#[test]
fn opens_and_reports_abi_version() {
    let Some(library) = open_library() else { return };
    assert_eq!(library.abi_version(), 1);
    assert_eq!(
        library.status_string(motionbricks_vulkan::Status::OK),
        "ok"
    );
}

#[test]
fn loads_model_and_skeleton() {
    let Some(library) = open_library() else { return };
    let model = load_model(&library);
    let parameter_count = model.parameter_count().expect("parameter count");
    assert!(parameter_count > 0, "parameter count must be positive");

    let joint_count = model.joint_count().expect("joint count");
    assert_eq!(joint_count, 34, "G1 skeleton has 34 joints");

    let skeleton = model.skeleton().expect("skeleton");
    assert_eq!(skeleton.len(), 34);
    assert_eq!(skeleton[0].parent, -1, "root joint has no parent");
    assert!(skeleton.iter().all(|joint| !joint.name.is_empty()));
    assert!(skeleton
        .iter()
        .all(|joint| joint.position.iter().all(|value| value.is_finite())));
}

#[test]
fn runtime_options_roundtrip() {
    let Some(library) = open_library() else { return };
    let options = RuntimeOptions::new(Arc::clone(&library)).expect("options");
    options.set_device(Device::Cpu).expect("set device");
    assert_eq!(options.device().expect("get device"), Device::Cpu);
    options.set_threads(4).expect("set threads");
    assert_eq!(options.threads().expect("get threads"), 4);
    let directory = std::env::temp_dir();
    options
        .set_backend_directory(Some(&directory))
        .expect("set backend directory");
    assert_eq!(
        options.backend_directory().expect("get backend directory"),
        Some(directory.to_string_lossy().into_owned())
    );
    options
        .set_backend_directory(None)
        .expect("clear backend directory");
    assert_eq!(options.backend_directory().expect("get backend directory"), None);
}

fn planned_motion(model: &Model, library: &Arc<Library>, style_path: &Path, seed: u64) -> Motion {
    let style = model.load_style(style_path).expect("load style");
    assert!(!style.name.is_empty());
    assert!(style.speed >= 0.0);

    let agent = model.agent().expect("create agent");
    agent.reset(&style).expect("reset agent");

    let mut command = Command::new(library).expect("create command");
    command.set_style(&style).expect("set style");
    command.set_movement_direction(1.0, 0.0, 0.0).expect("set movement");
    command.set_facing_direction(0.0, 0.0, 1.0).expect("set facing");
    command.set_target_speed(-1.0).expect("set speed"); // style speed
    command.set_sampling_argmax(false).expect("set sampling");
    command.set_seed(seed).expect("set seed");
    assert_eq!(command.seed().expect("get seed"), seed);
    assert_eq!(command.style().map(|style| style.name.as_str()), Some(style.name.as_str()));

    agent.plan(&command).expect("plan")
}

fn assert_motion_shape(motion: &Motion) {
    assert_eq!(motion.joints, 34);
    assert_eq!(motion.frames, 44, "expected one 44-frame segment");
    assert_eq!(motion.roots.len(), (motion.frames * 3) as usize);
    assert_eq!(motion.rotations.len(), (motion.frames * motion.joints * 4) as usize);
    assert!(motion.roots.iter().all(|value| value.is_finite()));
    assert!(motion.rotations.iter().all(|value| value.is_finite()));
    // Quaternions stay near unit length.
    for frame in 0..motion.frames as usize {
        for joint in 0..motion.joints as usize {
            let [x, y, z, w] = motion.rotation_at(frame, joint).expect("rotation");
            let norm = (x * x + y * y + z * z + w * w).sqrt();
            assert!(
                (norm - 1.0).abs() < 0.05,
                "rotation norm {norm} at frame {frame} joint {joint}"
            );
        }
    }
    let targets = motion.targets.as_ref().expect("targets");
    assert_eq!(targets.frames, 4, "four placed style-pose constraints");
    assert_eq!(targets.roots.len(), (targets.frames * 3) as usize);
    assert_eq!(targets.rotations.len(), (targets.frames * targets.joints * 4) as usize);
}

#[test]
fn plans_with_seed_reproducibly() {
    let Some(library) = open_library() else { return };
    let model = load_model(&library);
    let artifacts = artifacts().expect("artifacts");

    let first = planned_motion(&model, &library, &artifacts.style, 7);
    assert_motion_shape(&first);
    // Rotation accessors agree with the flat rows.
    assert_eq!(first.rotation_at(0, 0), Some(first.rotation_rows()[0]));
    assert_eq!(first.root_at(3), Some(first.root_rows()[3]));

    let second = planned_motion(&model, &library, &artifacts.style, 7);
    assert_eq!(first, second, "same seed must reproduce the same motion");

    let third = planned_motion(&model, &library, &artifacts.style, 8);
    assert_ne!(first, third, "different seed should change sampled motion");
}

#[test]
fn plans_batch_of_two() {
    let Some(library) = open_library() else { return };
    let model = load_model(&library);
    let artifacts = artifacts().expect("artifacts");
    let style = model.load_style(&artifacts.style).expect("load style");

    let agents: Vec<_> = (0..2).map(|_| model.agent().expect("agent")).collect();
    for agent in &agents {
        agent.reset(&style).expect("reset");
    }
    let commands: Vec<_> = (0..2)
        .map(|index| {
            let mut command = Command::new(&library).expect("command");
            command.set_style(&style).expect("set style");
            command.set_movement_direction(1.0, 0.0, 0.0).expect("set movement");
            command.set_facing_direction(0.0, 0.0, 1.0).expect("set facing");
            command.set_seed(100 + index).expect("set seed");
            command
        })
        .collect();

    let command_refs: Vec<&Command> = commands.iter().collect();
    let agent_refs: Vec<&motionbricks_vulkan::Agent> = agents.iter().collect();
    let motions = model
        .plan_batch(&agent_refs, &command_refs)
        .expect("plan batch");
    assert_eq!(motions.len(), 2);
    for motion in &motions {
        assert_motion_shape(motion);
    }

    // A repeated batch with equal inputs is reproducible.
    let again = model
        .plan_batch(&agent_refs, &command_refs)
        .expect("plan batch again");
    // Agents advanced between batches, so only shape parity is asserted here;
    // per-seed reproducibility is covered by plans_with_seed_reproducibly.
    assert_eq!(motions.len(), again.len());
    assert!(motions
        .iter()
        .zip(again.iter())
        .all(|(a, b)| a.frames == b.frames && a.joints == b.joints));
}

#[test]
fn batch_validates_mixed_handles() {
    let Some(library) = open_library() else { return };
    let model = load_model(&library);
    let artifacts = artifacts().expect("artifacts");
    let style = model.load_style(&artifacts.style).expect("load style");

    let agent = model.agent().expect("agent");
    agent.reset(&style).expect("reset");
    let command = Command::new(&library).expect("command");

    let second = Model::load(
        &library,
        &artifacts.bundle,
        None,
    )
    .expect("second model instance");
    let foreign_agent = second.agent().expect("agent on second model");

    // Foreign agent must be rejected.
    assert!(model
        .plan_batch(&[&foreign_agent], &[&command])
        .is_err());
    // Length mismatch must be rejected.
    assert!(model.plan_batch(&[&agent], &[]).is_err());
    // Empty batch must be rejected.
    assert!(model.plan_batch(&[&agent][..0], &command_refs_empty(&command)).is_err());
}

fn command_refs_empty(_command: &Command) -> Vec<&'static Command> {
    Vec::new()
}

#[test]
fn style_speed_roundtrip() {
    let Some(library) = open_library() else { return };
    let model = load_model(&library);
    let artifacts = artifacts().expect("artifacts");
    let style = model.load_style(&artifacts.style).expect("load style");
    let original = style.get_speed().expect("get speed");
    style.set_speed(original + 0.25).expect("set speed");
    assert!((style.get_speed().expect("get speed") - (original + 0.25)).abs() < 1e-6);
    style.set_speed(original).expect("restore speed");
    assert!((style.get_speed().expect("restored") - original).abs() < 1e-6);
}

#[test]
fn agent_set_context_validates_shape() {
    let Some(library) = open_library() else { return };
    let model = load_model(&library);
    let agent = model.agent().expect("agent");
    // Wrong shape must be rejected locally without touching the library.
    assert!(agent.set_context(&[0.0; 3], &[0.0; 4], 1, 34).is_err());
}
