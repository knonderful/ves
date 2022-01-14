use std::time::{Duration, Instant};
use iced::{Command, Container, Element, Length, PaneGrid, Rectangle, Subscription, Text};
use iced::canvas::{Cursor, Geometry};
use iced::pane_grid::{Axis, TitleBar};
use art_extractor_core::surface::Surface;
use ves_cache::SliceCache;
use ves_geom::SpaceUnit;
use crate::ToIced;
use crate::MovieMessage::NextFrame;
use crate::style::{AppBackgroundStyle, FONT_SIZE, PaneStyle, PaneTitleBarStyle};

const MOVIE_SCALE_FACTOR: u16 = 2;

#[derive(Debug, Clone)]
pub enum MovieMessage {
    NextFrame(Instant),
}

enum PaneType {
    MoviePlayback,
    SomethingElse,
}

impl PaneType {
    fn title(&self) -> &'static str {
        match self {
            PaneType::MoviePlayback => "Movie",
            PaneType::SomethingElse => "Sth Else",
        }
    }

    fn content<'a>(&self, movie: &'a art_extractor_core::movie::Movie, current_frame_nr: usize) -> Element<'a, MovieMessage> {
        match self {
            PaneType::MoviePlayback => {
                iced::Canvas::new(MovieScreen::new(movie, current_frame_nr))
                    .width(Length::Units(u16::try_from(movie.screen_size().width.raw()).unwrap() * MOVIE_SCALE_FACTOR))
                    .height(Length::Units(u16::try_from(movie.screen_size().height.raw()).unwrap() * MOVIE_SCALE_FACTOR))
                    .into()
            }
            PaneType::SomethingElse => {
                crate::style::form_label("Something else")
                    .into()
            }
        }
    }
}

pub struct SpriteMovie {
    movie: art_extractor_core::movie::Movie,
    current_frame_nr: usize,
    pane_grid_state: iced::pane_grid::State<PaneType>,
}

impl SpriteMovie {
    pub fn new(movie: art_extractor_core::movie::Movie) -> Self {
        let (mut pane_grid_state, playback_pane) = iced::pane_grid::State::new(PaneType::MoviePlayback);
        pane_grid_state.split(Axis::Horizontal, &playback_pane, PaneType::SomethingElse);

        Self {
            movie,
            pane_grid_state,
            current_frame_nr: 0,
        }
    }

    pub fn update(&mut self, message: MovieMessage) -> Command<MovieMessage> {
        match message {
            NextFrame(_) => {
                self.current_frame_nr = (self.current_frame_nr + 1) % self.movie.frames().len()
            }
        };

        Command::none()
    }

    pub fn subscription(&self) -> Subscription<MovieMessage> {
        let frame_interval = Duration::from_secs(1) / self.movie.frame_rate().fps();
        iced::time::every(frame_interval)
            .map(MovieMessage::NextFrame)
    }

    pub fn view(&mut self) -> Container<MovieMessage> {
        let pane_grid = PaneGrid::new(&mut self.pane_grid_state, |_pane, state| {
            let content = state.content(&self.movie, self.current_frame_nr);

            gui_pane(String::from(state.title()), content)
        })
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(2);

        iced::Container::new(pane_grid)
            .padding(2)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(AppBackgroundStyle)
            .into()
    }
}

fn gui_pane<'a, M: 'a>(title: impl Into<String>, content: impl Into<Element<'a, M>>) -> iced::pane_grid::Content<'a, M>
{
    iced::pane_grid::Content::new(
        Container::new(content)
            .padding(2)
            // .style(PaneStyle::Focused)
    )
        .title_bar(
            TitleBar::new(
                Container::new(
                    Text::new(title)
                        .size(FONT_SIZE)
                )
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
            )
                .style(PaneTitleBarStyle::Focused)
        )
        .style(PaneStyle::Focused)
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
        // frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), iced::Color::from_rgb8(0x31, 0x33, 0x35));
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