use eframe::egui;
use rust_i18n::t;

use crate::calculator::balance;
use crate::data::models::*;
use crate::ui::theme;

pub struct BalanceViewState {
    pub report: Option<BalanceReport>,
}

impl Default for BalanceViewState {
    fn default() -> Self {
        Self { report: None }
    }
}

/// Call this after chain calculation to generate balance report.
pub fn update_balance(
    state: &mut BalanceViewState,
    chain_node: Option<&ChainNode>,
    data: &GameData,
) {
    state.report = chain_node.map(|node| balance::analyze_balance(node, data));
}

pub fn show_balance_view(ui: &mut egui::Ui, state: &BalanceViewState) {
    ui.heading(t!("balance_title"));
    ui.separator();

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
            ui.end_row();

            for mt in &report.machine_totals {
                ui.label(&mt.machine_name);
                ui.label(format!("{:.2}", mt.count));
                ui.label(format!("{}", mt.count_ceil));
                ui.label(format!("{:.1} kW", mt.total_power));
                ui.end_row();
            }
        });

    ui.separator();
    ui.label(format!(
        "{}: {:.1} kW",
        t!("total_power"),
        report.total_power
    ));
}
