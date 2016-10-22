extern crate sdl2;
extern crate rand;

use std::thread;
use std::time::Duration;
use std::path::Path;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::env;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};
use sdl2::surface::Surface;

use rand::{thread_rng, Rng};

struct Font {
    texture: Texture,
    characters: [Character; 256],
}

#[derive(Copy, Clone, PartialEq)]
struct Character {
    x: u32,
    width: u32,
}

impl Font {
    fn load_bmp<P: AsRef<Path>>(renderer: &mut Renderer, path: P) -> Font {
        let mut characters = [Character { x: 0, width: 0 }; 256];
        let mut surface = Surface::load_bmp(path).unwrap();
        surface.with_lock(|pixels| {
            let mut count = 0;
            let mut last_x = 0;

            for (x, color) in pixels[3 as usize..(surface.width() * 3) as usize]
                .chunks(3)
                .enumerate() {
                if color == [255, 0, 255] {
                    characters[count].x = (last_x + 1) as u32;
                    characters[count].width = (x - last_x) as u32;
                    count += 1;
                    last_x = x + 1;
                }
            }
        });
        surface.set_color_key(true, Color::RGB(211, 203, 207)).unwrap();
        Font {
            texture: renderer.create_texture_from_surface(surface).unwrap(),
            characters: characters,
        }
    }

    fn draw(&self, renderer: &mut Renderer, x: i32, y: i32, text: &str) {
        let mut position = x;
        for byte in text.bytes() {
            position += if byte == ' ' as u8 {
                2
            } else {
                let character = self.characters[byte as usize - '!' as usize];
                renderer.copy(&self.texture,
                              Some(Rect::new(character.x as i32, 0, character.width, 10)),
                              Some(Rect::new(position, y, character.width, 9)));
                character.width as i32
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Tile {
    Empty,
    Wall(u8),
    Food,
    SnakeVertical,
    SnakeHorizontal,
    SnakeTurn(Direction, bool),
    SnakeHead(Direction),
    SnakeTail(Direction),
}

#[derive(Clone)]
struct Map {
    name: String,
    tiles: [[Tile; 23]; 32],
    snake_x: usize,
    snake_y: usize,
}

#[derive(Debug)]
enum MapError {
    Io(io::Error),
    InvalidFormat(String),
}

impl Map {
    fn new() -> Map {
        Map {
            name: "Default".to_string(),
            tiles: [[Tile::Empty; 23]; 32],
            snake_x: 5,
            snake_y: 5,
        }
    }

    fn is_wall(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= 32 || y < 0 || y >= 23 {
            return true;
        }
        match self.tiles[x as usize][y as usize] {
            Tile::Wall(_) => true,
            _ => false,
        }
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Map, MapError> {
        let file = try!(File::open(path).map_err(MapError::Io));
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let name = match lines.next() {
                Some(line) => try!(line.map_err(MapError::Io)),
                None => return Err(MapError::InvalidFormat("name required".to_string())),
            }
            .trim()
            .to_string();

        if name.is_empty() {
            return Err(MapError::InvalidFormat("empty name".to_string()));
        }

        let mut tiles = [[Tile::Empty; 23]; 32];
        let mut snake_pos = None;
        for (y, line) in lines.take(23).enumerate() {
            for (x, c) in try!(line.map_err(MapError::Io)).chars().take(32).enumerate() {
                match c {
                    'X' => tiles[x][y] = Tile::Wall(0),
                    ' ' => tiles[x][y] = Tile::Empty,
                    '@' => snake_pos = Some((x, y)),
                    _ => {}
                }
            }
        }

        let (snake_x, snake_y) =
            try!(snake_pos.ok_or(MapError::InvalidFormat("no snake".to_string())));
        let mut map = Map {
            name: name,
            tiles: tiles,
            snake_x: snake_x,
            snake_y: snake_y,
        };
        for x in 0..32 {
            for y in 0..23 {
                match map.tiles[x as usize][y as usize] {
                    Tile::Wall(i) => {
                        let mut new_i = i;
                        if map.is_wall(x, y - 1) {
                            new_i += 1;
                        }
                        if map.is_wall(x + 1, y) {
                            new_i += 2;
                        }
                        if map.is_wall(x, y + 1) {
                            new_i += 4;
                        }
                        if map.is_wall(x - 1, y) {
                            new_i += 8;
                        }
                        map.tiles[x as usize][y as usize] = Tile::Wall(new_i);
                    }
                    _ => {}
                }
            }
        }
        Ok(map)
    }
}

struct SnakeEnd {
    x: i32,
    y: i32,
    direction: Direction,
}

impl SnakeEnd {
    fn update(&mut self) {
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

struct Snake {
    head: SnakeEnd,
    tail: SnakeEnd,
    grow: u8,
}

impl Snake {
    fn new(x: i32, y: i32, direction: Direction) -> Snake {
        Snake {
            head: SnakeEnd {
                x: x,
                y: y,
                direction: direction,
            },
            tail: SnakeEnd {
                x: x,
                y: y,
                direction: direction,
            },
            grow: 10,
        }
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
            renderer.copy(&tiles,
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
                          Some(target_rect));
        }
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
        update_tile(renderer, &tiles, &map, x, y);
        break;
    }
}

struct Game {
    snake: Snake,
    score: u32,
    snake_alive: bool,
    snake_show: bool,
    map: Map,
}

impl Game {
    fn new(renderer: &mut Renderer, font: &Font, tiles: &Texture, map: &Map) -> Game {
        let mut game = Game {
            snake: Snake::new(map.snake_x as i32, map.snake_y as i32, Direction::Right),
            snake_alive: true,
            snake_show: true,
            score: 0,
            map: map.clone(),
        };

        renderer.set_draw_color(Color::RGB(255, 255, 255));
        renderer.clear();

        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.fill_rect(Rect::new(0, 0, 320, 10)).unwrap();
        font.draw(renderer, 1, 0, &format!("Score: {}", game.score));
        font.draw(renderer, 100, 0, &game.map.name);

        for x in 0..32 {
            for y in 0..23 {
                update_tile(renderer, &tiles, &game.map, x, y);
            }
        }

        place_food(&mut game.map, renderer, &tiles);

        game
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Natrix", 320, 240)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = window.renderer().present_vsync().build().unwrap();

    let tiles =
        renderer.create_texture_from_surface(Surface::load_bmp("data/images/tiles.bmp").unwrap())
            .unwrap();

    let font = Font::load_bmp(&mut renderer, "data/images/NeoSans.bmp");

    let mut event_pump = sdl_context.event_pump().unwrap();

    let map = if let Some(path) = env::args().nth(1) {
        Map::load(path).unwrap()
    } else {
        Map::new()
    };

    let mut game = Game::new(&mut renderer, &font, &tiles, &map);

    'running: loop {
        let mut next_direction = game.snake.head.direction;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { scancode: Some(scancode), .. } => {
                    if game.snake_alive {
                        next_direction = match scancode {
                            Scancode::W => Direction::Up,
                            Scancode::D => Direction::Right,
                            Scancode::S => Direction::Down,
                            Scancode::A => Direction::Left,
                            _ => next_direction,
                        }
                    } else {
                        if scancode == Scancode::R {
                            game = Game::new(&mut renderer, &font, &tiles, &map);
                            continue 'running;
                        }
                    }
                }
                _ => {}
            }
        }

        if game.snake_alive {
            if game.snake.grow > 0 {
                game.snake.grow -= 1;
            } else {
                game.map.tiles[game.snake.tail.x as usize][game.snake.tail.y as usize] =
                    Tile::Empty;

                update_tile(&mut renderer,
                            &tiles,
                            &game.map,
                            game.snake.tail.x,
                            game.snake.tail.y);

                game.snake.tail.update();

                match game.map.tiles[game.snake.tail.x as usize][game.snake.tail.y as usize] {
                    Tile::SnakeTurn(direction, _) => game.snake.tail.direction = direction,
                    Tile::SnakeVertical | Tile::SnakeHorizontal => {}
                    _ => unreachable!(),
                };

                game.map.tiles[game.snake.tail.x as usize][game.snake.tail.y as usize] =
                    Tile::SnakeTail(game.snake
                        .tail
                        .direction);

                update_tile(&mut renderer,
                            &tiles,
                            &game.map,
                            game.snake.tail.x,
                            game.snake.tail.y);
            }

            if game.snake.head.direction != next_direction &&
               next_direction.opposite() != game.snake.head.direction {
                game.map.tiles[game.snake.head.x as usize][game.snake.head.y as usize] =
                    Tile::SnakeTurn(next_direction,
                                    match (game.snake.head.direction, next_direction) {
                                        (Direction::Right, Direction::Down) |
                                        (Direction::Down, Direction::Left) |
                                        (Direction::Left, Direction::Up) |
                                        (Direction::Up, Direction::Right) => true,
                                        _ => false,
                                    });
                game.snake.head.direction = next_direction;
            } else {
                game.map.tiles[game.snake.head.x as usize][game.snake.head.y as usize] =
                    match game.snake.head.direction {
                        Direction::Up | Direction::Down => Tile::SnakeVertical,
                        Direction::Right | Direction::Left => Tile::SnakeHorizontal,
                    };
            }

            update_tile(&mut renderer,
                        &tiles,
                        &game.map,
                        game.snake.head.x,
                        game.snake.head.y);

            game.snake.head.update();

            match game.map.tiles[game.snake.head.x as usize][game.snake.head.y as usize] {
                Tile::Food => {
                    place_food(&mut game.map, &mut renderer, &tiles);
                    game.snake.grow += 5;
                    game.score += 1;
                    font.draw(&mut renderer, 1, 0, &format!("Score: {}", game.score));
                }
                Tile::Wall(_) |
                Tile::SnakeVertical |
                Tile::SnakeHorizontal |
                Tile::SnakeTurn(_, _) |
                Tile::SnakeTail(_) => {
                    game.snake_alive = false;
                }
                _ => {}
            }

            if game.snake_alive {
                game.map.tiles[game.snake.head.x as usize][game.snake.head.y as usize] =
                    Tile::SnakeHead(game.snake.head.direction);
                update_tile(&mut renderer,
                            &tiles,
                            &game.map,
                            game.snake.head.x,
                            game.snake.head.y);
            }
        } else {
            for x in 0..32 {
                for y in 0..23 {
                    if !game.snake_show &&
                       match game.map.tiles[x as usize][y as usize] {
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
                        update_tile(&mut renderer, &tiles, &game.map, x, y);
                    }
                }
            }
            if game.snake_show {
                font.draw(&mut renderer, 200, 0, "Press R to restart");
            } else {
                renderer.set_draw_color(Color::RGB(0, 0, 0));
                renderer.fill_rect(Rect::new(200, 0, 120, 10)).unwrap();
            }
            game.snake_show = !game.snake_show;
        }

        renderer.present();

        thread::sleep(Duration::from_millis(100));
    }
}
