use eframe::egui;
use rust_i18n::t;

use crate::calculator::balance;
use crate::data::models::*;
use crate::ui::theme;

pub struct BalanceEntry {
    pub recipe_id: String,
    pub machine_count: String,
}

pub struct BalanceViewState {
    pub entries: Vec<BalanceEntry>,
    pub report: Option<BalanceReport>,
}

impl Default for BalanceViewState {
    fn default() -> Self {
        Self {
            entries: vec![BalanceEntry {
                recipe_id: String::new(),
                machine_count: "1".to_string(),
            }],
            report: None,
        }
    }
}

pub fn show_balance_view(ui: &mut egui::Ui, state: &mut BalanceViewState, data: &GameData) {
    ui.heading(t!("balance_title"));
    ui.separator();

    // Recipe entry rows
    let mut to_remove: Option<usize> = None;

    for (i, entry) in state.entries.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            // Recipe dropdown
            let selected_label = if entry.recipe_id.is_empty() {
                t!("select_recipe").to_string()
            } else {
                data.recipes
                    .iter()
                    .find(|r| r.id == entry.recipe_id)
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| entry.recipe_id.clone())
            };

            egui::ComboBox::from_id_salt(format!("balance_recipe_{i}"))
                .selected_text(&selected_label)
                .width(250.0)
                .show_ui(ui, |ui| {
                    for recipe in &data.recipes {
                        ui.selectable_value(&mut entry.recipe_id, recipe.id.clone(), &recipe.name);
                    }
                });

            // Machine count input
            ui.label(t!("machine_count"));
            ui.add(egui::TextEdit::singleline(&mut entry.machine_count).desired_width(60.0));

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

    ui.horizontal(|ui| {
        // Add recipe entry button
        if ui.button(t!("add_recipe_entry")).clicked() {
            state.entries.push(BalanceEntry {
                recipe_id: String::new(),
                machine_count: "1".to_string(),
            });
        }

        // Calculate button
        if ui.button(t!("calculate_balance")).clicked() {
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

            if !parsed.is_empty() {
                state.report = Some(balance::analyze_balance_from_recipes(&parsed, data));
            }
        }
    });

    ui.separator();

    // Report display (same as before)
    let Some(ref report) = state.report else {
        ui.label(t!("no_results"));
        return;
    };

    // Resource balance table
    egui::Grid::new("balance_table")
        .striped(true)
        .min_col_width(80.0)
        .show(ui, |ui| {
            ui.strong(t!("resource_name"));
            ui.strong(t!("production_rate"));
            ui.strong(t!("consumption_rate"));
            ui.strong(t!("net_rate"));
            ui.strong(t!("status"));
            ui.end_row();

            for rb in &report.resource_balances {
                let (color, status_text) = match rb.status {
                    BalanceStatus::Surplus => (theme::surplus_color(), t!("surplus")),
                    BalanceStatus::Deficit => (theme::deficit_color(), t!("deficit")),
                    BalanceStatus::Balanced => (theme::balanced_color(), t!("balanced")),
                    BalanceStatus::Bottleneck => (theme::bottleneck_color(), t!("bottleneck")),
                };

                ui.label(&rb.resource_name);
                ui.label(format!("{:.2}/min", rb.production_rate));
                ui.label(format!("{:.2}/min", rb.consumption_rate));
                ui.colored_label(color, format!("{:.2}/min", rb.net_rate));
                ui.colored_label(color, status_text);
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
            ui.strong(t!("power_consumption"));
            ui.strong(t!("workers"));
            ui.strong(t!("computing"));
            ui.end_row();

            for mt in &report.machine_totals {
                ui.label(&mt.machine_name);
                ui.label(format!("{:.2}", mt.count));
                ui.label(format!("{}", mt.count_ceil));
                ui.label(format!("{:.1} kW", mt.total_power));
                ui.label(format!("{}", mt.total_workers));
                ui.label(if mt.total_computing > 0.0 {
                    format!("{:.1}", mt.total_computing)
                } else {
                    "-".to_string()
                });
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
}
