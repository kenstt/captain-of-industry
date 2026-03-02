//! 單配方計算器。
//!
//! 輸入目標產率（每分鐘），輸出所需機器數與投入/產出速率。

use crate::data::models::{CalculationResult, GameData, Ingredient};

/// 計算單一配方在目標產率下的需求。
///
/// - `target_output_per_min`：目標主產物速率（每分鐘）
/// - `primary_output_index`：多產物配方時，指定目標主產物索引
///
/// 回傳值包含：
/// - 精確機器數（未進位）
/// - 對應投入/產出速率（每分鐘）
/// - 進位後機器數導出的電力、人力、算力與維護成本
pub fn calculate_single(
    data: &GameData,
    recipe_id: &str,
    target_output_per_min: f64,
    primary_output_index: usize,
) -> Option<CalculationResult> {
    let recipe = data.recipes.iter().find(|r| r.id == recipe_id)?;
    let primary_output = recipe.outputs.get(primary_output_index)?;
    let machines_map = data.machines_map();
    let machine = machines_map.get(&recipe.machine_id)?;

    let output_per_duration = primary_output.amount;
    let durations_per_min = 60.0 / recipe.duration;
    let single_machine_output_per_min = output_per_duration * durations_per_min;

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

    let machines_ceil = machines_needed.ceil() as f64;
    let total_power = machine.power_consumption * machines_ceil;
    let total_workers = machine.workers as f64 * machines_ceil;
    let total_computing = machine.computing * machines_ceil;
    let total_unity = (recipe.unity_production - recipe.unity_consumption) * machines_needed;
    let maintenance_costs: Vec<Ingredient> = machine
        .maintenance
        .iter()
        .map(|m| Ingredient {
            resource_id: m.resource_id.clone(),
            amount: m.amount * machines_ceil,
        })
        .collect();

    Some(CalculationResult {
        recipe_name: recipe.name.clone(),
        machine_name: machine.name.clone(),
        machines_needed,
        inputs,
        outputs,
        total_power,
        total_workers,
        total_computing,
        total_unity,
        maintenance_costs,
    })
}
