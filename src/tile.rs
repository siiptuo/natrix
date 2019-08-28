// SPDX-FileCopyrightText: 2019 Tuomas Siipola
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::direction::Direction;

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
    Empty,
    Wall(u8),
    Food,
    SnakeVertical,
    SnakeHorizontal,
    SnakeTurn(Direction, bool),
    SnakeHead(Direction),
    SnakeTail(Direction),
}

impl Tile {
    pub fn is_empty(self) -> bool {
        match self {
            Tile::Empty => true,
            _ => false,
        }
    }

    pub fn is_wall(self) -> bool {
        match self {
            Tile::Wall(_) => true,
            _ => false,
        }
    }
}
