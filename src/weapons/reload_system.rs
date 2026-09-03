use bevy::prelude::*;

use crate::weapons::*;

pub fn reload_system(
    time: Res<Time>,
    mut turrets: Query<&mut Turret>,
) {
    for mut turret in turrets.iter_mut() {
        if turret.reload > 0.0 {
            turret.reload -= time.delta_secs();
        }
    }
}