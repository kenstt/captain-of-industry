use std::collections::HashMap;

use super::building::MaintenanceTier;
use super::ids::{BuildingId, RecipeId, ResourceId};
use super::recipe::Ingredient;
use super::resource::ResourceCategory;

/// 生產鏈中的單一生產節點
#[derive(Debug, Clone)]
pub struct ProductionNode {
    pub recipe_id: RecipeId,
    pub building_id: BuildingId,
    pub building_name: String,
    pub recipe_name: String,
    /// 精確機器數（f64，用於計算）
    pub machines_needed: f64,
    /// 實際機器數（ceil，用於建造）
    pub machines_actual: u32,
    /// 每分鐘各輸入速率
    pub inputs_per_min: Vec<Ingredient>,
    /// 每分鐘各輸出速率
    pub outputs_per_min: Vec<Ingredient>,
    /// 此節點的總耗電（KW）
    pub electricity_kw: f64,
    /// 維護消耗（等級 + 每分鐘消耗量）
    pub maintenance: Option<(MaintenanceTier, f64)>,
    /// 算力需求（TFlops）
    pub computing_tflops: f64,
    /// 凝聚力消耗（每分鐘）
    pub unity_per_min: f64,
    /// 此節點所需工人數
    pub workers: u32,
}

/// 完整生產鏈結果
#[derive(Debug)]
pub struct ProductionChain {
    pub target_resource: ResourceId,
    pub target_rate_per_min: f64,
    pub nodes: Vec<ProductionNode>,
    pub balance_sheet: BalanceSheet,
}

/// 資源平衡表中的單筆記錄
#[derive(Debug, Clone)]
pub struct BalanceEntry {
    pub resource_name: String,
    pub resource_name_en: String,
    pub category: ResourceCategory,
    /// 每分鐘產出
    pub produced_per_min: f64,
    /// 每分鐘消耗
    pub consumed_per_min: f64,
    /// 是否為必須外部供給的原料
    pub is_raw_input: bool,
}

impl BalanceEntry {
    /// 淨值：正=盈餘、負=赤字
    pub fn net_per_min(&self) -> f64 {
        self.produced_per_min - self.consumed_per_min
    }
}

/// 資源平衡表 — 追蹤所有資源每分鐘的產出/消耗
#[derive(Debug, Clone, Default)]
pub struct BalanceSheet {
    pub entries: HashMap<ResourceId, BalanceEntry>,
}

impl BalanceSheet {
    pub fn new() -> Self {
        Self::default()
    }

    /// 新增某資源的產出量
    pub fn add_production(&mut self, resource_id: &ResourceId, rate: f64, meta: EntryMeta) {
        let entry = self.get_or_insert(resource_id, meta);
        entry.produced_per_min += rate;
    }

    /// 新增某資源的消耗量
    pub fn add_consumption(&mut self, resource_id: &ResourceId, rate: f64, meta: EntryMeta) {
        let entry = self.get_or_insert(resource_id, meta);
        entry.consumed_per_min += rate;
    }

    /// 標記為原料需求
    pub fn mark_raw_input(&mut self, resource_id: &ResourceId) {
        if let Some(entry) = self.entries.get_mut(resource_id) {
            entry.is_raw_input = true;
        }
    }

    /// 合併另一張平衡表
    pub fn merge(&mut self, other: &BalanceSheet) {
        for (id, other_entry) in &other.entries {
            let entry = self.entries.entry(id.clone()).or_insert_with(|| BalanceEntry {
                resource_name: other_entry.resource_name.clone(),
                resource_name_en: other_entry.resource_name_en.clone(),
                category: other_entry.category.clone(),
                produced_per_min: 0.0,
                consumed_per_min: 0.0,
                is_raw_input: other_entry.is_raw_input,
            });
            entry.produced_per_min += other_entry.produced_per_min;
            entry.consumed_per_min += other_entry.consumed_per_min;
            entry.is_raw_input |= other_entry.is_raw_input;
        }
    }

    /// 取得所有赤字（淨值 < 0）的資源
    pub fn deficits(&self) -> Vec<(&ResourceId, &BalanceEntry)> {
        self.entries
            .iter()
            .filter(|(_, e)| e.net_per_min() < -f64::EPSILON)
            .collect()
    }

    /// 取得所有盈餘（淨值 > 0）的資源
    pub fn surpluses(&self) -> Vec<(&ResourceId, &BalanceEntry)> {
        self.entries
            .iter()
            .filter(|(_, e)| e.net_per_min() > f64::EPSILON)
            .collect()
    }

    fn get_or_insert(&mut self, resource_id: &ResourceId, meta: EntryMeta) -> &mut BalanceEntry {
        self.entries
            .entry(resource_id.clone())
            .or_insert_with(|| BalanceEntry {
                resource_name: meta.name,
                resource_name_en: meta.name_en,
                category: meta.category,
                produced_per_min: 0.0,
                consumed_per_min: 0.0,
                is_raw_input: false,
            })
    }
}

/// 建立 BalanceEntry 時所需的資源元資料
pub struct EntryMeta {
    pub name: String,
    pub name_en: String,
    pub category: ResourceCategory,
}

/// 缺口分析建議
#[derive(Debug)]
pub struct GapSuggestion {
    pub resource_id: ResourceId,
    pub resource_name: String,
    /// 每分鐘赤字量
    pub deficit_per_min: f64,
    pub suggested_building_id: BuildingId,
    pub suggested_building_name: String,
    pub suggested_recipe_id: RecipeId,
    pub suggested_recipe_name: String,
    /// 建議機器數
    pub machines_needed: f64,
}
