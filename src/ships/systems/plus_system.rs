use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;
use crate::ships::components::*;

pub fn plus_effect_system(
    grids: Query<&ShipGrid>,
    // 1. Add `Entity` to this query so we can look up the source cell
    plus_cells_pos: Query<(Entity, &CellPos, &ChildOf), With<PlusCell>>,
    mut cells: Query<&mut Cell>,
) {
    for (entity, pos, parent) in plus_cells_pos.iter() {
        
        if let Ok(grid) = grids.get(parent.get()) {
            let x = pos.x as i32;
            let y = pos.y as i32;
            
            // 2. CLEAN UNWRAP: Get the cell_type and immediately drop the borrow.
            // By returning just the `char`, we satisfy the borrow checker and 
            // free up `cells` so we can use `.get_mut()` safely in the loop below.
            let (cell_type, cell_plus) = if let Ok(cell) = cells.get(entity) {
                (cell.cell_type.clone(), cell.plus)
            } else {
                continue;
            };

            // 3. Define the base cardinal directions (for '+')
            let mut neighbors = vec![
                IVec2::new(x, y - 1), // Up
                IVec2::new(x, y + 1), // Down
                IVec2::new(x - 1, y), // Left
                IVec2::new(x + 1, y), // Right
            ];

            // 4. If it's a '*', append the 4 diagonal directions for AoE
            if cell_type == "*" {
                neighbors.extend([
                    IVec2::new(x - 1, y - 1), // Top-Left
                    IVec2::new(x + 1, y - 1), // Top-Right
                    IVec2::new(x - 1, y + 1), // Bottom-Left
                    IVec2::new(x + 1, y + 1), // Bottom-Right
                ]);
            }

            for n_pos in neighbors {
                if n_pos.x >= 0 && n_pos.x < grid.width as i32 && n_pos.y >= 0 && n_pos.y < grid.height as i32 {
                    
                    let index = (n_pos.y * grid.width as i32 + n_pos.x) as usize;
                    
                    if let Some(neighbor_entity) = grid.grid[index] {
                        if let Ok(mut neighbor_cell) = cells.get_mut(neighbor_entity) {
                            // Apply the buff!
                            neighbor_cell.plus += cell_plus;
                        }
                    }
                }
            }
        }
    }
}