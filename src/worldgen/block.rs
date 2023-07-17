use bevy::prelude::*;

pub const ATLAS_SIZE: (usize, usize) = (32, 32);

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Block {
    Grass,
    Dirt,
    Stone,
    Water,
    Air,
}

impl Block {
    pub fn get_texture_config(self) -> BlockTextureConfig {
        match self {
            Block::Dirt => BlockTextureConfig::new(0, 16),
            Block::Grass => BlockTextureConfig::new(0, 0),
            Block::Stone => BlockTextureConfig::new(16, 0),
            Block::Water => BlockTextureConfig::new(16, 16),
            _ => panic!(
                "Tried to query block texture config for a block that doesn't have a texture"
            ),
        }
    }

    pub fn is_opaque(self) -> bool {
        self != Block::Air
    }
}

#[derive(Clone, Copy)]
pub struct BlockTextureConfig {
    pub starting_x: u32,
    pub starting_y: u32,
}

impl BlockTextureConfig {
    pub fn new(starting_x: u32, starting_y: u32) -> Self {
        Self {
            starting_x,
            starting_y,
        }
    }
}
