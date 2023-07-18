use super::interface::*;
use crate::camera::*;
use crate::worldgen::chunk::{Chunk, CHUNK_SIZE};
use bevy::prelude::*;
use bevy::reflect::List;
use bevy::utils::{HashMap, HashSet};
use crossbeam::queue::SegQueue;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Resource)]
pub struct GeneratedChunks {
    pub map: Arc<Mutex<HashMap<(i32, i32, i32), Chunk>>>,
}

#[derive(Resource)]
pub struct ChunkQueue(pub Arc<SegQueue<(i32, i32, i32)>>);

/// Fills the chunk queue with values in the current render distance.
pub fn fill_chunk_queue(
    generated_chunks: ResMut<GeneratedChunks>,
    mut chunk_queue: ResMut<ChunkQueue>,

    camera_query: Query<&Transform, With<PlayerCamera>>,

    chunk_generation_timer: Res<ChunkGenerationTimer>,
) {
    if !chunk_generation_timer.0.just_finished() {
        return;
    }

    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;

    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    let map = generated_chunks.map.lock().unwrap();

    let mut num_chunks_added = 0;

    for x in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for y in -VIEW_DISTANCE..=VIEW_DISTANCE {
            for z in -VIEW_DISTANCE..=VIEW_DISTANCE {
                if num_chunks_added == MAX_CHUNKS_PROCESSED_PER_ITER {
                    return;
                }

                let pos = IVec3::new(x, y, z) + camera_chunk_pos;
                let pos_tuple = (pos.x, pos.y, pos.z);

                // A chunk was already generated, don't add it to the queue
                if map.contains_key(&pos_tuple) {
                    continue;
                }

                chunk_queue.0.push(pos_tuple);

                num_chunks_added += 1;
            }
        }
    }
}

/// Helper function that runs on each thread; generating chunks in the chunk queue
/// and storing it in the generated chunks map.
fn generate_chunks_worker(
    chunks: Arc<Mutex<HashMap<(i32, i32, i32), Chunk>>>,
    queue: Arc<SegQueue<(i32, i32, i32)>>,
) {
    while let Some(chunk_pos) = queue.pop() {
        let mut map = chunks.lock().unwrap();

        // Chunk has already been generated
        if map.contains_key(&chunk_pos) {
            continue;
        }

        let chunk_pos_vec = IVec3::from(chunk_pos);
        let mut chunk = Chunk::empty(chunk_pos_vec * CHUNK_SIZE as i32);
        chunk.generate();

        map.insert(chunk_pos, chunk);
    }
}

/// System that generates chunks in parallel. This doesn't actually load chunks, it just generates
/// them and stores them.
pub fn generate_chunks_multithreaded(
    generated_chunks: ResMut<GeneratedChunks>,
    chunk_queue: ResMut<ChunkQueue>,
) {
    let num_threads = num_cpus::get();

    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let chunks = Arc::clone(&generated_chunks.map);
        let queue = Arc::clone(&chunk_queue.0);

        let handle = thread::spawn(move || {
            generate_chunks_worker(chunks, queue);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// If a chunk position is in this list, then it is loaded.
#[derive(Resource)]
pub struct LoadedChunks {
    pub chunks: HashSet<(i32, i32, i32)>,
}

pub fn load_generated_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    generated_chunks: Res<GeneratedChunks>,
    asset_server: Res<AssetServer>,

    camera_query: Query<&Transform, With<PlayerCamera>>,
) {
    let map = generated_chunks.map.lock().unwrap();

    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;
    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    let range = (-VIEW_DISTANCE..=VIEW_DISTANCE);

    let mut chunks_spawned = 0;

    for (chunk_pos, chunk) in map.iter() {
        if chunks_spawned == MAX_CHUNKS_PROCESSED_PER_ITER {
            return;
        }

        let offsetted_chunk_pos = camera_chunk_pos - IVec3::from(*chunk_pos);

        // Only load the chunk if it's in the view distance
        // This is required to prevent this system from trying to spawn
        // chunks that the despawn system has just destroyed.
        if !(range.contains(&offsetted_chunk_pos.x)
            && range.contains(&offsetted_chunk_pos.y)
            && range.contains(&offsetted_chunk_pos.z))
        {
            continue;
        }

        // If the chunk is empty or already loaded, don't bother loading it
        if chunk.is_empty() || loaded_chunks.chunks.contains(chunk_pos) {
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

        loaded_chunks.chunks.insert(*chunk_pos);

        chunks_spawned += 1;
    }
}

pub fn unload_chunks(
    mut commands: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
    chunks_query: Query<(Entity, &Transform), With<ChunkedTerrain>>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
) {
    let camera_transform = camera_query.get_single().unwrap();
    let camera_pos = camera_transform.translation;
    let camera_chunk_pos = (camera_pos / CHUNK_SIZE as f32).as_ivec3();

    let range = (-VIEW_DISTANCE..=VIEW_DISTANCE);

    let mut chunks_despawned = 0;

    for (entity, transform) in chunks_query.iter() {
        if chunks_despawned == MAX_CHUNKS_PROCESSED_PER_ITER {
            return;
        }

        let chunk_position = transform.translation.as_ivec3() / CHUNK_SIZE as i32;

        let offsetted_chunk_pos = chunk_position - camera_chunk_pos;

        // If the chunk isn't in the view distance, despawn it
        if !(range.contains(&offsetted_chunk_pos.x)
            && range.contains(&offsetted_chunk_pos.y)
            && range.contains(&offsetted_chunk_pos.z))
        {
            commands.entity(entity).despawn();

            loaded_chunks
                .chunks
                .remove::<(i32, i32, i32)>(&chunk_position.into());

            chunks_despawned += 1;
        }
    }
}
