# motionbricks-rs

Safe Rust bindings for the **motionbricks C ABI (version 1)** — Rust systems
integrate motion-bricks motion generation independently: load a model bundle,
load styles, plan motions for one agent or a native batch of up to 64.

The dynamic library is **loaded at runtime** (`dlopen`/`LoadLibrary` via
`libloading`), exactly like the reference PureGo binding — no build-time
linking, no `build.rs`, no C struct layouts. A mismatched or older library
fails at `Library::open`, never mid-plan.

## Crates

| Crate | Flavour | Default device | For |
| --- | --- | --- | --- |
| [`motionbricks-mlx`](crates/motionbricks-mlx) | Apple/MLX | `Auto` (library's own choice) | macOS / Apple silicon hosts |
| [`motionbricks-vulkan`](crates/motionbricks-vulkan) | Vulkan | `Vulkan` | Vulkan-capable hosts; backend-directory configuration up front |

Both crates are **self-contained** (raw FFI lives in each crate's `pub mod
ffi`), bind the **same C ABI**, and expose the identical safe API — pick one
per target platform. Equal inputs and seeds produce identical motions through
either.

Each crate covers `motionbricks.h` completely: runtime options (device,
threads, backend directory), commands (style, movement/facing, speed, world
target, seed, argmax sampling), model + skeleton, styles, agent
(reset/advance/set_context/plan), `plan_batch`, and motion + target
keyframes. The SONIC, physics and stateless-inference headers (`sonic.h`,
`physics.h`, `inference.h`) are not bound yet.

## Quick start

```rust
use motionbricks_mlx::{Command, Device, Library}; // or motionbricks_vulkan

let library = Library::open("libmotionbricks.dylib")?; // .so elsewhere
let model = library.load_model("g1-f32", Device::Auto)?;
let style = model.load_style("generated/styles/walk.mbstyle")?;

let agent = model.agent()?;
agent.reset(&style)?;

let mut command = Command::new(&library)?;
command.set_style(&style)?;
command.set_movement_direction(1.0, 0.0, 0.0)?;
command.set_facing_direction(0.0, 0.0, 1.0)?;
command.set_seed(1)?;

let motion = agent.plan(&command)?;
println!("{} frames x {} joints", motion.frames, motion.joints);
```

Guarantees:

- Native handles (`Model`, `Style`, `Command`, `Agent`, `RuntimeOptions`)
  are reference-counted and freed on drop; clones keep the underlying handle
  alive, so pointers passed to the library can never dangle.
- `Agent::plan` and `Model::plan_batch` return **owned** `Motion` data
  (roots `[frames][3]`, rotations `[frames][joints][4]` XYZW) copied out of
  the library's borrowed buffers; the native motion is freed before return.
- Handles are `!Send`/`!Sync`, mirroring the C contract that calls sharing
  a model/agent/style/command are serialized. Use `plan_batch` for native
  parallel planning of independent robots.

## Building and testing

Prerequisite: a built `libmotionbricks` dynamic library and a model bundle
(default discovery: sibling `../motion-bricks-cpp/repo` checkout):

```sh
cargo build
cargo test        # 8 integration tests per crate against the real dylib
```

Tests are skipped with a message when artifacts are absent. Override
discovery with:

| Variable               | Meaning              | Default                                                       |
| ---------------------- | -------------------- | ------------------------------------------------------------- |
| `MOTIONBRICKS_LIBRARY` | dynamic library path | `../motion-bricks-cpp/repo/build/bench/libmotionbricks.dylib` |
| `MOTIONBRICKS_BUNDLE`  | model bundle         | `../motion-bricks-cpp/repo/generated/g1-f32`                  |
| `MOTIONBRICKS_STYLE`   | `.mbstyle` file      | `../motion-bricks-cpp/repo/generated/styles/walk.mbstyle`     |

Tests pin `Device::Cpu` for determinism on development machines (the Vulkan
crate's default device stays `Vulkan` for deployment); `Auto`/`Vulkan` are
runtime choices of the same library.

## Downstream integration

Both crates are published on crates.io:

```toml
[dependencies]
motionbricks-mlx = "0.1"       # Apple/MLX hosts
motionbricks-vulkan = "0.1"    # Vulkan hosts
```

Or pin the repo directly:

```toml
[dependencies]
motionbricks-mlx = { git = "https://github.com/neeboo/motionbricks-rs" }        # Apple hosts
motionbricks-vulkan = { git = "https://github.com/neeboo/motionbricks-rs" }     # Vulkan hosts
```

## License

Apache-2.0.
