use iced::{executor, Application, Clipboard, Command, Element, Settings, Rectangle, Length};
use iced::canvas::{Cursor, Geometry};
use art_extractor_core::geom_art::{ArtworkSpaceUnit, Point};
use art_extractor_core::movie::Movie;
use art_extractor_core::sprite::Color;
use art_extractor_core::surface::Surface;
use ves_cache::SliceCache;

pub fn main() -> iced::Result {
    ArtExtractorApp::run(Settings::default())
}

struct ArtExtractorApp {
    movie: Movie,
}

impl Application for ArtExtractorApp {
    type Executor = executor::Default;
    type Message = ();
    type Flags = ();

    fn new(_flags: ()) -> (ArtExtractorApp, Command<Self::Message>) {
        let mut input_file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input_file.push("resources/test/movie_10_frames.bincode");
        let file = std::fs::File::open(input_file).unwrap();
        let movie: Movie = bincode::deserialize_from(file).unwrap();

        (ArtExtractorApp { movie }, Command::none())
    }

    fn title(&self) -> String {
        String::from("VES Art Extractor")
    }

    fn update(&mut self, _message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        iced::canvas::Canvas::new(CanvasProgram::new(&self.movie))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct CanvasProgram<'a> {
    movie: &'a Movie,
}

impl<'a> CanvasProgram<'a> {
    fn new(movie: &'a Movie) -> Self {
        Self { movie }
    }
}

fn i32_from(value: ArtworkSpaceUnit) -> f32 {
    use ves_geom::SpaceUnit;
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

impl iced::canvas::Program<()> for CanvasProgram<'_> {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = iced::canvas::Frame::new(bounds.size());
        frame.scale(2.0);

        let palettes = SliceCache::new(self.movie.palettes());
        let tiles = SliceCache::new(self.movie.tiles());
        let movie_frame = &self.movie.frames()[0];
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