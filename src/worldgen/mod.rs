pub mod block;
pub mod chunk;
pub mod gen;

use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use crossbeam::queue::SegQueue;
use std::sync::{Arc, Mutex};

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(chunk::ChunkGenerationTimer(Timer::from_seconds(
            0.01,
            TimerMode::Repeating,
        )))
        .insert_resource(chunk::multithreading::GeneratedChunks {
            map: Arc::new(Mutex::new(HashMap::new())),
        })
        .insert_resource(chunk::multithreading::ChunkQueue(Arc::new(SegQueue::new())))
        .insert_resource(chunk::multithreading::LoadedChunks {
            chunks: HashSet::new(),
        })
        //.add_systems(Startup, chunk::spawn_initial_chunks)
        .add_systems(
            Update,
            (
                chunk::tick_chunk_generation_timer,
                chunk::multithreading::fill_chunk_queue,
                chunk::multithreading::generate_chunks_multithreaded,
                chunk::multithreading::load_generated_chunks,
                chunk::multithreading::unload_chunks,
            ),
        );
    }
}
