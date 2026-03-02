// BalanceSheet 結構定義在 model/results.rs
// 此模組提供額外的顯示與分析功能

use crate::model::results::BalanceSheet;
use crate::model::resource::ResourceCategory;

/// 平衡表摘要資訊
#[derive(Debug)]
pub struct BalanceSummary {
    pub total_workers: u32,
    pub total_electricity_kw: f64,
    pub deficit_count: usize,
    pub surplus_count: usize,
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
