use avian2d::prelude::*;

#[derive(PhysicsLayer, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameLayer {
    #[default]
    Default,
    Ship,
    Shield,
    Projectile,
}
