use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ResourceCategory {
    Mineral,
    Liquid,
    Gaseous,
    Food,
    Intermediate,
    Product,
    Waste,
    Crops,
    Other,
}

impl Default for ResourceCategory {
    fn default() -> Self {
        ResourceCategory::Other
    }
}

impl ResourceCategory {
    pub fn all() -> &'static [ResourceCategory] {
        &[
            ResourceCategory::Mineral,
            ResourceCategory::Liquid,
            ResourceCategory::Gaseous,
            ResourceCategory::Food,
            ResourceCategory::Intermediate,
            ResourceCategory::Product,
            ResourceCategory::Waste,
            ResourceCategory::Crops,
            ResourceCategory::Other,
        ]
    }

    pub fn i18n_key(&self) -> &'static str {
        match self {
            ResourceCategory::Mineral => "cat_mineral",
            ResourceCategory::Liquid => "cat_liquid",
            ResourceCategory::Gaseous => "cat_gaseous",
            ResourceCategory::Food => "cat_food",
            ResourceCategory::Intermediate => "cat_intermediate",
            ResourceCategory::Product => "cat_product",
            ResourceCategory::Waste => "cat_waste",
            ResourceCategory::Crops => "cat_crops",
            ResourceCategory::Other => "cat_other",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resource {
    pub id: ResourceId,
    pub name: String,
    #[serde(default)]
    pub name_zh: Option<String>,
    #[serde(default)]
    pub category: ResourceCategory,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ingredient {
    pub resource_id: ResourceId,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub name_zh: Option<String>,
    pub inputs: Vec<Ingredient>,
    pub outputs: Vec<Ingredient>,
    pub duration: f64,
    pub machine_id: String,
    #[serde(default)]
    pub tier: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_output_multiplier")]
    pub output_multiplier: f64,
}

fn default_output_multiplier() -> f64 {
    1.0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaintenanceItem {
    pub resource_id: ResourceId,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Machine {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub name_zh: Option<String>,
    #[serde(default)]
    pub power_consumption: f64,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub workers: u32,
    #[serde(default)]
    pub maintenance: Vec<MaintenanceItem>,
    #[serde(default)]
    pub computing: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GameData {
    #[serde(default)]
    pub resources: Vec<Resource>,
    #[serde(default)]
    pub recipes: Vec<Recipe>,
    #[serde(default)]
    pub machines: Vec<Machine>,
}

impl GameData {
    pub fn recipes_map(&self) -> HashMap<String, &Recipe> {
        self.recipes.iter().map(|r| (r.id.clone(), r)).collect()
    }

    pub fn machines_map(&self) -> HashMap<String, &Machine> {
        self.machines.iter().map(|m| (m.id.clone(), m)).collect()
    }

    pub fn resources_map(&self) -> HashMap<ResourceId, &Resource> {
        self.resources.iter().map(|r| (r.id.clone(), r)).collect()
    }

    pub fn merge(mut self, other: GameData) -> GameData {
        // Merge by adding non-duplicate entries from other
        let existing_recipe_ids: std::collections::HashSet<_> =
            self.recipes.iter().map(|r| r.id.clone()).collect();
        let existing_machine_ids: std::collections::HashSet<_> =
            self.machines.iter().map(|m| m.id.clone()).collect();
        let existing_resource_ids: std::collections::HashSet<_> =
            self.resources.iter().map(|r| r.id.clone()).collect();

        for r in other.recipes {
            if !existing_recipe_ids.contains(&r.id) {
                self.recipes.push(r);
            }
        }
        for m in other.machines {
            if !existing_machine_ids.contains(&m.id) {
                self.machines.push(m);
            }
        }
        for r in other.resources {
            if !existing_resource_ids.contains(&r.id) {
                self.resources.push(r);
            }
        }
        self
    }
}

/// Result of a single recipe calculation
#[derive(Debug, Clone)]
pub struct CalculationResult {
    pub recipe_name: String,
    pub machine_name: String,
    pub machines_needed: f64,
    pub inputs: Vec<Ingredient>,
    pub outputs: Vec<Ingredient>,
    pub total_power: f64,
    pub total_workers: f64,
    pub total_computing: f64,
    pub maintenance_costs: Vec<Ingredient>,
}

/// A node in the production chain tree
#[derive(Debug, Clone)]
pub struct ChainNode {
    pub recipe_id: String,
    pub recipe_name: String,
    pub machine_name: String,
    pub machines_needed: f64,
    pub inputs: Vec<Ingredient>,
    pub outputs: Vec<Ingredient>,
    pub children: Vec<ChainChild>,
    pub power: f64,
    pub workers: f64,
    pub computing: f64,
    pub maintenance_costs: Vec<Ingredient>,
}

#[derive(Debug, Clone)]
pub struct ChainChild {
    pub resource_id: ResourceId,
    pub required_rate: f64,
    pub source: ChainSource,
}

#[derive(Debug, Clone)]
pub enum ChainSource {
    Recipe(ChainNode),
    RawMaterial,
    Supplied,
    CycleDetected,
}

/// Resource balance report
#[derive(Debug, Clone)]
pub struct BalanceReport {
    pub resource_balances: Vec<ResourceBalance>,
    pub machine_totals: Vec<MachineTally>,
    pub total_power: f64,
    pub total_workers: f64,
    pub total_computing: f64,
    pub total_maintenance: Vec<Ingredient>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalFlows {
    #[serde(default)]
    pub supplies_per_min: Vec<Ingredient>,
    #[serde(default)]
    pub consumptions_per_min: Vec<Ingredient>,
}

#[derive(Debug, Clone)]
pub struct ResourceBalance {
    pub resource_id: ResourceId,
    pub resource_name: String,
    pub production_rate: f64,
    pub consumption_rate: f64,
    pub net_rate: f64,
    pub status: BalanceStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BalanceStatus {
    Surplus,
    Deficit,
    Balanced,
    Bottleneck,
}

#[derive(Debug, Clone)]
pub struct MachineTally {
    pub machine_id: String,
    pub machine_name: String,
    pub count: f64,
    pub count_ceil: u32,
    pub total_power: f64,
    pub total_workers: u32,
    pub total_computing: f64,
    pub maintenance_costs: Vec<Ingredient>,
}
