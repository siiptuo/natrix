// SPDX-FileCopyrightText: 2019 Tuomas Siipola
// SPDX-License-Identifier: GPL-3.0-or-later

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};
use std::path::Path;

const SPACE_WIDTH: u32 = 2;

pub struct Font<'a> {
    texture: Texture<'a>,
    characters: [Character; 256],
}

#[derive(Copy, Clone, PartialEq)]
struct Character {
    x: u32,
    width: u32,
}

impl<'a> Font<'a> {
    pub fn load_bmp<P: AsRef<Path>>(
        texture_creator: &TextureCreator<WindowContext>,
        path: P,
    ) -> Font {
        let mut characters = [Character { x: 0, width: 0 }; 256];
        let mut surface = Surface::load_bmp(path).unwrap();
        surface.with_lock(|pixels| {
            let mut count = 0;
            let mut last_x = 0;

            for (x, color) in pixels[3 as usize..(surface.width() * 3) as usize]
                .chunks(3)
                .enumerate()
            {
                if color == [255, 0, 255] {
                    characters[count].x = (last_x + 1) as u32;
                    characters[count].width = (x - last_x) as u32;
                    count += 1;
                    last_x = x + 1;
                }
            }
        });
        surface
            .set_color_key(true, Color::RGB(211, 203, 207))
            .unwrap();
        Font {
            texture: texture_creator
                .create_texture_from_surface(surface)
                .unwrap(),
            characters,
        }
    }

    fn get_character(&self, byte: u8) -> Character {
        self.characters[byte as usize - '!' as usize]
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, x: i32, y: i32, text: &str) {
        let mut position = x;
        for byte in text.bytes() {
            position += if byte == b' ' {
                SPACE_WIDTH as i32
            } else {
                let character = self.get_character(byte);
                canvas
                    .copy(
                        &self.texture,
                        Some(Rect::new(character.x as i32, 0, character.width, 10)),
                        Some(Rect::new(position, y, character.width, 9)),
                    )
                    .unwrap();
                character.width as i32
            }
        }
    }

    pub fn measure(&self, text: &str) -> u32 {
        text.bytes().fold(0, |acc, byte| {
            acc + if byte == b' ' {
                SPACE_WIDTH
            } else {
                self.get_character(byte).width
            }
        })
    }
}
