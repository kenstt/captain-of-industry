use captain_of_industry::{Calculator, Ingredient, Machine, Recipe, ResourceId};

fn main() {
    let mut calc = Calculator::new();

    // 建立機器
    calc.add_machine(Machine {
        id: "blast_furnace".to_string(),
        name: "高爐 (Blast Furnace)".to_string(),
    });

    // 建立配方：鐵水 (Molten Iron)
    // 12 鐵礦 + 3 焦炭 -> 12 鐵水, 耗時 20 秒
    calc.add_recipe(Recipe {
        id: "molten_iron".to_string(),
        name: "鐵水".to_string(),
        inputs: vec![
            Ingredient {
                resource_id: ResourceId("iron_ore".to_string()),
                amount: 12.0,
            },
            Ingredient {
                resource_id: ResourceId("coke".to_string()),
                amount: 3.0,
            },
        ],
        outputs: vec![Ingredient {
            resource_id: ResourceId("molten_iron".to_string()),
            amount: 12.0,
        }],
        duration: 20.0,
        machine_id: "blast_furnace".to_string(),
    });

    // 目標：每分鐘產出 60 單位鐵水
    let target_output = 60.0;
    if let Some(result) = calc.calculate_requirements("molten_iron", target_output) {
        println!("目標產出: {:.1} 鐵水 / 分鐘", target_output);
        println!("配方: {}", result.recipe_name);
        println!("機器: {} 需要 {:.2} 台", result.machine_name, result.machines_needed);

        println!("輸入:");
        for input in result.inputs {
            println!("  - {}: {:.2} / 分鐘", input.resource_id.0, input.amount);
        }

        println!("輸出:");
        for output in result.outputs {
            println!("  - {}: {:.2} / 分鐘", output.resource_id.0, output.amount);
        }
    } else {
        println!("找不到配方！");
    }
}
