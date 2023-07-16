mod camera;
mod worldgen;

use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;

use bevy::window::{CursorGrabMode, PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(WireframePlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(worldgen::WorldgenPlugin)
        .add_systems(Startup, (configure_window, spawn_light))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn configure_window(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = window_query.get_single_mut().unwrap();

    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}

fn spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(1.0, 0.8, 0.5),
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(50.0, 200.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
