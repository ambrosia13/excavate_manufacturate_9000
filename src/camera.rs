use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::camera::MipBias;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (update_camera_position, update_camera_rotation));
    }
}

#[derive(Component)]
pub struct PlayerCamera;

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PlayerCamera)
        .insert(MipBias(10.0));
}

fn update_camera_position(
    mut transform_query: Query<&mut Transform, With<PlayerCamera>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = transform_query.get_single_mut().unwrap();
    let delta = time.delta_seconds();

    let mut movement = Vec3::ZERO;

    let forward = {
        let forward = transform.forward();
        Vec3::new(forward.x, 0.0, forward.z).normalize()
    };

    let right = {
        let right = transform.right();
        Vec3::new(right.x, 0.0, right.z).normalize()
    };

    let up = Vec3::Y;

    if input.pressed(KeyCode::W) {
        movement += forward * delta;
    }
    if input.pressed(KeyCode::S) {
        movement -= forward * delta;
    }

    if input.pressed(KeyCode::D) {
        movement += right * delta;
    }
    if input.pressed(KeyCode::A) {
        movement -= right * delta;
    }

    if input.pressed(KeyCode::Space) {
        movement += up * delta;
    }
    if input.pressed(KeyCode::ShiftLeft) {
        movement -= up * delta;
    }

    if movement.length_squared() > 0.0 {
        const MOVEMENT_SPEED: f32 = 0.2;
        transform.translation += MOVEMENT_SPEED * movement.normalize();
    }
}

fn update_camera_rotation(
    mut transform_query: Query<&mut Transform, With<PlayerCamera>>,
    mut motion: EventReader<MouseMotion>,
) {
    let mut transform = transform_query.get_single_mut().unwrap();

    for event in motion.iter() {
        let delta = event.delta;

        const SENSITIVITY: f32 = 0.15;

        let pitch = (-delta.y * SENSITIVITY).clamp(-89.0, 89.0);
        let yaw = -delta.x * SENSITIVITY;

        let right = transform.right();

        transform.rotate_axis(right, f32::sin(pitch.to_radians()));
        transform.rotate_y(yaw.to_radians());
    }
}
