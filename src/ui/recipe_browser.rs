use eframe::egui;
use rust_i18n::t;

use crate::data::models::GameData;

pub struct RecipeBrowserState {
    pub search_text: String,
    pub selected_machine_filter: Option<String>,
    pub selected_tier_filter: Option<u32>,
    pub selected_recipe_id: Option<String>,
}

impl Default for RecipeBrowserState {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            selected_machine_filter: None,
            selected_tier_filter: None,
            selected_recipe_id: None,
        }
    }
}

pub fn show_recipe_browser(ui: &mut egui::Ui, state: &mut RecipeBrowserState, data: &GameData) {
    ui.heading(t!("recipe_browser"));
    ui.separator();

    // Filters row
    ui.horizontal(|ui| {
        // Search box
        ui.label(t!("search"));
        ui.text_edit_singleline(&mut state.search_text);

        ui.separator();

        // Machine filter
        egui::ComboBox::from_id_salt("machine_filter")
            .selected_text(
                state
                    .selected_machine_filter
                    .as_deref()
                    .unwrap_or(&t!("filter_all")),
            )
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(state.selected_machine_filter.is_none(), t!("filter_all"))
                    .clicked()
                {
                    state.selected_machine_filter = None;
                }
                for machine in &data.machines {
                    let label = &machine.name;
                    if ui
                        .selectable_label(
                            state.selected_machine_filter.as_deref() == Some(&machine.id),
                            label,
                        )
                        .clicked()
                    {
                        state.selected_machine_filter = Some(machine.id.clone());
                    }
                }
            });

        // Tier filter
        egui::ComboBox::from_id_salt("tier_filter")
            .selected_text(
                state
                    .selected_tier_filter
                    .map(|t| format!("T{}", t))
                    .unwrap_or_else(|| t!("filter_all").to_string()),
            )
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(state.selected_tier_filter.is_none(), t!("filter_all"))
                    .clicked()
                {
                    state.selected_tier_filter = None;
                }
                let mut tiers: Vec<u32> = data.recipes.iter().map(|r| r.tier).collect();
                tiers.sort();
                tiers.dedup();
                for tier in tiers {
                    let label = format!("T{}", tier);
                    if ui
                        .selectable_label(state.selected_tier_filter == Some(tier), &label)
                        .clicked()
                    {
                        state.selected_tier_filter = Some(tier);
                    }
                }
            });
    });

    ui.separator();

    // Filter recipes
    let search_lower = state.search_text.to_lowercase();
    let filtered: Vec<_> = data
        .recipes
        .iter()
        .filter(|r| {
            if !search_lower.is_empty() {
                let matches_name = r.name.to_lowercase().contains(&search_lower);
                let matches_zh = r
                    .name_zh
                    .as_deref()
                    .map(|n| n.to_lowercase().contains(&search_lower))
                    .unwrap_or(false);
                let matches_id = r.id.to_lowercase().contains(&search_lower);
                if !(matches_name || matches_zh || matches_id) {
                    return false;
                }
            }
            if let Some(ref mid) = state.selected_machine_filter {
                if r.machine_id != *mid {
                    return false;
                }
            }
            if let Some(tier) = state.selected_tier_filter {
                if r.tier != tier {
                    return false;
                }
            }
            true
        })
        .collect();

    // Split: recipe list on left, detail on right
    ui.columns(2, |cols| {
        // Left column: recipe list
        egui::ScrollArea::vertical()
            .id_salt("recipe_list")
            .show(&mut cols[0], |ui| {
                if filtered.is_empty() {
                    ui.label(t!("no_results"));
                }
                for recipe in &filtered {
                    let display_name = recipe
                        .name_zh
                        .as_deref()
                        .map(|zh| format!("{} ({})", zh, recipe.name))
                        .unwrap_or_else(|| recipe.name.clone());

                    let selected = state.selected_recipe_id.as_deref() == Some(&recipe.id);
                    if ui.selectable_label(selected, &display_name).clicked() {
                        state.selected_recipe_id = Some(recipe.id.clone());
                    }
                }
            });

        // Right column: recipe detail
        egui::ScrollArea::vertical()
            .id_salt("recipe_detail")
            .show(&mut cols[1], |ui| {
                if let Some(ref selected_id) = state.selected_recipe_id {
                    if let Some(recipe) = data.recipes.iter().find(|r| r.id == *selected_id) {
                        ui.heading(&recipe.name);
                        if let Some(ref zh) = recipe.name_zh {
                            ui.label(zh);
                        }
                        ui.separator();

                        let machines_map = data.machines_map();
                        if let Some(machine) = machines_map.get(&recipe.machine_id) {
                            ui.label(format!("{}: {}", t!("machine"), machine.name));
                        }
                        ui.label(format!("{}: {}s", t!("duration"), recipe.duration));
                        if recipe.tier > 0 {
                            ui.label(format!("{}: T{}", t!("tier"), recipe.tier));
                        }

                        ui.separator();
                        ui.strong(t!("inputs"));
                        for input in &recipe.inputs {
                            ui.label(format!("  {} × {}", input.resource_id.0, input.amount));
                        }

                        ui.separator();
                        ui.strong(t!("outputs"));
                        for output in &recipe.outputs {
                            ui.label(format!("  {} × {}", output.resource_id.0, output.amount));
                        }

                        if !recipe.tags.is_empty() {
                            ui.separator();
                            ui.label(format!("{}: {}", t!("tags"), recipe.tags.join(", ")));
                        }
                    }
                } else {
                    ui.label(t!("select_recipe"));
                }
            });
    });
}
