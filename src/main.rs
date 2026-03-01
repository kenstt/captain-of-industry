rust_i18n::i18n!("locales", fallback = "en");

mod app;
mod calculator;
mod data;
mod i18n;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Captain of Industry - Production Calculator"),
        ..Default::default()
    };

    eframe::run_native(
        "Captain of Industry Calculator",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
