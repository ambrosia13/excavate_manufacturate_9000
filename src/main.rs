#![feature(let_chains)]

mod camera;
mod worldgen;

use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

use bevy::window::{CursorGrabMode, PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WireframePlugin)
        .add_plugins(camera::CameraPlugin)
        .add_systems(Startup, (setup, configure_window))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn configure_window(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = window_query.get_single_mut().unwrap();

    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    let render_distance = 8;

    for x in -render_distance..=render_distance {
        for y in -render_distance..=render_distance {
            for z in -render_distance..=render_distance {
                let pos = Vec3::new(
                    (x * worldgen::chunk::CHUNK_SIZE as i32) as f32,
                    (y * worldgen::chunk::CHUNK_SIZE as i32) as f32,
                    (z * worldgen::chunk::CHUNK_SIZE as i32) as f32,
                );

                let mut chunk = worldgen::chunk::Chunk::empty(pos);
                chunk.generate();

                // Skip the whole mesh-making-process for empty chunks
                // Might not be very optimal to perform this check for every chunk
                // Maybe combine it with Chunk::generate?
                if chunk.is_empty() {
                    continue;
                }

                let mesh = chunk.get_mesh();

                commands.spawn(PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(1.0, 0.0, 0.0),
                        base_color_texture: None,
                        emissive: Color::BLACK,
                        emissive_texture: None,
                        perceptual_roughness: 1.0,
                        metallic: 0.0,
                        metallic_roughness_texture: None,
                        reflectance: 1.0,
                        normal_map_texture: None,
                        flip_normal_map_y: false,
                        occlusion_texture: None,
                        double_sided: false,
                        cull_mode: Some(Face::Back),
                        unlit: false,
                        fog_enabled: false,
                        alpha_mode: Default::default(),
                        depth_bias: 0.0,
                        depth_map: None,
                        parallax_depth_scale: 0.0,
                        parallax_mapping_method: Default::default(),
                        max_parallax_layer_count: 0.0,
                    }),
                    transform: Transform::from_translation(chunk.pos),
                    ..default()
                });
            }
        }
    }

    // let pos = Vec3::new(0.0, -10.0, 0.0);
    //
    // let mut chunk = worldgen::chunk::Chunk::empty(pos);
    // chunk.generate();
    //
    // let mesh = chunk.get_mesh();
    //
    // commands
    //     .spawn(PbrBundle {
    //         mesh: meshes.add(mesh),
    //         material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
    //         transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //         ..default()
    //     })
    //     .insert(Wireframe);
}
