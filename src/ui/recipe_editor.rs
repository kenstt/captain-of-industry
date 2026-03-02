use eframe::egui;
use rust_i18n::t;

use crate::data::models::*;

pub struct RecipeEditorState {
    pub editing: bool,
    pub recipe_id: String,
    pub name_en: String,
    pub name_zh: String,
    pub machine_id: String,
    pub duration: String,
    pub tier: String,
    pub tags: String,
    pub inputs: Vec<(String, String)>,  // (resource_id, amount)
    pub outputs: Vec<(String, String)>, // (resource_id, amount)
    pub status_message: Option<String>,
}

impl Default for RecipeEditorState {
    fn default() -> Self {
        Self {
            editing: false,
            recipe_id: String::new(),
            name_en: String::new(),
            name_zh: String::new(),
            machine_id: String::new(),
            duration: "10".to_string(),
            tier: "0".to_string(),
            tags: String::new(),
            inputs: vec![("".to_string(), "1".to_string())],
            outputs: vec![("".to_string(), "1".to_string())],
            status_message: None,
        }
    }
}

impl RecipeEditorState {
    pub fn load_recipe(&mut self, recipe: &Recipe) {
        self.editing = true;
        self.recipe_id = recipe.id.clone();
        self.name_en = recipe.name.clone();
        self.name_zh = recipe.name_zh.clone().unwrap_or_default();
        self.machine_id = recipe.machine_id.clone();
        self.duration = recipe.duration.to_string();
        self.tier = recipe.tier.to_string();
        self.tags = recipe.tags.join(", ");
        self.inputs = recipe
            .inputs
            .iter()
            .map(|i| (i.resource_id.0.clone(), i.amount.to_string()))
            .collect();
        self.outputs = recipe
            .outputs
            .iter()
            .map(|o| (o.resource_id.0.clone(), o.amount.to_string()))
            .collect();
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn build_recipe(&self) -> Option<Recipe> {
        let duration: f64 = self.duration.parse().ok()?;
        let tier: u32 = self.tier.parse().unwrap_or(0);

        if self.recipe_id.is_empty() || self.name_en.is_empty() || self.machine_id.is_empty() {
            return None;
        }

        let inputs: Option<Vec<Ingredient>> = self
            .inputs
            .iter()
            .filter(|(id, _)| !id.is_empty())
            .map(|(id, amt)| {
                let amount: f64 = amt.parse().ok()?;
                Some(Ingredient {
                    resource_id: ResourceId(id.clone()),
                    amount,
                })
            })
            .collect();

        let outputs: Option<Vec<Ingredient>> = self
            .outputs
            .iter()
            .filter(|(id, _)| !id.is_empty())
            .map(|(id, amt)| {
                let amount: f64 = amt.parse().ok()?;
                Some(Ingredient {
                    resource_id: ResourceId(id.clone()),
                    amount,
                })
            })
            .collect();

        let tags: Vec<String> = self
            .tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Some(Recipe {
            id: self.recipe_id.clone(),
            name: self.name_en.clone(),
            name_zh: if self.name_zh.is_empty() {
                None
            } else {
                Some(self.name_zh.clone())
            },
            inputs: inputs?,
            outputs: outputs?,
            duration,
            machine_id: self.machine_id.clone(),
            tier,
            tags,
        })
    }
}

/// Returns true if the recipe was saved (so caller can update data).
pub fn show_recipe_editor(
    ui: &mut egui::Ui,
    state: &mut RecipeEditorState,
    data: &mut GameData,
) -> Option<Recipe> {
    let mut saved_recipe = None;

    ui.heading(if state.editing {
        t!("edit_recipe")
    } else {
        t!("new_recipe")
    });
    ui.separator();

    egui::Grid::new("recipe_editor_grid")
        .num_columns(2)
        .spacing([10.0, 6.0])
        .show(ui, |ui| {
            ui.label("ID:");
            ui.text_edit_singleline(&mut state.recipe_id);
            ui.end_row();

            ui.label(t!("recipe_name_en"));
            ui.text_edit_singleline(&mut state.name_en);
            ui.end_row();

            ui.label(t!("recipe_name_zh"));
            ui.text_edit_singleline(&mut state.name_zh);
            ui.end_row();

            ui.label(t!("machine"));
            egui::ComboBox::from_id_salt("editor_machine")
                .selected_text(if state.machine_id.is_empty() {
                    "---".to_string()
                } else {
                    data.machines
                        .iter()
                        .find(|m| m.id == state.machine_id)
                        .map(|m| m.name.clone())
                        .unwrap_or_else(|| state.machine_id.clone())
                })
                .show_ui(ui, |ui| {
                    for machine in &data.machines {
                        if ui
                            .selectable_label(state.machine_id == machine.id, &machine.name)
                            .clicked()
                        {
                            state.machine_id = machine.id.clone();
                        }
                    }
                });
            ui.end_row();

            ui.label(t!("duration"));
            ui.text_edit_singleline(&mut state.duration);
            ui.end_row();

            ui.label(t!("tier"));
            ui.text_edit_singleline(&mut state.tier);
            ui.end_row();

            ui.label(t!("tags"));
            ui.text_edit_singleline(&mut state.tags);
            ui.end_row();
        });

    // Machine properties editor (workers, maintenance, computing)
    if !state.machine_id.is_empty() {
        ui.separator();
        ui.strong(format!("{} - {}", t!("machine"), &state.machine_id));

        if let Some(machine) = data.machines.iter_mut().find(|m| m.id == state.machine_id) {
            egui::Grid::new("machine_props_grid")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    ui.label(t!("power_consumption"));
                    let mut power_str = machine.power_consumption.to_string();
                    if ui.text_edit_singleline(&mut power_str).changed() {
                        if let Ok(v) = power_str.parse::<f64>() {
                            machine.power_consumption = v;
                        }
                    }
                    ui.end_row();

                    ui.label(t!("workers"));
                    let mut workers_str = machine.workers.to_string();
                    if ui.text_edit_singleline(&mut workers_str).changed() {
                        if let Ok(v) = workers_str.parse::<u32>() {
                            machine.workers = v;
                        }
                    }
                    ui.end_row();

                    ui.label(t!("computing"));
                    let mut computing_str = machine.computing.to_string();
                    if ui.text_edit_singleline(&mut computing_str).changed() {
                        if let Ok(v) = computing_str.parse::<f64>() {
                            machine.computing = v;
                        }
                    }
                    ui.end_row();
                });

            // Maintenance items
            ui.label(t!("maintenance"));
            let mut maint_to_remove = None;
            for (i, item) in machine.maintenance.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", t!("resource")));
                    ui.text_edit_singleline(&mut item.resource_id.0);
                    ui.label(format!("{}:", t!("amount")));
                    let mut amt_str = item.amount.to_string();
                    if ui
                        .add(egui::TextEdit::singleline(&mut amt_str).desired_width(60.0))
                        .changed()
                    {
                        if let Ok(v) = amt_str.parse::<f64>() {
                            item.amount = v;
                        }
                    }
                    if ui.small_button(t!("remove")).clicked() {
                        maint_to_remove = Some(i);
                    }
                });
            }
            if let Some(i) = maint_to_remove {
                machine.maintenance.remove(i);
            }
            if ui.button(t!("add_maintenance")).clicked() {
                machine.maintenance.push(crate::data::models::MaintenanceItem {
                    resource_id: ResourceId("maintenance_1".to_string()),
                    amount: 0.5,
                });
            }
        }
    }

    ui.separator();

    // Inputs section
    ui.strong(t!("inputs"));
    let mut input_to_remove = None;
    for (i, (id, amt)) in state.inputs.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", t!("resource")));
            ui.text_edit_singleline(id);
            ui.label(format!("{}:", t!("amount")));
            ui.add(egui::TextEdit::singleline(amt).desired_width(60.0));
            if ui.small_button(t!("remove")).clicked() {
                input_to_remove = Some(i);
            }
        });
    }
    if let Some(i) = input_to_remove {
        state.inputs.remove(i);
    }
    if ui.button(t!("add_input")).clicked() {
        state.inputs.push(("".to_string(), "1".to_string()));
    }

    ui.separator();

    // Outputs section
    ui.strong(t!("outputs"));
    let mut output_to_remove = None;
    for (i, (id, amt)) in state.outputs.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", t!("resource")));
            ui.text_edit_singleline(id);
            ui.label(format!("{}:", t!("amount")));
            ui.add(egui::TextEdit::singleline(amt).desired_width(60.0));
            if ui.small_button(t!("remove")).clicked() {
                output_to_remove = Some(i);
            }
        });
    }
    if let Some(i) = output_to_remove {
        state.outputs.remove(i);
    }
    if ui.button(t!("add_output")).clicked() {
        state.outputs.push(("".to_string(), "1".to_string()));
    }

    ui.separator();

    // Action buttons
    ui.horizontal(|ui| {
        if ui.button(t!("save")).clicked() {
            if let Some(recipe) = state.build_recipe() {
                saved_recipe = Some(recipe);
                state.status_message = Some(t!("export_success").to_string());
            } else {
                state.status_message = Some("Invalid recipe data".to_string());
            }
        }
        if ui.button(t!("cancel")).clicked() {
            state.clear();
        }
    });

    // Import/Export
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button(t!("import_json")).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                match crate::data::loader::load_from_json(&path) {
                    Ok(imported) => {
                        let count = imported.recipes.len();
                        state.status_message =
                            Some(t!("import_success", count = count).to_string());
                        // Return the first recipe for editing, but the full import
                        // should be handled by the caller
                    }
                    Err(e) => {
                        state.status_message = Some(t!("file_error", error = e).to_string());
                    }
                }
            }
        }
        if ui.button(t!("export_json")).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("recipes.json")
                .save_file()
            {
                let export_data = GameData {
                    recipes: data.recipes.clone(),
                    machines: data.machines.clone(),
                    resources: data.resources.clone(),
                };
                match crate::data::loader::save_to_json(&export_data, &path) {
                    Ok(()) => {
                        state.status_message = Some(t!("export_success").to_string());
                    }
                    Err(e) => {
                        state.status_message = Some(t!("file_error", error = e).to_string());
                    }
                }
            }
        }
    });

    if let Some(ref msg) = state.status_message {
        ui.label(msg.as_str());
    }

    saved_recipe
}
