use std::time::{Duration, Instant};
use iced::{executor, Application, Clipboard, Command, Element, Settings, Rectangle, Length, Subscription};
use iced::canvas::{Cursor, Geometry};
use art_extractor_core::geom_art::{ArtworkSpaceUnit, Point};
use art_extractor_core::movie::Movie;
use art_extractor_core::sprite::Color;
use art_extractor_core::surface::Surface;
use ves_cache::SliceCache;
use ves_geom::SpaceUnit;

pub fn main() -> iced::Result {
    ArtExtractorApp::run(Settings::default())
}

struct ArtExtractorApp {
    movie: Movie,
    current_frame_nr: usize,
}

#[derive(Debug)]
enum AppMessage {
    NextMovieFrame(Instant),
}

const MOVIE_SCALE_FACTOR: u16 = 2;

impl Application for ArtExtractorApp {
    type Executor = executor::Default;
    type Message = AppMessage;
    type Flags = ();

    fn new(_flags: ()) -> (ArtExtractorApp, Command<Self::Message>) {
        let mut input_file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input_file.push("resources/test/movie_10_frames.bincode");
        let file = std::fs::File::open(input_file).unwrap();
        let movie: Movie = bincode::deserialize_from(file).unwrap();

        (ArtExtractorApp { movie, current_frame_nr: 0 }, Command::none())
    }

    fn title(&self) -> String {
        String::from("VES Art Extractor")
    }

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            AppMessage::NextMovieFrame(_) => {
                self.current_frame_nr = (self.current_frame_nr + 1) % self.movie.frames().len()
            },
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let frame_interval = Duration::from_secs(1) / self.movie.frame_rate().fps();
        iced::time::every(frame_interval)
            .map(AppMessage::NextMovieFrame)
    }

    fn view(&mut self) -> Element<Self::Message> {
        let canvas = iced::Canvas::new(CanvasProgram::new(&self.movie, self.current_frame_nr))
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

struct CanvasProgram<'a> {
    movie: &'a Movie,
    current_frame_nr: usize,
}

impl<'a> CanvasProgram<'a> {
    fn new(movie: &'a Movie, current_frame_nr: usize) -> Self {
        Self { movie, current_frame_nr }
    }
}

fn i32_from(value: ArtworkSpaceUnit) -> f32 {
    u16::try_from(value.raw()).unwrap().into()
}

fn iced_point_from(point: Point) -> iced::Point {
    let x = i32_from(point.x);
    let y = i32_from(point.y);
    iced::Point::new(x, y)
}

fn iced_color_from(color: &Color) -> iced::Color {
    match color {
        Color::Opaque(rgb) => iced::Color::from_rgb8(rgb.r, rgb.g, rgb.b),
        Color::Transparent => iced::Color::from_rgba8(0, 0, 0, 0.0),
    }
}

impl iced::canvas::Program<AppMessage> for CanvasProgram<'_> {
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
                        iced_point_from(pos),
                        iced::Size::UNIT,
                        iced_color_from(color),
                    )
                }).unwrap();
        }


        vec![frame.into_geometry()]
    }
}