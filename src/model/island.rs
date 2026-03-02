use super::ids::{BuildingId, RecipeId};

/// 全島帳本中的單一建築條目
#[derive(Clone, Debug)]
pub struct IslandEntry {
    pub id: u32,
    pub building_id: BuildingId,
    pub recipe_id: RecipeId,
    pub count: f64,
}

/// 使用者選擇的食物來源（每個食物分類選一種食物）
#[derive(Clone, Debug)]
pub struct FoodChoice {
    /// 食物分類 key（如 "carbs", "protein"）
    pub category_key: String,
    /// 該分類中選用的食物 resource_id（如 "bread", "meat"）
    pub food_id: String,
    /// 是否啟用此分類
    pub enabled: bool,
}

/// 人口設定
#[derive(Clone, Debug)]
pub struct PopulationSettings {
    pub population: u32,
    pub housing_tier: u32,
    pub food_choices: Vec<FoodChoice>,
}

impl Default for PopulationSettings {
    fn default() -> Self {
        Self {
            population: 0,
            housing_tier: 1,
            food_choices: vec![
                FoodChoice {
                    category_key: "carbs".into(),
                    food_id: "bread".into(),
                    enabled: true,
                },
                FoodChoice {
                    category_key: "protein".into(),
                    food_id: "meat".into(),
                    enabled: true,
                },
                FoodChoice {
                    category_key: "vitamins".into(),
                    food_id: "vegetables".into(),
                    enabled: true,
                },
                FoodChoice {
                    category_key: "treats".into(),
                    food_id: "snack".into(),
                    enabled: false,
                },
            ],
        }
    }
}
