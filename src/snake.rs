// SPDX-FileCopyrightText: 2019 Tuomas Siipola
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::direction::Direction;

pub struct SnakeEnd {
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
}

impl SnakeEnd {
    pub fn update(&mut self) {
        match self.direction {
            Direction::Up => {
                if self.y == 0 {
                    self.y = 23 - 1;
                } else {
                    self.y -= 1;
                }
            }
            Direction::Right => {
                if self.x == 32 - 1 {
                    self.x = 0;
                } else {
                    self.x += 1;
                }
            }
            Direction::Down => {
                if self.y == 23 - 1 {
                    self.y = 0;
                } else {
                    self.y += 1;
                }
            }
            Direction::Left => {
                if self.x == 0 {
                    self.x = 32 - 1;
                } else {
                    self.x -= 1;
                }
            }
        }
    }
}

pub struct Snake {
    pub head: SnakeEnd,
    pub tail: SnakeEnd,
    pub grow: u8,
}

impl Snake {
    pub fn new(x: i32, y: i32, direction: Direction) -> Snake {
        Snake {
            head: SnakeEnd { x, y, direction },
            tail: SnakeEnd { x, y, direction },
            grow: 10,
        }
    }
}
