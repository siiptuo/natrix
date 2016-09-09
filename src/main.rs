extern crate sdl2;
extern crate rand;

use std::thread;
use std::time::Duration;
use std::path::Path;

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
    Food,
    SnakeVertical,
    SnakeHorizontal,
    SnakeTurn(Direction, bool),
    SnakeHead(Direction),
    SnakeTail(Direction),
}

type Map = [[Tile; 23]; 32];

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
    match map[x as usize][y as usize] {
        Tile::Empty => {
            renderer.set_draw_color(Color::RGB(255, 255, 255));
            renderer.fill_rect(target_rect).unwrap();
        }
        tile => {
            renderer.copy(&tiles,
                          Some(Rect::new(match tile {
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
        if map[x as usize][y as usize] != Tile::Empty {
            continue;
        }
        map[x as usize][y as usize] = Tile::Food;
        update_tile(renderer, &tiles, &map, x, y);
        break;
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

    let tiles = renderer.create_texture_from_surface(Surface::load_bmp("data/tiles.bmp").unwrap())
        .unwrap();

    let font = Font::load_bmp(&mut renderer, "data/NeoSans.bmp");

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut snake = Snake::new(5, 5, Direction::Right);

    let mut map: Map = [[Tile::Empty; 23]; 32];

    let mut score = 0;

    renderer.set_draw_color(Color::RGB(255, 255, 255));
    renderer.clear();

    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.fill_rect(Rect::new(0, 0, 320, 10)).unwrap();
    font.draw(&mut renderer, 1, 0, &format!("Score: {}", score));

    place_food(&mut map, &mut renderer, &tiles);

    'running: loop {
        let mut next_direction = snake.head.direction;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { scancode: Some(scancode), .. } => {
                    next_direction = match scancode {
                        Scancode::W => Direction::Up,
                        Scancode::D => Direction::Right,
                        Scancode::S => Direction::Down,
                        Scancode::A => Direction::Left,
                        _ => next_direction,
                    }
                }
                _ => {}
            }
        }

        if snake.grow > 0 {
            snake.grow -= 1;
        } else {
            map[snake.tail.x as usize][snake.tail.y as usize] = Tile::Empty;

            update_tile(&mut renderer, &tiles, &map, snake.tail.x, snake.tail.y);

            snake.tail.update();

            match map[snake.tail.x as usize][snake.tail.y as usize] {
                Tile::SnakeTurn(direction, _) => snake.tail.direction = direction,
                Tile::SnakeVertical | Tile::SnakeHorizontal => {}
                _ => unreachable!(),
            };

            map[snake.tail.x as usize][snake.tail.y as usize] = Tile::SnakeTail(snake.tail
                .direction);

            update_tile(&mut renderer, &tiles, &map, snake.tail.x, snake.tail.y);
        }

        if snake.head.direction != next_direction &&
           next_direction.opposite() != snake.head.direction {
            map[snake.head.x as usize][snake.head.y as usize] =
                Tile::SnakeTurn(next_direction,
                                match (snake.head.direction, next_direction) {
                                    (Direction::Right, Direction::Down) |
                                    (Direction::Down, Direction::Left) |
                                    (Direction::Left, Direction::Up) |
                                    (Direction::Up, Direction::Right) => true,
                                    _ => false,
                                });
            snake.head.direction = next_direction;
        } else {
            map[snake.head.x as usize][snake.head.y as usize] = match snake.head.direction {
                Direction::Up | Direction::Down => Tile::SnakeVertical,
                Direction::Right | Direction::Left => Tile::SnakeHorizontal,
            };
        }

        update_tile(&mut renderer, &tiles, &map, snake.head.x, snake.head.y);

        snake.head.update();

        match map[snake.head.x as usize][snake.head.y as usize] {
            Tile::Food => {
                place_food(&mut map, &mut renderer, &tiles);
                snake.grow += 5;
                score += 1;
                font.draw(&mut renderer, 1, 0, &format!("Score: {}", score));
            }
            Tile::SnakeVertical |
            Tile::SnakeHorizontal |
            Tile::SnakeTurn(_, _) |
            Tile::SnakeTail(_) => panic!("You die!"),
            _ => {}
        }

        map[snake.head.x as usize][snake.head.y as usize] = Tile::SnakeHead(snake.head.direction);

        update_tile(&mut renderer, &tiles, &map, snake.head.x, snake.head.y);

        renderer.present();

        thread::sleep(Duration::from_millis(100));
    }
}
