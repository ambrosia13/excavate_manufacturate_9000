use crate::camera::PlayerCamera;
use crate::worldgen::chunk::generation::*;
use bevy::prelude::*;
use bevy::utils::HashMap;

// Should eventually be configurable
pub const VIEW_DISTANCE: i32 = 8;

/// Marker for whether this entity is terrain generated from the chunk meshing process.
#[derive(Component)]
pub struct ChunkedTerrain;

/// A hashmap containing all loaded chunks
#[derive(Resource)]
pub struct GeneratedChunks(pub HashMap<(i32, i32, i32), Chunk>);

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
            }
        }
    }
}

pub fn spawn_new_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut generated_chunks: ResMut<GeneratedChunks>,
    timer: Res<ChunkGenerationTimer>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
) {
    if !timer.0.just_finished() {
        return;
    }

    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;

    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    for x in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for y in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for z in -VIEW_DISTANCE..=VIEW_DISTANCE {
                let pos = IVec3::new(x, y, z) + camera_chunk_pos;

                // This chunk was already generated, don't waste time generating it again
                if generated_chunks.0.contains_key(&(pos.x, pos.y, pos.z)) {
                    continue;
                }

                let mut chunk = Chunk::empty(pos * CHUNK_SIZE as i32);
                chunk.generate();

                if chunk.is_empty() {
                    generated_chunks.0.insert((pos.x, pos.y, pos.z), chunk);

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
            }
        }
    }
}

pub fn despawn_old_chunks(
    mut commands: Commands,
    chunk_entity_query: Query<(Entity, &Transform), With<ChunkedTerrain>>,
    mut generated_chunks: ResMut<GeneratedChunks>,
    timer: Res<ChunkGenerationTimer>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
) {
    if !timer.0.just_finished() {
        return;
    }

    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;

    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    for (entity, transform) in chunk_entity_query.iter() {
        let chunk_position = transform.translation.as_ivec3() / CHUNK_SIZE as i32;

        let diff = chunk_position - camera_chunk_pos;

        if !(-VIEW_DISTANCE..=VIEW_DISTANCE).contains(&diff.x)
            || !(-VIEW_DISTANCE..=VIEW_DISTANCE).contains(&diff.y)
            || !(-VIEW_DISTANCE..=VIEW_DISTANCE).contains(&diff.z)
        {
            commands.entity(entity).despawn();

            generated_chunks
                .0
                .remove(&(chunk_position.x, chunk_position.y, chunk_position.z));
        }
    }
}
