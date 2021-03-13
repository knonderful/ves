use libretro_backend::{AudioVideoInfo, CoreInfo, GameData, LoadGameResult, RuntimeHandle};

use crate::game::Game;
use std::path::Path;
use crate::gfx::{Rgba8888, Unit2D, Surface};

const SCREEN_WIDTH: Unit2D = 320;
const SCREEN_HEIGHT: Unit2D = 240;

crate::surface!(FrameBuffer, Rgba8888, SCREEN_WIDTH, SCREEN_HEIGHT);

pub struct ProtoCore {
    game: Option<Game>,
    game_data: Option<GameData>,
    frame_buffer: FrameBuffer,
    frame_count: u64,
}

impl ProtoCore {
    pub fn new() -> Self {
        ProtoCore {
            game: Option::None,
            game_data: Option::None,
            frame_buffer: FrameBuffer::default(),
            frame_count: 0,
        }
    }
}

impl Default for ProtoCore {
    fn default() -> Self {
        ProtoCore::new()
    }
}

impl libretro_backend::Core for ProtoCore {
    fn info() -> CoreInfo {
        CoreInfo::new("ves-proto", "0.0.1")
            .supports_roms_with_extension("wasm")
    }

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        if let Some(path) = game_data.path() {
            if let Ok(game) = Game::from_path(Path::new(path)) {
                self.game.replace(game);
                self.game_data.replace(game_data);
                let av_info = AudioVideoInfo::new()
                    .video(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, 60.0, libretro_backend::PixelFormat::ARGB8888);
                return LoadGameResult::Success(av_info);
            }
        }

        LoadGameResult::Failed(game_data)
    }

    fn on_unload_game(&mut self) -> GameData {
        self.game_data.take().unwrap()
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {
        self.frame_count += 1;

        let game = self.game.as_ref().unwrap();
        game.step();
        game.render(&mut self.frame_buffer);

        handle.upload_video_frame(self.frame_buffer.data());

        // No audio for now, but we need to call this
        handle.upload_audio_frame(&[]);
    }

    fn on_reset(&mut self) {
        // TODO: Handle reset.
    }
}
