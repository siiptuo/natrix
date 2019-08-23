// SPDX-FileCopyrightText: 2019 Tuomas Siipola
// SPDX-License-Identifier: GPL-3.0-or-later

use rand::{thread_rng, Rng};
use std::thread;
use std::time::Duration;

use sdl2::event::{Event, EventPollIterator};
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::direction::Direction;
use crate::font::Font;
use crate::map::Map;
use crate::menu::Menu;
use crate::snake::Snake;
use crate::state::{Action, State};
use crate::tile::Tile;

pub struct Game {
    snake: Snake,
    score: u32,
    snake_alive: bool,
    snake_show: bool,
    initial_map: Map,
    map: Map,
}

impl Game {
    fn redraw(canvas: &mut Canvas<Window>, font: &Font, tiles: &Texture, game: &Game) {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.fill_rect(Rect::new(0, 0, 320, 10)).unwrap();
        font.draw(canvas, 1, 0, &format!("Score: {}", game.score));
        font.draw(
            canvas,
            ((320 - font.measure(&game.map.name)) / 2) as i32,
            0,
            &game.map.name,
        );

        for x in 0..32 {
            for y in 0..23 {
                update_tile(canvas, tiles, &game.map, x, y);
            }
        }
    }

    pub fn new(canvas: &mut Canvas<Window>, font: &Font, tiles: &Texture, map: &Map) -> Game {
        let mut game = Game {
            snake: Snake::new(map.snake_x as i32, map.snake_y as i32, Direction::Right),
            snake_alive: true,
            snake_show: true,
            score: 0,
            initial_map: map.clone(),
            map: map.clone(),
        };

        Game::redraw(canvas, font, tiles, &game);

        place_food(&mut game.map, canvas, tiles);

        game
    }
}

impl State for Game {
    fn update(
        &mut self,
        events: EventPollIterator,
        canvas: &mut Canvas<Window>,
        font: &Font,
        tiles: &Texture,
        _logo: &Texture,
    ) -> Action {
        let mut next_direction = self.snake.head.direction;

        for event in events {
            match event {
                Event::Quit { .. } => return Action::Quit,
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    if self.snake_alive {
                        next_direction = match scancode {
                            Scancode::W => Direction::Up,
                            Scancode::D => Direction::Right,
                            Scancode::S => Direction::Down,
                            Scancode::A => Direction::Left,
                            _ => next_direction,
                        }
                    } else {
                        match scancode {
                            Scancode::R => {
                                self.map = self.initial_map.clone();
                                self.snake = Snake::new(
                                    self.map.snake_x as i32,
                                    self.map.snake_y as i32,
                                    Direction::Right,
                                );
                                self.snake_alive = true;
                                self.snake_show = true;
                                self.score = 0;
                                place_food(&mut self.map, canvas, tiles);
                                Game::redraw(canvas, font, tiles, self);
                                return Action::None;
                            }
                            Scancode::M => {
                                return Action::Change(Box::new(Menu::new()));
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if self.snake_alive {
            if self.snake.grow > 0 {
                self.snake.grow -= 1;
            } else {
                self.map.tiles[self.snake.tail.x as usize][self.snake.tail.y as usize] =
                    Tile::Empty;

                update_tile(
                    canvas,
                    tiles,
                    &self.map,
                    self.snake.tail.x,
                    self.snake.tail.y,
                );

                self.snake.tail.update();

                match self.map.tiles[self.snake.tail.x as usize][self.snake.tail.y as usize] {
                    Tile::SnakeTurn(direction, _) => self.snake.tail.direction = direction,
                    Tile::SnakeVertical | Tile::SnakeHorizontal => {}
                    _ => unreachable!(),
                };

                self.map.tiles[self.snake.tail.x as usize][self.snake.tail.y as usize] =
                    Tile::SnakeTail(self.snake.tail.direction);

                update_tile(
                    canvas,
                    tiles,
                    &self.map,
                    self.snake.tail.x,
                    self.snake.tail.y,
                );
            }

            if self.snake.head.direction != next_direction
                && next_direction.opposite() != self.snake.head.direction
            {
                self.map.tiles[self.snake.head.x as usize][self.snake.head.y as usize] =
                    Tile::SnakeTurn(
                        next_direction,
                        match (self.snake.head.direction, next_direction) {
                            (Direction::Right, Direction::Down)
                            | (Direction::Down, Direction::Left)
                            | (Direction::Left, Direction::Up)
                            | (Direction::Up, Direction::Right) => true,
                            _ => false,
                        },
                    );
                self.snake.head.direction = next_direction;
            } else {
                self.map.tiles[self.snake.head.x as usize][self.snake.head.y as usize] =
                    match self.snake.head.direction {
                        Direction::Up | Direction::Down => Tile::SnakeVertical,
                        Direction::Right | Direction::Left => Tile::SnakeHorizontal,
                    };
            }

            update_tile(
                canvas,
                tiles,
                &self.map,
                self.snake.head.x,
                self.snake.head.y,
            );

            self.snake.head.update();

            match self.map.tiles[self.snake.head.x as usize][self.snake.head.y as usize] {
                Tile::Food => {
                    place_food(&mut self.map, canvas, tiles);
                    self.snake.grow += 5;
                    self.score += 1;
                    font.draw(canvas, 1, 0, &format!("Score: {}", self.score));
                }
                Tile::Wall(_)
                | Tile::SnakeVertical
                | Tile::SnakeHorizontal
                | Tile::SnakeTurn(_, _)
                | Tile::SnakeTail(_) => {
                    self.snake_alive = false;
                }
                _ => {}
            }

            if self.snake_alive {
                self.map.tiles[self.snake.head.x as usize][self.snake.head.y as usize] =
                    Tile::SnakeHead(self.snake.head.direction);
                update_tile(
                    canvas,
                    tiles,
                    &self.map,
                    self.snake.head.x,
                    self.snake.head.y,
                );
            }
        } else {
            for x in 0..32 {
                for y in 0..23 {
                    if !self.snake_show
                        && match self.map.tiles[x as usize][y as usize] {
                            Tile::SnakeVertical
                            | Tile::SnakeHorizontal
                            | Tile::SnakeTurn(_, _)
                            | Tile::SnakeTail(_) => true,
                            _ => false,
                        }
                    {
                        let target_rect = Rect::new(x * 10, 10 + y * 10, 10, 10);
                        canvas.set_draw_color(Color::RGB(215, 227, 244));
                        canvas.fill_rect(target_rect).unwrap();
                    } else {
                        update_tile(canvas, tiles, &self.map, x, y);
                    }
                }
            }
            if self.snake_show {
                font.draw(
                    canvas,
                    320 - 1 - font.measure("R restart   M menu") as i32,
                    0,
                    "R restart   M menu",
                );
            } else {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.fill_rect(Rect::new(200, 0, 120, 10)).unwrap();
            }
            self.snake_show = !self.snake_show;
        }

        canvas.present();

        thread::sleep(Duration::from_millis(100));

        Action::None
    }
}

fn place_food(map: &mut Map, canvas: &mut Canvas<Window>, tiles: &Texture) {
    let mut rng = thread_rng();
    loop {
        let (x, y) = (rng.gen_range(0, 32), rng.gen_range(0, 23));
        if map.tiles[x as usize][y as usize] != Tile::Empty {
            continue;
        }
        map.tiles[x as usize][y as usize] = Tile::Food;
        update_tile(canvas, tiles, map, x, y);
        break;
    }
}

fn update_tile(canvas: &mut Canvas<Window>, tiles: &Texture, map: &Map, x: i32, y: i32) {
    let target_rect = Rect::new(x * 10, 10 + y * 10, 10, 10);
    match map.tiles[x as usize][y as usize] {
        Tile::Empty => {
            canvas.set_draw_color(Color::RGB(215, 227, 244));
            canvas.fill_rect(target_rect).unwrap();
        }
        tile => {
            canvas
                .copy(
                    tiles,
                    Some(Rect::new(
                        match tile {
                            Tile::Wall(i) => 150 + 10 * i32::from(i),
                            Tile::Food => 140,
                            Tile::SnakeVertical => 120,
                            Tile::SnakeHorizontal => 130,
                            Tile::SnakeTail(Direction::Up) => 60,
                            Tile::SnakeTail(Direction::Right) => 70,
                            Tile::SnakeTail(Direction::Down) => 40,
                            Tile::SnakeTail(Direction::Left) => 50,
                            Tile::SnakeHead(Direction::Up) => 0,
                            Tile::SnakeHead(Direction::Right) => 10,
                            Tile::SnakeHead(Direction::Down) => 20,
                            Tile::SnakeHead(Direction::Left) => 30,
                            Tile::SnakeTurn(Direction::Up, false)
                            | Tile::SnakeTurn(Direction::Left, true) => 110,
                            Tile::SnakeTurn(Direction::Right, false)
                            | Tile::SnakeTurn(Direction::Up, true) => 80,
                            Tile::SnakeTurn(Direction::Down, false)
                            | Tile::SnakeTurn(Direction::Right, true) => 90,
                            Tile::SnakeTurn(Direction::Left, false)
                            | Tile::SnakeTurn(Direction::Down, true) => 100,
                            _ => unreachable!(),
                        },
                        0,
                        10,
                        10,
                    )),
                    Some(target_rect),
                )
                .unwrap();
        }
    }
}
