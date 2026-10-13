# motionbricks-mlx

Safe Rust bindings for the **motionbricks C ABI (version 1)** — the
Apple/MLX flavour (default device `Auto`). Load a model bundle, load
styles, plan motions for one agent or a native batch of up to 64.

The dynamic library (`libmotionbricks.dylib`) is **loaded at runtime**
(`dlopen` via `libloading`) — no build-time linking, no `build.rs`, no C
struct layouts. A mismatched or older library fails at `Library::open`,
never mid-plan.

For Vulkan hosts use the sibling crate
[`motionbricks-vulkan`](https://crates.io/crates/motionbricks-vulkan);
both bind the same ABI and expose the identical safe API.

## Quick start

```rust
use motionbricks_mlx::{Command, Device, Library};

let library = Library::open("libmotionbricks.dylib")?;
let model = library.load_model("g1-f32", Device::Auto)?;
let style = model.load_style("styles/walk.mbstyle")?;

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

## Guarantees

- Native handles are reference-counted, freed on drop; clones keep the
  underlying handle alive, so pointers can never dangle.
- `Agent::plan` / `Model::plan_batch` return **owned** `Motion` data
  (roots `[frames][3]`, rotations `[frames][joints][4]` XYZW) copied out
  of the library's borrowed buffers.
- Handles are `!Send`/`!Sync`, mirroring the C serialization contract;
  use `plan_batch` for native parallel planning.

Coverage is complete for `motionbricks.h`; `sonic.h`, `physics.h` and
`inference.h` are not bound yet.

See the [repository](https://github.com/neeboo/motionbricks-rs) for the
full guide and testing instructions.

Licensed under Apache-2.0.
