pub mod block;
pub mod chunk;
pub mod gen;

use bevy::prelude::*;
use bevy::utils::HashMap;

pub struct WorldgenPlugin;

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(chunk::GeneratedChunks(HashMap::new()))
            .insert_resource(chunk::ChunkGenerationTimer(Timer::from_seconds(
                1.0,
                TimerMode::Repeating,
            )))
            .add_systems(Startup, chunk::spawn_initial_chunks)
            .add_systems(
                Update,
                (
                    chunk::tick_chunk_generation_timer,
                    chunk::spawn_new_chunks,
                    chunk::despawn_old_chunks,
                ),
            );
    }
}
