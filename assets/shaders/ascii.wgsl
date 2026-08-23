#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var depth_texture: texture_depth_2d;
@group(0) @binding(2) var texture_sampler: sampler;
@group(0) @binding(3) var font_texture: texture_2d<f32>; // <-- NEW: Font input

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let tex_dim = vec2<f32>(textureDimensions(screen_texture));
    let grid_size = vec2<f32>(40.0, 40.0); 
    
    // 1. Calculate Grid and Local UV
    let block_uv = floor(in.uv * tex_dim / grid_size) * grid_size / tex_dim;
    let center_uv = block_uv + (grid_size / 2.0) / tex_dim;
    let center_coord = vec2<i32>(center_uv * tex_dim);
    
    // Calculate where exactly we are INSIDE the 8x8 block (0.0 to 1.0)
    let local_uv = fract(in.uv * tex_dim / grid_size);

    // 2. Depth Check (Anti-Smearing)
    let depth_center = textureLoad(depth_texture, center_coord, 0);
    if depth_center == 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Default character data
    var char_index = 0.0;
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    
    // 3. SAMPLE SEMANTIC COLOR (Cogmind layer)
    let data_color = textureSample(screen_texture, texture_sampler, center_uv);
    
    if data_color.g > 0.5 && data_color.r < 0.5 {
        char_index = 87.0; // ASCII 'W' for Weapon
        final_color = vec4<f32>(0.0, 1.0, 0.0, 1.0); // Neon Green
    } else {
        // 4. STRUCTURAL EDGE DETECTION (Obra Dinn layer)
        let offset_x = vec2<i32>(i32(grid_size.x), 0);
        let offset_y = vec2<i32>(0, i32(grid_size.y));

        let depth_n = textureLoad(depth_texture, center_coord - offset_y, 0);
        let depth_s = textureLoad(depth_texture, center_coord + offset_y, 0);
        let depth_e = textureLoad(depth_texture, center_coord + offset_x, 0);
        let depth_w = textureLoad(depth_texture, center_coord - offset_x, 0);
        
        let edge_x = abs(depth_e - depth_w);
        let edge_y = abs(depth_s - depth_n);
        let edge_magnitude = edge_x + edge_y;

        if edge_magnitude > 0.001 {
            // Determine line direction based on the steepest gradient
            if edge_y > edge_x {
                char_index = 45.0; // ASCII '-' for Horizontal Edge
            } else {
                char_index = 124.0; // ASCII '|' for Vertical Edge
            }
            final_color = vec4<f32>(1.0, 1.0, 1.0, 1.0); // Bright White
        } else {
            char_index = 35.0; // ASCII '#' for Flat Hull
            final_color = vec4<f32>(0.2, 0.0, 0.0, 1.0); // Dark Red
        }
    }

    // 5. SAMPLE THE FONT ATLAS
    // Calculate the row and column of the character in a 12x12 grid
    let font_col = char_index % 12.0;
    let font_row = floor(char_index / 12.0);
    
    // Offset the local UV to the correct character in the atlas
    let font_uv = (vec2<f32>(font_col, font_row) + local_uv) / 12.0;
    
    // Sample the font image
    let font_sample = textureSample(font_texture, texture_sampler, font_uv);
    
    // Multiply our chosen color by the font's brightness to draw the letter
    // (Assuming your font image has white text on a black/transparent background)
    return final_color * font_sample.r;
}
