/// Interfaces the chunk logic with Bevy APIs for use with ECS.
pub mod interface;
/// Contains chunk generation logic; somewhat disconnected from Bevy (still uses Bevy types)
pub mod generation;

pub use interface::*;
pub use generation::*;