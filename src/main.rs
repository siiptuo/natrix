extern crate sdl2;
extern crate rand;

use std::thread;
use std::time::Duration;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use rand::{thread_rng, Rng};

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
    Snake(Direction),
}

type Map = [[Tile; 24]; 32];

struct SnakeEnd {
    x: i32,
    y: i32,
    direction: Direction,
}

impl SnakeEnd {
    fn update(&mut self) {
        match self.direction {
            Direction::Up => self.y -= 1,
            Direction::Right => self.x += 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
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

fn place_food(map: &mut Map, renderer: &mut Renderer) {
    let mut rng = thread_rng();
    loop {
        let (x, y) = (rng.gen_range(0, 32), rng.gen_range(0, 24));
        if map[x as usize][y as usize] != Tile::Empty {
            continue;
        }
        map[x as usize][y as usize] = Tile::Food;
        renderer.set_draw_color(Color::RGB(255, 0, 0));
        renderer.fill_rect(Rect::new(x * 10, y * 10, 10, 10)).unwrap();
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

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut snake = Snake::new(5, 5, Direction::Right);

    let mut map: Map = [[Tile::Empty; 24]; 32];
    map[snake.head.x as usize][snake.head.y as usize] = Tile::Snake(snake.head.direction);

    renderer.set_draw_color(Color::RGB(255, 255, 255));
    renderer.clear();

    place_food(&mut map, &mut renderer);

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
            renderer.set_draw_color(Color::RGB(255, 255, 255));
            renderer.fill_rect(Rect::new(snake.tail.x * 10, snake.tail.y * 10, 10, 10)).unwrap();

            snake.tail.direction = match map[snake.tail.x as usize][snake.tail.y as usize] {
                Tile::Snake(direction) => direction,
                _ => unreachable!(),
            };

            map[snake.tail.x as usize][snake.tail.y as usize] = Tile::Empty;

            snake.tail.update();
        }

        if next_direction.opposite() != snake.head.direction {
            snake.head.direction = next_direction;
        }

        map[snake.head.x as usize][snake.head.y as usize] = Tile::Snake(snake.head.direction);

        snake.head.update();

        match map[snake.head.x as usize][snake.head.y as usize] {
            Tile::Food => {
                place_food(&mut map, &mut renderer);
                snake.grow += 5;
            }
            Tile::Snake(_) => panic!("You die!"),
            Tile::Empty => {}
        }

        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.fill_rect(Rect::new(snake.head.x * 10, snake.head.y * 10, 10, 10)).unwrap();

        renderer.present();

        thread::sleep(Duration::from_millis(100));
    }
}
