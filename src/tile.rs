use direction::Direction;

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
    pub fn is_wall(&self) -> bool {
        match *self {
            Tile::Wall(_) => true,
            _ => false,
        }
    }
}
