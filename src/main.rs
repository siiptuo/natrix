// SPDX-FileCopyrightText: 2019 Tuomas Siipola
// SPDX-License-Identifier: GPL-3.0-or-later

use sdl2::surface::Surface;

mod direction;
mod font;
mod game;
mod map;
mod menu;
mod snake;
mod state;
mod tile;

use crate::font::Font;
use crate::menu::Menu;
use crate::state::{Action, State};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let scale = 2;

    let window = video_subsystem
        .window("Natrix", scale * 320, scale * 240)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    canvas.set_scale(scale as f32, scale as f32).unwrap();

    let texture_creator = canvas.texture_creator();

    let tiles = texture_creator
        .create_texture_from_surface(Surface::load_bmp("data/images/tiles.bmp").unwrap())
        .unwrap();

    let logo = texture_creator
        .create_texture_from_surface(Surface::load_bmp("data/images/logo.bmp").unwrap())
        .unwrap();

    let mut font = Font::load_bmp(&texture_creator, "data/images/NeoSans.bmp");

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut current_state: Box<dyn State> = Box::new(Menu::new());

    loop {
        match current_state.update(
            event_pump.poll_iter(),
            &mut canvas,
            &mut font,
            &tiles,
            &logo,
        ) {
            Action::Quit => break,
            Action::Change(next_state) => current_state = next_state,
            Action::None => {}
        }
    }
}
