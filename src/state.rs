// SPDX-FileCopyrightText: 2019 Tuomas Siipola
// SPDX-License-Identifier: GPL-3.0-or-later

use sdl2::event::EventPollIterator;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::font::Font;

pub enum Action {
    None,
    Quit,
    Change(Box<dyn State>),
}

pub trait State {
    fn update(
        &mut self,
        events: EventPollIterator,
        canvas: &mut Canvas<Window>,
        font: &mut Font,
        tiles: &Texture,
        logo: &Texture,
    ) -> Action;
}
