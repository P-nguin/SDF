use bevy::prelude::*;

#[derive(Component)]
pub struct VoxelGrid {
    pub height: usize,
    pub width: usize,
    // array of voxelchunks
    pub grid: Vec<Option<Entity>>
}
impl VoxelGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![None; width * height],
        }
    }
}
impl VoxelGrid {
    /// Iterates over a generic 2D blueprint, calculates local coordinates, 
    /// and uses a provided closure to spawn the specific entities.
    pub fn create_voxel_grid<T, F>(blueprint: &[Vec<T>], mut spawn_chunk: F) -> Self
    where
        // The closure takes (grid_x, grid_y, cell_data, local_pos_x, local_pos_y) 
        // and returns an Option<Entity>
        F: FnMut(usize, usize, &T, f32, f32) -> Option<Entity>,
    {
        let height = blueprint.len();
        let width = if height > 0 { blueprint[0].len() } else { 0 };
        let mut grid = vec![None; width * height];
        
        let chunk_size = 16.0;
        let half_width = (width as f32 - 1.0) / 2.0;
        let half_height = (height as f32 - 1.0) / 2.0;

        for (y, row) in blueprint.iter().enumerate() {
            for (x, cell_data) in row.iter().enumerate() {
                // Calculate the visual transform offset automatically
                let local_x = (x as f32 - half_width) * chunk_size;
                let local_y = -(y as f32 - half_height) * chunk_size;

                // Let the specific module (Ship/Shield) handle the actual spawning
                if let Some(entity) = spawn_chunk(x, y, cell_data, local_x, local_y) {
                    let index = y * width + x;
                    grid[index] = Some(entity);
                }
            }
        }
        
        Self { width, height, grid }
    }
}