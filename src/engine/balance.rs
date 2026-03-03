// BalanceSheet 結構定義在 model/results.rs
// 此模組提供額外的顯示與分析功能

use crate::data::GameData;
use crate::model::building::MaintenanceTier;
use crate::model::island::{IslandEntry, NeedKind, PopulationSettings};
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
        let workers = building.workers * machines_actual;
        total_workers += workers;
        if workers > 0 {
            balance.add_consumption(
                &ResourceId::new("workers"),
                workers as f64,
                EntryMeta {
                    name: "工人（建築）".into(),
                    name_en: "Workers (Buildings)".into(),
                    category: ResourceCategory::Service,
                },
            );
        }
    }

    // === 人口消耗 ===
    if population.population > 0 {
        let pop = population.population as f64;
        let pop_data = &game_data.population_data;

        // 食物消耗：只計算啟用的食物需求
        let enabled_food_count = population
            .needs
            .iter()
            .filter(|n| n.kind == NeedKind::Food && n.enabled)
            .count();

        for need in &population.needs {
            if !need.enabled {
                continue;
            }

            match need.kind {
                NeedKind::Food => {
                    let cat_key = need.key.strip_prefix("food:").unwrap_or(&need.key);
                    if need.selected_items.is_empty() {
                        continue;
                    }
                    if let Some(cat) = pop_data.food_categories.get(cat_key) {
                        let item_count = need.selected_items.len() as f64;
                        for food_id in &need.selected_items {
                            if let Some(item) = cat.items.get(food_id.as_str()) {
                                let rate = item.per_100_pop_per_month * (pop / 100.0)
                                    / (enabled_food_count as f64)
                                    * difficulty.food_consumption_multiplier
                                    / 60.0
                                    / item_count;
                                let food_name = item.name.as_deref().unwrap_or(food_id);
                                let food_name_en = item.name_en.as_deref().unwrap_or(food_id);
                                balance.add_consumption(
                                    &ResourceId::new(food_id),
                                    rate,
                                    EntryMeta {
                                        name: food_name.to_string(),
                                        name_en: food_name_en.to_string(),
                                        category: ResourceCategory::Food,
                                    },
                                );
                            }
                        }
                    }
                }
                NeedKind::Service => {
                    let service_key = need.key.strip_prefix("service:").unwrap_or(&need.key);
                    let base_rate = match pop_data.services_per_1000_pop_per_60s.get(service_key) {
                        Some(&r) => r,
                        None => continue,
                    };

                    let tier_key = population.housing_tier.to_string();
                    let tier_mult = pop_data
                        .housing_tier_multipliers
                        .get(&tier_key)
                        .cloned()
                        .unwrap_or_default();

                    let tier_multiplier = match service_key {
                        "electricity_kw" => tier_mult.electricity,
                        "water" => tier_mult.water,
                        "household_goods" => tier_mult.household_goods,
                        "household_appliances" => tier_mult.household_appliances,
                        "luxury_goods" => tier_mult.luxury_goods,
                        _ => 1.0,
                    };

                    let difficulty_mult = match service_key {
                        "household_goods" | "household_appliances" | "luxury_goods" | "consumer_electronics"
                            => difficulty.goods_services_multiplier,
                        _ => 1.0,
                    };

                    let rate = base_rate * (pop / 1000.0) * tier_multiplier * difficulty_mult / 60.0;

                    let (res_id, name, name_en, category) = match service_key {
                        "electricity_kw" => (
                            "electricity",
                            "電力（住宅）",
                            "Electricity (Housing)",
                            ResourceCategory::Electricity,
                        ),
                        "water" => ("water", "水", "Water", ResourceCategory::Service),
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
                NeedKind::Waste => {
                    let waste_key = need.key.strip_prefix("waste:").unwrap_or(&need.key);
                    match waste_key {
                        "base_waste" => {
                            if let Some(&base_rate) = pop_data.waste_per_1000_pop_per_60s.get("base_waste") {
                                let rate = base_rate * (pop / 1000.0) / 60.0;
                                balance.add_production(
                                    &ResourceId::new("waste"),
                                    rate,
                                    EntryMeta {
                                        name: "廢棄物".into(),
                                        name_en: "Waste".into(),
                                        category: ResourceCategory::Waste,
                                    },
                                );
                            }
                        }
                        "waste_water" => {
                            if let Some(&ww_rate) = pop_data.services_per_1000_pop_per_60s.get("waste_water") {
                                let rate = ww_rate * (pop / 1000.0) / 60.0;
                                balance.add_production(
                                    &ResourceId::new("waste_water"),
                                    rate,
                                    EntryMeta {
                                        name: "廢水".into(),
                                        name_en: "Waste Water".into(),
                                        category: ResourceCategory::Waste,
                                    },
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // 凝聚力產出：每個啟用的食物分類貢獻 Unity
        if enabled_food_count > 0 {
            let tier_key = population.housing_tier.to_string();
            let tier_mult = pop_data
                .housing_tier_multipliers
                .get(&tier_key)
                .cloned()
                .unwrap_or_default();
            let unity_rate = (enabled_food_count as f64) * tier_mult.unity_bonus * (pop / 1000.0) / 60.0;
            balance.add_production(
                &ResourceId::new("unity"),
                unity_rate,
                EntryMeta {
                    name: "凝聚力（人口）".into(),
                    name_en: "Unity (Population)".into(),
                    category: ResourceCategory::Unity,
                },
            );
        }

        // 工人供給（人口）
        balance.add_production(
            &ResourceId::new("workers"),
            pop,
            EntryMeta {
                name: "工人（人口）".into(),
                name_en: "Workers (Population)".into(),
                category: ResourceCategory::Service,
            },
        );
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
