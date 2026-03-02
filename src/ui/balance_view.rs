use eframe::egui;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::calculator::balance;
use crate::data::models::*;
use crate::ui::balance_widgets;

#[derive(Serialize, Deserialize)]
struct BalanceConfig {
    entries: Vec<BalanceConfigEntry>,
}

#[derive(Serialize, Deserialize)]
struct BalanceConfigEntry {
    recipe_id: String,
    machine_count: String,
    selected_tag: Option<String>,
}

fn format_maintenance(
    costs: &[Ingredient],
    resources_map: &std::collections::HashMap<ResourceId, &Resource>,
) -> String {
    if costs.is_empty() {
        return "-".to_string();
    }
    costs
        .iter()
        .map(|c| {
            let name = resources_map
                .get(&c.resource_id)
                .and_then(|r| r.name_zh.as_deref())
                .map(|zh| zh.to_string())
                .unwrap_or_else(|| {
                    resources_map
                        .get(&c.resource_id)
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| c.resource_id.0.clone())
                });
            format!("{} x{}", name, c.amount)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_count(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

fn recipe_label(r: &Recipe, data: &GameData) -> String {
    let base = r
        .name_zh
        .as_deref()
        .map(|zh| format!("{} ({})", zh, r.name))
        .unwrap_or_else(|| r.name.clone());

    let machine_name = data
        .machines
        .iter()
        .find(|m| m.id == r.machine_id)
        .and_then(|m| {
            m.name_zh
                .as_deref()
                .map(|zh| format!("{} ({})", zh, m.name))
                .or(Some(m.name.clone()))
        })
        .unwrap_or_default();

    let with_machine = if machine_name.is_empty() {
        base
    } else {
        format!("{} - {}", base, machine_name)
    };

    if r.tier > 0 {
        format!("[T{}] {}", r.tier, with_machine)
    } else {
        with_machine
    }
}

pub struct BalanceEntry {
    pub recipe_id: String,
    pub machine_count: String,
    pub selected_tag: Option<String>,
}

pub struct BalanceViewState {
    pub entries: Vec<BalanceEntry>,
    pub report: Option<BalanceReport>,
    last_fingerprint: String,
}

impl Default for BalanceViewState {
    fn default() -> Self {
        Self {
            entries: vec![BalanceEntry {
                recipe_id: String::new(),
                machine_count: "1".to_string(),
                selected_tag: None,
            }],
            report: None,
            last_fingerprint: String::new(),
        }
    }
}

pub fn show_balance_view(ui: &mut egui::Ui, state: &mut BalanceViewState, data: &GameData) {
    ui.heading(t!("balance_title"));
    ui.separator();

    // Collect all unique tags from recipes
    let mut all_tags: Vec<String> = data
        .recipes
        .iter()
        .flat_map(|r| r.tags.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    all_tags.sort();

    // Save/Load config buttons
    ui.horizontal(|ui| {
        if ui.button(t!("save_balance_config")).clicked() {
            let config = BalanceConfig {
                entries: state
                    .entries
                    .iter()
                    .filter(|e| !e.recipe_id.is_empty())
                    .map(|e| BalanceConfigEntry {
                        recipe_id: e.recipe_id.clone(),
                        machine_count: e.machine_count.clone(),
                        selected_tag: e.selected_tag.clone(),
                    })
                    .collect(),
            };
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("balance_config.json")
                .save_file()
            {
                if let Ok(json) = serde_json::to_string_pretty(&config) {
                    let _ = std::fs::write(path, json);
                }
            }
        }
        if ui.button(t!("load_balance_config")).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<BalanceConfig>(&contents) {
                        state.entries = config
                            .entries
                            .into_iter()
                            .map(|e| BalanceEntry {
                                recipe_id: e.recipe_id,
                                machine_count: e.machine_count,
                                selected_tag: e.selected_tag,
                            })
                            .collect();
                        if state.entries.is_empty() {
                            state.entries.push(BalanceEntry {
                                recipe_id: String::new(),
                                machine_count: "1".to_string(),
                                selected_tag: None,
                            });
                        }
                        state.last_fingerprint = String::new();
                    }
                }
            }
        }
    });

    // Recipe entry rows
    let mut to_remove: Option<usize> = None;

    for (i, entry) in state.entries.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            // Tag filter dropdown
            let tag_label = entry
                .selected_tag
                .as_deref()
                .unwrap_or(&t!("all_tags").to_string())
                .to_string();

            egui::ComboBox::from_id_salt(format!("balance_tag_{i}"))
                .selected_text(&tag_label)
                .width(120.0)
                .show_ui(ui, |ui| {
                    // "All" option
                    let is_all = entry.selected_tag.is_none();
                    if ui
                        .selectable_label(is_all, t!("all_tags").to_string())
                        .clicked()
                    {
                        entry.selected_tag = None;
                    }
                    // Individual tags
                    for tag in &all_tags {
                        let selected = entry.selected_tag.as_deref() == Some(tag.as_str());
                        if ui.selectable_label(selected, tag).clicked() {
                            entry.selected_tag = Some(tag.clone());
                        }
                    }
                });

            // Recipe dropdown — filtered by selected tag
            let selected_label = if entry.recipe_id.is_empty() {
                t!("select_recipe").to_string()
            } else {
                data.recipes
                    .iter()
                    .find(|r| r.id == entry.recipe_id)
                    .map(|r| recipe_label(r, data))
                    .unwrap_or_else(|| entry.recipe_id.clone())
            };

            egui::ComboBox::from_id_salt(format!("balance_recipe_{i}"))
                .selected_text(&selected_label)
                .width(350.0)
                .show_ui(ui, |ui| {
                    for recipe in &data.recipes {
                        // Apply tag filter
                        if let Some(ref tag) = entry.selected_tag {
                            if !recipe.tags.contains(tag) {
                                continue;
                            }
                        }
                        let label = recipe_label(recipe, data);
                        ui.selectable_value(&mut entry.recipe_id, recipe.id.clone(), &label);
                    }
                });

            // Machine count input
            if ui.small_button("-").clicked() {
                if let Ok(v) = entry.machine_count.parse::<f64>() {
                    let new_v = (v - 1.0).max(0.0);
                    entry.machine_count = format_count(new_v);
                }
            }
            ui.add(egui::TextEdit::singleline(&mut entry.machine_count).desired_width(60.0));
            if ui.small_button("+").clicked() {
                if let Ok(v) = entry.machine_count.parse::<f64>() {
                    entry.machine_count = format_count(v + 1.0);
                }
            }

            // Delete button
            if ui.button(t!("remove")).clicked() {
                to_remove = Some(i);
            }
        });
    }

    if let Some(idx) = to_remove {
        if state.entries.len() > 1 {
            state.entries.remove(idx);
        }
    }

    // Auto-add blank entry if the last entry already has a recipe selected
    if state
        .entries
        .last()
        .map(|e| !e.recipe_id.is_empty())
        .unwrap_or(true)
    {
        state.entries.push(BalanceEntry {
            recipe_id: String::new(),
            machine_count: "1".to_string(),
            selected_tag: None,
        });
    }

    // Auto-calculate balance only when inputs change
    let fingerprint: String = state
        .entries
        .iter()
        .map(|e| format!("{}:{}", e.recipe_id, e.machine_count))
        .collect::<Vec<_>>()
        .join("|");

    if fingerprint != state.last_fingerprint {
        state.last_fingerprint = fingerprint;

        let parsed: Vec<(String, f64)> = state
            .entries
            .iter()
            .filter(|e| !e.recipe_id.is_empty())
            .filter_map(|e| {
                e.machine_count
                    .parse::<f64>()
                    .ok()
                    .filter(|&v| v > 0.0)
                    .map(|v| (e.recipe_id.clone(), v))
            })
            .collect();

        state.report = if parsed.is_empty() {
            None
        } else {
            Some(balance::analyze_balance_from_recipes(&parsed, data))
        };
    }

    ui.separator();

    // Report display
    let Some(ref report) = state.report else {
        ui.label(t!("no_results"));
        return;
    };

    let resources_map = data.resources_map();
    let machines_map = data.machines_map();

    // Resource balance table
    egui::Grid::new("balance_table")
        .striped(true)
        .min_col_width(80.0)
        .show(ui, |ui| {
            ui.strong(t!("resource_name"));
            ui.strong(format!("{} (/min)", t!("production_rate")));
            ui.strong(format!("{} (/min)", t!("consumption_rate")));
            ui.strong(format!("{} (/min)", t!("net_rate")));
            ui.strong(t!("status"));
            ui.end_row();

            for rb in &report.resource_balances {
                let (color, status_text) = balance_widgets::balance_status_visual(&rb.status);

                let res_name = resources_map
                    .get(&rb.resource_id)
                    .and_then(|r| r.name_zh.as_deref())
                    .map(|zh| format!("{} ({})", zh, rb.resource_name))
                    .unwrap_or_else(|| rb.resource_name.clone());

                ui.label(res_name);
                ui.label(format!("{:.2}", rb.production_rate));
                ui.label(format!("{:.2}", rb.consumption_rate));
                ui.colored_label(color, format!("{:.2}", rb.net_rate));
                ui.colored_label(color, &status_text);
                ui.end_row();
            }
        });

    ui.separator();

    // Machine summary
    ui.heading(t!("machine_summary"));
    egui::Grid::new("machine_table")
        .striped(true)
        .min_col_width(80.0)
        .show(ui, |ui| {
            ui.strong(t!("machine_name"));
            ui.strong(t!("count"));
            ui.strong(t!("machines_needed_ceil"));
            ui.strong(format!("{} (kW)", t!("power_consumption")));
            ui.strong(t!("workers"));
            ui.strong(format!("{} (TFLOPs)", t!("computing")));
            ui.strong(t!("maintenance_per_month"));
            ui.end_row();

            for mt in &report.machine_totals {
                let machine_name = machines_map
                    .get(mt.machine_id.as_str())
                    .and_then(|m| m.name_zh.as_deref())
                    .map(|zh| format!("{} ({})", zh, mt.machine_name))
                    .unwrap_or_else(|| mt.machine_name.clone());

                ui.label(machine_name);
                ui.label(format!("{:.2}", mt.count));
                ui.label(format!("{}", mt.count_ceil));
                ui.label(format!("{:.1}", mt.total_power));
                ui.label(format!("{}", mt.total_workers));
                ui.label(if mt.total_computing > 0.0 {
                    format!("{:.1}", mt.total_computing)
                } else {
                    "-".to_string()
                });
                ui.label(format_maintenance(&mt.maintenance_costs, &resources_map));
                ui.end_row();
            }
        });

    ui.separator();
    ui.label(format!(
        "{}: {:.1} kW",
        t!("total_power"),
        report.total_power
    ));
    ui.label(format!(
        "{}: {:.0}",
        t!("total_workers"),
        report.total_workers
    ));
    if report.total_computing > 0.0 {
        ui.label(format!(
            "{}: {:.1} TFLOPs",
            t!("total_computing"),
            report.total_computing
        ));
    }
    if !report.total_maintenance.is_empty() {
        ui.label(format!(
            "{}: {}",
            t!("total_maintenance"),
            format_maintenance(&report.total_maintenance, &resources_map)
        ));
    }
}
