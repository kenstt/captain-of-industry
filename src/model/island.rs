use super::ids::{BuildingId, RecipeId};
use crate::model::population::PopulationData;

/// 全島帳本中的單一建築條目
#[derive(Clone, Debug)]
pub struct IslandEntry {
    pub id: u32,
    pub building_id: BuildingId,
    pub recipe_id: RecipeId,
    pub count: f64,
}

/// 需求種類
#[derive(Clone, Debug, PartialEq)]
pub enum NeedKind {
    Food,    // 食物分類 (carbs, protein, vitamins, treats)
    Service, // 服務 (water, household_goods, etc.)
    Waste,   // 廢棄物產出 (base_waste, waste_water)
}

/// 人口需求項目（統一表示食物/服務/廢棄物）
#[derive(Clone, Debug)]
pub struct PopulationNeed {
    /// 唯一 key，如 "food:carbs", "service:water", "waste:base_waste"
    pub key: String,
    /// 繁體中文顯示名
    pub name: String,
    /// 需求種類
    pub kind: NeedKind,
    /// 是否啟用
    pub enabled: bool,
    /// 食物類型專用：選用的具體食物 ID（複選）
    pub selected_items: Vec<String>,
}

/// 人口設定
#[derive(Clone, Debug)]
pub struct PopulationSettings {
    pub population: u32,
    pub housing_tier: u32,
    pub needs: Vec<PopulationNeed>,
}

impl PopulationSettings {
    /// 從 PopulationData 建立預設需求列表
    ///
    /// 預設：carbs/protein/vitamins 啟用，treats 關閉，所有服務關閉，所有廢棄物關閉
    pub fn build_default_needs(pop_data: &PopulationData) -> Vec<PopulationNeed> {
        let mut needs = Vec::new();

        // 食物分類（固定順序）
        let food_order = ["carbs", "protein", "vitamins", "treats"];
        let default_foods = [
            ("carbs", "bread", true),
            ("protein", "meat", true),
            ("vitamins", "vegetables", true),
            ("treats", "snack", false),
        ];

        for &(cat_key, default_food, default_enabled) in &default_foods {
            if let Some(cat) = pop_data.food_categories.get(cat_key) {
                let selected = if cat.items.contains_key(default_food) {
                    default_food.to_string()
                } else {
                    cat.items.keys().next().cloned().unwrap_or_default()
                };
                needs.push(PopulationNeed {
                    key: format!("food:{cat_key}"),
                    name: cat.name.clone(),
                    kind: NeedKind::Food,
                    enabled: default_enabled,
                    selected_items: vec![selected],
                });
            }
        }

        // 也處理 food_order 中未列出的分類
        let mut sorted_cats: Vec<_> = pop_data.food_categories.keys().collect();
        sorted_cats.sort();
        for cat_key in sorted_cats {
            if food_order.contains(&cat_key.as_str()) {
                continue;
            }
            let cat = &pop_data.food_categories[cat_key];
            let first_item = cat.items.keys().next().cloned().unwrap_or_default();
            needs.push(PopulationNeed {
                key: format!("food:{cat_key}"),
                name: cat.name.clone(),
                kind: NeedKind::Food,
                enabled: false,
                selected_items: vec![first_item],
            });
        }

        // 服務（固定順序）
        let service_order = [
            ("electricity_kw", "電力（住宅）"),
            ("water", "水"),
            ("household_goods", "家用品"),
            ("household_appliances", "家電"),
            ("luxury_goods", "奢侈品"),
            ("consumer_electronics", "消費電子"),
            ("medical_supplies", "醫療用品"),
            ("computing_tflops", "算力（住宅）"),
        ];

        for &(svc_key, name) in &service_order {
            if svc_key == "waste_water" {
                continue; // waste_water 歸類為廢棄物
            }
            if pop_data.services_per_1000_pop_per_60s.contains_key(svc_key) {
                needs.push(PopulationNeed {
                    key: format!("service:{svc_key}"),
                    name: name.to_string(),
                    kind: NeedKind::Service,
                    enabled: false,
                    selected_items: vec![],
                });
            }
        }

        // 也處理 service_order 中未列出的服務
        let mut sorted_svcs: Vec<_> = pop_data.services_per_1000_pop_per_60s.keys().collect();
        sorted_svcs.sort();
        for svc_key in sorted_svcs {
            if svc_key == "waste_water" {
                continue;
            }
            if service_order.iter().any(|(k, _)| *k == svc_key.as_str()) {
                continue;
            }
            needs.push(PopulationNeed {
                key: format!("service:{svc_key}"),
                name: svc_key.clone(),
                kind: NeedKind::Service,
                enabled: false,
                selected_items: vec![],
            });
        }

        // 廢棄物
        let waste_order = [
            ("base_waste", "廢棄物"),
        ];

        for &(waste_key, name) in &waste_order {
            if pop_data.waste_per_1000_pop_per_60s.contains_key(waste_key) {
                needs.push(PopulationNeed {
                    key: format!("waste:{waste_key}"),
                    name: name.to_string(),
                    kind: NeedKind::Waste,
                    enabled: false,
                    selected_items: vec![],
                });
            }
        }

        // waste_water（從 services 資料來，但歸類為廢棄物）
        if pop_data.services_per_1000_pop_per_60s.contains_key("waste_water") {
            needs.push(PopulationNeed {
                key: "waste:waste_water".to_string(),
                name: "廢水".to_string(),
                kind: NeedKind::Waste,
                enabled: false,
                selected_items: vec![],
            });
        }

        // 也處理 waste_order 中未列出的廢棄物
        let mut sorted_waste: Vec<_> = pop_data.waste_per_1000_pop_per_60s.keys().collect();
        sorted_waste.sort();
        for waste_key in sorted_waste {
            if waste_order.iter().any(|(k, _)| *k == waste_key.as_str()) {
                continue;
            }
            needs.push(PopulationNeed {
                key: format!("waste:{waste_key}"),
                name: waste_key.clone(),
                kind: NeedKind::Waste,
                enabled: false,
                selected_items: vec![],
            });
        }

        needs
    }
}

impl Default for PopulationSettings {
    fn default() -> Self {
        Self {
            population: 0,
            housing_tier: 1,
            needs: Vec::new(), // 延遲初始化：需要 PopulationData 才能建立
        }
    }
}
