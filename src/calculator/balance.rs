use std::collections::HashMap;

use crate::data::models::*;

/// Analyze resource balance across an entire production chain.
pub fn analyze_balance(node: &ChainNode, data: &GameData) -> BalanceReport {
    let mut production: HashMap<ResourceId, f64> = HashMap::new();
    let mut consumption: HashMap<ResourceId, f64> = HashMap::new();
    let mut machine_counts: HashMap<String, (String, f64)> = HashMap::new();

    collect_rates(node, &mut production, &mut consumption, &mut machine_counts);

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
        order(&a.status)
            .cmp(&order(&b.status))
            .then_with(|| a.net_rate.partial_cmp(&b.net_rate).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Mark the worst deficit as bottleneck
    if let Some(first) = resource_balances.first_mut() {
        if first.status == BalanceStatus::Deficit {
            first.status = BalanceStatus::Bottleneck;
        }
    }

    // Build machine tallies
    let mut total_power = 0.0;
    let machine_totals: Vec<MachineTally> = machine_counts
        .into_iter()
        .map(|(mid, (name, count))| {
            let power_per = machines_map
                .get(&mid)
                .map(|m| m.power_consumption)
                .unwrap_or(0.0);
            let tp = power_per * count.ceil() as f64;
            total_power += tp;
            MachineTally {
                machine_id: mid,
                machine_name: name,
                count,
                count_ceil: count.ceil() as u32,
                total_power: tp,
            }
        })
        .collect();

    BalanceReport {
        resource_balances,
        machine_totals,
        total_power,
    }
}

fn collect_rates(
    node: &ChainNode,
    production: &mut HashMap<ResourceId, f64>,
    consumption: &mut HashMap<ResourceId, f64>,
    machine_counts: &mut HashMap<String, (String, f64)>,
) {
    // This node produces its outputs and consumes its inputs
    for output in &node.outputs {
        *production.entry(output.resource_id.clone()).or_insert(0.0) += output.amount;
    }
    for input in &node.inputs {
        *consumption.entry(input.resource_id.clone()).or_insert(0.0) += input.amount;
    }

    // Accumulate machine count
    let entry = machine_counts
        .entry(node.recipe_id.clone())
        .or_insert_with(|| (node.machine_name.clone(), 0.0));
    entry.1 += node.machines_needed;

    // Recurse into children
    for child in &node.children {
        if let ChainSource::Recipe(ref child_node) = child.source {
            collect_rates(child_node, production, consumption, machine_counts);
        }
    }
}
