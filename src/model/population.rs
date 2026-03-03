use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 單項食物消耗率
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FoodItem {
    /// 每 100 人每遊戲月消耗量
    pub per_100_pop_per_month: f64,
    /// 繁體中文名稱
    #[serde(default)]
    pub name: Option<String>,
    /// 英文名稱
    #[serde(default)]
    pub name_en: Option<String>,
}

/// 食物分類（碳水/蛋白質/維生素/零食）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FoodCategory {
    pub name: String,
    pub name_en: String,
    pub items: HashMap<String, FoodItem>,
}

/// 住宅等級倍率
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HousingTierMultiplier {
    #[serde(default = "one")]
    pub electricity: f64,
    #[serde(default = "one")]
    pub water: f64,
    #[serde(default = "one")]
    pub household_goods: f64,
    #[serde(default = "one")]
    pub household_appliances: f64,
    #[serde(default = "one")]
    pub luxury_goods: f64,
    #[serde(default = "one")]
    pub unity_bonus: f64,
}

fn one() -> f64 {
    1.0
}

/// 人口消耗靜態資料（從 population.json 載入）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PopulationData {
    #[serde(default)]
    pub comment: String,
    pub food_categories: HashMap<String, FoodCategory>,
    pub services_per_1000_pop_per_60s: HashMap<String, f64>,
    pub waste_per_1000_pop_per_60s: HashMap<String, f64>,
    pub housing_tier_multipliers: HashMap<String, HousingTierMultiplier>,
}
