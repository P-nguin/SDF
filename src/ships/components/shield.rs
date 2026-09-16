use bevy::prelude::*;

use crate::voxels::components::*; 

#[derive(Component)]
pub struct ShieldCell;

#[derive(Component)]
pub struct ShieldStats {
    pub shield_type: String,

    pub shield_max_health: i32,
    pub shield_health: i32,
    pub shield_regen: i32,
    pub shield_respawn: i16,
    
    pub projection_distance: i32, 

    pub projection: Option<Entity> // this will be a ShieldProjection
}

#[derive(Bundle)]
pub struct ShieldCellBundle {
    pub shield_tag: ShieldCell,
    pub shield_stats: ShieldStats,
    pub transform: Transform,
}

#[derive(Component)]
pub struct ShieldProjection;

#[derive(Component)]
pub struct ShieldProjectionStats {
    pub shield_max_health: i32,
    pub shield_health: i32,

    pub shield_respawn: i16,
}

#[derive(Bundle)]
pub struct ShieldProjectionBundle {
    pub marker: ShieldProjection,
    pub grid: VoxelGrid,
    pub transform: Transform,
    pub visibility: Visibility,
}
