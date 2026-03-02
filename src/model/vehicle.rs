use serde::{Deserialize, Serialize};

use super::building::MaintenanceCost;
use super::ids::{ResearchId, ResourceId, VehicleId};

/// 遊戲車輛定義
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vehicle {
    pub id: VehicleId,
    pub name: String,
    pub name_en: String,
    /// 使用的燃料類型
    pub fuel_resource: ResourceId,
    /// 每遊戲月（60 秒）燃油消耗
    pub fuel_per_month: f64,
    pub maintenance: MaintenanceCost,
    /// 載貨容量
    pub capacity: u32,
    #[serde(default)]
    pub research_required: Option<ResearchId>,
}
