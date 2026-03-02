//! 產線鏈計算器。
//!
//! 由目標配方往上游遞迴展開，直到遇到：
//! - 使用者標記為已供應的資源
//! - 找不到上游配方的原料
//! - 偵測到循環依賴

use std::collections::HashSet;

use crate::data::models::*;

/// 計算完整產線鏈（遞迴追溯上游配方）。
///
/// `supplied` 代表使用者已外部供應的資源，遇到時不再向上展開。
pub fn calculate_chain(
    data: &GameData,
    recipe_id: &str,
    target_output_per_min: f64,
    primary_output_index: usize,
    supplied: &HashSet<ResourceId>,
) -> Option<ChainNode> {
    let mut visited = HashSet::new();
    build_chain_node(
        data,
        recipe_id,
        target_output_per_min,
        primary_output_index,
        supplied,
        &mut visited,
    )
}

fn build_chain_node(
    data: &GameData,
    recipe_id: &str,
    target_output_per_min: f64,
    primary_output_index: usize,
    supplied: &HashSet<ResourceId>,
    visited: &mut HashSet<String>,
) -> Option<ChainNode> {
    let recipe = data.recipes.iter().find(|r| r.id == recipe_id)?;
    let machines_map = data.machines_map();
    let machine = machines_map.get(&recipe.machine_id)?;

    let primary_output = recipe.outputs.get(primary_output_index)?;
    let durations_per_min = 60.0 / recipe.duration;
    let single_machine_output_per_min = primary_output.amount * durations_per_min;
    let machines_needed = target_output_per_min / single_machine_output_per_min;

    let inputs: Vec<Ingredient> = recipe
        .inputs
        .iter()
        .map(|input| Ingredient {
            resource_id: input.resource_id.clone(),
            amount: input.amount * durations_per_min * machines_needed,
        })
        .collect();

    let outputs: Vec<Ingredient> = recipe
        .outputs
        .iter()
        .map(|output| Ingredient {
            resource_id: output.resource_id.clone(),
            amount: output.amount * durations_per_min * machines_needed,
        })
        .collect();

    // 記錄訪問路徑，用於循環偵測。
    visited.insert(recipe_id.to_string());

    let mut children = Vec::new();
    for input in &inputs {
        let required_rate = input.amount;
        let source = if supplied.contains(&input.resource_id) {
            ChainSource::Supplied
        } else if let Some((upstream_recipe, output_idx)) =
            find_recipe_for_resource(data, &input.resource_id)
        {
            if visited.contains(&upstream_recipe.id) {
                ChainSource::CycleDetected
            } else {
                match build_chain_node(
                    data,
                    &upstream_recipe.id,
                    required_rate,
                    output_idx,
                    supplied,
                    visited,
                ) {
                    Some(node) => ChainSource::Recipe(node),
                    None => ChainSource::RawMaterial,
                }
            }
        } else {
            ChainSource::RawMaterial
        };

        children.push(ChainChild {
            resource_id: input.resource_id.clone(),
            required_rate,
            source,
        });
    }

    visited.remove(recipe_id);

    let machines_ceil = machines_needed.ceil() as f64;
    let power = machine.power_consumption * machines_ceil;
    let workers = machine.workers as f64 * machines_ceil;
    let computing = machine.computing * machines_ceil;
    let maintenance_costs: Vec<Ingredient> = machine
        .maintenance
        .iter()
        .map(|m| Ingredient {
            resource_id: m.resource_id.clone(),
            amount: m.amount * machines_ceil,
        })
        .collect();

    Some(ChainNode {
        recipe_id: recipe_id.to_string(),
        recipe_name: recipe.name.clone(),
        machine_id: recipe.machine_id.clone(),
        machine_name: machine.name.clone(),
        machines_needed,
        inputs,
        outputs,
        children,
        power,
        workers,
        computing,
        maintenance_costs,
    })
}

/// 尋找可產生指定資源的配方，回傳 `(配方, 該資源在 outputs 的索引)`。
fn find_recipe_for_resource<'a>(
    data: &'a GameData,
    resource_id: &ResourceId,
) -> Option<(&'a Recipe, usize)> {
    for recipe in &data.recipes {
        for (i, output) in recipe.outputs.iter().enumerate() {
            if output.resource_id == *resource_id {
                return Some((recipe, i));
            }
        }
    }
    None
}
