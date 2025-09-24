use eframe::egui;

mod app;
mod ui;

use app::RustyRootApp;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Rusty ROOT - CERN ROOT File Analyzer"),
        ..Default::default()
    };

    eframe::run_native(
        "Rusty ROOT",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            Ok(Box::new(RustyRootApp::new(cc)))
        }),
    )
}