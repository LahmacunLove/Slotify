use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::modes::{Mode, DjMode, GuestMode, AdminMode};

pub struct DjSystemApp {
    rt: Arc<Runtime>,
    current_mode: Mode,
    dj_mode: DjMode,
    guest_mode: GuestMode,
    admin_mode: AdminMode,
    api_base_url: String,
    logo_texture: Option<egui::TextureHandle>,
}

impl DjSystemApp {
    pub fn new(rt: Arc<Runtime>) -> Self {
        let api_base_url = "http://localhost:3000/api".to_string();

        Self {
            rt: rt.clone(),
            current_mode: Mode::DJ,
            dj_mode: DjMode::new(rt.clone(), api_base_url.clone()),
            guest_mode: GuestMode::new(rt.clone(), api_base_url.clone()),
            admin_mode: AdminMode::new(rt.clone(), api_base_url.clone()),
            api_base_url,
            logo_texture: None,
        }
    }

    fn load_logo(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_some() {
            return; // Already loaded
        }

        // Try to load logo from assets folder
        let logo_path = "assets/logo.jpg";
        if let Ok(image_data) = std::fs::read(logo_path) {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice(),
                );
                self.logo_texture = Some(ctx.load_texture(
                    "logo",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
    }

    fn render_mode_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().button_padding = egui::vec2(20.0, 10.0);
            
            if ui.selectable_label(
                matches!(self.current_mode, Mode::DJ),
                "📝 DJ Registration"
            ).clicked() {
                self.current_mode = Mode::DJ;
            }

            if ui.selectable_label(
                matches!(self.current_mode, Mode::Guest),
                "🎵 Session"
            ).clicked() {
                self.current_mode = Mode::Guest;
            }

            if ui.selectable_label(
                matches!(self.current_mode, Mode::Admin),
                "⚙️ Admin Mode"
            ).clicked() {
                self.current_mode = Mode::Admin;
            }
        });
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("🔗 API Status:");
            ui.colored_label(egui::Color32::GREEN, "Connected");
            
            ui.separator();
            
            ui.label("🕐 Current Time:");
            ui.label(chrono::Utc::now().format("%H:%M:%S").to_string());
        });
    }
}

impl eframe::App for DjSystemApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Load logo on first frame
        self.load_logo(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            // Title with logo
            ui.vertical_centered(|ui| {
                // Display logo if available
                if let Some(logo) = &self.logo_texture {
                    let logo_size = egui::vec2(150.0, 150.0); // Adjust size as needed
                    ui.add(egui::Image::new(logo).fit_to_exact_size(logo_size));
                    ui.add_space(5.0);
                }
                ui.heading("🎵 DA Slotify");
                ui.add_space(10.0);
            });

            // Mode selector
            self.render_mode_selector(ui);
            ui.add_space(20.0);

            // Main content area
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.current_mode {
                    Mode::DJ => self.dj_mode.render(ui),
                    Mode::Guest => self.guest_mode.render(ui),
                    Mode::Admin => self.admin_mode.render(ui),
                }
            });

            // Status bar at the bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                self.render_status_bar(ui);
            });
        });

        // Request repaint for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}