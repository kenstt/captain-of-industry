use serde::{Deserialize, Serialize};

use super::ids::{BuildingId, RecipeId, ResearchId, ResourceId};

/// 維護等級（I/II/III 不可替代）
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MaintenanceTier {
    Maintenance1,
    Maintenance2,
    Maintenance3,
}

/// 建築的維護消耗
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaintenanceCost {
    pub tier: MaintenanceTier,
    /// 每遊戲月（60 秒）消耗量
    pub amount_per_month: f64,
    /// 閒置時消耗比例（遊戲預設 0.33）
    #[serde(default = "default_idle_fraction")]
    pub idle_fraction: f64,
}

fn default_idle_fraction() -> f64 {
    0.33
}

/// 建築佔地
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Footprint {
    pub width: u32,
    pub height: u32,
}

/// 建築分類
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum BuildingCategory {
    Mining,
    Smelting,
    Manufacturing,
    FoodProduction,
    Power,
    WasteProcessing,
    Storage,
    Housing,
    Services,
    Farming,
    Transport,
    Research,
    Logistics,
    Other,
}

/// 建造成本項目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConstructionCost {
    pub resource_id: ResourceId,
    pub amount: f64,
}

/// 遊戲建築定義
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Building {
    pub id: BuildingId,
    /// 繁體中文名稱
    pub name: String,
    /// 英文名稱
    pub name_en: String,
    pub category: BuildingCategory,
    pub footprint: Footprint,
    pub construction_costs: Vec<ConstructionCost>,
    /// 所需工人數
    pub workers: u32,
    /// 基礎耗電（KW），負值表示發電
    #[serde(default)]
    pub base_electricity_kw: f64,
    /// 算力需求（TFlops），0 表示不需要
    #[serde(default)]
    pub computing_tflops: f64,
    /// 維護消耗（部分建築無需維護）
    #[serde(default)]
    pub maintenance: Option<MaintenanceCost>,
    /// 凝聚力消耗（每月），如研發中心
    #[serde(default)]
    pub unity_consumption_per_month: f64,
    /// 此建築可執行的配方列表
    pub available_recipes: Vec<RecipeId>,
    /// 解鎖此建築的研究節點
    #[serde(default)]
    pub research_required: Option<ResearchId>,
    /// 凝聚力加速生產的倍率（None = 不支援加速）
    #[serde(default)]
    pub unity_boost: Option<f64>,
}
