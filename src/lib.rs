pub mod data;
pub mod calculator;

// Re-export core types for backward compatibility
pub use data::models::{
    CalculationResult, GameData, Ingredient, Machine, Recipe, Resource, ResourceId,
};

use std::collections::HashMap;

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
    pub fn calculate_requirements(
        &self,
        recipe_id: &str,
        target_output_per_min: f64,
    ) -> Option<CalculationResult> {
        let recipe = self.recipes.get(recipe_id)?;

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

        let machine = self.machines.get(&recipe.machine_id)?;
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
            total_unity: (recipe.unity_production - recipe.unity_consumption) * machines_needed,
            maintenance_costs,
        })
    }
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
            name_zh: None,
            power_consumption: 0.0,
            category: String::new(),
            workers: 0,
            maintenance: vec![],
            computing: 0.0,
        });
        calc.add_recipe(Recipe {
            id: "copper".to_string(),
            name: "Copper Plate".to_string(),
            name_zh: None,
            inputs: vec![Ingredient {
                resource_id: ResourceId("copper_ore".to_string()),
                amount: 10.0,
            }],
            outputs: vec![Ingredient {
                resource_id: ResourceId("copper_plate".to_string()),
                amount: 10.0,
            }],
            duration: 10.0,
            machine_id: "furnace".to_string(),
            tier: 0,
            tags: vec![],
            output_multiplier: 1.0,
            unity_consumption: 0.0,
            unity_production: 0.0,
        });

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
            name_zh: None,
            power_consumption: 0.0,
            category: String::new(),
            workers: 0,
            maintenance: vec![],
            computing: 0.0,
        });
        calc.add_recipe(Recipe {
            id: "cracking".to_string(),
            name: "Oil Cracking".to_string(),
            name_zh: None,
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
            tier: 0,
            tags: vec![],
            output_multiplier: 1.0,
            unity_consumption: 0.0,
            unity_production: 0.0,
        });

        let result = calc.calculate_requirements("cracking", 36.0).unwrap();
        assert_eq!(result.machines_needed, 1.0);
        assert_eq!(result.inputs[0].amount, 60.0);
        assert_eq!(result.outputs[0].amount, 36.0);
        assert_eq!(result.outputs[1].amount, 24.0);
    }

    #[test]
    fn test_chain_calculation() {
        use std::collections::HashSet;

        let data = GameData {
            resources: vec![],
            machines: vec![
                Machine {
                    id: "furnace".to_string(),
                    name: "Furnace".to_string(),
                    name_zh: None,
                    power_consumption: 50.0,
                    category: String::new(),
                    workers: 0,
                    maintenance: vec![],
                    computing: 0.0,
                },
                Machine {
                    id: "smelter".to_string(),
                    name: "Smelter".to_string(),
                    name_zh: None,
                    power_consumption: 100.0,
                    category: String::new(),
                    workers: 0,
                    maintenance: vec![],
                    computing: 0.0,
                },
            ],
            recipes: vec![
                Recipe {
                    id: "iron_plate".to_string(),
                    name: "Iron Plate".to_string(),
                    name_zh: None,
                    inputs: vec![Ingredient {
                        resource_id: ResourceId("molten_iron".to_string()),
                        amount: 10.0,
                    }],
                    outputs: vec![Ingredient {
                        resource_id: ResourceId("iron_plate".to_string()),
                        amount: 10.0,
                    }],
                    duration: 10.0,
                    machine_id: "smelter".to_string(),
                    tier: 0,
                    tags: vec![],
                    output_multiplier: 1.0,
                    unity_consumption: 0.0,
                    unity_production: 0.0,
                },
                Recipe {
                    id: "molten_iron".to_string(),
                    name: "Molten Iron".to_string(),
                    name_zh: None,
                    inputs: vec![Ingredient {
                        resource_id: ResourceId("iron_ore".to_string()),
                        amount: 12.0,
                    }],
                    outputs: vec![Ingredient {
                        resource_id: ResourceId("molten_iron".to_string()),
                        amount: 12.0,
                    }],
                    duration: 20.0,
                    machine_id: "furnace".to_string(),
                    tier: 0,
                    tags: vec![],
                    output_multiplier: 1.0,
                    unity_consumption: 0.0,
                    unity_production: 0.0,
                },
            ],
        };

        let chain = calculator::chain::calculate_chain(
            &data,
            "iron_plate",
            60.0,
            0,
            &HashSet::new(),
        )
        .unwrap();

        // 60 iron plate/min needs 1 smelter (10 per 10s = 60/min)
        assert_eq!(chain.machines_needed, 1.0);
        // It needs 60 molten_iron/min
        assert_eq!(chain.children.len(), 1);
        // The upstream molten_iron recipe: 12 per 20s = 36/min per machine
        // Need 60/36 = 1.667 furnaces
        if let data::models::ChainSource::Recipe(ref upstream) = chain.children[0].source {
            assert!((upstream.machines_needed - 60.0 / 36.0).abs() < 0.01);
        } else {
            panic!("Expected upstream recipe");
        }
    }

    #[test]
    fn test_balance_analysis() {
        use std::collections::HashSet;

        let data = GameData {
            resources: vec![],
            machines: vec![Machine {
                id: "furnace".to_string(),
                name: "Furnace".to_string(),
                name_zh: None,
                power_consumption: 50.0,
                category: String::new(),
                workers: 0,
                maintenance: vec![],
                computing: 0.0,
            }],
            recipes: vec![Recipe {
                id: "copper".to_string(),
                name: "Copper Plate".to_string(),
                name_zh: None,
                inputs: vec![Ingredient {
                    resource_id: ResourceId("copper_ore".to_string()),
                    amount: 10.0,
                }],
                outputs: vec![Ingredient {
                    resource_id: ResourceId("copper_plate".to_string()),
                    amount: 10.0,
                }],
                duration: 10.0,
                machine_id: "furnace".to_string(),
                tier: 0,
                tags: vec![],
                output_multiplier: 1.0,
                unity_consumption: 0.0,
                unity_production: 0.0,
            }],
        };

        let chain = calculator::chain::calculate_chain(
            &data,
            "copper",
            60.0,
            0,
            &HashSet::new(),
        )
        .unwrap();

        let report = calculator::balance::analyze_balance(&chain, &data);

        // copper_plate is produced (surplus), copper_ore is consumed (deficit as raw material)
        assert!(!report.resource_balances.is_empty());
        assert!(!report.machine_totals.is_empty());
    }
}
