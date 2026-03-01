use eframe::egui::{self, Color32, Visuals};

/// Apply a dark industrial theme inspired by the game.
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Darker background for industrial feel
    visuals.panel_fill = Color32::from_rgb(25, 25, 30);
    visuals.window_fill = Color32::from_rgb(30, 30, 35);
    visuals.extreme_bg_color = Color32::from_rgb(15, 15, 20);
    visuals.faint_bg_color = Color32::from_rgb(35, 35, 42);

    // Accent colors
    visuals.selection.bg_fill = Color32::from_rgb(60, 90, 140);
    visuals.hyperlink_color = Color32::from_rgb(100, 160, 240);

    ctx.set_visuals(visuals);
}

/// Color for surplus (positive net rate)
pub fn surplus_color() -> Color32 {
    Color32::from_rgb(80, 200, 80)
}

/// Color for deficit (negative net rate)
pub fn deficit_color() -> Color32 {
    Color32::from_rgb(220, 60, 60)
}

/// Color for balanced (zero net rate)
pub fn balanced_color() -> Color32 {
    Color32::from_rgb(220, 200, 60)
}

/// Color for bottleneck
pub fn bottleneck_color() -> Color32 {
    Color32::from_rgb(255, 80, 40)
}
