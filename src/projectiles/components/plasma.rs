use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct PlasmaProjectile;

#[derive(Component)]
pub struct PlasmaProjectileStats {
    pub damage: i32,
    pub energy: i32,
    pub energy_loss_rate: i32,
    pub source_ship: Option<Entity>,
}

#[derive(Bundle)]
pub struct PlasmaProjectileBundle {
    pub plasma_projectile_tag: PlasmaProjectile,
    pub plasma_projectile_stats: PlasmaProjectileStats,

    pub sprite: Sprite,
    pub transform: Transform,
    pub visibility: Visibility,

    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_layers: CollisionLayers,
    pub sensor: Sensor,
    pub velocity: LinearVelocity,
    pub colliding_entities: CollidingEntities,
}