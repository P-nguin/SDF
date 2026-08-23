Here is the architectural roadmap for building the smart ASCII rendering pipeline. This gives us a clear checklist before we start wiring up the rendering graph.

### Tactical ASCII Pipeline: Implementation Plan

#### Phase 1: Camera & Depth Pass Preparation (Our Next Step)

To do edge detection on the GPU, the shader needs to know how far away the meshes are. Bevy doesn't pass depth to post-processing by default.

- **Update `camera/plugin.rs**`: Add the `DepthPrepass` component to the 3D camera.
- **Result**: Bevy will automatically generate a high-precision depth texture every frame before our custom shader runs.

#### Phase 2: The Rust Render Graph (`ascii/` module)

We need to build the bridge between the CPU game world and the GPU shader.

- **`ascii/pipeline.rs`**: Implement the `ViewNode` trait. This code will grab three things and bundle them for the GPU:

1. The Main Color Texture (our red/green semantic masks).
2. The Depth Texture (from Phase 1).
3. The Font Atlas Texture (our PNG of ASCII characters).

- **`ascii/plugin.rs`**: Insert this node into Bevy's `Core3d` render graph right after the main pass but before the UI renders.

#### Phase 3: WGSL Foundation & Semantics (The "Cogmind" Layer)

We write the first half of `ascii_post_process.wgsl` to handle the grid and semantic colors.

- **Grid Math**: Divide the screen into fixed 8x8 or 8x16 pixel blocks.
- **Anti-Shimmering**: Sample 4 sub-pixels within the block and average them to stabilize characters during rotation.
- **Semantic Check**: If the average color is pure Green or Blue, bypass the depth math and immediately output a specific component character (e.g., `W` or `≈`).

#### Phase 4: Depth & Edge Detection (The "Obra Dinn" Layer)

We write the structural logic in the WGSL shader for the red hull blocks.

- **Sobel Filter**: Sample the depth buffer at the North, South, East, and West edges of the current block.
- **Gradient Calculation**: If an edge is found, use `atan2(Y, X)` to calculate the exact mathematical angle of the geometry underneath.
- **Character Mapping**: Map that angle to `/`, `\`, `|`, or `-`.
- **Negative Space**: If no edge is found (flat hull), output a faint `.` or empty space ` `.

#### Phase 5: Polish & Vibe (The "Caves of Qud" Layer)

Apply global post-processing over the assembled ASCII grid.

- **Scanlines**: Darken alternating pixel rows.
- **CRT Warp**: Apply a slight barrel distortion to the UV coordinates to simulate curved monitor glass.
- **Phosphor Bloom**: Add a slight localized blur to the brightest characters (weapons/engines) so they glow against the dark terminal background.

---
