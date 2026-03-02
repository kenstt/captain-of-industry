use super::models::GameData;
use std::path::Path;

pub fn load_from_json(path: &Path) -> Result<GameData, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn save_to_json(data: &GameData, path: &Path) -> Result<(), String> {
    let content =
        serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

pub fn load_builtin_data() -> GameData {
    let mut data = GameData::default();

    // Try loading from assets/data/ relative to the executable
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    let search_dirs: Vec<std::path::PathBuf> = vec![
        // Current directory
        std::path::PathBuf::from("assets/data"),
        // Exe directory
        exe_dir
            .clone()
            .map(|d| d.join("assets/data"))
            .unwrap_or_default(),
        // Parent of exe (for cargo run)
        exe_dir
            .and_then(|d| d.parent().map(|p| p.join("assets/data")))
            .unwrap_or_default(),
    ];

    for dir in &search_dirs {
        let recipes_path = dir.join("recipes.json");
        let machines_path = dir.join("machines.json");

        if recipes_path.exists() || machines_path.exists() {
            if let Ok(loaded) = load_from_json_files(&recipes_path, &machines_path) {
                data = data.merge(loaded);
                break;
            }
        }
    }

    data
}

fn load_from_json_files(recipes_path: &Path, machines_path: &Path) -> Result<GameData, String> {
    let mut data = GameData::default();

    if recipes_path.exists() {
        let content = std::fs::read_to_string(recipes_path)
            .map_err(|e| format!("Failed to read recipes: {}", e))?;
        let loaded: GameData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse recipes: {}", e))?;
        data = data.merge(loaded);
    }

    if machines_path.exists() {
        let content = std::fs::read_to_string(machines_path)
            .map_err(|e| format!("Failed to read machines: {}", e))?;
        let loaded: GameData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse machines: {}", e))?;
        data = data.merge(loaded);
    }

    Ok(data)
}
