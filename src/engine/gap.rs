use std::collections::HashSet;

use crate::data::GameData;
use crate::model::ids::ResourceId;
use crate::model::recipe::Recipe;
use crate::model::resource::ResourceCategory;
use crate::model::results::{BalanceSheet, EntryMeta, GapSuggestion};

use super::solver::{Engine, SolverSettings};

/// 為單一赤字資源尋找最佳建築建議
fn find_suggestion_for_deficit(
    resource_id: &ResourceId,
    deficit_per_min: f64,
    engine: &Engine,
    settings: &SolverSettings,
    game_data: &GameData,
) -> Option<GapSuggestion> {
    let resource = game_data.resources.get(resource_id)?;

    // 收集所有能產出此資源的可用配方
    let candidates: Vec<&Recipe> = game_data
        .recipes
        .values()
        .filter(|r| r.outputs.iter().any(|o| o.resource_id == *resource_id))
        .filter(|r| engine.is_recipe_available(r, settings))
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // 對每個候選配方計算分數，選最佳者
    let mut best: Option<(GapSuggestion, f64)> = None;

    for recipe in &candidates {
        let building = match game_data.buildings.get(&recipe.building_id) {
            Some(b) => b,
            None => continue,
        };

        // 計算此配方對目標資源的每分鐘單機產量
        let output_per_cycle = recipe
            .outputs
            .iter()
            .find(|o| o.resource_id == *resource_id)
            .map(|o| o.amount)
            .unwrap_or(0.0);

        if output_per_cycle <= 0.0 {
            continue;
        }

        let cycles_per_min = 60.0 / recipe.duration;
        let single_machine_rate = output_per_cycle * cycles_per_min;
        let machines_needed = deficit_per_min / single_machine_rate;
        let machines_actual = machines_needed.ceil() as u32;

        let workers = building.workers * machines_actual;
        let footprint_area = building.footprint.width * building.footprint.height * machines_actual;

        // 分數：越低越好
        let score = workers as f64 + footprint_area as f64 * 0.5;

        let suggestion = GapSuggestion {
            resource_id: resource_id.clone(),
            resource_name: resource.name.clone(),
            deficit_per_min,
            suggested_building_id: building.id.clone(),
            suggested_building_name: building.name.clone(),
            suggested_recipe_id: recipe.id.clone(),
            suggested_recipe_name: recipe.name.clone(),
            machines_needed,
        };

        if best.as_ref().map_or(true, |(_, s)| score < *s) {
            best = Some((suggestion, score));
        }
    }

    best.map(|(s, _)| s)
}

/// 分析平衡表中的所有赤字，為每個赤字提供建築建議
pub fn analyze_gaps(
    balance: &BalanceSheet,
    engine: &Engine,
    settings: &SolverSettings,
    game_data: &GameData,
) -> Vec<GapSuggestion> {
    let mut suggestions = Vec::new();

    for (resource_id, entry) in &balance.entries {
        let net = entry.net_per_min();
        // 跳過非赤字、原料輸入、污染
        if net >= -f64::EPSILON {
            continue;
        }
        if entry.is_raw_input {
            continue;
        }
        if entry.category == ResourceCategory::Pollution {
            continue;
        }

        let deficit = net.abs();
        if let Some(suggestion) =
            find_suggestion_for_deficit(resource_id, deficit, engine, settings, game_data)
        {
            suggestions.push(suggestion);
        }
    }

    // 按資源名稱排序，確保輸出穩定
    suggestions.sort_by(|a, b| a.resource_name.cmp(&b.resource_name));
    suggestions
}

/// 迭代式缺口解決：反覆分析赤字並加入建議的產出，直到只剩原料需求
pub fn resolve_gaps_iteratively(
    balance: &BalanceSheet,
    engine: &Engine,
    settings: &SolverSettings,
    game_data: &GameData,
) -> (BalanceSheet, Vec<GapSuggestion>) {
    let mut current_balance = balance.clone();
    let mut all_suggestions = Vec::new();
    let mut resolved_resources: HashSet<ResourceId> = HashSet::new();

    const MAX_ITERATIONS: usize = 20;

    for _ in 0..MAX_ITERATIONS {
        let suggestions = analyze_gaps(&current_balance, engine, settings, game_data);

        // 過濾掉已經處理過的資源，避免無限循環
        let new_suggestions: Vec<GapSuggestion> = suggestions
            .into_iter()
            .filter(|s| !resolved_resources.contains(&s.resource_id))
            .collect();

        if new_suggestions.is_empty() {
            break;
        }

        // 將每個建議的產出/消耗加入平衡表
        for suggestion in &new_suggestions {
            resolved_resources.insert(suggestion.resource_id.clone());

            let recipe = match game_data.recipes.get(&suggestion.suggested_recipe_id) {
                Some(r) => r,
                None => continue,
            };

            let cycles_per_min = 60.0 / recipe.duration;

            // 加入所有輸出
            for output in &recipe.outputs {
                let out_resource = game_data.resources.get(&output.resource_id);
                let meta = EntryMeta {
                    name: out_resource
                        .map_or_else(|| output.resource_id.0.clone(), |r| r.name.clone()),
                    name_en: out_resource
                        .map_or_else(|| output.resource_id.0.clone(), |r| r.name_en.clone()),
                    category: out_resource
                        .map_or(ResourceCategory::Intermediate, |r| r.category.clone()),
                };
                current_balance.add_production(
                    &output.resource_id,
                    output.amount * cycles_per_min * suggestion.machines_needed,
                    meta,
                );
            }

            // 加入所有輸入消耗
            for input in &recipe.inputs {
                let in_resource = game_data.resources.get(&input.resource_id);
                let meta = EntryMeta {
                    name: in_resource
                        .map_or_else(|| input.resource_id.0.clone(), |r| r.name.clone()),
                    name_en: in_resource
                        .map_or_else(|| input.resource_id.0.clone(), |r| r.name_en.clone()),
                    category: in_resource
                        .map_or(ResourceCategory::Intermediate, |r| r.category.clone()),
                };
                current_balance.add_consumption(
                    &input.resource_id,
                    input.amount * cycles_per_min * suggestion.machines_needed,
                    meta,
                );
            }

            // 標記原料輸入
            for input in &recipe.inputs {
                if let Some(res) = game_data.resources.get(&input.resource_id) {
                    if res.is_primary {
                        current_balance.mark_raw_input(&input.resource_id);
                    }
                }
            }
        }

        all_suggestions.extend(new_suggestions);
    }

    all_suggestions.sort_by(|a, b| a.resource_name.cmp(&b.resource_name));
    (current_balance, all_suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_game_data;
    use crate::model::ids::ResearchId;
    use std::path::PathBuf;

    fn load_test_data() -> GameData {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
        load_game_data(&data_dir).expect("載入測試資料")
    }

    #[test]
    fn test_analyze_gaps_finds_suggestions() {
        let game_data = load_test_data();
        let engine = Engine::new(game_data.clone());
        let settings = SolverSettings {
            unlocked_research: HashSet::from([ResearchId::new("iron_smelting_scrap")]),
            ..Default::default()
        };

        // 先計算一條生產鏈，取得有赤字的平衡表
        let chain = engine
            .solve_chain(&ResourceId::new("iron_plate"), 12.0, &settings)
            .expect("應成功求解");

        let suggestions = analyze_gaps(&chain.balance_sheet, &engine, &settings, &game_data);

        // 赤字中應排除原料（is_raw_input）和污染，
        // 但可能會有電力等赤字的建議
        // 主要驗證函數不會 panic 且返回合理結果
        for s in &suggestions {
            assert!(s.deficit_per_min > 0.0, "赤字應為正值");
            assert!(s.machines_needed > 0.0, "需要機器數應為正值");
        }
    }

    #[test]
    fn test_analyze_gaps_no_deficit() {
        let game_data = load_test_data();
        let engine = Engine::new(game_data.clone());
        let settings = SolverSettings::default();

        // 空平衡表應無建議
        let balance = BalanceSheet::new();
        let suggestions = analyze_gaps(&balance, &engine, &settings, &game_data);
        assert!(suggestions.is_empty(), "空平衡表不應有建議");
    }
}
