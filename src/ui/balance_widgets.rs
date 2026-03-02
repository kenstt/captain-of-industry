use eframe::egui::Color32;
use rust_i18n::t;

use crate::data::models::BalanceStatus;
use crate::ui::theme;

pub fn balance_status_visual(status: &BalanceStatus) -> (Color32, String) {
    match status {
        BalanceStatus::Surplus => (theme::surplus_color(), t!("surplus").to_string()),
        BalanceStatus::Deficit => (theme::deficit_color(), t!("deficit").to_string()),
        BalanceStatus::Balanced => (theme::balanced_color(), t!("balanced").to_string()),
        BalanceStatus::Bottleneck => (theme::bottleneck_color(), t!("bottleneck").to_string()),
    }
}
