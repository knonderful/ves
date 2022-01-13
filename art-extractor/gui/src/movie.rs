use std::time::{Duration, Instant};
use iced::{Command, Container, Length, Rectangle, Subscription};
use iced::canvas::{Cursor, Geometry};
use art_extractor_core::surface::Surface;
use ves_cache::SliceCache;
use ves_geom::SpaceUnit;
use crate::ToIced;
use crate::MovieMessage::NextFrame;

const MOVIE_SCALE_FACTOR: u16 = 2;

#[derive(Debug)]
pub enum MovieMessage {
    NextFrame(Instant),
}

pub struct SpriteMovie {
    movie: art_extractor_core::movie::Movie,
    current_frame_nr: usize,
}

impl SpriteMovie {
    pub fn new(movie: art_extractor_core::movie::Movie) -> Self {
        Self {
            movie,
            current_frame_nr: 0,
        }
    }

    pub fn update(&mut self, message: MovieMessage) -> Command<MovieMessage> {
        match message {
            NextFrame(_) => {
                self.current_frame_nr = (self.current_frame_nr + 1) % self.movie.frames().len()
            },
        };

        Command::none()
    }

    pub fn subscription(&self) -> Subscription<MovieMessage> {
        let frame_interval = Duration::from_secs(1) / self.movie.frame_rate().fps();
        iced::time::every(frame_interval)
            .map(MovieMessage::NextFrame)
    }

    pub fn view(&self) -> Container<MovieMessage> {
        let canvas = iced::Canvas::new(MovieScreen::new(&self.movie, self.current_frame_nr))
            .width(Length::Units(u16::try_from(self.movie.screen_size().width.raw()).unwrap() * MOVIE_SCALE_FACTOR))
            .height(Length::Units(u16::try_from(self.movie.screen_size().height.raw()).unwrap() * MOVIE_SCALE_FACTOR));

        iced::Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

struct MovieScreen<'a> {
    movie: &'a art_extractor_core::movie::Movie,
    current_frame_nr: usize,
}

impl<'a> MovieScreen<'a> {
    fn new(movie: &'a art_extractor_core::movie::Movie, current_frame_nr: usize) -> Self {
        Self { movie, current_frame_nr }
    }
}

impl iced::canvas::Program<MovieMessage> for MovieScreen<'_> {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = iced::canvas::Frame::new(bounds.size());
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), iced::Color::from_rgb8(200, 200, 200));
        frame.scale(MOVIE_SCALE_FACTOR.into());

        let palettes = SliceCache::new(self.movie.palettes());
        let tiles = SliceCache::new(self.movie.tiles());
        let movie_frame = &self.movie.frames()[self.current_frame_nr];
        let screen_size = self.movie.screen_size();
        for sprite in movie_frame.sprites().iter().rev() {
            let palette = &palettes[sprite.palette()];
            let tile = &tiles[sprite.tile()];
            let surf = tile.surface();
            let surf_data = surf.data();
            // Using surface_iterate_2 instead of surface_iterate to map the coordinates to our target space easily
            art_extractor_core::surface::surface_iterate_2(
                surf.size(), surf.size().as_rect(),
                screen_size, sprite.position(),
                sprite.h_flip(), sprite.v_flip(),
                |_, idx, pos, _| {
                    let color = &palette[surf_data[idx]];
                    frame.fill_rectangle(
                        pos.to_iced(),
                        iced::Size::UNIT,
                        color.to_iced(),
                    )
                }).unwrap();
        }

        vec![frame.into_geometry()]
    }
}