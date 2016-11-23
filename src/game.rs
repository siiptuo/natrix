use rand::{thread_rng, Rng};

use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::render::{Renderer, Texture};

use direction::Direction;
use tile::Tile;
use map::Map;
use snake::Snake;
use font::Font;

pub struct Game {
    pub snake: Snake,
    pub score: u32,
    pub snake_alive: bool,
    pub snake_show: bool,
    pub map: Map,
}

impl Game {
    pub fn new(renderer: &mut Renderer, font: &Font, tiles: &Texture, map: &Map) -> Game {
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
        font.draw(renderer,
                  ((320 - font.measure(&game.map.name)) / 2) as i32,
                  0,
                  &game.map.name);

        for x in 0..32 {
            for y in 0..23 {
                update_tile(renderer, tiles, &game.map, x, y);
            }
        }

        place_food(&mut game.map, renderer, tiles);

        game
    }
}

pub fn place_food(map: &mut Map, renderer: &mut Renderer, tiles: &Texture) {
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

pub fn update_tile(renderer: &mut Renderer, tiles: &Texture, map: &Map, x: i32, y: i32) {
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
                          Some(target_rect));
        }
    }
}
