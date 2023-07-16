use bevy::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Block {
    Grass,
    Dirt,
    Stone,
    Water,
    Air
}

impl Block {
    pub fn get_color(self) -> Color {
        match self {
            Block::Grass => Color::rgb(0.5, 1.0, 0.5),
            Block::Dirt => Color::rgb(0.8, 0.6, 0.2),
            Block::Stone => Color::rgb(0.5, 0.5, 0.5),
            Block::Water => Color::rgb(0.2, 0.5, 1.0),
            _ => panic!("Tried to get block color for air, which should never be drawn")
        }
    }

    pub fn is_opaque(self) -> bool {
        self != Block::Air
    }
}
