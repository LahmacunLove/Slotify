use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;

mod app;
mod components;
mod modes;
mod api_client;

use app::DjSystemApp;

fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    env_logger::init();

    // Create Tokio runtime for async operations
    let rt = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("DJ Session Recorder & Lottery System"),
        ..Default::default()
    };

    eframe::run_native(
        "DJ System",
        options,
        Box::new(|cc| {
            // Set up fonts for better touch interface
            setup_custom_style(&cc.egui_ctx);

            Ok(Box::new(DjSystemApp::new(rt)))
        }),
    )
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Increase default font size for touchscreen
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(18.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(28.0, egui::FontFamily::Proportional),
    );

    // Increase spacing for touch-friendly interface
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(10.0);

    ctx.set_style(style);
}