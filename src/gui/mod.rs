pub mod state;
pub mod app;
pub mod components;

use std::path::PathBuf;

use crate::data::loader::load_game_data;
use state::AppState;
use app::App;

/// 啟動 Dioxus 桌面 GUI
pub fn run() {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");

    let game_data = match load_game_data(&data_dir) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("載入遊戲資料失敗: {e}");
            std::process::exit(1);
        }
    };

    let app_state = AppState::new(game_data);

    dioxus::LaunchBuilder::desktop()
        .with_context(app_state)
        .launch(App);
}
