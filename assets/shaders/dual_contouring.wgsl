// ---------------------------------------------------------
// 1. MEMORY BINDINGS
// ---------------------------------------------------------
@group(0) @binding(0) var<storage, read> sdf_data: array<f32>;
@group(0) @binding(1) var<storage, read_write> vertices: array<f32>; 
@group(0) @binding(2) var<storage, read_write> indices: array<u32>;
@group(0) @binding(3) var<storage, read_write> index_counter: atomic<u32>;

const GRID_SIZE: u32 = 64u;

// ---------------------------------------------------------
// 2. HELPER FUNCTIONS
// ---------------------------------------------------------
fn get_voxel_index(x: u32, y: u32, z: u32) -> u32 {
    return (z * GRID_SIZE * GRID_SIZE) + (y * GRID_SIZE) + x;
}

fn get_distance(x: u32, y: u32, z: u32) -> f32 {
    if (x >= GRID_SIZE || y >= GRID_SIZE || z >= GRID_SIZE) {
        return 1.0;
    }
    return sdf_data[get_voxel_index(x, y, z)];
}

// Safe central difference normal calculation
fn get_normal(x: f32, y: f32, z: f32) -> vec3<f32> {
    let ix = u32(clamp(round(x), 1.0, f32(GRID_SIZE - 2u)));
    let iy = u32(clamp(round(y), 1.0, f32(GRID_SIZE - 2u)));
    let iz = u32(clamp(round(z), 1.0, f32(GRID_SIZE - 2u)));
    
    let nx = get_distance(ix + 1u, iy, iz) - get_distance(ix - 1u, iy, iz);
    let ny = get_distance(ix, iy + 1u, iz) - get_distance(ix, iy - 1u, iz);
    let nz = get_distance(ix, iy, iz + 1u) - get_distance(ix, iy, iz - 1u);
    
    let v = vec3<f32>(nx, ny, nz);
    let len = length(v);
    if (len < 1e-5) {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    return v / len;
}

fn invert_mat3(m: mat3x3<f32>) -> mat3x3<f32> {
    let det = dot(m[0], cross(m[1], m[2]));
    
    if (abs(det) < 1e-4) {
        return mat3x3<f32>(
            vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0)
        );
    }
    
    let inv_det = 1.0 / det;
    return mat3x3<f32>(
        cross(m[1], m[2]) * inv_det,
        cross(m[2], m[0]) * inv_det,
        cross(m[0], m[1]) * inv_det
    );
}

// ---------------------------------------------------------
// 3. PASS ONE: SOLVE VERTICES
// ---------------------------------------------------------
@compute @workgroup_size(8, 8, 8)
fn solve_qef(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    let z = global_id.z;

    if (x >= GRID_SIZE - 1u || y >= GRID_SIZE - 1u || z >= GRID_SIZE - 1u) {
        return;
    }

    let corners = array<vec3<f32>, 8>(
        vec3<f32>(f32(x), f32(y), f32(z)),
        vec3<f32>(f32(x + 1u), f32(y), f32(z)),
        vec3<f32>(f32(x + 1u), f32(y + 1u), f32(z)),
        vec3<f32>(f32(x), f32(y + 1u), f32(z)),
        vec3<f32>(f32(x), f32(y), f32(z + 1u)),
        vec3<f32>(f32(x + 1u), f32(y), f32(z + 1u)),
        vec3<f32>(f32(x + 1u), f32(y + 1u), f32(z + 1u)),
        vec3<f32>(f32(x), f32(y + 1u), f32(z + 1u))
    );

    let edge_pairs = array<vec2<u32>, 12>(
        vec2<u32>(0u, 1u), vec2<u32>(1u, 2u), vec2<u32>(2u, 3u), vec2<u32>(3u, 0u),
        vec2<u32>(4u, 5u), vec2<u32>(5u, 6u), vec2<u32>(6u, 7u), vec2<u32>(7u, 4u),
        vec2<u32>(0u, 4u), vec2<u32>(1u, 5u), vec2<u32>(2u, 6u), vec2<u32>(3u, 7u)
    );

    var ata = mat3x3<f32>(vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0));
    var atb = vec3<f32>(0.0, 0.0, 0.0);
    var mass_point = vec3<f32>(0.0, 0.0, 0.0);
    var intersection_count = 0.0;

    for (var i = 0u; i < 12u; i = i + 1u) {
        let p1 = corners[edge_pairs[i].x];
        let p2 = corners[edge_pairs[i].y];
        
        let d1 = get_distance(u32(p1.x), u32(p1.y), u32(p1.z));
        let d2 = get_distance(u32(p2.x), u32(p2.y), u32(p2.z));

        if ((d1 <= 0.0) != (d2 <= 0.0)) {
            let t = clamp(d1 / (d1 - d2), 0.0, 1.0);
            let p = p1 + t * (p2 - p1);
            
            let normal = get_normal(p.x, p.y, p.z);
            let outer_product = mat3x3<f32>(
                normal * normal.x,
                normal * normal.y,
                normal * normal.z
            );
            
            ata = ata + outer_product;
            atb = atb + (normal * dot(normal, p));
            mass_point = mass_point + p;
            intersection_count = intersection_count + 1.0;
        }
    }

    let voxel_idx = get_voxel_index(x, y, z);
    let v_idx = voxel_idx * 3u; 
    
    if (intersection_count > 0.0) {
        let mass_center = mass_point / intersection_count;
        let pseudo_inv = invert_mat3(ata);
        
        let correction = pseudo_inv * (atb - (ata * mass_center));
        var optimal_vertex = mass_center + correction;
        
        let cell_min = vec3<f32>(f32(x), f32(y), f32(z));
        let cell_max = vec3<f32>(f32(x) + 1.0, f32(y) + 1.0, f32(z) + 1.0);
        
        // Safety check: Fall back to mass center if SVD diverges or produces NaNs
        if (any(optimal_vertex < cell_min) || any(optimal_vertex > cell_max) || dot(correction, correction) > 2.0) {
            optimal_vertex = mass_center;
        }

        let mesh_center = f32(GRID_SIZE - 1u) * 0.5;
        let voxel_size = 16.0 / f32(GRID_SIZE - 1u);
        
        optimal_vertex = (optimal_vertex - vec3<f32>(mesh_center)) * voxel_size;

        vertices[v_idx] = optimal_vertex.x;
        vertices[v_idx + 1u] = optimal_vertex.y;
        vertices[v_idx + 2u] = optimal_vertex.z;
    } else {
        vertices[v_idx] = 0.0;
        vertices[v_idx + 1u] = 0.0;
        vertices[v_idx + 2u] = 0.0;
    }
}

// ---------------------------------------------------------
// 4. PASS TWO: GENERATE QUADS (INDICES)
// ---------------------------------------------------------
@compute @workgroup_size(8, 8, 8)
fn generate_quads(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    let z = global_id.z;

    if (x < 1u || y < 1u || z < 1u || x >= GRID_SIZE - 1u || y >= GRID_SIZE - 1u || z >= GRID_SIZE - 1u) {
        return;
    }

    let d0 = get_distance(x, y, z);

    // X-AXIS EDGE
    let d_x = get_distance(x + 1u, y, z);
    if ((d0 <= 0.0) != (d_x <= 0.0)) {
        let v1 = get_voxel_index(x, y, z);
        let v2 = get_voxel_index(x, y - 1u, z);
        let v3 = get_voxel_index(x, y - 1u, z - 1u);
        let v4 = get_voxel_index(x, y, z - 1u);

        let idx = atomicAdd(&index_counter, 6u);
        
        if (d0 <= 0.0) {
            indices[idx] = v1; indices[idx+1] = v2; indices[idx+2] = v3;
            indices[idx+3] = v1; indices[idx+4] = v3; indices[idx+5] = v4;
        } else {
            indices[idx] = v1; indices[idx+1] = v4; indices[idx+2] = v3;
            indices[idx+3] = v1; indices[idx+4] = v3; indices[idx+5] = v2;
        }
    }

    // Y-AXIS EDGE
    let d_y = get_distance(x, y + 1u, z);
    if ((d0 <= 0.0) != (d_y <= 0.0)) {
        let v1 = get_voxel_index(x, y, z);
        let v2 = get_voxel_index(x, y, z - 1u);
        let v3 = get_voxel_index(x - 1u, y, z - 1u);
        let v4 = get_voxel_index(x - 1u, y, z);
        
        let idx = atomicAdd(&index_counter, 6u);
        
        if (d0 <= 0.0) {
            indices[idx] = v1; indices[idx+1] = v2; indices[idx+2] = v3;
            indices[idx+3] = v1; indices[idx+4] = v3; indices[idx+5] = v4;
        } else {
            indices[idx] = v1; indices[idx+1] = v4; indices[idx+2] = v3;
            indices[idx+3] = v1; indices[idx+4] = v3; indices[idx+5] = v2;
        }
    }

    // Z-AXIS EDGE
    let d_z = get_distance(x, y, z + 1u);
    if ((d0 <= 0.0) != (d_z <= 0.0)) {
        let v1 = get_voxel_index(x, y, z);
        let v2 = get_voxel_index(x - 1u, y, z);
        let v3 = get_voxel_index(x - 1u, y - 1u, z);
        let v4 = get_voxel_index(x, y - 1u, z);

        let idx = atomicAdd(&index_counter, 6u);
        
        if (d0 <= 0.0) {
            indices[idx] = v1; indices[idx+1] = v2; indices[idx+2] = v3;
            indices[idx+3] = v1; indices[idx+4] = v3; indices[idx+5] = v4;
        } else {
            indices[idx] = v1; indices[idx+1] = v4; indices[idx+2] = v3;
            indices[idx+3] = v1; indices[idx+4] = v3; indices[idx+5] = v2;
        }
    }
}