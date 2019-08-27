// SPDX-FileCopyrightText: 2019 Tuomas Siipola
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::thread;
use std::time::Duration;

use sdl2::event::{Event, EventPollIterator};
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::font::Font;
use crate::game::Game;
use crate::map::Map;
use crate::state::{Action, State};

pub struct Menu {
    maps: Vec<Map>,
    selected_map: usize,
}

fn read_maps() -> Vec<Map> {
    match fs::read_dir("data/maps") {
        Ok(entries) => entries
            .filter_map(|entry| match entry {
                Ok(entry) => Map::load(entry.path()).ok(),
                Err(_) => None,
            })
            .collect(),
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
        font: &mut Font,
        tiles: &Texture,
        logo: &Texture,
    ) -> Action {
        for event in events {
            match event {
                Event::Quit { .. } => return Action::Quit,
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => match scancode {
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
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(215, 227, 244));
        canvas.clear();

        canvas
            .copy(logo, None, Rect::new((320 - 175) / 2, 40, 175, 40))
            .unwrap();

        for (i, map) in self.maps.iter().enumerate() {
            font.draw(
                canvas,
                if i == self.selected_map { 120 } else { 110 },
                110 + i as i32 * 10,
                &map.name,
                Color::RGB(0, 0, 0),
            );
        }

        canvas.present();

        thread::sleep(Duration::from_millis(100));

        Action::None
    }
}
