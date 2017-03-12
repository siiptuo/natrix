extern crate sdl2;
extern crate rand;

use sdl2::surface::Surface;

mod game;
mod font;
mod snake;
mod direction;
mod map;
mod tile;
mod state;
mod menu;

use font::Font;
use menu::Menu;
use state::{State, Action};

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

    let mut current_state: Box<State> = Box::new(Menu::new());

    loop {
        match current_state.update(event_pump.poll_iter(), &mut renderer, &font, &tiles) {
            Action::Quit => break,
            Action::Change(next_state) => current_state = next_state,
            Action::None => {}
        }
    }
}
