use std::io::{self, BufReader};
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

use tile::Tile;

#[derive(Clone)]
pub struct Map {
    pub name: String,
    pub tiles: [[Tile; 23]; 32],
    pub snake_x: usize,
    pub snake_y: usize,
}

#[derive(Debug)]
pub enum MapError {
    Io(io::Error),
    InvalidFormat(String),
}

impl Map {
    pub fn new() -> Map {
        Map {
            name: "Default".to_string(),
            tiles: [[Tile::Empty; 23]; 32],
            snake_x: 5,
            snake_y: 5,
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Map, MapError> {
        let file = try!(File::open(path).map_err(MapError::Io));
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let name = match lines.next() {
            Some(line) => try!(line.map_err(MapError::Io)),
            None => return Err(MapError::InvalidFormat("name required".to_string())),
        }.trim()
            .to_string();

        if name.is_empty() {
            return Err(MapError::InvalidFormat("empty name".to_string()));
        }

        let mut tiles = [[Tile::Empty; 23]; 32];
        let mut snake_pos = None;
        for (y, line) in lines.take(23).enumerate() {
            for (x, c) in try!(line.map_err(MapError::Io))
                .chars()
                .take(32)
                .enumerate()
            {
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

        for x in 0..32 {
            for y in 0..23 {
                if let Tile::Wall(i) = tiles[x][y] {
                    let mut new_i = i;
                    if y == 0 || tiles[x][y - 1].is_wall() {
                        new_i += 1;
                    }
                    if x == 31 || tiles[x + 1][y].is_wall() {
                        new_i += 2;
                    }
                    if y == 22 || tiles[x][y + 1].is_wall() {
                        new_i += 4;
                    }
                    if x == 0 || tiles[x - 1][y].is_wall() {
                        new_i += 8;
                    }
                    tiles[x][y] = Tile::Wall(new_i);
                }
            }
        }

        Ok(Map {
            name: name,
            tiles: tiles,
            snake_x: snake_x,
            snake_y: snake_y,
        })
    }
}
