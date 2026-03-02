use eframe::egui;
use rust_i18n::t;

use crate::data::loader;
use crate::data::models::*;
use crate::i18n::{self, Locale};
use crate::ui::balance_view::{self, BalanceViewState};
use crate::ui::calculator_panel::{self, CalculatorPanelState};
use crate::ui::chain_view::{self, ChainViewState};
use crate::ui::recipe_browser::{self, RecipeBrowserState};
use crate::ui::recipe_editor::{self, RecipeEditorState};
use crate::ui::theme;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    RecipeBrowser,
    RecipeEditor,
    Calculator,
    ProductionChain,
    ResourceBalance,
}

pub struct App {
    current_tab: Tab,
    locale: Locale,
    data: GameData,
    browser_state: RecipeBrowserState,
    editor_state: RecipeEditorState,
    calc_state: CalculatorPanelState,
    chain_state: ChainViewState,
    balance_state: BalanceViewState,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load CJK font for Chinese support
        let mut fonts = egui::FontDefinitions::default();

        // Try to load NotoSansCJK from assets
        let font_paths = [
            "assets/fonts/NotoSansCJK-Regular.ttc",
            "assets/fonts/NotoSansSC-Regular.ttf",
            "assets/fonts/NotoSansTC-Regular.ttf",
        ];

        let mut cjk_loaded = false;
        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "cjk".to_owned(),
                    egui::FontData::from_owned(font_data).into(),
                );
                // Add CJK font as fallback for proportional and monospace
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("cjk".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("cjk".to_owned());
                cjk_loaded = true;
                break;
            }
        }

        // If no bundled font, try system fonts
        if !cjk_loaded {
            let system_fonts = [
                // macOS
                "/System/Library/Fonts/STHeiti Medium.ttc",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/System/Library/Fonts/Supplemental/Songti.ttc",
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
                "/System/Library/Fonts/PingFang.ttc",
                // Windows
                "C:/Windows/Fonts/msjh.ttc",       // Microsoft JhengHei (繁體)
                "C:/Windows/Fonts/msyh.ttc",       // Microsoft YaHei (簡體)
                "C:/Windows/Fonts/simsun.ttc",     // SimSun
                // Linux
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            ];
            for font_path in &system_fonts {
                if let Ok(font_data) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "cjk".to_owned(),
                        egui::FontData::from_owned(font_data).into(),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .push("cjk".to_owned());
                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("cjk".to_owned());
                    break;
                }
            }
        }

        cc.egui_ctx.set_fonts(fonts);

        // Apply dark theme
        theme::apply_theme(&cc.egui_ctx);

        // Load builtin data
        let data = loader::load_builtin_data();

        // Default to Chinese
        i18n::set_locale(Locale::ZhTW);

        Self {
            current_tab: Tab::RecipeBrowser,
            locale: Locale::ZhTW,
            data,
            browser_state: RecipeBrowserState::default(),
            editor_state: RecipeEditorState::default(),
            calc_state: CalculatorPanelState::default(),
            chain_state: ChainViewState::default(),
            balance_state: BalanceViewState::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Left side panel: navigation
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .default_width(160.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("CoI Calc");
                });
                ui.separator();

                // Language switcher
                ui.horizontal(|ui| {
                    ui.label(t!("language"));
                    egui::ComboBox::from_id_salt("locale_switcher")
                        .selected_text(self.locale.label())
                        .show_ui(ui, |ui| {
                            for locale in Locale::all() {
                                if ui
                                    .selectable_label(self.locale == *locale, locale.label())
                                    .clicked()
                                {
                                    self.locale = *locale;
                                    i18n::set_locale(*locale);
                                }
                            }
                        });
                });

                ui.separator();

                // Navigation buttons
                let tabs = [
                    (Tab::RecipeBrowser, "nav_recipes"),
                    (Tab::RecipeEditor, "recipe_editor"),
                    (Tab::Calculator, "nav_calculator"),
                    (Tab::ProductionChain, "nav_chain"),
                    (Tab::ResourceBalance, "nav_balance"),
                ];

                for (tab, key) in tabs {
                    if ui
                        .selectable_label(self.current_tab == tab, t!(key))
                        .clicked()
                    {
                        self.current_tab = tab;
                    }
                }

                ui.separator();

                // Import/Export buttons at bottom
                if ui.button(t!("import_json")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file()
                    {
                        if let Ok(imported) = loader::load_from_json(&path) {
                            self.data = self.data.clone().merge(imported);
                        }
                    }
                }
                if ui.button(t!("export_json")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_file_name("game_data.json")
                        .save_file()
                    {
                        let _ = loader::save_to_json(&self.data, &path);
                    }
                }
            });

        // Main content panel
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::RecipeBrowser => {
                    recipe_browser::show_recipe_browser(ui, &mut self.browser_state, &self.data);
                }
                Tab::RecipeEditor => {
                    if let Some(recipe) =
                        recipe_editor::show_recipe_editor(ui, &mut self.editor_state, &mut self.data)
                    {
                        // Update or add recipe to data
                        if let Some(existing) =
                            self.data.recipes.iter_mut().find(|r| r.id == recipe.id)
                        {
                            *existing = recipe;
                        } else {
                            self.data.recipes.push(recipe);
                        }
                    }
                }
                Tab::Calculator => {
                    calculator_panel::show_calculator_panel(
                        ui,
                        &mut self.calc_state,
                        &self.data,
                    );
                }
                Tab::ProductionChain => {
                    chain_view::show_chain_view(ui, &mut self.chain_state, &self.data);
                }
                Tab::ResourceBalance => {
                    balance_view::show_balance_view(
                        ui,
                        &mut self.balance_state,
                        &self.data,
                    );
                }
            }
        });
    }
}
