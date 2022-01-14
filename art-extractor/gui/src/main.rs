pub(crate) mod style;
mod movie;

use iced::{executor, Application, Clipboard, Command, Element, Settings, Subscription};
use crate::AppMessage::Movie;
use crate::movie::{MovieMessage, SpriteMovie};

pub fn main() -> iced::Result {
    ArtExtractorApp::run(Settings::default())
}

struct ArtExtractorApp {
    movie: SpriteMovie,
}

#[derive(Debug, Clone)]
enum AppMessage {
    Movie(MovieMessage),
}

impl From<MovieMessage> for AppMessage {
    fn from(msg: MovieMessage) -> Self {
        AppMessage::Movie(msg)
    }
}

impl Application for ArtExtractorApp {
    type Executor = executor::Default;
    type Message = AppMessage;
    type Flags = ();

    fn new(_flags: ()) -> (ArtExtractorApp, Command<Self::Message>) {
        let mut input_file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        input_file.push("resources/test/movie_10_frames.bincode");
        let file = std::fs::File::open(input_file).unwrap();
        let movie: art_extractor_core::movie::Movie = bincode::deserialize_from(file).unwrap();

        (ArtExtractorApp { movie: SpriteMovie::new(movie) }, Command::none())
    }

    fn title(&self) -> String {
        String::from("VES Art Extractor")
    }

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Movie(msg) => {
                self.movie.update(msg)
                    .map(From::from)
            },
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.movie.subscription()
            .map(From::from)
    }

    fn view(&mut self) -> Element<Self::Message> {
        Element::<MovieMessage>::from(self.movie.view())
            .map(From::from)
    }
}

fn f32_from(value: art_extractor_core::geom_art::ArtworkSpaceUnit) -> f32 {
    use ves_geom::SpaceUnit;
    u16::try_from(value.raw()).unwrap().into()
}

/// Trait for converting types into their "iced" counterparts.
trait ToIced {
    type Out;

    /// Converts the type.
    fn to_iced(&self) -> Self::Out;
}

impl ToIced for art_extractor_core::geom_art::Point {
    type Out = iced::Point;

    fn to_iced(&self) -> Self::Out {
        iced::Point::new(f32_from(self.x), f32_from(self.y))
    }
}

impl ToIced for art_extractor_core::sprite::Color {
    type Out = iced::Color;

    fn to_iced(&self) -> Self::Out {
        match self {
            art_extractor_core::sprite::Color::Opaque(rgb) => iced::Color::from_rgb8(rgb.r, rgb.g, rgb.b),
            art_extractor_core::sprite::Color::Transparent => iced::Color::TRANSPARENT,
        }
    }
}
