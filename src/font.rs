use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use std::path::Path;

pub struct Font {
    texture: Texture,
    characters: [Character; 256],
}

#[derive(Copy, Clone, PartialEq)]
struct Character {
    x: u32,
    width: u32,
}

impl Font {
    pub fn load_bmp<P: AsRef<Path>>(renderer: &mut Renderer, path: P) -> Font {
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

    pub fn draw(&self, renderer: &mut Renderer, x: i32, y: i32, text: &str) {
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
