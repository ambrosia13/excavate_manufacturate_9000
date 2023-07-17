use crate::camera::PlayerCamera;
use crate::worldgen::chunk::generation::*;

use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy::utils::HashMap;



// Should eventually be configurable
pub const VIEW_DISTANCE: i32 = 8;
pub const MAX_CHUNKS_PROCESSED_PER_ITER: usize = 16;

/// The material to use for the chunk mesh.
const CHUNK_MATERIAL: StandardMaterial = StandardMaterial {
    base_color: Color::WHITE,
    base_color_texture: None,
    emissive: Color::BLACK,
    emissive_texture: None,
    perceptual_roughness: 0.5,
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
    alpha_mode: AlphaMode::Opaque,
    depth_bias: 0.0,
    depth_map: None,
    parallax_depth_scale: 0.0,
    parallax_mapping_method: ParallaxMappingMethod::Occlusion,
    max_parallax_layer_count: 0.0,
};

pub fn chunk_material(asset_server: &Res<AssetServer>) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/atlas.png")),
        ..CHUNK_MATERIAL
    }
}

/// Marker for whether this entity is terrain generated from the chunk meshing process.
#[derive(Component)]
pub struct ChunkedTerrain;

/// A hashmap containing all chunks that have been generated
#[derive(Resource)]
pub struct GeneratedChunksOld(pub HashMap<(i32, i32, i32), Chunk>);

/// A hashmap containing all chunks that are currently loaded,
/// regardless of whether they have been generated
#[derive(Resource)]
pub struct LoadedChunksOld(pub HashMap<(i32, i32, i32), bool>);

/// Used to delay the generation of chunks so it doesn't happen constantly
#[derive(Resource)]
pub struct ChunkGenerationTimer(pub Timer);

pub fn tick_chunk_generation_timer(mut timer: ResMut<ChunkGenerationTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
}

pub fn spawn_initial_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut generated_chunks: ResMut<GeneratedChunksOld>,
    mut loaded_chunks: ResMut<LoadedChunksOld>,
    asset_server: Res<AssetServer>,
) {
    // Initial render distance.
    // This doesn't need to be high, because other chunks will be loaded during runtime;
    // these chunks are the chunks that are loaded before the game starts.
    let initial_view_distance = 2;

    for x in -initial_view_distance..=initial_view_distance {
        for y in -initial_view_distance..=initial_view_distance {
            for z in -initial_view_distance..=initial_view_distance {
                let pos = IVec3::new(x, y, z);

                let mut chunk = Chunk::empty(pos * CHUNK_SIZE as i32);
                chunk.generate();

                // Skip the whole mesh-making-process for empty chunks
                if chunk.is_empty() {
                    generated_chunks.0.insert((pos.x, pos.y, pos.z), chunk);
                    loaded_chunks.0.insert((pos.x, pos.y, pos.z), true);

                    continue;
                }

                let mesh = chunk.get_mesh();

                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(mesh),
                        material: materials.add(chunk_material(&asset_server)),
                        transform: Transform::from_translation(chunk.pos.as_vec3()),
                        ..default()
                    })
                    .insert(ChunkedTerrain);

                generated_chunks.0.insert((pos.x, pos.y, pos.z), chunk);
                loaded_chunks.0.insert((pos.x, pos.y, pos.z), true);
            }
        }
    }
}

pub fn spawn_chunks_in_region<T>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut generated_chunks: ResMut<GeneratedChunksOld>,
    mut loaded_chunks: ResMut<LoadedChunksOld>,
    asset_server: Res<AssetServer>,
    timer: Res<ChunkGenerationTimer>,
    camera_query: Query<&Transform, With<PlayerCamera>>,

    region_bounds_x: T,
    region_bounds_y: T,
    region_bounds_z: T,
) where
    T: Iterator<Item = i32> + Clone,
{
    if !timer.0.just_finished() {
        return;
    }

    let mut chunks_processed = 0;

    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;

    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    let i = -1..1;

    for x in region_bounds_x.clone() {
        for y in region_bounds_y.clone() {
            for z in region_bounds_z.clone() {
                if chunks_processed == MAX_CHUNKS_PROCESSED_PER_ITER {
                    return;
                }

                let pos = IVec3::new(x, y, z) + camera_chunk_pos;
                let pos_tuple = (pos.x, pos.y, pos.z);

                // Chunk is already loaded, don't load it again
                if loaded_chunks.0.contains_key(&pos_tuple) {
                    continue;
                }

                let mesh;
                let chunk_translation;

                if generated_chunks.0.contains_key(&pos_tuple) {
                    // Chunk is generated, so don't regenerate it
                    let chunk = generated_chunks.0.get(&pos_tuple).unwrap();

                    mesh = chunk.get_mesh();
                    chunk_translation = chunk.pos.as_vec3();
                } else {
                    // Chunk has not been generated yet, so we need to regenerate it

                    let mut chunk = Chunk::empty(pos * CHUNK_SIZE as i32);
                    chunk.generate();

                    // If the chunk is empty, mark it as generated and loaded, and move on
                    if chunk.is_empty() {
                        generated_chunks.0.insert(pos_tuple, chunk);
                        loaded_chunks.0.insert(pos_tuple, true);

                        continue;
                    }

                    mesh = chunk.get_mesh();
                    chunk_translation = chunk.pos.as_vec3();

                    // Now that the chunk has been generated and we don't need it anymore,
                    // move it to the hashmap
                    generated_chunks.0.insert(pos_tuple, chunk);
                }

                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(mesh),
                        material: materials.add(chunk_material(&asset_server)),
                        transform: Transform::from_translation(chunk_translation),
                        ..default()
                    })
                    .insert(ChunkedTerrain);

                loaded_chunks.0.insert(pos_tuple, true);

                chunks_processed += 1;
            }
        }
    }
}

pub fn despawn_old_chunks(
    mut commands: Commands,
    chunk_entity_query: Query<(Entity, &Transform), With<ChunkedTerrain>>,
    mut loaded_chunks: ResMut<LoadedChunksOld>,
    timer: Res<ChunkGenerationTimer>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
) {
    if !timer.0.just_finished() {
        return;
    }

    let mut chunks_processed = 0;

    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;

    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    for (entity, transform) in chunk_entity_query.iter() {
        if chunks_processed == MAX_CHUNKS_PROCESSED_PER_ITER {
            return;
        }

        let chunk_position = transform.translation.as_ivec3() / CHUNK_SIZE as i32;

        let diff = chunk_position - camera_chunk_pos;

        // If the chunk isn't in the view distance, despawn it
        if !(-VIEW_DISTANCE..=VIEW_DISTANCE).contains(&diff.x)
            || !(-VIEW_DISTANCE..=VIEW_DISTANCE).contains(&diff.y)
            || !(-VIEW_DISTANCE..=VIEW_DISTANCE).contains(&diff.z)
        {
            commands.entity(entity).despawn();

            loaded_chunks
                .0
                .remove(&(chunk_position.x, chunk_position.y, chunk_position.z));

            chunks_processed += 1;
        }
    }
}
