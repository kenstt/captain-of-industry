use std::path::PathBuf;

use captain_of_industry::data::loader::load_game_data;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cli_mode = args.iter().any(|a| a == "--cli");

    if cli_mode {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");

        let game_data = match load_game_data(&data_dir) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("載入遊戲資料失敗: {e}");
                std::process::exit(1);
            }
        };

        captain_of_industry::cli::run(game_data);
    } else {
        captain_of_industry::gui::run();
    }
}
