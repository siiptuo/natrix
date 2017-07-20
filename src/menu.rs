use std::fs;
use std::thread;
use std::time::Duration;

use sdl2::event::{Event, EventPollIterator};
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use font::Font;
use game::Game;
use map::Map;
use state::{State, Action};

pub struct Menu {
    maps: Vec<Map>,
    selected_map: usize,
}

fn read_maps() -> Vec<Map> {
    match fs::read_dir("data/maps") {
        Ok(entries) => {
            entries
                .filter_map(|entry| match entry {
                    Ok(entry) => Map::load(entry.path()).ok(),
                    Err(_) => None,
                })
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

impl Menu {
    pub fn new() -> Menu {
        let maps = read_maps();
        Menu {
            selected_map: 0,
            maps: if maps.is_empty() {
                vec![Map::new()]
            } else {
                maps
            },
        }
    }
}

impl State for Menu {
    fn update(
        &mut self,
        events: EventPollIterator,
        canvas: &mut Canvas<Window>,
        font: &Font,
        tiles: &Texture,
    ) -> Action {
        for event in events {
            match event {
                Event::Quit { .. } => return Action::Quit,
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    match scancode {
                        Scancode::Space => {
                            return Action::Change(Box::new(Game::new(
                                canvas,
                                font,
                                tiles,
                                &self.maps[self.selected_map],
                            )));
                        }
                        Scancode::W => {
                            if self.selected_map == 0 {
                                self.selected_map = self.maps.len() - 1;
                            } else {
                                self.selected_map -= 1;
                            }
                        }
                        Scancode::S => {
                            if self.selected_map == self.maps.len() - 1 {
                                self.selected_map = 0;
                            } else {
                                self.selected_map += 1;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        font.draw(canvas, 10, 10, "Natrix");

        for (i, map) in self.maps.iter().enumerate() {
            font.draw(
                canvas,
                if i == self.selected_map { 20 } else { 10 },
                30 + i as i32 * 10,
                &map.name,
            );
        }

        canvas.present();

        thread::sleep(Duration::from_millis(100));

        Action::None
    }
}
