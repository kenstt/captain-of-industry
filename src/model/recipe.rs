use serde::{Deserialize, Serialize};

use super::ids::{BuildingId, RecipeId, ResearchId, ResourceId};

/// 配方中的單項材料（輸入或輸出）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ingredient {
    pub resource_id: ResourceId,
    /// 每次配方週期的數量
    pub amount: f64,
}

/// 遊戲配方定義
///
/// 一個配方屬於一棟建築，但建築與配方的解鎖研究各自獨立。
/// 例：高爐解鎖後只有基礎鐵水配方，銅水配方需額外研究「銅冶煉」。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recipe {
    pub id: RecipeId,
    /// 繁體中文名稱
    pub name: String,
    /// 英文名稱
    pub name_en: String,
    /// 輸入材料
    pub inputs: Vec<Ingredient>,
    /// 輸出產物（含副產物如礦渣、廢氣）
    pub outputs: Vec<Ingredient>,
    /// 每次週期秒數
    pub duration: f64,
    /// 執行此配方的建築
    pub building_id: BuildingId,
    /// 是否為該建築的預設配方
    #[serde(default)]
    pub is_default: bool,
    /// 耗電倍率（預設 1.0）。實際耗電 = building.base_electricity_kw × 此值
    #[serde(default = "default_electricity_multiplier")]
    pub electricity_multiplier: f64,
    /// 解鎖此配方的研究節點（獨立於建築解鎖）
    #[serde(default)]
    pub research_required: Option<ResearchId>,
}

fn default_electricity_multiplier() -> f64 {
    1.0
}
