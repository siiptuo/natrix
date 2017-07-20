use sdl2::event::EventPollIterator;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use font::Font;

pub enum Action {
    None,
    Quit,
    Change(Box<State>),
}

pub trait State {
    fn update(
        &mut self,
        events: EventPollIterator,
        canvas: &mut Canvas<Window>,
        font: &Font,
        tiles: &Texture,
    ) -> Action;
}
