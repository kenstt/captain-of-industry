use serde::{Deserialize, Serialize};

use super::ids::{EdictId, ResearchId};

/// 政策效果的作用目標
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EdictTarget {
    FoodConsumption,
    VehicleFuel,
    ShipFuel,
    MaintenanceConsumption,
    TruckCapacity,
    TruckMaintenance,
    RecyclingEfficiency,
    FarmYield,
    FarmWater,
    WaterConsumption,
    SolarOutput,
    PopulationGrowth,
    HealthPoints,
    UnityFromGoods,
    HouseholdGoodsConsumption,
    HouseholdAppliancesConsumption,
    ConsumerElectronicsConsumption,
}

/// 效果類型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ModifierType {
    /// 乘法倍率（例：0.85 = 減少 15%）
    Multiply,
    /// 加法數值（例：+10 生命值）
    Add,
}

/// 單項政策效果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdictEffect {
    pub target: EdictTarget,
    pub modifier_type: ModifierType,
    pub value: f64,
}

/// 政策定義
///
/// 政策消耗或產出凝聚力，並對特定數值施加倍率。
/// 多個政策的效果以乘法疊加。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edict {
    pub id: EdictId,
    pub name: String,
    pub name_en: String,
    /// 凝聚力成本（正數=產出, 負數=消耗）
    pub unity_cost_per_month: f64,
    pub effects: Vec<EdictEffect>,
    #[serde(default)]
    pub research_required: Option<ResearchId>,
}
