use bevy::prelude::*;

#[derive(Component)]
pub struct NavTarget(pub Vec3);

pub fn process_nav_commands(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &NavTarget)>,
) {
    let speed = 10.0;
    let turn_speed = 5.0;

    for (entity, mut transform, target) in query.iter_mut() {
        let current_pos = transform.translation;
        // Force Y to remain flat with the current height
        let target_pos = Vec3::new(target.0.x, current_pos.y, target.0.z);

        let direction = target_pos - current_pos;
        let distance = direction.length();

        // If within 0.1 units, we have arrived
        if distance < 0.1 {
            commands.entity(entity).remove::<NavTarget>();
            info!("Ship {:?} reached destination.", entity);
        } else {
            // 1. Move linearly
            let dir_normalized = direction / distance;
            transform.translation += dir_normalized * speed * time.delta_secs();
            
            // 2. Rotate smoothly to face the movement vector
            let target_rotation = Transform::from_translation(current_pos)
                .looking_at(target_pos, Vec3::Y)
                .rotation;
                
            transform.rotation = transform.rotation.slerp(target_rotation, time.delta_secs() * turn_speed);
        }
    }
}