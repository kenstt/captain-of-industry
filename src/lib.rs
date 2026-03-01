use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resource {
    pub id: ResourceId,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ingredient {
    pub resource_id: ResourceId,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub inputs: Vec<Ingredient>,
    pub outputs: Vec<Ingredient>,
    pub duration: f64, // seconds
    pub machine_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Machine {
    pub id: String,
    pub name: String,
}

pub struct Calculator {
    pub recipes: HashMap<String, Recipe>,
    pub machines: HashMap<String, Machine>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            recipes: HashMap::new(),
            machines: HashMap::new(),
        }
    }

    pub fn add_recipe(&mut self, recipe: Recipe) {
        self.recipes.insert(recipe.id.clone(), recipe);
    }

    pub fn add_machine(&mut self, machine: Machine) {
        self.machines.insert(machine.id.clone(), machine);
    }

    /// Calculate requirements for a target output rate (units per minute)
    pub fn calculate_requirements(&self, recipe_id: &str, target_output_per_min: f64) -> Option<CalculationResult> {
        let recipe = self.recipes.get(recipe_id)?;

        // Find the primary output (assume first for now, or find the one that matches if needed)
        // In Captain of Industry, a recipe can have multiple outputs.
        // We usually calculate based on one desired output.
        let primary_output = recipe.outputs.first()?;
        let output_per_duration = primary_output.amount;
        let durations_per_min = 60.0 / recipe.duration;
        let single_machine_output_per_min = output_per_duration * durations_per_min;

        let machines_needed = target_output_per_min / single_machine_output_per_min;

        let mut inputs = Vec::new();
        for input in &recipe.inputs {
            let rate_per_min = (input.amount * durations_per_min) * machines_needed;
            inputs.push(Ingredient {
                resource_id: input.resource_id.clone(),
                amount: rate_per_min,
            });
        }

        let mut outputs = Vec::new();
        for output in &recipe.outputs {
            let rate_per_min = (output.amount * durations_per_min) * machines_needed;
            outputs.push(Ingredient {
                resource_id: output.resource_id.clone(),
                amount: rate_per_min,
            });
        }

        Some(CalculationResult {
            recipe_name: recipe.name.clone(),
            machine_name: self.machines.get(&recipe.machine_id)?.name.clone(),
            machines_needed,
            inputs,
            outputs,
        })
    }
}

#[derive(Debug)]
pub struct CalculationResult {
    pub recipe_name: String,
    pub machine_name: String,
    pub machines_needed: f64,
    pub inputs: Vec<Ingredient>,
    pub outputs: Vec<Ingredient>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculation() {
        let mut calc = Calculator::new();
        calc.add_machine(Machine {
            id: "furnace".to_string(),
            name: "Furnace".to_string(),
        });
        calc.add_recipe(Recipe {
            id: "copper".to_string(),
            name: "Copper Plate".to_string(),
            inputs: vec![Ingredient {
                resource_id: ResourceId("copper_ore".to_string()),
                amount: 10.0,
            }],
            outputs: vec![Ingredient {
                resource_id: ResourceId("copper_plate".to_string()),
                amount: 10.0,
            }],
            duration: 10.0, // 每 10 秒 10 個，即每分鐘 60 個
            machine_id: "furnace".to_string(),
        });

        // 目標每分鐘 120 個產出，需要 2 台機器
        let result = calc.calculate_requirements("copper", 120.0).unwrap();
        assert_eq!(result.machines_needed, 2.0);
        assert_eq!(result.inputs[0].amount, 120.0);
        assert_eq!(result.outputs[0].amount, 120.0);
    }

    #[test]
    fn test_multiple_outputs() {
        let mut calc = Calculator::new();
        calc.add_machine(Machine {
            id: "cracking".to_string(),
            name: "Cracking".to_string(),
        });
        // 假設配方：10 單位原油 -> 6 單位汽油 + 4 單位渣油，耗時 10 秒
        calc.add_recipe(Recipe {
            id: "cracking".to_string(),
            name: "Oil Cracking".to_string(),
            inputs: vec![Ingredient {
                resource_id: ResourceId("crude_oil".to_string()),
                amount: 10.0,
            }],
            outputs: vec![
                Ingredient {
                    resource_id: ResourceId("gasoline".to_string()),
                    amount: 6.0,
                },
                Ingredient {
                    resource_id: ResourceId("slag".to_string()),
                    amount: 4.0,
                },
            ],
            duration: 10.0,
            machine_id: "cracking".to_string(),
        });

        // 每分鐘產出 36 單位汽油 (6/10秒 = 36/分鐘)
        // 36 汽油 / 36 單機 = 1 台機器
        let result = calc.calculate_requirements("cracking", 36.0).unwrap();
        assert_eq!(result.machines_needed, 1.0);
        assert_eq!(result.inputs[0].amount, 60.0);
        assert_eq!(result.outputs[0].amount, 36.0); // 汽油
        assert_eq!(result.outputs[1].amount, 24.0); // 渣油
    }
}
