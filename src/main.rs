extern crate sdl2;
extern crate rand;

use std::thread;
use std::time::Duration;
use std::env;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::rect::Rect;
use sdl2::surface::Surface;

mod game;
mod font;
mod snake;
mod direction;
mod map;
mod tile;

use tile::Tile;
use direction::Direction;
use font::Font;
use map::Map;
use game::{Game, place_food, update_tile};

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
