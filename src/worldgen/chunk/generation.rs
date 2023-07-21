use crate::camera::PlayerCamera;
use crate::worldgen::chunk::timer::ChunkGenerationTimer;
use crate::worldgen::chunk::{
    Chunk, ChunkQueue, GeneratedChunks, CHUNK_SIZE, MAX_CHUNKS_PROCESSED_PER_ITER, VIEW_DISTANCE,
};
use bevy::prelude::*;
use bevy::utils::HashMap;
use crossbeam::queue::SegQueue;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Fills the chunk queue with values in the current render distance.
pub fn fill_chunk_queue(
    generated_chunks: ResMut<GeneratedChunks>,
    chunk_queue: ResMut<ChunkQueue>,

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

    // Generate chunks in a bigger radius than the view distance
    let dist_mult = 1;

    for x in (-VIEW_DISTANCE.0 * dist_mult)..=(VIEW_DISTANCE.0 * dist_mult) {
        for y in (-VIEW_DISTANCE.1 * dist_mult)..=(VIEW_DISTANCE.1 * dist_mult) {
            for z in (-VIEW_DISTANCE.2 * dist_mult)..=(VIEW_DISTANCE.2 * dist_mult) {
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
///
/// This is probably the biggest bottleneck when it comes to performance.
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
}
