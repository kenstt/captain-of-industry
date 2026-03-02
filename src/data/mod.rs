pub mod loader;

use std::collections::HashMap;

use crate::model::{
    Building, BuildingId, CargoShip, Edict, EdictId, Recipe, RecipeId, Research, ResearchId,
    Resource, ResourceId, Vehicle, VehicleId,
};

/// 遊戲全部靜態資料
///
/// 啟動時從 `data/` 目錄載入，執行期間不可變。
#[derive(Debug, Clone)]
pub struct GameData {
    pub resources: HashMap<ResourceId, Resource>,
    pub buildings: HashMap<BuildingId, Building>,
    pub recipes: HashMap<RecipeId, Recipe>,
    pub vehicles: HashMap<VehicleId, Vehicle>,
    pub research: HashMap<ResearchId, Research>,
    pub edicts: HashMap<EdictId, Edict>,
    pub cargo_ships: Vec<CargoShip>,
}
