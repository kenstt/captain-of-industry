use std::collections::HashSet;

use eframe::egui;
use rust_i18n::t;

use crate::calculator::chain;
use crate::data::models::*;

pub struct ChainViewState {
    pub selected_recipe_id: Option<String>,
    pub target_output: String,
    pub primary_output_index: usize,
    pub supplied_resources: HashSet<ResourceId>,
    pub supply_input: String,
    pub chain_result: Option<ChainNode>,
    pub expanded_nodes: HashSet<String>,
}

impl Default for ChainViewState {
    fn default() -> Self {
        Self {
            selected_recipe_id: None,
            target_output: "60".to_string(),
            primary_output_index: 0,
            supplied_resources: HashSet::new(),
            supply_input: String::new(),
            chain_result: None,
            expanded_nodes: HashSet::new(),
        }
    }
}

pub fn show_chain_view(
    ui: &mut egui::Ui,
    state: &mut ChainViewState,
    data: &GameData,
) {
    ui.heading(t!("chain_title"));
    ui.separator();

    // Recipe selection
    ui.horizontal(|ui| {
        ui.label(t!("select_recipe"));
        egui::ComboBox::from_id_salt("chain_recipe")
            .selected_text(
                state
                    .selected_recipe_id
                    .as_ref()
                    .and_then(|id| data.recipes.iter().find(|r| r.id == *id))
                    .map(|r| {
                        r.name_zh
                            .as_deref()
                            .map(|zh| format!("{} ({})", zh, r.name))
                            .unwrap_or_else(|| r.name.clone())
                    })
                    .unwrap_or_else(|| "---".to_string()),
            )
            .show_ui(ui, |ui| {
                for recipe in &data.recipes {
                    let label = recipe
                        .name_zh
                        .as_deref()
                        .map(|zh| format!("{} ({})", zh, recipe.name))
                        .unwrap_or_else(|| recipe.name.clone());
                    if ui
                        .selectable_label(
                            state.selected_recipe_id.as_deref() == Some(&recipe.id),
                            &label,
                        )
                        .clicked()
                    {
                        state.selected_recipe_id = Some(recipe.id.clone());
                        state.primary_output_index = 0;
                        state.chain_result = None;
                    }
                }
            });
    });

    // Primary output for multi-output
    if let Some(ref recipe_id) = state.selected_recipe_id {
        if let Some(recipe) = data.recipes.iter().find(|r| r.id == *recipe_id) {
            if recipe.outputs.len() > 1 {
                ui.horizontal(|ui| {
                    ui.label(t!("primary_output"));
                    egui::ComboBox::from_id_salt("chain_primary_output")
                        .selected_text(
                            recipe
                                .outputs
                                .get(state.primary_output_index)
                                .map(|o| o.resource_id.0.as_str())
                                .unwrap_or("---"),
                        )
                        .show_ui(ui, |ui| {
                            for (i, output) in recipe.outputs.iter().enumerate() {
                                if ui
                                    .selectable_label(
                                        state.primary_output_index == i,
                                        &output.resource_id.0,
                                    )
                                    .clicked()
                                {
                                    state.primary_output_index = i;
                                    state.chain_result = None;
                                }
                            }
                        });
                });
            }
        }
    }

    // Target output
    ui.horizontal(|ui| {
        ui.label(t!("target_output"));
        ui.text_edit_singleline(&mut state.target_output);
    });

    // Supplied resources
    ui.separator();
    ui.strong(t!("supplied_resources"));
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.supply_input);
        if ui.button(t!("mark_supplied")).clicked() && !state.supply_input.is_empty() {
            state
                .supplied_resources
                .insert(ResourceId(state.supply_input.clone()));
            state.supply_input.clear();
        }
    });

    let mut to_remove = None;
    for rid in &state.supplied_resources {
        ui.horizontal(|ui| {
            ui.label(format!("  {}", rid.0));
            if ui.small_button(t!("unmark_supplied")).clicked() {
                to_remove = Some(rid.clone());
            }
        });
    }
    if let Some(rid) = to_remove {
        state.supplied_resources.remove(&rid);
    }

    ui.separator();

    // Calculate button
    if ui.button(t!("chain_calculate")).clicked() {
        if let Some(ref recipe_id) = state.selected_recipe_id {
            if let Ok(target) = state.target_output.parse::<f64>() {
                state.chain_result = chain::calculate_chain(
                    data,
                    recipe_id,
                    target,
                    state.primary_output_index,
                    &state.supplied_resources,
                );
                state.expanded_nodes.clear();
            }
        }
    }

    // Display chain result
    if let Some(ref node) = state.chain_result {
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("chain_tree")
            .show(ui, |ui| {
                show_chain_node(ui, node, 0, &mut state.expanded_nodes);
            });
    }
}

fn show_chain_node(
    ui: &mut egui::Ui,
    node: &ChainNode,
    depth: usize,
    expanded: &mut HashSet<String>,
) {
    let indent = "  ".repeat(depth);
    let node_key = format!("{}_{}", node.recipe_id, depth);
    let is_expanded = expanded.contains(&node_key);

    let header = format!(
        "{}{} -> {:.2} x {}",
        indent, node.recipe_name, node.machines_needed, node.machine_name
    );

    let toggle = if node.children.is_empty() {
        ui.label(&header);
        return;
    } else if is_expanded {
        ui.selectable_label(false, format!("▼ {}", header))
    } else {
        ui.selectable_label(false, format!("▶ {}", header))
    };

    if toggle.clicked() {
        if is_expanded {
            expanded.remove(&node_key);
        } else {
            expanded.insert(node_key);
        }
    }

    if is_expanded {
        for child in &node.children {
            let child_indent = "  ".repeat(depth + 1);
            match &child.source {
                ChainSource::Recipe(child_node) => {
                    show_chain_node(ui, child_node, depth + 1, expanded);
                }
                ChainSource::RawMaterial => {
                    ui.colored_label(
                        eframe::egui::Color32::from_rgb(180, 180, 100),
                        format!(
                            "{}{} ({:.2}/min) [{}]",
                            child_indent,
                            child.resource_id.0,
                            child.required_rate,
                            t!("raw_material")
                        ),
                    );
                }
                ChainSource::Supplied => {
                    ui.colored_label(
                        eframe::egui::Color32::from_rgb(100, 180, 100),
                        format!(
                            "{}{} ({:.2}/min) [{}]",
                            child_indent,
                            child.resource_id.0,
                            child.required_rate,
                            t!("supplied")
                        ),
                    );
                }
                ChainSource::CycleDetected => {
                    ui.colored_label(
                        eframe::egui::Color32::from_rgb(220, 100, 100),
                        format!(
                            "{}{} ({:.2}/min) [{}]",
                            child_indent,
                            child.resource_id.0,
                            child.required_rate,
                            t!("cycle_detected")
                        ),
                    );
                }
            }
        }
    }
}
