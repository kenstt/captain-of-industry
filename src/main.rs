use captain_of_industry::{Calculator, Ingredient, Machine, Recipe, ResourceId};
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Captain of Industry Calculator",
        native_options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(App::new()) as Box<dyn eframe::App>)
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 優先順序：macOS 苹方 (PingFang) -> 黑體 (STHeiti) -> 宋體 (Songti) -> Windows 微軟正黑體 (msjh.ttc)
    let font_paths = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/Supplemental/Songti.ttc",
        "C:\\Windows\\Fonts\\msjh.ttc",
        "C:\\Windows\\Fonts\\msjhbd.ttc",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\kaiu.ttf",
    ];

    let mut loaded_count = 0;
    for path in font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            let font_name = format!("chinese_font_{}", loaded_count);
            fonts.font_data.insert(
                font_name.clone(),
                egui::FontData::from_owned(font_data).into(),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, font_name.clone());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(font_name);

            loaded_count += 1;
        }
    }

    if loaded_count > 0 {
        ctx.set_fonts(fonts);
    }
}

struct App {
    calc: Calculator,
    active_recipes: Vec<(String, f64)>,
    new_recipe_id: String,
    new_recipe_count: String,
    lang: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    ZhTw,
    En,
}

impl Language {
    fn name(&self) -> &'static str {
        match self {
            Language::ZhTw => "正體中文",
            Language::En => "English",
        }
    }

    fn t(&self, key: &str) -> String {
        let text = match self {
            Language::ZhTw => match key {
                "title" => "Captain of Industry 生產線計算器",
                "select_recipe" => "選擇配方：",
                "count" => "數量：",
                "add_machine" => "新增設備",
                "current_line" => "當前生產線",
                "resource_balance" => "資源平衡 (每分鐘)",
                "machines_unit" => "台",
                "unknown" => "未知",
                _ => key,
            },
            Language::En => match key {
                "title" => "Captain of Industry Production Line Calculator",
                "select_recipe" => "Select Recipe:",
                "count" => "Count:",
                "add_machine" => "Add Machine",
                "current_line" => "Current Production Line",
                "resource_balance" => "Resource Balance (per min)",
                "machines_unit" => "units",
                "unknown" => "Unknown",
                _ => key,
            },
        };
        text.to_string()
    }
}

impl App {
    fn new() -> Self {
        let mut calc = Calculator::new();

        // Add some default data
        calc.add_machine(Machine {
            id: "blast_furnace".to_string(),
            name: "高爐 (Blast Furnace)".to_string(),
        });
        calc.add_machine(Machine {
            id: "assembly".to_string(),
            name: "組裝機 (Assembly)".to_string(),
        });

        calc.add_recipe(Recipe {
            id: "molten_iron".to_string(),
            name: "熔融鐵".to_string(),
            inputs: vec![
                Ingredient { resource_id: ResourceId("iron_ore".to_string()), amount: 12.0 },
                Ingredient { resource_id: ResourceId("coke".to_string()), amount: 3.0 },
            ],
            outputs: vec![Ingredient { resource_id: ResourceId("molten_iron".to_string()), amount: 12.0 }],
            duration: 20.0,
            machine_id: "blast_furnace".to_string(),
        });

        calc.add_recipe(Recipe {
            id: "iron_plate".to_string(),
            name: "鐵板".to_string(),
            inputs: vec![
                Ingredient { resource_id: ResourceId("molten_iron".to_string()), amount: 12.0 },
            ],
            outputs: vec![Ingredient { resource_id: ResourceId("iron_plate".to_string()), amount: 12.0 }],
            duration: 20.0,
            machine_id: "blast_furnace".to_string(),
        });

        Self {
            calc,
            active_recipes: Vec::new(),
            new_recipe_id: "molten_iron".to_string(),
            new_recipe_count: "1.0".to_string(),
            lang: Language::ZhTw,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.lang.t("title"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::new("lang_select", "")
                        .selected_text(self.lang.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.lang, Language::ZhTw, Language::ZhTw.name());
                            ui.selectable_value(&mut self.lang, Language::En, Language::En.name());
                        });
                });
            });

            ui.horizontal(|ui| {
                ui.label(self.lang.t("select_recipe"));
                egui::ComboBox::new("recipe_select", "")
                    .selected_text(self.calc.recipes.get(&self.new_recipe_id).map(|r| r.name.as_str()).unwrap_or(""))
                    .show_ui(ui, |ui| {
                        for recipe in self.calc.recipes.values() {
                            ui.selectable_value(&mut self.new_recipe_id, recipe.id.clone(), &recipe.name);
                        }
                    });

                ui.label(self.lang.t("count"));
                ui.text_edit_singleline(&mut self.new_recipe_count);

                if ui.button(self.lang.t("add_machine")).clicked() {
                    if let Ok(count) = self.new_recipe_count.parse::<f64>() {
                        self.active_recipes.push((self.new_recipe_id.clone(), count));
                    }
                }
            });

            ui.separator();

            ui.columns(2, |cols| {
                cols[0].heading(self.lang.t("current_line"));
                let mut to_remove = None;
                for (i, (id, count)) in self.active_recipes.iter().enumerate() {
                    cols[0].horizontal(|ui| {
                        let unknown = self.lang.t("unknown");
                        let name = self.calc.recipes.get(id).map(|r| r.name.as_str()).unwrap_or(&unknown);
                        ui.label(format!("{}: {:.2} {}", name, count, self.lang.t("machines_unit")));
                        if ui.button("🗑").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }
                if let Some(i) = to_remove {
                    self.active_recipes.remove(i);
                }

                cols[1].heading(self.lang.t("resource_balance"));
                let flows = self.calc.calculate_net_flow(&self.active_recipes);
                let mut sorted_flows: Vec<_> = flows.into_iter().collect();
                sorted_flows.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));

                for (res_id, amount) in sorted_flows {
                    let color = if amount < 0.0 {
                        egui::Color32::RED
                    } else if amount > 0.0 {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::GRAY
                    };
                    cols[1].colored_label(color, format!("{}: {:.2}", res_id.0, amount));
                }
            });
        });
    }
}
