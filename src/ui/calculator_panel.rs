use eframe::egui;
use rust_i18n::t;

use crate::calculator::single;
use crate::data::models::*;

pub struct CalculatorPanelState {
    pub selected_recipe_id: Option<String>,
    pub target_output: String,
    pub primary_output_index: usize,
    pub round_up: bool,
    pub result: Option<CalculationResult>,
}

impl Default for CalculatorPanelState {
    fn default() -> Self {
        Self {
            selected_recipe_id: None,
            target_output: "60".to_string(),
            primary_output_index: 0,
            round_up: true,
            result: None,
        }
    }
}

pub fn show_calculator_panel(ui: &mut egui::Ui, state: &mut CalculatorPanelState, data: &GameData) {
    ui.heading(t!("nav_calculator"));
    ui.separator();

    // Recipe selection
    ui.horizontal(|ui| {
        ui.label(t!("select_recipe"));
        egui::ComboBox::from_id_salt("calc_recipe")
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
                        state.result = None;
                    }
                }
            });
    });

    // Primary output selection (for multi-output recipes)
    if let Some(ref recipe_id) = state.selected_recipe_id {
        if let Some(recipe) = data.recipes.iter().find(|r| r.id == *recipe_id) {
            if recipe.outputs.len() > 1 {
                ui.horizontal(|ui| {
                    ui.label(t!("primary_output"));
                    egui::ComboBox::from_id_salt("primary_output")
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
                                    state.result = None;
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

    // Round up toggle
    ui.checkbox(&mut state.round_up, t!("rounding"));

    ui.separator();

    // Calculate button
    if ui.button(t!("calculate")).clicked() {
        if let Some(ref recipe_id) = state.selected_recipe_id {
            if let Ok(target) = state.target_output.parse::<f64>() {
                state.result =
                    single::calculate_single(data, recipe_id, target, state.primary_output_index);
            }
        }
    }

    // Results
    if let Some(ref result) = state.result {
        ui.separator();
        ui.heading(&result.recipe_name);

        let machines_display = if state.round_up {
            format!(
                "{}: {} (≈ {})",
                t!("machines_needed"),
                format_f64(result.machines_needed),
                result.machines_needed.ceil() as u32
            )
        } else {
            format!(
                "{}: {}",
                t!("machines_needed"),
                format_f64(result.machines_needed)
            )
        };
        ui.label(format!("{}: {}", t!("machine"), result.machine_name));
        ui.label(machines_display);
        ui.label(format!(
            "{}: {:.1} kW",
            t!("power_consumption"),
            result.total_power
        ));
        ui.label(format!(
            "{}: {}",
            t!("workers"),
            format_f64(result.total_workers)
        ));
        if result.total_computing > 0.0 {
            ui.label(format!(
                "{}: {} TFLOPs",
                t!("computing"),
                format_f64(result.total_computing)
            ));
        }
        if !result.maintenance_costs.is_empty() {
            ui.label(format!("{}:", t!("maintenance")));
            for mc in &result.maintenance_costs {
                ui.label(format!(
                    "  {}: {}/month",
                    mc.resource_id.0,
                    format_f64(mc.amount)
                ));
            }
        }

        ui.separator();
        ui.strong(t!("input_rates"));
        egui::Grid::new("calc_inputs").striped(true).show(ui, |ui| {
            ui.strong(t!("resource"));
            ui.strong(t!("amount"));
            ui.end_row();
            for input in &result.inputs {
                ui.label(&input.resource_id.0);
                ui.label(format!("{}/min", format_f64(input.amount)));
                ui.end_row();
            }
        });

        ui.separator();
        ui.strong(t!("output_rates"));
        egui::Grid::new("calc_outputs")
            .striped(true)
            .show(ui, |ui| {
                ui.strong(t!("resource"));
                ui.strong(t!("amount"));
                ui.end_row();
                for output in &result.outputs {
                    ui.label(&output.resource_id.0);
                    ui.label(format!("{}/min", format_f64(output.amount)));
                    ui.end_row();
                }
            });
    }
}

fn format_f64(v: f64) -> String {
    if (v - v.round()).abs() < 0.001 {
        format!("{:.0}", v)
    } else {
        format!("{:.2}", v)
    }
}
