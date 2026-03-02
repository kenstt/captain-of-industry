//! 資源平衡分析器。
//!
//! 提供兩種入口：
//! - `analyze_balance`：從產線鏈樹分析
//! - `analyze_balance_from_recipes`：從「配方 + 機器數」清單分析

use std::collections::HashMap;

use crate::data::models::*;

/// 針對完整產線鏈進行資源平衡分析。
///
/// 會輸出：
/// - 各資源生產/消耗/淨值與狀態
/// - 機器數量匯總與總電力/人力/算力
/// - 維護成本對應的額外消耗
pub fn analyze_balance(node: &ChainNode, data: &GameData) -> BalanceReport {
    let mut production: HashMap<ResourceId, f64> = HashMap::new();
    let mut consumption: HashMap<ResourceId, f64> = HashMap::new();
    let mut machine_counts: HashMap<String, (String, f64)> = HashMap::new();
    let mut total_unity = 0.0;

    collect_rates(node, &mut production, &mut consumption, &mut machine_counts, &mut total_unity);

    let machines_map = data.machines_map();

    // Build resource balances
    let mut all_resources: std::collections::HashSet<ResourceId> = std::collections::HashSet::new();
    all_resources.extend(production.keys().cloned());
    all_resources.extend(consumption.keys().cloned());

    let resources_map = data.resources_map();

    let mut resource_balances: Vec<ResourceBalance> = all_resources
        .into_iter()
        .map(|rid| {
            let prod = production.get(&rid).copied().unwrap_or(0.0);
            let cons = consumption.get(&rid).copied().unwrap_or(0.0);
            let net = prod - cons;
            let name = resources_map
                .get(&rid)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| rid.0.clone());

            let status = if net < -0.001 {
                BalanceStatus::Deficit
            } else if net > 0.001 {
                BalanceStatus::Surplus
            } else {
                BalanceStatus::Balanced
            };

            ResourceBalance {
                resource_id: rid,
                resource_name: name,
                production_rate: prod,
                consumption_rate: cons,
                net_rate: net,
                status,
            }
        })
        .collect();

    // Sort: deficits first, then balanced, then surplus
    resource_balances.sort_by(|a, b| {
        let order = |s: &BalanceStatus| -> i32 {
            match s {
                BalanceStatus::Deficit | BalanceStatus::Bottleneck => 0,
                BalanceStatus::Balanced => 1,
                BalanceStatus::Surplus => 2,
            }
        };
        order(&a.status).cmp(&order(&b.status)).then_with(|| {
            a.net_rate
                .partial_cmp(&b.net_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    // Mark the worst deficit as bottleneck
    if let Some(first) = resource_balances.first_mut() {
        if first.status == BalanceStatus::Deficit {
            first.status = BalanceStatus::Bottleneck;
        }
    }

    // Build machine tallies
    let mut total_power = 0.0;
    let mut total_workers = 0.0;
    let mut total_computing = 0.0;
    // total_unity is handled in collect_rates for analyze_balance
    let machine_totals: Vec<MachineTally> = machine_counts
        .into_iter()
        .map(|(mid, (name, count))| {
            let machine = machines_map.get(&mid);
            let power_per = machine.map(|m| m.power_consumption).unwrap_or(0.0);
            let workers_per = machine.map(|m| m.workers).unwrap_or(0);
            let computing_per = machine.map(|m| m.computing).unwrap_or(0.0);
            let count_ceil = count.ceil() as u32;

            let tp = power_per * count_ceil as f64;
            let tw = workers_per * count_ceil;
            let tc = computing_per * count_ceil as f64;

            total_power += tp;
            total_workers += tw as f64;
            total_computing += tc;

            // Compute maintenance costs for this machine type
            let maintenance_costs: Vec<Ingredient> = machine
                .map(|m| {
                    m.maintenance
                        .iter()
                        .map(|mi| Ingredient {
                            resource_id: mi.resource_id.clone(),
                            amount: mi.amount * count_ceil as f64,
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Add maintenance to consumption in resource balances
            for mc in &maintenance_costs {
                // Find existing balance entry or we'll add after
                if let Some(rb) = resource_balances
                    .iter_mut()
                    .find(|rb| rb.resource_id == mc.resource_id)
                {
                    rb.consumption_rate += mc.amount;
                    rb.net_rate = rb.production_rate - rb.consumption_rate;
                    rb.status = if rb.net_rate < -0.001 {
                        BalanceStatus::Deficit
                    } else if rb.net_rate > 0.001 {
                        BalanceStatus::Surplus
                    } else {
                        BalanceStatus::Balanced
                    };
                } else {
                    let name = resources_map
                        .get(&mc.resource_id)
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| mc.resource_id.0.clone());
                    resource_balances.push(ResourceBalance {
                        resource_id: mc.resource_id.clone(),
                        resource_name: name,
                        production_rate: 0.0,
                        consumption_rate: mc.amount,
                        net_rate: -mc.amount,
                        status: BalanceStatus::Deficit,
                    });
                }
            }

            MachineTally {
                machine_id: mid,
                machine_name: name,
                count,
                count_ceil,
                total_power: tp,
                total_workers: tw,
                total_computing: tc,
                total_unity: 0.0, // Individually not attributed here
                maintenance_costs,
            }
        })
        .collect();

    // Re-sort after adding maintenance consumption
    resource_balances.sort_by(|a, b| {
        let order = |s: &BalanceStatus| -> i32 {
            match s {
                BalanceStatus::Deficit | BalanceStatus::Bottleneck => 0,
                BalanceStatus::Balanced => 1,
                BalanceStatus::Surplus => 2,
            }
        };
        order(&a.status).cmp(&order(&b.status)).then_with(|| {
            a.net_rate
                .partial_cmp(&b.net_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let total_maintenance = aggregate_maintenance(&machine_totals);

    BalanceReport {
        resource_balances,
        machine_totals,
        total_power,
        total_workers,
        total_computing,
        total_unity,
        total_maintenance,
    }
}

fn aggregate_maintenance(machine_totals: &[MachineTally]) -> Vec<Ingredient> {
    let mut map: HashMap<ResourceId, f64> = HashMap::new();
    for mt in machine_totals {
        for mc in &mt.maintenance_costs {
            *map.entry(mc.resource_id.clone()).or_insert(0.0) += mc.amount;
        }
    }
    let mut result: Vec<Ingredient> = map
        .into_iter()
        .map(|(resource_id, amount)| Ingredient {
            resource_id,
            amount,
        })
        .collect();
    result.sort_by(|a, b| a.resource_id.0.cmp(&b.resource_id.0));
    result
}

/// Analyze resource balance from a list of (recipe_id, machine_count) entries.
pub fn analyze_balance_from_recipes(
    entries: &[(String, f64)],
    data: &GameData,
    external: &ExternalFlows,
) -> BalanceReport {
    let recipes_map = data.recipes_map();
    let machines_map = data.machines_map();
    let resources_map = data.resources_map();

    let mut production: HashMap<ResourceId, f64> = HashMap::new();
    let mut consumption: HashMap<ResourceId, f64> = HashMap::new();
    let mut total_unity = 0.0;

    // Apply external supplies/consumptions (per minute)
    for ing in &external.supplies_per_min {
        *production.entry(ing.resource_id.clone()).or_insert(0.0) += ing.amount;
    }
    for ing in &external.consumptions_per_min {
        *consumption.entry(ing.resource_id.clone()).or_insert(0.0) += ing.amount;
    }

    // machine_id -> (machine_name, total_count)
    let mut machine_counts: HashMap<String, (String, f64)> = HashMap::new();

    for (recipe_id, machine_count) in entries {
        let Some(recipe) = recipes_map.get(recipe_id.as_str()) else {
            continue;
        };
        let rate_multiplier = (60.0 / recipe.duration) * machine_count;

        for input in &recipe.inputs {
            *consumption.entry(input.resource_id.clone()).or_insert(0.0) +=
                input.amount * rate_multiplier;
        }
        for output in &recipe.outputs {
            *production.entry(output.resource_id.clone()).or_insert(0.0) +=
                output.amount * rate_multiplier * recipe.output_multiplier;
        }

        total_unity += (recipe.unity_production - recipe.unity_consumption) * machine_count;

        let machine_name = machines_map
            .get(recipe.machine_id.as_str())
            .map(|m| m.name.clone())
            .unwrap_or_else(|| recipe.machine_id.clone());
        let entry = machine_counts
            .entry(recipe.machine_id.clone())
            .or_insert_with(|| (machine_name, 0.0));
        entry.1 += machine_count;
    }

    // Build resource balances
    let mut all_resources: std::collections::HashSet<ResourceId> = std::collections::HashSet::new();
    all_resources.extend(production.keys().cloned());
    all_resources.extend(consumption.keys().cloned());

    let mut resource_balances: Vec<ResourceBalance> = all_resources
        .into_iter()
        .map(|rid| {
            let prod = production.get(&rid).copied().unwrap_or(0.0);
            let cons = consumption.get(&rid).copied().unwrap_or(0.0);
            let net = prod - cons;
            let name = resources_map
                .get(&rid)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| rid.0.clone());
            let status = if net < -0.001 {
                BalanceStatus::Deficit
            } else if net > 0.001 {
                BalanceStatus::Surplus
            } else {
                BalanceStatus::Balanced
            };
            ResourceBalance {
                resource_id: rid,
                resource_name: name,
                production_rate: prod,
                consumption_rate: cons,
                net_rate: net,
                status,
            }
        })
        .collect();

    // Sort: deficits first, then balanced, then surplus
    resource_balances.sort_by(|a, b| {
        let order = |s: &BalanceStatus| -> i32 {
            match s {
                BalanceStatus::Deficit | BalanceStatus::Bottleneck => 0,
                BalanceStatus::Balanced => 1,
                BalanceStatus::Surplus => 2,
            }
        };
        order(&a.status).cmp(&order(&b.status)).then_with(|| {
            a.net_rate
                .partial_cmp(&b.net_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    // Mark the worst deficit as bottleneck
    if let Some(first) = resource_balances.first_mut() {
        if first.status == BalanceStatus::Deficit {
            first.status = BalanceStatus::Bottleneck;
        }
    }

    // Build machine tallies
    let mut total_power = 0.0;
    let mut total_workers = 0.0;
    let mut total_computing = 0.0;
    // total_unity is already calculated from recipes
    let machine_totals: Vec<MachineTally> = machine_counts
        .into_iter()
        .map(|(mid, (name, count))| {
            let machine = machines_map.get(mid.as_str());
            let power_per = machine.map(|m| m.power_consumption).unwrap_or(0.0);
            let workers_per = machine.map(|m| m.workers).unwrap_or(0);
            let computing_per = machine.map(|m| m.computing).unwrap_or(0.0);
            let count_ceil = count.ceil() as u32;

            let tp = power_per * count_ceil as f64;
            let tw = workers_per * count_ceil;
            let tc = computing_per * count_ceil as f64;

            total_power += tp;
            total_workers += tw as f64;
            total_computing += tc;

            let maintenance_costs: Vec<Ingredient> = machine
                .map(|m| {
                    m.maintenance
                        .iter()
                        .map(|mi| Ingredient {
                            resource_id: mi.resource_id.clone(),
                            amount: mi.amount * count_ceil as f64,
                        })
                        .collect()
                })
                .unwrap_or_default();

            for mc in &maintenance_costs {
                if let Some(rb) = resource_balances
                    .iter_mut()
                    .find(|rb| rb.resource_id == mc.resource_id)
                {
                    rb.consumption_rate += mc.amount;
                    rb.net_rate = rb.production_rate - rb.consumption_rate;
                    rb.status = if rb.net_rate < -0.001 {
                        BalanceStatus::Deficit
                    } else if rb.net_rate > 0.001 {
                        BalanceStatus::Surplus
                    } else {
                        BalanceStatus::Balanced
                    };
                } else {
                    let rname = resources_map
                        .get(&mc.resource_id)
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| mc.resource_id.0.clone());
                    resource_balances.push(ResourceBalance {
                        resource_id: mc.resource_id.clone(),
                        resource_name: rname,
                        production_rate: 0.0,
                        consumption_rate: mc.amount,
                        net_rate: -mc.amount,
                        status: BalanceStatus::Deficit,
                    });
                }
            }

            MachineTally {
                machine_id: mid,
                machine_name: name,
                count,
                count_ceil,
                total_power: tp,
                total_workers: tw,
                total_computing: tc,
                total_unity: 0.0,
                maintenance_costs,
            }
        })
        .collect();

    // Re-sort after adding maintenance consumption
    resource_balances.sort_by(|a, b| {
        let order = |s: &BalanceStatus| -> i32 {
            match s {
                BalanceStatus::Deficit | BalanceStatus::Bottleneck => 0,
                BalanceStatus::Balanced => 1,
                BalanceStatus::Surplus => 2,
            }
        };
        order(&a.status).cmp(&order(&b.status)).then_with(|| {
            a.net_rate
                .partial_cmp(&b.net_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let total_maintenance = aggregate_maintenance(&machine_totals);

    BalanceReport {
        resource_balances,
        machine_totals,
        total_power,
        total_workers,
        total_computing,
        total_unity,
        total_maintenance,
    }
}

fn collect_rates(
    node: &ChainNode,
    production: &mut HashMap<ResourceId, f64>,
    consumption: &mut HashMap<ResourceId, f64>,
    machine_counts: &mut HashMap<String, (String, f64)>,
    total_unity: &mut f64,
) {
    // 節點本身：產出 + 消耗。
    for output in &node.outputs {
        *production.entry(output.resource_id.clone()).or_insert(0.0) += output.amount;
    }
    for input in &node.inputs {
        *consumption.entry(input.resource_id.clone()).or_insert(0.0) += input.amount;
    }

    *total_unity += node.unity;

    // Accumulate machine count
    // 以 machine_id 聚合機器數，確保後續查表一致。
    let entry = machine_counts
        .entry(node.machine_id.clone())
        .or_insert_with(|| (node.machine_name.clone(), 0.0));
    entry.1 += node.machines_needed;

    // 遞迴累計子節點。
    for child in &node.children {
        if let ChainSource::Recipe(ref child_node) = child.source {
            collect_rates(child_node, production, consumption, machine_counts, total_unity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chain_node() -> ChainNode {
        ChainNode {
            recipe_id: "iron_ingot_recipe".to_string(),
            recipe_name: "Iron Ingot".to_string(),
            machine_id: "arc_furnace".to_string(),
            machine_name: "Arc Furnace".to_string(),
            machines_needed: 1.5,
            inputs: vec![Ingredient {
                resource_id: ResourceId("iron_ore".to_string()),
                amount: 90.0,
            }],
            outputs: vec![Ingredient {
                resource_id: ResourceId("iron_ingot".to_string()),
                amount: 60.0,
            }],
            children: vec![],
            power: 0.0,
            workers: 0.0,
            computing: 0.0,
            maintenance_costs: vec![],
        }
    }

    #[test]
    fn analyze_balance_uses_machine_id_for_machine_totals() {
        let data = GameData {
            resources: vec![],
            recipes: vec![],
            machines: vec![Machine {
                id: "arc_furnace".to_string(),
                name: "Arc Furnace".to_string(),
                name_zh: None,
                power_consumption: 120.0,
                category: String::new(),
                workers: 3,
                maintenance: vec![],
                computing: 0.0,
            }],
        };

        let report = analyze_balance(&sample_chain_node(), &data);

        assert_eq!(report.machine_totals.len(), 1);
        assert_eq!(report.machine_totals[0].machine_id, "arc_furnace");
        assert_eq!(report.machine_totals[0].count_ceil, 2);
        assert_eq!(report.machine_totals[0].total_power, 240.0);
        assert_eq!(report.machine_totals[0].total_workers, 6);
        assert_eq!(report.total_power, 240.0);
        assert_eq!(report.total_workers, 6.0);
    }
}
