use crate::data::models::{CalculationResult, GameData, Ingredient};

/// Calculate requirements for a single recipe given a target output rate (per minute).
/// `primary_output_index` specifies which output to target (for multi-output recipes).
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
        maintenance_costs,
    })
}
