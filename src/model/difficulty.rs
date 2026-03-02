use serde::{Deserialize, Serialize};

/// 遊戲難度設定
///
/// 各倍率預設為 1.0（標準難度）。
/// 傳送帶/倉儲耗電可獨立開關。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DifficultySettings {
    /// 維護消耗倍率
    #[serde(default = "one")]
    pub maintenance_multiplier: f64,
    /// 燃油消耗倍率
    #[serde(default = "one")]
    pub fuel_multiplier: f64,
    /// 食物消耗倍率
    #[serde(default = "one")]
    pub food_consumption_multiplier: f64,
    /// 商品及服務消耗倍率
    #[serde(default = "one")]
    pub goods_services_multiplier: f64,
    /// 傳送帶是否消耗電力
    #[serde(default = "yes")]
    pub conveyor_power_enabled: bool,
    /// 倉儲是否消耗電力
    #[serde(default = "yes")]
    pub storage_power_enabled: bool,
}

fn one() -> f64 {
    1.0
}
fn yes() -> bool {
    true
}

impl Default for DifficultySettings {
    fn default() -> Self {
        Self {
            maintenance_multiplier: 1.0,
            fuel_multiplier: 1.0,
            food_consumption_multiplier: 1.0,
            goods_services_multiplier: 1.0,
            conveyor_power_enabled: true,
            storage_power_enabled: true,
        }
    }
}
