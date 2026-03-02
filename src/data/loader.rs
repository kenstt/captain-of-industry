use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::AppError;
use crate::model::*;

use super::GameData;

/// 從指定目錄載入所有遊戲資料 JSON 檔
pub fn load_game_data(data_dir: &Path) -> Result<GameData, AppError> {
    let resources = load_resources(data_dir)?;
    let buildings = load_buildings(data_dir)?;
    let recipes = load_recipes(data_dir)?;
    let vehicles = load_list_as_map::<Vehicle, VehicleId>(
        &data_dir.join("vehicles.json"),
        |v| v.id.clone(),
    )?;
    let research = load_list_as_map::<Research, ResearchId>(
        &data_dir.join("research.json"),
        |r| r.id.clone(),
    )?;
    let edicts = load_list_as_map::<Edict, EdictId>(
        &data_dir.join("edicts.json"),
        |e| e.id.clone(),
    )?;
    let cargo_ships = load_json_file::<Vec<CargoShip>>(&data_dir.join("cargo_ships.json"))?;

    let game_data = GameData {
        resources,
        buildings,
        recipes,
        vehicles,
        research,
        edicts,
        cargo_ships,
    };

    validate(&game_data)?;

    Ok(game_data)
}

/// 載入 resources.json
fn load_resources(data_dir: &Path) -> Result<HashMap<ResourceId, Resource>, AppError> {
    let list: Vec<Resource> = load_json_file(&data_dir.join("resources.json"))?;
    Ok(list.into_iter().map(|r| (r.id.clone(), r)).collect())
}

/// 載入 buildings/ 目錄下所有 JSON 檔
fn load_buildings(data_dir: &Path) -> Result<HashMap<BuildingId, Building>, AppError> {
    load_directory_json(&data_dir.join("buildings"), |b: &Building| b.id.clone())
}

/// 載入 recipes/ 目錄下所有 JSON 檔
fn load_recipes(data_dir: &Path) -> Result<HashMap<RecipeId, Recipe>, AppError> {
    load_directory_json(&data_dir.join("recipes"), |r: &Recipe| r.id.clone())
}

/// 讀取單一 JSON 檔案並反序列化
fn load_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    if !path.exists() {
        // 檔案不存在時回傳空預設值（僅限陣列型別）
        let empty = "[]";
        return Ok(serde_json::from_str(empty)?);
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

/// 讀取一個 JSON 陣列檔案，轉為 HashMap
fn load_list_as_map<T, K>(
    path: &Path,
    key_fn: impl Fn(&T) -> K,
) -> Result<HashMap<K, T>, AppError>
where
    T: serde::de::DeserializeOwned,
    K: std::hash::Hash + Eq,
{
    let list: Vec<T> = load_json_file(path)?;
    Ok(list.into_iter().map(|item| (key_fn(&item), item)).collect())
}

/// 讀取目錄下所有 .json 檔，合併為一個 HashMap
fn load_directory_json<T, K>(
    dir: &Path,
    key_fn: impl Fn(&T) -> K,
) -> Result<HashMap<K, T>, AppError>
where
    T: serde::de::DeserializeOwned,
    K: std::hash::Hash + Eq,
{
    let mut map = HashMap::new();

    if !dir.exists() {
        return Ok(map);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let list: Vec<T> = load_json_file(&path)?;
            for item in list {
                map.insert(key_fn(&item), item);
            }
        }
    }

    Ok(map)
}

/// 交叉驗證所有 ID 引用是否存在
fn validate(data: &GameData) -> Result<(), AppError> {
    // 驗證配方引用的建築和資源
    for recipe in data.recipes.values() {
        if !data.buildings.contains_key(&recipe.building_id) {
            return Err(AppError::ValidationError(format!(
                "配方 '{}' 引用不存在的建築: {}",
                recipe.id, recipe.building_id
            )));
        }
        for input in &recipe.inputs {
            if !data.resources.contains_key(&input.resource_id) {
                return Err(AppError::ValidationError(format!(
                    "配方 '{}' 的輸入引用不存在的資源: {}",
                    recipe.id, input.resource_id
                )));
            }
        }
        for output in &recipe.outputs {
            if !data.resources.contains_key(&output.resource_id) {
                return Err(AppError::ValidationError(format!(
                    "配方 '{}' 的輸出引用不存在的資源: {}",
                    recipe.id, output.resource_id
                )));
            }
        }
    }

    // 驗證建築引用的配方
    for building in data.buildings.values() {
        for recipe_id in &building.available_recipes {
            if !data.recipes.contains_key(recipe_id) {
                return Err(AppError::ValidationError(format!(
                    "建築 '{}' 引用不存在的配方: {}",
                    building.id, recipe_id
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_game_data() {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        let game_data = load_game_data(&data_dir).expect("應成功載入遊戲資料");

        assert!(!game_data.resources.is_empty(), "資源不應為空");
        assert!(!game_data.buildings.is_empty(), "建築不應為空");
        assert!(!game_data.recipes.is_empty(), "配方不應為空");
    }
}
