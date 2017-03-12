use sdl2::event::EventPollIterator;
use sdl2::render::{Renderer, Texture};

use font::Font;

pub enum Action {
    None,
    Quit,
    Change(Box<State>),
}

pub trait State {
    fn update(&mut self,
              events: EventPollIterator,
              renderer: &mut Renderer,
              font: &Font,
              tiles: &Texture)
              -> Action;
}
