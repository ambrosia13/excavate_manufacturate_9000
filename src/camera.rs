use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (
                handle_input_movement,
                handle_input_jump,
                handle_input_rotation,
                tick_player_movement,
                dampen_velocity,
            ),
        );
    }
}

/// Marker for whether an entity is the player camera. There should only be one.
#[derive(Component)]
pub struct PlayerCamera;

/// Controls the physics of the player camera. The position is just the camera transform's translation.
#[derive(Component)]
pub struct PlayerCameraMovement {
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 20.0, 0.0),
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::capsule(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            0.2,
        ))
        .insert(KinematicCharacterController::default())
        .insert(PlayerCamera)
        .insert(PlayerCameraMovement {
            velocity: Vec3::ZERO,
            acceleration: Vec3::new(0.0, -1.0, 0.0),
        });
}

/// Updates the PlayerCameraMovement component, and assigns it to the character controller's
/// translation field.
fn tick_player_movement(
    mut query: Query<
        (&mut PlayerCameraMovement, &mut KinematicCharacterController),
        With<PlayerCamera>,
    >,
    time: Res<Time>,
) {
    let (mut movement, mut controller) = query.get_single_mut().unwrap();

    let delta_seconds = time.delta_seconds();

    let accel = movement.acceleration;
    movement.velocity += accel * delta_seconds;

    movement.velocity = movement.velocity.clamp_length_max(1.0);

    controller.translation = Some(movement.velocity);
}

/// Dampens the velocity under certain conditions.
fn dampen_velocity(
    mut query: Query<
        (
            &mut PlayerCameraMovement,
            &KinematicCharacterControllerOutput,
        ),
        With<PlayerCamera>,
    >,
) {
    if let Ok((mut movement, output)) = query.get_single_mut() {
        if output.grounded {
            movement.velocity.y = movement.velocity.y.max(0.0);
        }

        movement.velocity.x *= 0.8;
        movement.velocity.z *= 0.8;
    }
}

/// Updates the PlayerCameraMovement depending on user input.
/// Should be before the tick_player_movement system, but I doubt it matters.
fn handle_input_movement(
    mut query: Query<(&Transform, &mut PlayerCameraMovement), With<PlayerCamera>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (transform, mut player_movement) = query.get_single_mut().unwrap();
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

    if input.pressed(KeyCode::W) {
        movement += forward;
    }
    if input.pressed(KeyCode::S) {
        movement -= forward;
    }

    if input.pressed(KeyCode::D) {
        movement += right;
    }
    if input.pressed(KeyCode::A) {
        movement -= right;
    }

    const MOVEMENT_SPEED: f32 = 1.5;
    let movement = MOVEMENT_SPEED * movement.normalize_or_zero();

    player_movement.velocity += movement * delta;
}

fn handle_input_jump(
    mut query: Query<
        (
            &mut PlayerCameraMovement,
            &KinematicCharacterControllerOutput,
        ),
        With<PlayerCamera>,
    >,
    input: Res<Input<KeyCode>>,
) {
    if let Ok((mut movement, output)) = query.get_single_mut() {
        const JUMP_VELOCITY: f32 = 0.25;

        if output.grounded && input.just_pressed(KeyCode::Space) {
            movement.velocity += Vec3::new(0.0, JUMP_VELOCITY, 0.0);
        }
    }
}

/// The same system as above, but for rotation.
fn handle_input_rotation(
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
