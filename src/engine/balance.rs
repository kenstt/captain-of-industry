// BalanceSheet 結構定義在 model/results.rs
// 此模組提供額外的顯示與分析功能

use crate::data::GameData;
use crate::model::building::MaintenanceTier;
use crate::model::island::{IslandEntry, PopulationSettings};
use crate::model::results::{BalanceSheet, EntryMeta};
use crate::model::resource::ResourceCategory;
use crate::model::*;

/// 平衡表摘要資訊
#[derive(Debug)]
pub struct BalanceSummary {
    pub total_workers: u32,
    pub total_electricity_kw: f64,
    pub total_maintenance: f64,
    pub deficit_count: usize,
    pub surplus_count: usize,
}

/// 全島平衡表計算結果（包含平衡表 + 每條目的摘要）
#[derive(Debug)]
pub struct IslandBalance {
    pub balance_sheet: BalanceSheet,
    pub total_workers: u32,
    pub total_electricity_kw: f64,
    pub total_maintenance: f64,
}

/// 計算全島資源平衡表
///
/// 遍歷所有 IslandEntry，累加 inputs/outputs/electricity/maintenance/workers，
/// 再加入人口消耗。
pub fn compute_island_balance(
    entries: &[IslandEntry],
    population: &PopulationSettings,
    game_data: &GameData,
    difficulty: &DifficultySettings,
) -> IslandBalance {
    let mut balance = BalanceSheet::new();
    let mut total_workers: u32 = 0;
    let mut total_electricity_kw: f64 = 0.0;
    let mut total_maintenance: f64 = 0.0;

    for entry in entries {
        let recipe = match game_data.recipes.get(&entry.recipe_id) {
            Some(r) => r,
            None => continue,
        };
        let building = match game_data.buildings.get(&entry.building_id) {
            Some(b) => b,
            None => continue,
        };

        let count = entry.count;
        let cycles_per_min = 60.0 / recipe.duration;

        // 輸入
        for input in &recipe.inputs {
            let rate = input.amount * cycles_per_min * count;
            let resource = game_data.resources.get(&input.resource_id);
            balance.add_consumption(
                &input.resource_id,
                rate,
                resource_meta(&input.resource_id, resource),
            );
        }

        // 輸出
        for output in &recipe.outputs {
            let rate = output.amount * cycles_per_min * count;
            let resource = game_data.resources.get(&output.resource_id);
            balance.add_production(
                &output.resource_id,
                rate,
                resource_meta(&output.resource_id, resource),
            );
        }

        // 電力
        let elec = building.base_electricity_kw * recipe.electricity_multiplier * count;
        if elec.abs() > f64::EPSILON {
            total_electricity_kw += elec;
            if elec > 0.0 {
                balance.add_consumption(
                    &ResourceId::new("electricity"),
                    elec,
                    EntryMeta {
                        name: "電力".into(),
                        name_en: "Electricity".into(),
                        category: ResourceCategory::Electricity,
                    },
                );
            } else {
                balance.add_production(
                    &ResourceId::new("electricity"),
                    -elec,
                    EntryMeta {
                        name: "電力".into(),
                        name_en: "Electricity".into(),
                        category: ResourceCategory::Electricity,
                    },
                );
            }
        }

        // 算力
        let computing = building.computing_tflops * count;
        if computing > 0.0 {
            balance.add_consumption(
                &ResourceId::new("computing"),
                computing,
                EntryMeta {
                    name: "算力".into(),
                    name_en: "Computing".into(),
                    category: ResourceCategory::Computing,
                },
            );
        }

        // 凝聚力
        let unity = building.unity_consumption_per_month * count;
        if unity > 0.0 {
            balance.add_consumption(
                &ResourceId::new("unity"),
                unity,
                EntryMeta {
                    name: "凝聚力".into(),
                    name_en: "Unity".into(),
                    category: ResourceCategory::Unity,
                },
            );
        }

        // 維護
        if let Some(ref m) = building.maintenance {
            let amount = m.amount_per_month * count * difficulty.maintenance_multiplier;
            total_maintenance += amount;
            let maint_id = match m.tier {
                MaintenanceTier::Maintenance1 => ResourceId::new("maintenance_1"),
                MaintenanceTier::Maintenance2 => ResourceId::new("maintenance_2"),
                MaintenanceTier::Maintenance3 => ResourceId::new("maintenance_3"),
            };
            let tier_label = match m.tier {
                MaintenanceTier::Maintenance1 => ("維護 I", "Maintenance I"),
                MaintenanceTier::Maintenance2 => ("維護 II", "Maintenance II"),
                MaintenanceTier::Maintenance3 => ("維護 III", "Maintenance III"),
            };
            balance.add_consumption(
                &maint_id,
                amount,
                EntryMeta {
                    name: tier_label.0.into(),
                    name_en: tier_label.1.into(),
                    category: ResourceCategory::Maintenance,
                },
            );
        }

        // 工人（以 ceil 計算）
        let machines_actual = count.ceil() as u32;
        total_workers += building.workers * machines_actual;
    }

    // === 人口消耗 ===
    if population.population > 0 {
        let pop = population.population as f64;
        let pop_data = &game_data.population_data;

        // 食物消耗：每個啟用的食物分類
        let enabled_count = population.food_choices.iter().filter(|c| c.enabled).count();
        for choice in &population.food_choices {
            if !choice.enabled {
                continue;
            }
            if let Some(cat) = pop_data.food_categories.get(&choice.category_key) {
                if let Some(item) = cat.items.get(&choice.food_id) {
                    // wiki 公式: per_100_pop_per_month / (Nc × N)
                    // Nc = 啟用的分類數, N = 該分類中選用的食物數（我們只選一種，N=1）
                    // 換算: per_100_pop_per_month × (pop / 100) / enabled_count / 60s → per_min
                    // 遊戲月 = 60 秒
                    let rate = item.per_100_pop_per_month * (pop / 100.0)
                        / (enabled_count as f64)
                        * difficulty.food_consumption_multiplier
                        / 60.0;
                    balance.add_consumption(
                        &ResourceId::new(&choice.food_id),
                        rate,
                        EntryMeta {
                            name: choice.food_id.clone(),
                            name_en: choice.food_id.clone(),
                            category: ResourceCategory::Food,
                        },
                    );
                }
            }
        }

        // 服務消耗
        let tier_key = population.housing_tier.to_string();
        let tier_mult = pop_data
            .housing_tier_multipliers
            .get(&tier_key)
            .cloned()
            .unwrap_or_default();

        for (service_key, &base_rate) in &pop_data.services_per_1000_pop_per_60s {
            let multiplier = match service_key.as_str() {
                "electricity_kw" => tier_mult.electricity,
                "water" => tier_mult.water,
                "household_goods" => tier_mult.household_goods,
                "household_appliances" => tier_mult.household_appliances,
                "luxury_goods" => tier_mult.luxury_goods,
                _ => 1.0,
            };

            let rate = base_rate * (pop / 1000.0) * multiplier / 60.0;

            let (res_id, name, name_en, category) = match service_key.as_str() {
                "electricity_kw" => (
                    "electricity",
                    "電力（住宅）",
                    "Electricity (Housing)",
                    ResourceCategory::Electricity,
                ),
                "water" => ("water", "水", "Water", ResourceCategory::Service),
                "waste_water" => ("waste_water", "廢水", "Waste Water", ResourceCategory::Waste),
                "household_goods" => (
                    "household_goods",
                    "家用品",
                    "Household Goods",
                    ResourceCategory::Service,
                ),
                "household_appliances" => (
                    "household_appliances",
                    "家電",
                    "Household Appliances",
                    ResourceCategory::Service,
                ),
                "luxury_goods" => (
                    "luxury_goods",
                    "奢侈品",
                    "Luxury Goods",
                    ResourceCategory::Service,
                ),
                "consumer_electronics" => (
                    "consumer_electronics",
                    "消費電子",
                    "Consumer Electronics",
                    ResourceCategory::Service,
                ),
                "medical_supplies" => (
                    "medical_supplies",
                    "醫療用品",
                    "Medical Supplies",
                    ResourceCategory::Service,
                ),
                "computing_tflops" => (
                    "computing",
                    "算力（住宅）",
                    "Computing (Housing)",
                    ResourceCategory::Computing,
                ),
                _ => continue,
            };

            balance.add_consumption(
                &ResourceId::new(res_id),
                rate,
                EntryMeta {
                    name: name.into(),
                    name_en: name_en.into(),
                    category,
                },
            );
        }

        // 廢棄物產出
        for (waste_key, &base_rate) in &pop_data.waste_per_1000_pop_per_60s {
            let rate = base_rate * (pop / 1000.0) / 60.0;
            let (res_id, name, name_en) = match waste_key.as_str() {
                "base_waste" => ("waste", "廢棄物", "Waste"),
                _ => continue,
            };
            balance.add_production(
                &ResourceId::new(res_id),
                rate,
                EntryMeta {
                    name: name.into(),
                    name_en: name_en.into(),
                    category: ResourceCategory::Waste,
                },
            );
        }
    }

    IslandBalance {
        balance_sheet: balance,
        total_workers,
        total_electricity_kw,
        total_maintenance,
    }
}

fn resource_meta(id: &ResourceId, resource: Option<&Resource>) -> EntryMeta {
    EntryMeta {
        name: resource.map_or_else(|| id.0.clone(), |r| r.name.clone()),
        name_en: resource.map_or_else(|| id.0.clone(), |r| r.name_en.clone()),
        category: resource.map_or(ResourceCategory::Intermediate, |r| r.category.clone()),
    }
}

impl BalanceSheet {
    /// 產生摘要
    pub fn summary(&self) -> BalanceSummary {
        let total_electricity_kw = self
            .entries
            .values()
            .filter(|e| e.category == ResourceCategory::Electricity)
            .map(|e| e.net_per_min())
            .sum();

        BalanceSummary {
            total_workers: 0, // 工人數在 ProductionNode 層追蹤
            total_electricity_kw,
            total_maintenance: 0.0,
            deficit_count: self.deficits().len(),
            surplus_count: self.surpluses().len(),
        }
    }

    /// 依類別排序的所有條目（用於顯示）
    pub fn sorted_entries(&self) -> Vec<(&crate::model::ResourceId, &crate::model::BalanceEntry)> {
        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by(|a, b| {
            category_order(&a.1.category)
                .cmp(&category_order(&b.1.category))
                .then_with(|| a.1.resource_name.cmp(&b.1.resource_name))
        });
        entries
    }
}

/// 類別顯示順序
fn category_order(category: &ResourceCategory) -> u8 {
    match category {
        ResourceCategory::RawMaterial => 0,
        ResourceCategory::Intermediate => 1,
        ResourceCategory::FinalProduct => 2,
        ResourceCategory::MoltenMaterial => 3,
        ResourceCategory::Food => 4,
        ResourceCategory::Fuel => 5,
        ResourceCategory::Electricity => 6,
        ResourceCategory::Computing => 7,
        ResourceCategory::Unity => 8,
        ResourceCategory::Maintenance => 9,
        ResourceCategory::Service => 10,
        ResourceCategory::Housing => 11,
        ResourceCategory::Waste => 12,
        ResourceCategory::Pollution => 13,
    }
}
