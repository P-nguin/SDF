# SDF 3D

A minimal Bevy 3D starter project.

## Run

```sh
cargo run
```

The starter scene opens a window with a camera, point light, ground plane, and two rotating 3D meshes.

## Adding Source Files

Rust source files must be linked from the module tree before rust-analyzer can provide full IDE services.

For a file like `src/transform.rs`, add this near the top of `src/main.rs`:

```rust
mod transform;
```

For nested modules, either add `mod child;` inside the parent module file or create a `mod.rs` for that folder.

## Debug

VS Code is configured for Rust debugging through CodeLLDB.

1. Install the recommended extensions when VS Code prompts you.
2. Run `Developer: Reload Window` so CodeLLDB picks up the workspace `PATH`.
3. Open the Run and Debug view.
4. Select `Debug Bevy app`.
5. Press `F5`.

The debug launch sets `RUST_BACKTRACE=1` and builds the `sdf_3d` binary before starting it.

If that launch still fails, try `Debug Bevy app (prebuilt)` after running `cargo build` once. That profile skips CodeLLDB's Cargo integration and launches `target/debug/sdf_3d` directly.
