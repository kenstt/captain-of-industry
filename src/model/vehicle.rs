use serde::{Deserialize, Serialize};

use super::building::MaintenanceCost;
use super::ids::{ResearchId, ResourceId, VehicleId};

/// 車輛分類
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum VehicleCategory {
    Hauling,
    Excavation,
    Forestry,
    Rocket,
}

/// 遊戲車輛定義
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vehicle {
    pub id: VehicleId,
    pub name: String,
    pub name_en: String,
    pub category: VehicleCategory,
    /// 使用的燃料類型
    pub fuel_resource: ResourceId,
    /// 每遊戲月（60 秒）燃油消耗
    pub fuel_per_month: f64,
    /// 燃料箱容量
    #[serde(default)]
    pub fuel_tank: f64,
    /// 維護消耗（火箭等無維護）
    #[serde(default)]
    pub maintenance: Option<MaintenanceCost>,
    /// 載貨容量
    pub capacity: u32,
    /// 所需工人數
    #[serde(default = "default_workers")]
    pub workers: u32,
    /// 污染量
    #[serde(default)]
    pub pollution: f64,
    #[serde(default)]
    pub research_required: Option<ResearchId>,
}

fn default_workers() -> u32 {
    1
}
