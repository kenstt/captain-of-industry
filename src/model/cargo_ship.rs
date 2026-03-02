use serde::{Deserialize, Serialize};

use super::building::MaintenanceCost;
use super::ids::ResourceId;

/// 貨船定義
///
/// 貨船的燃油、船員、維護消耗都需計入資源平衡表。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CargoShip {
    /// 船體模組數 (2/4/6/8)
    pub size: u32,
    /// 使用的燃料類型（柴油/重油/氫氣）
    pub fuel_type: ResourceId,
    /// 每趟正常模式燃油消耗
    pub fuel_per_trip: f64,
    /// 每趟省油模式燃油消耗
    pub fuel_per_trip_save_mode: f64,
    /// 單位/散裝容量（每模組 360）
    pub capacity_unit: u32,
    /// 流體容量（每模組 440）
    pub capacity_fluid: u32,
    /// 所需船員數（12-36 隨 size 變化）
    pub workers: u32,
    /// 正常模式單趟時間（秒）
    pub travel_time_normal: f64,
    /// 省油模式單趟時間（秒）
    pub travel_time_save_fuel: f64,
    pub maintenance: MaintenanceCost,
}
