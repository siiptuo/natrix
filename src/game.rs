use rand::{thread_rng, Rng};
use std::thread;
use std::time::Duration;

use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::render::{Renderer, Texture};
use sdl2::keyboard::Scancode;
use sdl2::event::{Event, EventPollIterator};

use direction::Direction;
use tile::Tile;
use map::Map;
use snake::Snake;
use font::Font;
use state::{State, Action};
use menu::Menu;

pub struct Game {
    snake: Snake,
    score: u32,
    snake_alive: bool,
    snake_show: bool,
    initial_map: Map,
    map: Map,
}

impl Game {
    fn redraw(renderer: &mut Renderer, font: &Font, tiles: &Texture, game: &Game) {
        renderer.set_draw_color(Color::RGB(255, 255, 255));
        renderer.clear();

        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.fill_rect(Rect::new(0, 0, 320, 10)).unwrap();
        font.draw(renderer, 1, 0, &format!("Score: {}", game.score));
        font.draw(renderer,
                  ((320 - font.measure(&game.map.name)) / 2) as i32,
                  0,
                  &game.map.name);

        for x in 0..32 {
            for y in 0..23 {
                update_tile(renderer, tiles, &game.map, x, y);
            }
        }
    }

    pub fn new(renderer: &mut Renderer, font: &Font, tiles: &Texture, map: &Map) -> Game {
        let mut game = Game {
            snake: Snake::new(map.snake_x as i32, map.snake_y as i32, Direction::Right),
            snake_alive: true,
            snake_show: true,
            score: 0,
            initial_map: map.clone(),
            map: map.clone(),
        };

        Game::redraw(renderer, font, tiles, &game);

        place_food(&mut game.map, renderer, tiles);

        game
    }
}

impl State for Game {
    fn update(&mut self,
              events: EventPollIterator,
              renderer: &mut Renderer,
              font: &Font,
              tiles: &Texture)
              -> Action {
        let mut next_direction = self.snake.head.direction;

        for event in events {
            match event {
                Event::Quit { .. } => return Action::Quit,
                Event::KeyDown { scancode: Some(scancode), .. } => {
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
                                self.snake = Snake::new(self.map.snake_x as i32,
                                                        self.map.snake_y as i32,
                                                        Direction::Right);
                                self.snake_alive = true;
                                self.snake_show = true;
                                self.score = 0;
                                place_food(&mut self.map, renderer, tiles);
                                Game::redraw(renderer, font, tiles, self);
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

                update_tile(renderer,
                            tiles,
                            &self.map,
                            self.snake.tail.x,
                            self.snake.tail.y);

                self.snake.tail.update();

                match self.map.tiles[self.snake.tail.x as usize][self.snake.tail.y as usize] {
                    Tile::SnakeTurn(direction, _) => self.snake.tail.direction = direction,
                    Tile::SnakeVertical | Tile::SnakeHorizontal => {}
                    _ => unreachable!(),
                };

                self.map.tiles[self.snake.tail.x as usize][self.snake.tail.y as usize] =
                    Tile::SnakeTail(self.snake
                        .tail
                        .direction);

                update_tile(renderer,
                            tiles,
                            &self.map,
                            self.snake.tail.x,
                            self.snake.tail.y);
            }

            if self.snake.head.direction != next_direction &&
               next_direction.opposite() != self.snake.head.direction {
                self.map.tiles[self.snake.head.x as usize][self.snake.head.y as usize] =
                    Tile::SnakeTurn(next_direction,
                                    match (self.snake.head.direction, next_direction) {
                                        (Direction::Right, Direction::Down) |
                                        (Direction::Down, Direction::Left) |
                                        (Direction::Left, Direction::Up) |
                                        (Direction::Up, Direction::Right) => true,
                                        _ => false,
                                    });
                self.snake.head.direction = next_direction;
            } else {
                self.map.tiles[self.snake.head.x as usize][self.snake.head.y as usize] =
                    match self.snake.head.direction {
                        Direction::Up | Direction::Down => Tile::SnakeVertical,
                        Direction::Right | Direction::Left => Tile::SnakeHorizontal,
                    };
            }

            update_tile(renderer,
                        tiles,
                        &self.map,
                        self.snake.head.x,
                        self.snake.head.y);

            self.snake.head.update();

            match self.map.tiles[self.snake.head.x as usize][self.snake.head.y as usize] {
                Tile::Food => {
                    place_food(&mut self.map, renderer, tiles);
                    self.snake.grow += 5;
                    self.score += 1;
                    font.draw(renderer, 1, 0, &format!("Score: {}", self.score));
                }
                Tile::Wall(_) |
                Tile::SnakeVertical |
                Tile::SnakeHorizontal |
                Tile::SnakeTurn(_, _) |
                Tile::SnakeTail(_) => {
                    self.snake_alive = false;
                }
                _ => {}
            }

            if self.snake_alive {
                self.map.tiles[self.snake.head.x as usize][self.snake.head.y as usize] =
                    Tile::SnakeHead(self.snake.head.direction);
                update_tile(renderer,
                            tiles,
                            &self.map,
                            self.snake.head.x,
                            self.snake.head.y);
            }
        } else {
            for x in 0..32 {
                for y in 0..23 {
                    if !self.snake_show &&
                       match self.map.tiles[x as usize][y as usize] {
                        Tile::SnakeVertical |
                        Tile::SnakeHorizontal |
                        Tile::SnakeTurn(_, _) |
                        Tile::SnakeTail(_) => true,
                        _ => false,
                    } {
                        let target_rect = Rect::new(x * 10, 10 + y * 10, 10, 10);
                        renderer.set_draw_color(Color::RGB(215, 227, 244));
                        renderer.fill_rect(target_rect).unwrap();
                    } else {
                        update_tile(renderer, tiles, &self.map, x, y);
                    }
                }
            }
            if self.snake_show {
                font.draw(renderer,
                          320 - 1 - font.measure("R restart   M menu") as i32,
                          0,
                          "R restart   M menu");
            } else {
                renderer.set_draw_color(Color::RGB(0, 0, 0));
                renderer.fill_rect(Rect::new(200, 0, 120, 10)).unwrap();
            }
            self.snake_show = !self.snake_show;
        }

        renderer.present();

        thread::sleep(Duration::from_millis(100));

        Action::None
    }
}

fn place_food(map: &mut Map, renderer: &mut Renderer, tiles: &Texture) {
    let mut rng = thread_rng();
    loop {
        let (x, y) = (rng.gen_range(0, 32), rng.gen_range(0, 23));
        if map.tiles[x as usize][y as usize] != Tile::Empty {
            continue;
        }
        map.tiles[x as usize][y as usize] = Tile::Food;
        update_tile(renderer, tiles, map, x, y);
        break;
    }
}

fn update_tile(renderer: &mut Renderer, tiles: &Texture, map: &Map, x: i32, y: i32) {
    let target_rect = Rect::new(x * 10, 10 + y * 10, 10, 10);
    match map.tiles[x as usize][y as usize] {
        Tile::Empty => {
            renderer.set_draw_color(Color::RGB(215, 227, 244));
            renderer.fill_rect(target_rect).unwrap();
        }
        tile => {
            renderer.copy(tiles,
                          Some(Rect::new(match tile {
                                             Tile::Wall(i) => 150 + 10 * i as i32,
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
                                             Tile::SnakeTurn(Direction::Up, false) |
                                             Tile::SnakeTurn(Direction::Left, true) => 110,
                                             Tile::SnakeTurn(Direction::Right, false) |
                                             Tile::SnakeTurn(Direction::Up, true) => 80,
                                             Tile::SnakeTurn(Direction::Down, false) |
                                             Tile::SnakeTurn(Direction::Right, true) => 90,
                                             Tile::SnakeTurn(Direction::Left, false) |
                                             Tile::SnakeTurn(Direction::Down, true) => 100,
                                             _ => unreachable!(),
                                         },
                                         0,
                                         10,
                                         10)),
                          Some(target_rect)).unwrap();
        }
    }
}
