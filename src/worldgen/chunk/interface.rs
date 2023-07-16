use crate::camera::PlayerCamera;
use crate::worldgen::chunk::generation::*;
use bevy::prelude::*;
use bevy::utils::HashMap;

// Should eventually be configurable
pub const VIEW_DISTANCE: i32 = 8;
const MAX_CHUNKS_PROCESSED_PER_ITER: usize = 16;

/// Marker for whether this entity is terrain generated from the chunk meshing process.
#[derive(Component)]
pub struct ChunkedTerrain;

/// A hashmap containing all chunks that have been generated
#[derive(Resource)]
pub struct GeneratedChunks(pub HashMap<(i32, i32, i32), Chunk>);

/// A hashmap containing all chunks that are currently loaded,
/// regardless of whether they have been generated
#[derive(Resource)]
pub struct LoadedChunks(pub HashMap<(i32, i32, i32), bool>);

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
    mut generated_chunks: ResMut<GeneratedChunks>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    // Initial render distance.
    // This doesn't need to be high, because other chunks will be loaded during runtime;
    // these chunks are the chunks that are loaded before the game starts.
    let initial_view_distance = 1;

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
                        material: materials.add(CHUNK_MATERIAL),
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

pub fn spawn_new_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut generated_chunks: ResMut<GeneratedChunks>,
    mut loaded_chunks: ResMut<LoadedChunks>,
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

    for x in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for y in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for z in -VIEW_DISTANCE..=VIEW_DISTANCE {
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
                        material: materials.add(CHUNK_MATERIAL),
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
    mut loaded_chunks: ResMut<LoadedChunks>,
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
